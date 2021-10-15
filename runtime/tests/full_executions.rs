use garnish_lang_runtime::*;

#[test]
fn adding_numbers_with_sub_expression() {
    let mut runtime = GarnishLangRuntime::new();

    runtime.add_data(ExpressionData::integer(100)).unwrap();
    runtime.add_data(ExpressionData::integer(200)).unwrap();
    runtime.add_data(ExpressionData::integer(300)).unwrap();

    runtime.add_input_reference(2).unwrap();

    runtime.add_instruction(Instruction::Put, Some(0)).unwrap();
    runtime.add_instruction(Instruction::Put, Some(1)).unwrap();
    runtime
        .add_instruction(Instruction::PerformAddition, None)
        .unwrap();
    runtime
        .add_instruction(Instruction::EndExpression, None)
        .unwrap();

    runtime.add_instruction(Instruction::Put, Some(2)).unwrap();
    runtime
        .add_instruction(Instruction::ExecuteExpression, Some(1))
        .unwrap();
    runtime
        .add_instruction(Instruction::PerformAddition, None)
        .unwrap();
    runtime
        .add_instruction(Instruction::EndExpression, None)
        .unwrap();

    runtime.set_instruction_cursor(4).unwrap();

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

    assert_eq!(runtime.get_result(0).unwrap().as_integer().unwrap(), 600)
}

#[test]
fn conditionals_and_inputs() {
    let mut runtime = GarnishLangRuntime::new();

    runtime.add_data(ExpressionData::integer(100)).unwrap();
    runtime.add_data(ExpressionData::integer(200)).unwrap();
    runtime.add_data(ExpressionData::integer(300)).unwrap();
    runtime
        .add_data(ExpressionData::symbol(&"false".to_string(), 0))
        .unwrap();

    runtime.add_input_reference(3).unwrap();

    runtime.add_instruction(Instruction::Put, Some(0)).unwrap();
    runtime.add_instruction(Instruction::Put, Some(1)).unwrap();
    runtime
        .add_instruction(Instruction::PerformAddition, None)
        .unwrap();
    runtime
        .add_instruction(Instruction::EndExpression, None)
        .unwrap();

    runtime.add_instruction(Instruction::Put, Some(1)).unwrap(); // 5
    runtime.add_instruction(Instruction::Put, Some(2)).unwrap();
    runtime
        .add_instruction(Instruction::PerformAddition, None)
        .unwrap();
    runtime
        .add_instruction(Instruction::EndExpression, None)
        .unwrap();

    runtime.add_instruction(Instruction::Put, Some(0)).unwrap(); // 9
    runtime.add_instruction(Instruction::Put, Some(3)).unwrap();
    runtime
        .add_instruction(Instruction::PerformAddition, None)
        .unwrap();
    runtime
        .add_instruction(Instruction::EndExpression, None)
        .unwrap();

    // 13
    runtime
        .add_instruction(Instruction::ExecuteIfTrue, Some(9))
        .unwrap();
    runtime
        .add_instruction(Instruction::ExecuteIfFalse, Some(1))
        .unwrap();

    runtime
        .add_instruction(Instruction::ExecuteExpression, Some(5))
        .unwrap();
    runtime
        .add_instruction(Instruction::PerformAddition, None)
        .unwrap();
    runtime
        .add_instruction(Instruction::EndExpression, None)
        .unwrap();

    runtime.set_instruction_cursor(13).unwrap();

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

    assert_eq!(runtime.get_result(0).unwrap().as_integer().unwrap(), 800)
}
