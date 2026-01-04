use colored::Colorize;
use garnish_lang::compiler::build::build;
use garnish_lang::compiler::lex::lex;
use garnish_lang::compiler::parse::{ParseNode, parse};
use garnish_lang::{GarnishData, simple::{BasicGarnishData, SimpleRuntimeState, execute_current_instruction, ops}};
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
    let manual_filter: Vec<&str> = vec![
        // "access:symbol_list:number"
    ];
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

fn execute_script(script_path: &String, create_dump_files: bool) -> TestResult {
    let mut data: BasicGarnishData<()> = match BasicGarnishData::new() {
        Ok(d) => d,
        Err(e) => return TestResult::Error(format!("Failed to create BasicGarnishData: {}", e)),
    };

    let mut dump_path = PathBuf::from("./tmp").join(script_path);
    if create_dump_files {
        dump_path.set_extension("");
        match fs::create_dir_all(&dump_path) {
            Ok(_) => {}
            Err(e) => error!("failed to create dump directories: {}", e),
        }
    }
    match read_to_string(PathBuf::from(&script_path))
        .or_else(|e| Err(format!("{}", e)))
        .and_then(|file| {
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
                let blocks_output = data.dump_all_blocks();
                let output_path = dump_path.join("blocks.txt");
                match fs::write(output_path, blocks_output).or_else(|e| Err(format!("{}", e))) {
                    Ok(_) => {}
                    Err(e) => println!("error writing blocks dump: {}", e),
                }
            }

            let start = match data.get_from_jump_table(0) {
                Some(jump_point) => jump_point,
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

            let mut instruction_count = 0;

            loop {
                match execute_current_instruction(&mut data) {
                    Err(e) => {
                        return TestResult::Error(format!("({}): {}", &script_path, e));
                    }
                    Ok(runtime_info) => match runtime_info.get_state() {
                        SimpleRuntimeState::Running => (),
                        SimpleRuntimeState::End => break,
                    },
                }

                instruction_count += 1;
                if instruction_count > 100000 {
                    return TestResult::Error(String::from("Reached executed instruction limit."));
                }
            }

            if create_dump_files {
                println!("Dumping blocks after execution");
                let blocks_output = data.dump_all_blocks();
                let output_path = dump_path.join("blocks_after_execution.txt");
                match fs::write(output_path, blocks_output).or_else(|e| Err(format!("{}", e))) {
                    Ok(_) => {}
                    Err(e) => println!("error writing post-execution blocks dump: {}", e),
                }
            }

            let result_index = match data.get_current_value() {
                Some(i) => i,
                None => {
                    return TestResult::Error(format!("({}) No current value after execution", &script_path));
                }
            };

            let (left, right) = match data.get_pair(result_index) {
                Ok((l, r)) => (l, r),
                Err(e) => return TestResult::Error(format!("({}) expected a Pair value, got error: {}", &script_path, e)),
            };

            let result = ops::put(&mut data, left)
                .and_then(|_| ops::put(&mut data, right))
                .and_then(|_| ops::equal(&mut data))
                .and_then(|_| ops::push_value(&mut data));

            match result {
                Err(e) => TestResult::Error(format!("Error making comparison: {}", e)),
                Ok(_) => {
                    let comparison_result_index = match data.get_current_value() {
                        Some(i) => i,
                        None => return TestResult::Error(String::from("No current value after comparison")),
                    };
        
                    match data.get_data_type(comparison_result_index) {
                        Ok(garnish_lang::GarnishDataType::True) => TestResult::Success,
                        Ok(garnish_lang::GarnishDataType::False) => TestResult::Failure(format!(
                            "[{} = {}]",
                            data.get_string_for_data_at(left).unwrap_or_else(|_| format!("[error: {}]", left)),
                            data.get_string_for_data_at(right).unwrap_or_else(|_| format!("[error: {}]", right))
                        )),
                        Ok(t) => TestResult::Error(format!("Got non-boolean result after comparison, got {:?}", t)),
                        Err(e) => TestResult::Error(format!("Error getting data type: {}", e)),
                    }
                }
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
