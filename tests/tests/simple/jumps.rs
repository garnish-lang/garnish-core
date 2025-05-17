#[cfg(test)]
mod tests {

    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang_traits::{GarnishData, GarnishRuntime, Instruction};

    #[test]
    fn end_expression_no_existing_value_no_path() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(1)).unwrap();
        let i1 = runtime.get_data_mut().push_instruction(Instruction::EndExpression, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(1)).unwrap();

        runtime.get_data_mut().set_instruction_cursor(i1).unwrap();
        runtime.get_data_mut().push_register(int1).unwrap();

        let err = runtime.end_expression().err().unwrap();

        assert_eq!(err.get_message(), &"No inputs available to update during end expression operation.".to_string())
    }

    #[test]
    fn end_expression_with_existing_value_and_no_path() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let int2 = runtime.get_data_mut().add_number(20.into()).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(1)).unwrap();
        let i1 = runtime.get_data_mut().push_instruction(Instruction::EndExpression, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(1)).unwrap();

        runtime.get_data_mut().set_instruction_cursor(i1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.get_data_mut().push_value_stack(int1).unwrap();

        let next = runtime.end_expression().unwrap();

        assert_eq!(runtime.get_data().get_registers().len(), 0);
        assert_eq!(runtime.get_data().get_value_stack_len(), 1);

        let i = runtime.get_data_mut().get_current_value().unwrap();
        assert_eq!(
            next.unwrap(),
            runtime.get_data_mut().get_instruction_len()
        );
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 20.into());
    }

    #[test]
    fn end_expression_with_path() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::EndExpression, Some(0)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::EmptyApply, None).unwrap();

        runtime.get_data_mut().push_jump_path(4).unwrap();
        runtime.get_data_mut().push_register(1).unwrap();
        runtime.get_data_mut().set_instruction_cursor(2).unwrap();

        runtime.get_data_mut().push_value_stack(1).unwrap();
        runtime.get_data_mut().push_value_stack(2).unwrap();

        let next = runtime.end_expression().unwrap();

        assert_eq!(next.unwrap(), 4);
        assert_eq!(runtime.get_data().get_register_len(), 1);
        assert_eq!(runtime.get_data().get_value_stack_len(), 1);
        assert_eq!(runtime.get_data().get_value(0).unwrap(), 1);
    }

    #[test]
    fn jump() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().push_instruction(Instruction::JumpIfFalse, Some(3)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::JumpTo, Some(0)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        let i1 = runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();

        runtime.get_data_mut().push_jump_point(i1).unwrap();

        let next = runtime.jump(0).unwrap();

        assert_eq!(next.unwrap(), i1);
    }

    #[test]
    fn jump_if_true_no_ref_is_err() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().add_true().unwrap();
        runtime.get_data_mut().push_instruction(Instruction::JumpIfTrue, Some(3)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();

        runtime.get_data_mut().push_jump_point(3).unwrap();

        let result = runtime.jump_if_true(0);

        assert!(result.is_err());
    }

    #[test]
    fn jump_if_false_no_ref_is_error() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().add_true().unwrap();
        runtime.get_data_mut().push_instruction(Instruction::JumpIfFalse, Some(3)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();

        runtime.get_data_mut().push_jump_point(3).unwrap();

        let result = runtime.jump_if_true(0);

        assert!(result.is_err());
    }

    #[test]
    fn jump_if_true_when_true() {
        let mut runtime = create_simple_runtime();

        let ta = runtime.get_data_mut().add_true().unwrap();
        let i1 = runtime.get_data_mut().push_instruction(Instruction::JumpIfTrue, Some(0)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        let i2 = runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();

        runtime.get_data_mut().push_jump_point(i2).unwrap();

        runtime.get_data_mut().set_instruction_cursor(i1).unwrap();

        runtime.get_data_mut().push_register(ta).unwrap();

        let next = runtime.jump_if_true(0).unwrap();

        assert_eq!(runtime.get_data_mut().get_register_len(), 0);
        assert_eq!(runtime.get_data_mut().get_data_len(), 3);
        assert_eq!(next.unwrap(), i2);
    }

    #[test]
    fn jump_if_true_when_unit() {
        let mut runtime = create_simple_runtime();

        let ua = runtime.get_data_mut().add_unit().unwrap();
        runtime.get_data_mut().push_instruction(Instruction::JumpIfTrue, Some(0)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();

        runtime.get_data_mut().push_jump_point(3).unwrap();
        runtime.get_data_mut().set_instruction_cursor(1).unwrap();
        runtime.get_data_mut().push_register(ua).unwrap();

        let next = runtime.jump_if_true(0).unwrap();

        assert_eq!(runtime.get_data_mut().get_register_len(), 0);
        assert_eq!(runtime.get_data_mut().get_data_len(), 3);
        assert_eq!(next, None);
    }

    #[test]
    fn jump_if_true_when_false() {
        let mut runtime = create_simple_runtime();

        let fa = runtime.get_data_mut().add_false().unwrap();
        runtime.get_data_mut().push_instruction(Instruction::JumpIfTrue, Some(0)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();

        runtime.get_data_mut().push_jump_point(3).unwrap();
        runtime.get_data_mut().set_instruction_cursor(1).unwrap();
        runtime.get_data_mut().push_register(fa).unwrap();

        let next = runtime.jump_if_true(0).unwrap();

        assert_eq!(runtime.get_data_mut().get_register_len(), 0);
        assert_eq!(runtime.get_data_mut().get_data_len(), 3);
        assert_eq!(next, None);
    }

    #[test]
    fn jump_if_false_when_true() {
        let mut runtime = create_simple_runtime();

        let ta = runtime.get_data_mut().add_true().unwrap();
        runtime.get_data_mut().push_instruction(Instruction::JumpIfFalse, Some(0)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();

        runtime.get_data_mut().push_jump_point(3).unwrap();
        runtime.get_data_mut().set_instruction_cursor(1).unwrap();
        runtime.get_data_mut().push_register(ta).unwrap();

        let next = runtime.jump_if_false(0).unwrap();

        assert_eq!(runtime.get_data_mut().get_register_len(), 0);
        assert_eq!(runtime.get_data_mut().get_data_len(), 3);
        assert_eq!(next, None);
    }

    #[test]
    fn jump_if_false_when_unit() {
        let mut runtime = create_simple_runtime();

        let ua = runtime.get_data_mut().add_unit().unwrap();
        let i1 = runtime.get_data_mut().push_instruction(Instruction::JumpIfFalse, Some(0)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        let i2 = runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();

        runtime.get_data_mut().push_jump_point(i2).unwrap();
        runtime.get_data_mut().set_instruction_cursor(i1).unwrap();
        runtime.get_data_mut().push_register(ua).unwrap();

        let next = runtime.jump_if_false(0).unwrap();

        assert_eq!(runtime.get_data_mut().get_register_len(), 0);
        assert_eq!(runtime.get_data_mut().get_data_len(), 3);
        assert_eq!(next.unwrap(), i2);
    }

    #[test]
    fn jump_if_false_when_false() {
        let mut runtime = create_simple_runtime();

        let fa = runtime.get_data_mut().add_false().unwrap();
        let i1 = runtime.get_data_mut().push_instruction(Instruction::JumpIfFalse, Some(0)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        let i2 = runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();

        runtime.get_data_mut().push_jump_point(i2).unwrap();
        runtime.get_data_mut().set_instruction_cursor(i1).unwrap();
        runtime.get_data_mut().push_register(fa).unwrap();

        let next = runtime.jump_if_false(0).unwrap();

        assert_eq!(runtime.get_data_mut().get_register_len(), 0);
        assert_eq!(runtime.get_data_mut().get_data_len(), 3);
        assert_eq!(next.unwrap(), i2);
    }
}
