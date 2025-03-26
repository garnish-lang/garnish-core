mod context;

use garnish_lang_compiler::build::build_with_data;
use garnish_lang_compiler::lex::lex;
use garnish_lang_compiler::parse::{parse};
use garnish_lang_simple_data::{SimpleData, SimpleGarnishData};
use log::{error, info};
use std::fs::read_to_string;
use colored::Colorize;
use garnish_lang_runtime::{SimpleGarnishRuntime, SimpleRuntimeState};
use garnish_lang_traits::GarnishData;
use crate::context::TestingContext;

fn main() {
    env_logger::init();

    let mut data = SimpleGarnishData::new();
    match read_to_string("tests/scripts/addition.garnish")
        .or_else(|e| Err(format!("{}", e)))
        .and_then(|file| {
            lex(&file)
                .or_else(|e| Err(format!("{}", e)))
                .and_then(|tokens| parse(&tokens)
                    .or_else(|e| Err(format!("{}", e))))
                .and_then(|parse_result| {
                    build_with_data(parse_result.get_root(), parse_result.get_nodes().clone(), &mut data)
                        .or_else(|e| Err(format!("{}", e)))
                })
                .or_else(|e| Err(format!("{}", e)))
        }) {
        Err(e) => error!("Test failed: {}", e),
        Ok(_) => {
            let start = match data.get_jump_points().get(0) {
                Some(jump_points) => *jump_points,
                None => {
                    error!("Test failed: No jump point");
                    return;
                },
            };

            match data.set_instruction_cursor(start) {
                Ok(_) => (),
                Err(e) => {
                    error!("Test failed: {}", e);
                    return;
                },
            }

            match data.add_unit().and_then(|e| data.push_value_stack(e)) {
                Ok(_) => (),
                Err(e) => {
                    error!("Test failed: {}", e);
                    return;
                },
            }

            let mut context = TestingContext::default();
            let mut runtime = SimpleGarnishRuntime::new(data);

            loop {
                match runtime.execute_current_instruction(Some(&mut context)) {
                    Err(e) => {
                        error!("Test failed: {}", e);
                        return;
                    }
                    Ok(data) => match data.get_state() {
                        SimpleRuntimeState::Running => (),
                        SimpleRuntimeState::End => break,
                    },
                }
            }

            let data = runtime.get_data_owned();

            let result = match data.get_current_value()
                .and_then(|i| data.get_data().get(i)) {
                Some(value) => value,
                None => {
                    error!("Test failed: No current value after execution");
                    return;
                }
            };

            match result {
                SimpleData::True => println!("{}: {}", "addition", "passed".green()),
                SimpleData::False => println!("{}: {}", "addition", "failed".red()),
                t => println!("{}{}", "addition", format!("Test failed: Expected a True or False value, got {:?}", t).yellow())
            }
        }
    }
}
