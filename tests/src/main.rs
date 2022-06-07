use std::fs::*;
use std::path::{Path, PathBuf};
use std::process::{exit};
use std::{env, io};

use colored::Colorize;

use garnish_data::data::SimpleData;
use garnish_data::SimpleRuntimeData;
use garnish_lang_compiler::{build_with_data, lex, parse};
use garnish_lang_runtime::runtime_impls::SimpleGarnishRuntime;
use garnish_traits::{GarnishLangRuntimeData, GarnishRuntime};

use crate::test_annotation::{execute_tests, extract_tests, ExecutionResult, TestExtractionError};

mod test_annotation;

fn main() {
    env_logger::init();
    let mut args = env::args().skip(1);
    let test_directory = match args.next() {
        Some(dir) => dir,
        None => {
            println!("Please provide test file or directory.");
            exit(1);
        }
    };

    let dir = Path::new(&test_directory);

    let mut paths = vec![];

    match get_all_paths(dir, &mut paths) {
        Err(e) => {
            println!("Failed to read all paths.\n{}", e);
            exit(1);
        }
        Ok(_) => (),
    }

    let mut overall_status = 0;

    for path in paths {
        println!("{}", path.to_string_lossy());
        let text = match read_to_string(path) {
            Err(e) => {
                println!("Failed to read file.\n{}", e);
                overall_status = 1;
                continue;
            }
            Ok(t) => t,
        };

        let data = SimpleRuntimeData::new();
        let mut runtime = SimpleGarnishRuntime::new(data);
        let results = match run_tests(&mut runtime, &text) {
            Err(e) => {
                println!("Failed to run tests.\n{}", e);
                overall_status = 1;
                continue;
            }
            Ok(r) => r,
        };

        for result in results.get_results() {
            let name = match result.name() {
                None => "[No name found for test]".to_string(),
                Some(addr) => match runtime.get_data().get_raw_data(addr) {
                    None => "[No name found for test]".to_string(),
                    Some(name) => match name {
                        SimpleData::CharList(s) => s,
                        _ => "[No name found for test]".to_string(),
                    },
                },
            };

            let s = format!("{}: {}", name, match result.is_success() {
                true => "pass",
                false => "failure"
            });

            if !result.is_success() {
                overall_status = 1;
                println!("{}", s.bright_red());
                match result.error() {
                    Some(e) => {
                        println!("\t{}", e);
                    }
                    None => ()
                }
            } else {
                println!("{}", s.bright_green());
            }
        }
    }

    exit(overall_status);
}

fn run_tests(runtime: &mut SimpleGarnishRuntime<SimpleRuntimeData>, text: &String) -> Result<ExecutionResult<SimpleRuntimeData>, String> {
    let input = lex(text.as_str())?;
    let tests = extract_tests(&input)?;
    let parse_result =
        parse(tests.get_expression().clone()).or_else(|err| Err(TestExtractionError::with_tokens(err.get_message(), tests.get_expression())))?;
    let top_expression = runtime.get_data().get_jump_table_len();
    build_with_data(parse_result.get_root(), parse_result.get_nodes().clone(), runtime.get_data_mut())?;
    let results = execute_tests(runtime, &tests, Some(top_expression))?;

    Ok(results)
}

fn get_all_paths(current_dir: &Path, paths: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in read_dir(current_dir)? {
        let entry = entry?;
        if entry.path().is_dir() {
            get_all_paths(entry.path().as_path(), paths)?;
        } else if entry.path().is_file() {
            paths.push(entry.path())
        }
    }

    Ok(())
}
