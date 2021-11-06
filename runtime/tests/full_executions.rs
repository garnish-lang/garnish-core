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
    runtime.add_instruction(Instruction::Jump, Some(16)).unwrap();

    runtime.add_instruction(Instruction::Put, Some(1)).unwrap(); // 5
    runtime.add_instruction(Instruction::Put, Some(2)).unwrap();
    runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
    runtime.add_instruction(Instruction::EndExpression, None).unwrap();

    runtime.add_instruction(Instruction::Put, Some(0)).unwrap(); // 9
    runtime.add_instruction(Instruction::Put, Some(2)).unwrap();
    runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
    runtime.add_instruction(Instruction::Jump, Some(16)).unwrap();

    // 13
    runtime.add_instruction(Instruction::PutInput, None).unwrap();
    runtime.add_instruction(Instruction::JumpIfTrue, Some(9)).unwrap();
    runtime.add_instruction(Instruction::JumpIfFalse, Some(1)).unwrap();

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
        runtime.clear_results().unwrap();
    }
}

#[test]
fn multiple_conditions() {
    let mut runtime = GarnishLangRuntime::new();

    runtime.add_data(ExpressionData::integer(100)).unwrap();
    runtime.add_data(ExpressionData::integer(200)).unwrap();
    runtime.add_data(ExpressionData::integer(300)).unwrap();
    runtime.add_data(ExpressionData::integer(1)).unwrap();
    runtime.add_data(ExpressionData::integer(2)).unwrap();

    runtime.add_instruction(Instruction::Put, Some(0)).unwrap(); // 1
    runtime.add_instruction(Instruction::Jump, Some(16)).unwrap();

    runtime.add_instruction(Instruction::Put, Some(1)).unwrap(); // 3
    runtime.add_instruction(Instruction::Jump, Some(16)).unwrap();

    runtime.add_instruction(Instruction::Put, Some(2)).unwrap(); // 5
    runtime.add_instruction(Instruction::Jump, Some(16)).unwrap();

    // 7
    runtime.add_instruction(Instruction::PutInput, None).unwrap();
    runtime.add_instruction(Instruction::Put, Some(3)).unwrap();
    runtime.add_instruction(Instruction::EqualityComparison, None).unwrap();

    runtime.add_instruction(Instruction::JumpIfTrue, Some(1)).unwrap();

    runtime.add_instruction(Instruction::PutInput, None).unwrap();
    runtime.add_instruction(Instruction::Put, Some(4)).unwrap();
    runtime.add_instruction(Instruction::EqualityComparison, None).unwrap();

    runtime.add_instruction(Instruction::JumpIfFalse, Some(5)).unwrap();
    runtime.add_instruction(Instruction::JumpIfTrue, Some(3)).unwrap();

    runtime.add_instruction(Instruction::EndExpression, None).unwrap();

    let inputs_expected_result = [
        (ExpressionData::integer(1), 100),
        (ExpressionData::integer(2), 200),
        (ExpressionData::integer(3), 300),
    ];

    for (input, expected) in inputs_expected_result {
        runtime.set_instruction_cursor(7).unwrap();

        let addr = runtime.add_data(input.clone()).unwrap();
        runtime.add_input_reference(addr).unwrap();

        execute_all_instructions(&mut runtime);

        assert_eq!(runtime.get_result(0).unwrap().as_integer().unwrap(), expected);

        runtime.remove_data(addr).unwrap();
        runtime.clear_results().unwrap();
    }
}

