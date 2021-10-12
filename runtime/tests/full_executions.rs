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
    runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
    runtime.add_instruction(Instruction::EndExpression, None).unwrap();

    runtime.add_instruction(Instruction::Put, Some(2)).unwrap();
    runtime.add_instruction(Instruction::ExecuteExpression, Some(1)).unwrap();
    runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
    runtime.add_instruction(Instruction::EndExpression, None).unwrap();

    runtime.set_instruction_cursor(4).unwrap();

    loop {
        match runtime.execute_current_instruction() {
            Err(e) => {
                println!("{}", e);
                break;
            },
            Ok(_) => match runtime.advance_instruction() {
                Err(e) => {
                    println!("{}", e);
                    break;
                }
                Ok(_) => ()
            }
        }
    }

    assert_eq!(runtime.get_result(0).unwrap().as_integer().unwrap(), 600)
}