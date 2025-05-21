mod context;

use crate::context::TestingContext;
use colored::Colorize;
use garnish_lang_compiler::build::build;
use garnish_lang_compiler::lex::lex;
use garnish_lang_compiler::parse::{ParseNode, parse};
use garnish_lang_runtime::{execute_current_instruction, ops, SimpleRuntimeState};
use garnish_lang_simple_data::{SimpleData, SimpleGarnishData};
use garnish_lang_traits::{GarnishData, Instruction};
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

    // future cli options
    let manual_filter: Vec<&str> = vec![];
    let display_successes = false;
    let create_dump_files = false;

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

        let result = execute_script(&script_path, create_dump_files);

        match result {
            TestResult::Success => {
                successes += 1;
                if display_successes {
                    messages.push(format!("{} - {}", script_name, "success".green()))
                }
            }
            TestResult::Failure(s) => {
                failures += 1;
                messages.push(format!("{} - {}", script_name, format!("failure - {}", s).red()));
            }
            TestResult::Error(s) => {
                failures += 1;

                messages.push(format!("{} - {}", script_name, format!("error - {}", s).yellow()));
            }
        };
    }

    println!("{} | {}", format!("{} successes", successes).green(), format!("{} failures", failures).red());
    for m in messages {
        println!("{}", m);
    }
}

enum TestResult {
    Success,
    Failure(String),
    Error(String),
}

fn execute_script(script_path: &String, create_dump_files: bool) -> TestResult {
    let mut data = SimpleGarnishData::new();

    let mut dump_path = PathBuf::from("./tmp").join(script_path);
    if create_dump_files {
        dump_path.set_extension("");
        match fs::create_dir_all(&dump_path) {
            Ok(_) => {}
            Err(e) => error!("failed to create dump directories: {}", e),
        }
    }
    match read_to_string(PathBuf::from(&script_path)).or_else(|e| Err(format!("{}", e))).and_then(|file| {
        lex(&file)
            .or_else(|e| Err(format!("{}", e)))
            .and_then(|tokens| {
                if create_dump_files {
                    let output = tokens
                        .iter()
                        .map(|t| format!("{:?} - \"{}\"", t.get_token_type(), escape_invisible_chars(t.get_text())))
                        .collect::<Vec<String>>()
                        .join("\n");
                    let output_path = dump_path.join("tokens.txt");
                    match fs::write(output_path, output).or_else(|e| Err(format!("{}", e))) {
                        Ok(_) => {}
                        Err(e) => println!("error writing token dump: {}", e),
                    }
                }
                parse(&tokens).or_else(|e| Err(format!("{}", e)))
            })
            .and_then(|parse_result| {
                match parse_result.get_node(parse_result.get_root()) {
                    None => println!("Could not dump parse tree"),
                    Some(root) => {
                        if create_dump_files {
                            let dump = dump_parse_tree(root, parse_result.get_nodes());
                            let output_path = dump_path.join("tree.txt");
                            match fs::write(output_path, dump).or_else(|e| Err(format!("{}", e))) {
                                Ok(_) => {}
                                Err(e) => println!("error writing parse tree dump: {}", e),
                            }
                        }
                    }
                }

                build(parse_result.get_root(), parse_result.get_nodes().clone(), &mut data).or_else(|e| Err(format!("{}", e)))
            })
            .or_else(|e| Err(format!("{}", e)))
    }) {
        Err(e) => TestResult::Error(format!("({}): {}", &script_path, e)),
        Ok(_) => {
            if create_dump_files {
                let instruction_output = data
                    .get_instructions()
                    .iter()
                    .enumerate()
                    .map(|(instruction_addr, instruction)| {
                        let jump_index = data.get_jump_points().iter().enumerate().find(|(_, i)| **i == instruction_addr);
                        format!(
                            "{:03} | {} | {:?} - {}",
                            instruction_addr,
                            match jump_index {
                                Some((index, _)) => format!("{:03}", index),
                                None => "   ".to_string(),
                            },
                            instruction.instruction,
                            match instruction.data {
                                Some(index) => match instruction.instruction {
                                    Instruction::Put =>  data.get_data().display_for_item(index),
                                    _ => format!("{:?}", index),
                                }
                                None => "".to_string(),
                            }
                        )
                    })
                    .collect::<Vec<String>>()
                    .join("\n");

                let output_path = dump_path.join("instructions.txt");
                match fs::write(output_path, instruction_output).or_else(|e| Err(format!("{}", e))) {
                    Ok(_) => {}
                    Err(e) => println!("error writing parse tree dump: {}", e),
                }
            }

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

            let mut instruction_count = 0;

            loop {
                match execute_current_instruction(&mut data, Some(&mut context)) {
                    Err(e) => {
                        return TestResult::Error(format!("({}): {}", &script_path, e));
                    }
                    Ok(data) => match data.get_state() {
                        SimpleRuntimeState::Running => (),
                        SimpleRuntimeState::End => break,
                    },
                }

                instruction_count += 1;
                if instruction_count > 100000 {
                    return TestResult::Error(String::from("Reached executed instruction limit."));
                }
            }

            let result = match data.get_current_value().and_then(|i| data.get_data().get(i)) {
                Some(value) => value,
                None => {
                    return TestResult::Error(format!("({}) No current value after execution", &script_path));
                }
            };

            let (left, right) = match result {
                SimpleData::Pair(left_index, right_index) => (*left_index, *right_index),
                t => return TestResult::Error(format!("expected a Pair value, got {:?}", t)),
            };

            ops::put(&mut data, left).unwrap();
            ops::put(&mut data, right).unwrap();
            ops::equal(&mut data).unwrap();
            ops::push_value(&mut data).unwrap();

            match data.get_current_value().and_then(|i| data.get_data().get(i)) {
                Some(SimpleData::True) => TestResult::Success,
                Some(SimpleData::False) => TestResult::Failure(format!("[{} = {}]", data.get_data().display_for_item(left), data.get_data().display_for_item(right))),
                Some(v) => TestResult::Error(format!("Got non-boolean result after comparison, got {:?}", v.display_simple())),
                None => TestResult::Error(String::from("No current value after comparison")),
            }
        }
    }
}

fn dump_parse_tree(tree: &ParseNode, nodes: &Vec<ParseNode>) -> String {
    let mut stack = vec![(0, tree)];
    let mut lines = vec![];

    while let Some((nesting, node)) = stack.pop() {
        lines.push(format!(
            "{}{:?} {}",
            "\t".repeat(nesting),
            node.get_definition(),
            escape_invisible_chars(node.get_lex_token().get_text())
        ));

        match node.get_left().and_then(|index| nodes.get(index)) {
            None => (),
            Some(node) => {
                stack.push((nesting + 1, node));
            }
        }

        match node.get_right().and_then(|index| nodes.get(index)) {
            None => (),
            Some(node) => {
                stack.push((nesting + 1, node));
            }
        }
    }

    lines.join("\n")
}

fn escape_invisible_chars(input: &str) -> String {
    let mut output = String::new();
    for c in input.chars() {
        match c {
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\\' => output.push_str("\\\\"), // Escape backslashes themselves
            '"' => output.push_str("\\\""),  // Escape double quotes (if needed for string literals)
            '\x00'..='\x1F' => {
                // Control characters (ASCII 0-31)
                output.push_str(&format!("\\x{:02x}", c as u8));
            }
            _ => output.push(c),
        }
    }
    output
}