#[test]
fn value_before_jump() {
    let mut runtime = GarnishLangRuntime::new();

    runtime.add_data(ExpressionData::integer(100)).unwrap();
    runtime.add_data(ExpressionData::integer(200)).unwrap();
    runtime.add_data(ExpressionData::integer(300)).unwrap();

    runtime.add_instruction(Instruction::Put, Some(0)).unwrap(); // 1
    runtime.add_instruction(Instruction::Jump, Some(8)).unwrap();

    // 3
    runtime.add_instruction(Instruction::Put, Some(0)).unwrap();
    runtime.add_instruction(Instruction::Put, Some(2)).unwrap();
    runtime.add_instruction(Instruction::Put, Some(2)).unwrap();
    runtime.add_instruction(Instruction::EqualityComparison, None).unwrap();
    runtime.add_instruction(Instruction::JumpIfTrue, Some(1)).unwrap();

    // 8
    runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
    runtime.add_instruction(Instruction::EndExpression, None).unwrap();

    runtime.set_instruction_cursor(3).unwrap();

    execute_all_instructions(&mut runtime);

    assert_eq!(runtime.get_result(0).unwrap().as_integer().unwrap(), 200);
}

#[test]
fn pair_with_pair() {
    let mut runtime = GarnishLangRuntime::new();

    runtime.add_data(ExpressionData::integer(100)).unwrap();
    runtime.add_data(ExpressionData::integer(200)).unwrap();
    runtime.add_data(ExpressionData::integer(300)).unwrap();

    // 1
    runtime.add_instruction(Instruction::Put, Some(1)).unwrap();
    runtime.add_instruction(Instruction::Put, Some(2)).unwrap();
    runtime.add_instruction(Instruction::MakePair, None).unwrap();
    runtime.add_instruction(Instruction::Put, Some(0)).unwrap();
    runtime.add_instruction(Instruction::MakePair, None).unwrap();
    runtime.add_instruction(Instruction::EndExpression, None).unwrap();

    execute_all_instructions(&mut runtime);

    let (first_left, first_right) = runtime.get_result(0).unwrap().as_pair().unwrap();
    let (second_left, second_right) = runtime.get_data(first_left).unwrap().as_pair().unwrap();

    assert_eq!(runtime.get_data(first_right).unwrap().as_integer().unwrap(), 100);
    assert_eq!(runtime.get_data(second_left).unwrap().as_integer().unwrap(), 200);
    assert_eq!(runtime.get_data(second_right).unwrap().as_integer().unwrap(), 300);
}

#[test]
fn add_5_loop() {
    // 5 ~> {
    //     $ + 5 @ get number and add 5 to make next element of new list

    //     ? == 25 => ? !> ^~ ?
    // }

    let mut runtime = GarnishLangRuntime::new();

    runtime.add_data(ExpressionData::integer(5)).unwrap();
    runtime.add_data(ExpressionData::integer(25)).unwrap();
    runtime.add_data(ExpressionData::expression(0)).unwrap();

    // 1 - subexpression
    runtime.add_instruction(Instruction::PutInput, None).unwrap();
    runtime.add_instruction(Instruction::Put, Some(0)).unwrap();
    runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
    runtime.add_instruction(Instruction::PushResult, None).unwrap();

    // 5
    runtime.add_instruction(Instruction::PutResult, None).unwrap();
    runtime.add_instruction(Instruction::Put, Some(1)).unwrap();
    runtime.add_instruction(Instruction::EqualityComparison, None).unwrap();
    runtime.add_instruction(Instruction::JumpIfFalse, Some(11)).unwrap();
    runtime.add_instruction(Instruction::PutResult, None).unwrap();
    runtime.add_instruction(Instruction::Jump, Some(13)).unwrap();

    // 11
    runtime.add_instruction(Instruction::PutResult, None).unwrap();
    runtime.add_instruction(Instruction::Reapply, Some(1)).unwrap();

    // 13
    runtime.add_instruction(Instruction::EndExpression, None).unwrap();

    // 14 - main
    runtime.add_instruction(Instruction::Put, Some(2)).unwrap();
    runtime.add_instruction(Instruction::Put, Some(0)).unwrap();
    runtime.add_instruction(Instruction::Apply, None).unwrap();
    runtime.add_instruction(Instruction::EndExpression, None).unwrap();

    runtime.add_expression(1).unwrap();

    runtime.set_instruction_cursor(14).unwrap();

    execute_all_instructions(&mut runtime);

    println!("{:#?}", runtime);

    let last_result = runtime.get_result(runtime.result_count() - 1).unwrap();

    assert_eq!(last_result.as_integer().unwrap(), 25);
}
