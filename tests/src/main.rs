mod context;

use crate::context::TestingContext;
use colored::Colorize;
use garnish_lang_compiler::build::build_with_data;
use garnish_lang_compiler::lex::lex;
use garnish_lang_compiler::parse::parse;
use garnish_lang_runtime::{SimpleGarnishRuntime, SimpleRuntimeState};
use garnish_lang_simple_data::{SimpleData, SimpleGarnishData};
use garnish_lang_traits::{GarnishData, GarnishRuntime};
use log::error;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::{fs, path};

fn collect_scripts(path: String) -> Vec<String> {
    let mut dirs = vec![path];
    let mut paths = vec![];

    while let Some(dir) = dirs.pop() {
        match fs::read_dir(dir) {
            Err(e) => error!("{}", e),
            Ok(contents) => {
                for file in contents {
                    match file {
                        Err(e) => error!("{}", e),
                        Ok(entry) => {
                            if entry.path().is_dir() {
                                dirs.push(entry.path().to_string_lossy().into_owned());
                            } else {
                                paths.push(entry.path().to_string_lossy().into_owned());
                            }
                        }
                    }
                }
            }
        }
    }

    paths
}

fn main() {
    env_logger::init();

    let script_paths = collect_scripts(String::from("tests/scripts"));
    let mut successes = 0;
    let mut failures = 0;
    let mut messages = vec![];
    let manual_filter: [&str; 0] = [
        
    ];

    for script_path in script_paths {
        let mut path = PathBuf::from(&script_path);
        while let Some(_) = path.extension() {
            path.set_extension("");
        }
        let script_name = path
            .to_string_lossy()
            .replace("tests/scripts", "")
            .trim_matches(path::MAIN_SEPARATOR)
            .replace(path::MAIN_SEPARATOR, ":");

        if !manual_filter.is_empty() && !manual_filter.contains(&script_name.as_str()) {
            continue;
        }

        let result = execute_script(&script_path);

        let message = match result {
            TestResult::Success => {
                successes += 1;
                "success".green()
            }
            TestResult::Failure(s) => {
                failures += 1;
                format!("failure - {}", s).red()
            }
            TestResult::Error(s) => {
                failures += 1;
                format!("error - {}", s).yellow()
            }
        };

        messages.push(format!("{} - {}", script_name, message));
    }

    println!(
        "{} | {}",
        format!("{} successes", successes).green(),
        format!("{} failures", failures).red()
    );
    for m in messages {
        println!("{}", m);
    }
}

enum TestResult {
    Success,
    Failure(String),
    Error(String),
}

fn execute_script(script_path: &String) -> TestResult {
    let mut data = SimpleGarnishData::new();
    match read_to_string(PathBuf::from(&script_path))
        .or_else(|e| Err(format!("{}", e)))
        .and_then(|file| {
            lex(&file)
                .or_else(|e| Err(format!("{}", e)))
                .and_then(|tokens| parse(&tokens).or_else(|e| Err(format!("{}", e))))
                .and_then(|parse_result| {
                    build_with_data(parse_result.get_root(), parse_result.get_nodes().clone(), &mut data).or_else(|e| Err(format!("{}", e)))
                })
                .or_else(|e| Err(format!("{}", e)))
        }) {
        Err(e) => TestResult::Error(format!("({}): {}", &script_path, e)),
        Ok(_) => {
            let start = match data.get_jump_points().get(0) {
                Some(jump_points) => *jump_points,
                None => {
                    return TestResult::Error(format!("({}) No jump point", &script_path));
                }
            };

            match data.set_instruction_cursor(start) {
                Ok(_) => (),
                Err(e) => {
                    return TestResult::Error(format!("({}): {}", &script_path, e));
                }
            }

            match data.add_unit().and_then(|e| data.push_value_stack(e)) {
                Ok(_) => (),
                Err(e) => {
                    return TestResult::Error(format!("({}): {}", &script_path, e));
                }
            }

            let mut context = TestingContext::default();
            let mut runtime = SimpleGarnishRuntime::new(data);

            loop {
                match runtime.execute_current_instruction(Some(&mut context)) {
                    Err(e) => {
                        return TestResult::Error(format!("({}): {}", &script_path, e));
                    }
                    Ok(data) => match data.get_state() {
                        SimpleRuntimeState::Running => (),
                        SimpleRuntimeState::End => break,
                    },
                }
            }

            let result = match runtime.get_data().get_current_value().and_then(|i| runtime.get_data().get_data().get(i)) {
                Some(value) => value,
                None => {
                    return TestResult::Error(format!("({}) No current value after execution", &script_path));
                }
            };

            let (left, right) = match result {
                SimpleData::Pair(left_index, right_index) => (*left_index, *right_index),
                t => return TestResult::Error(format!("expected a Pair value, got {:?}", t)),
            };

            runtime.put(left).unwrap();
            runtime.put(right).unwrap();
            runtime.equal().unwrap();
            runtime.push_value().unwrap();

            let data = runtime.get_data_owned();

            match data.get_current_value().and_then(|i| data.get_data().get(i)) {
                Some(SimpleData::True) => TestResult::Success,
                Some(SimpleData::False) => TestResult::Failure(format!(
                    "[{} = {}]",
                    data.get_data().display_for_item(left),
                    data.get_data().display_for_item(right)
                )),
                Some(v) => TestResult::Error(format!("Got non-boolean result after comparison, got {:?}", v.display_simple())),
                None => TestResult::Error(String::from("No current value after comparison")),
            }

            // if left == right {
            //     TestResult::Success
            // } else {
            //     TestResult::Failure(format!("[{} = {}]",
            //                                 left.display_simple(),
            //                                 right.display_simple()
            //                                 // data.get_data().display_for_item(*left_index),
            //                                 // data.get_data().display_for_item(*right_index)
            //     ))
            // }
            //     match (runtime.get_data().get_data().get(*left_index), runtime.get_data().get_data().get(*right_index)) {
            //         (Some(left), Some(right)) => (left, right),
            //         (l, r) => return TestResult::Error(format!("invalid Pair value. left = {:?}, right = {:?}", l, r))
            //     }
            // }
        }
    }
}
