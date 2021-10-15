use garnish_lang_runtime::*;

fn execute_all_instructions(runtime: &mut GarnishLangRuntime) {
    loop {
        match runtime.execute_current_instruction() {
            Err(e) => {
                println!("{:?}", e);
                break;
            }
            Ok(data) => match data.get_state() {
                GarnishLangRuntimeState::Running => (),
                GarnishLangRuntimeState::End => break,
            },
        }
    }
}

#[test]
fn adding_numbers_with_sub_expression() {
    let mut runtime = GarnishLangRuntime::new();

    runtime.add_data(ExpressionData::integer(100)).unwrap();
    runtime.add_data(ExpressionData::integer(200)).unwrap();
    runtime.add_data(ExpressionData::integer(300)).unwrap();

    runtime.add_input_reference(2).unwrap();

    runtime.add_instruction(Instruction::Put, Some(0)).unwrap();
    runtime.add_instruction(Instruction::Put, Some(1)).unwrap();
    runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
    runtime.add_instruction(Instruction::EndExpression, None).unwrap();

    runtime.add_instruction(Instruction::Put, Some(2)).unwrap();
    runtime.add_instruction(Instruction::ExecuteExpression, Some(1)).unwrap();
    runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
    runtime.add_instruction(Instruction::EndExpression, None).unwrap();

    runtime.set_instruction_cursor(4).unwrap();

    execute_all_instructions(&mut runtime);

    assert_eq!(runtime.get_result(0).unwrap().as_integer().unwrap(), 600)
}

#[test]
fn conditionals_and_inputs() {
    let mut runtime = GarnishLangRuntime::new();

    runtime.add_data(ExpressionData::integer(100)).unwrap();
    runtime.add_data(ExpressionData::integer(200)).unwrap();
    runtime.add_data(ExpressionData::integer(300)).unwrap();

    runtime.add_instruction(Instruction::Put, Some(0)).unwrap();
    runtime.add_instruction(Instruction::Put, Some(1)).unwrap();
    runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
    runtime.add_instruction(Instruction::EndExpression, None).unwrap();

    runtime.add_instruction(Instruction::Put, Some(1)).unwrap(); // 5
    runtime.add_instruction(Instruction::Put, Some(2)).unwrap();
    runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
    runtime.add_instruction(Instruction::EndExpression, None).unwrap();

    runtime.add_instruction(Instruction::Put, Some(0)).unwrap(); // 9
    runtime.add_instruction(Instruction::Put, Some(2)).unwrap();
    runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
    runtime.add_instruction(Instruction::EndExpression, None).unwrap();

    // 13
    runtime.add_instruction(Instruction::PutInput, None).unwrap();
    runtime.add_instruction(Instruction::ExecuteIfTrue, Some(9)).unwrap();
    runtime.add_instruction(Instruction::ExecuteIfFalse, Some(1)).unwrap();

    runtime.add_instruction(Instruction::ExecuteExpression, Some(5)).unwrap();
    runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
    runtime.add_instruction(Instruction::EndExpression, None).unwrap();

    runtime.set_instruction_cursor(13).unwrap();

    let inputs_expected_result = [
        (ExpressionData::symbol(&"false".to_string(), 0), 800),
        (ExpressionData::symbol(&"true".to_string(), 1), 900),
    ];

    for (input, expected) in inputs_expected_result {
        let addr = runtime.add_data(input.clone()).unwrap();
        runtime.add_input_reference(addr).unwrap();

        execute_all_instructions(&mut runtime);

        assert_eq!(runtime.get_result(0).unwrap().as_integer().unwrap(), expected);

        runtime.remove_data(addr).unwrap();
        runtime.set_instruction_cursor(13).unwrap();
    }
}
