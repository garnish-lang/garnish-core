use garnish_lang_runtime::*;

fn execute_all_instructions(runtime: &mut GarnishLangRuntime<SimpleRuntimeData>) {
    loop {
        match runtime.execute_current_instruction::<EmptyContext>(None) {
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
    let mut runtime = GarnishLangRuntime::simple();

    runtime.add_data(ExpressionData::integer(100)).unwrap();
    runtime.add_data(ExpressionData::integer(200)).unwrap();
    runtime.add_data(ExpressionData::integer(300)).unwrap();
    runtime.add_data(ExpressionData::expression(0)).unwrap();
    runtime.end_constant_data().unwrap();

    runtime.add_input_reference(2).unwrap();

    runtime.add_instruction(Instruction::Put, Some(1)).unwrap();
    runtime.add_instruction(Instruction::Put, Some(2)).unwrap();
    runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
    runtime.add_instruction(Instruction::EndExpression, None).unwrap();

    runtime.add_instruction(Instruction::Put, Some(3)).unwrap();
    runtime.add_instruction(Instruction::Put, Some(4)).unwrap();
    runtime.add_instruction(Instruction::EmptyApply, None).unwrap();
    runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
    runtime.add_instruction(Instruction::EndExpression, None).unwrap();

    runtime.add_expression(1).unwrap();

    runtime.set_instruction_cursor(5).unwrap();

    execute_all_instructions(&mut runtime);

    assert_eq!(
        runtime
            .get_data_pool()
            .get_integer(runtime.get_data_pool().get_result().unwrap())
            .unwrap(),
        600
    )
}

#[test]
fn value_before_jump() {
    let mut runtime = GarnishLangRuntime::simple();

    runtime.add_data(ExpressionData::integer(100)).unwrap();
    runtime.add_data(ExpressionData::integer(200)).unwrap();
    runtime.add_data(ExpressionData::integer(300)).unwrap();
    runtime.end_constant_data().unwrap();

    runtime.add_instruction(Instruction::Put, Some(1)).unwrap(); // 1
    runtime.add_instruction(Instruction::JumpTo, Some(0)).unwrap();

    // 3
    runtime.add_instruction(Instruction::Put, Some(1)).unwrap();
    runtime.add_instruction(Instruction::Put, Some(3)).unwrap();
    runtime.add_instruction(Instruction::Put, Some(3)).unwrap();
    runtime.add_instruction(Instruction::EqualityComparison, None).unwrap();
    runtime.add_instruction(Instruction::JumpIfTrue, Some(1)).unwrap();

    // 8
    runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
    runtime.add_instruction(Instruction::EndExpression, None).unwrap();

    runtime.add_expression(8).unwrap();
    runtime.add_expression(1).unwrap();

    runtime.set_instruction_cursor(3).unwrap();

    execute_all_instructions(&mut runtime);

    assert_eq!(
        runtime
            .get_data_pool()
            .get_integer(runtime.get_data_pool().get_result().unwrap())
            .unwrap(),
        200
    );
}

#[test]
fn pair_with_pair() {
    let mut runtime = GarnishLangRuntime::simple();

    runtime.add_data(ExpressionData::integer(100)).unwrap();
    runtime.add_data(ExpressionData::integer(200)).unwrap();
    runtime.add_data(ExpressionData::integer(300)).unwrap();
    runtime.end_constant_data().unwrap();

    // 1
    runtime.add_instruction(Instruction::Put, Some(2)).unwrap();
    runtime.add_instruction(Instruction::Put, Some(3)).unwrap();
    runtime.add_instruction(Instruction::MakePair, None).unwrap();
    runtime.add_instruction(Instruction::Put, Some(1)).unwrap();
    runtime.add_instruction(Instruction::MakePair, None).unwrap();
    runtime.add_instruction(Instruction::EndExpression, None).unwrap();

    runtime.set_instruction_cursor(1).unwrap();

    execute_all_instructions(&mut runtime);

    let pool = runtime.get_data_pool();

    let (first_left, first_right) = pool.get_pair(pool.get_result().unwrap()).unwrap();
    let (second_left, second_right) = pool.get_pair(first_left).unwrap();

    assert_eq!(pool.get_integer(first_right).unwrap(), 100);
    assert_eq!(pool.get_integer(second_left).unwrap(), 200);
    assert_eq!(pool.get_integer(second_right).unwrap(), 300);
}

#[test]
fn add_5_loop() {
    // 5 ~> {
    //     $ + 5 @ get number and add 5 to make next element of new list

    //     ? == 25 ?> ? |> ^~ ?
    // }

    let mut runtime = GarnishLangRuntime::simple();

    runtime.add_data(ExpressionData::integer(5)).unwrap();
    runtime.add_data(ExpressionData::integer(25)).unwrap();
    runtime.add_data(ExpressionData::expression(0)).unwrap();
    runtime.end_constant_data().unwrap();

    // 1 - subexpression
    runtime.add_instruction(Instruction::PutInput, None).unwrap();
    runtime.add_instruction(Instruction::Put, Some(1)).unwrap();
    runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
    runtime.add_instruction(Instruction::PushResult, None).unwrap();

    // 5
    runtime.add_instruction(Instruction::PutResult, None).unwrap();
    runtime.add_instruction(Instruction::Put, Some(2)).unwrap();
    runtime.add_instruction(Instruction::EqualityComparison, None).unwrap();
    runtime.add_instruction(Instruction::JumpIfFalse, Some(2)).unwrap();
    runtime.add_instruction(Instruction::PutResult, None).unwrap();
    runtime.add_instruction(Instruction::JumpTo, Some(1)).unwrap();

    // 11
    runtime.add_instruction(Instruction::PutResult, None).unwrap();
    runtime.add_instruction(Instruction::Reapply, Some(0)).unwrap();

    // 13
    runtime.add_instruction(Instruction::EndExpression, None).unwrap();

    // 14 - main
    runtime.add_instruction(Instruction::Put, Some(3)).unwrap();
    runtime.add_instruction(Instruction::Put, Some(1)).unwrap();
    runtime.add_instruction(Instruction::Apply, None).unwrap();
    runtime.add_instruction(Instruction::EndExpression, None).unwrap();

    runtime.add_expression(1).unwrap();
    runtime.add_expression(13).unwrap();
    runtime.add_expression(11).unwrap();

    runtime.set_instruction_cursor(14).unwrap();

    execute_all_instructions(&mut runtime);

    assert_eq!(
        runtime
            .get_data_pool()
            .get_integer(runtime.get_data_pool().get_result().unwrap())
            .unwrap(),
        25
    );
}
