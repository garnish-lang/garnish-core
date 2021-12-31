#[cfg(test)]
mod tests {
    use crate::{runtime::GarnishRuntime, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

    #[test]
    fn end_expression() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_integer(10).unwrap();
        runtime.push_instruction(Instruction::Put, Some(1)).unwrap();

        runtime.set_instruction_cursor(1).unwrap();
        runtime.push_register(1).unwrap();

        runtime.end_expression().unwrap();

        assert_eq!(runtime.get_instruction_cursor(), 2);
        assert_eq!(runtime.get_integer(runtime.get_current_value().unwrap()).unwrap(), 10);
    }

    #[test]
    fn end_expression_last_register_is_result() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_integer(10).unwrap();
        runtime.add_integer(20).unwrap();
        runtime.add_integer(30).unwrap();
        runtime.push_instruction(Instruction::Put, Some(1)).unwrap();

        runtime.set_instruction_cursor(1).unwrap();
        runtime.push_register(2).unwrap();

        runtime.end_expression().unwrap();

        assert_eq!(runtime.get_instruction_cursor(), 2);
        assert_eq!(runtime.get_integer(runtime.get_current_value().unwrap()).unwrap(), 20);
    }

    #[test]
    fn end_expression_with_path() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_integer(10).unwrap();
        runtime.push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.push_instruction(Instruction::EndExpression, Some(0)).unwrap();
        runtime.push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.push_instruction(Instruction::EmptyApply, None).unwrap();

        runtime.push_register(1).unwrap();
        runtime.push_jump_path(4).unwrap();
        runtime.set_instruction_cursor(2).unwrap();

        runtime.end_expression().unwrap();

        assert_eq!(runtime.get_instruction_cursor(), 4);
    }

    #[test]
    fn jump() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_symbol("false").unwrap();
        runtime.push_instruction(Instruction::JumpIfFalse, Some(3)).unwrap();
        runtime.push_instruction(Instruction::JumpTo, Some(0)).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.push_jump_point(4).unwrap();

        runtime.jump(0).unwrap();

        assert!(runtime.get_jump_path_vec().is_empty());
        assert_eq!(runtime.get_instruction_cursor(), 3);
    }

    #[test]
    fn jump_if_true_no_ref_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_true().unwrap();
        runtime.push_instruction(Instruction::JumpIfTrue, Some(3)).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.push_jump_point(3).unwrap();

        let result = runtime.jump_if_true(0);

        assert!(result.is_err());
    }

    #[test]
    fn jump_if_false_no_ref_is_error() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_true().unwrap();
        runtime.push_instruction(Instruction::JumpIfFalse, Some(3)).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.push_jump_point(3).unwrap();

        let result = runtime.jump_if_false(0);

        assert!(result.is_err());
    }

    #[test]
    fn jump_if_true_when_true() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_true().unwrap();
        runtime.push_instruction(Instruction::JumpIfTrue, Some(0)).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.push_jump_point(3).unwrap();
        runtime.set_instruction_cursor(1).unwrap();
        runtime.push_register(1).unwrap();

        runtime.jump_if_true(0).unwrap();

        assert!(runtime.get_register().is_empty());
        assert_eq!(runtime.get_data_len(), 2);
        assert_eq!(runtime.get_instruction_cursor(), 2);
    }

    #[test]
    fn jump_if_true_when_unit() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_unit().unwrap();
        runtime.push_instruction(Instruction::JumpIfTrue, Some(0)).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.push_jump_point(3).unwrap();
        runtime.set_instruction_cursor(1).unwrap();
        runtime.push_register(0).unwrap();

        runtime.jump_if_true(0).unwrap();

        assert!(runtime.get_register().is_empty());
        assert_eq!(runtime.get_data_len(), 1);
        assert_eq!(runtime.get_instruction_cursor(), 1);
    }

    #[test]
    fn jump_if_true_when_false() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_false().unwrap();
        runtime.push_instruction(Instruction::JumpIfTrue, Some(0)).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.push_jump_point(3).unwrap();
        runtime.set_instruction_cursor(1).unwrap();
        runtime.push_register(1).unwrap();

        runtime.jump_if_true(0).unwrap();

        assert!(runtime.get_register().is_empty());
        assert_eq!(runtime.get_data_len(), 2);
        assert_eq!(runtime.get_instruction_cursor(), 1);
    }

    #[test]
    fn jump_if_false_when_true() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_true().unwrap();
        runtime.push_instruction(Instruction::JumpIfFalse, Some(0)).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.push_jump_point(3).unwrap();
        runtime.set_instruction_cursor(1).unwrap();
        runtime.push_register(1).unwrap();

        runtime.jump_if_false(0).unwrap();

        assert!(runtime.get_register().is_empty());
        assert_eq!(runtime.get_data_len(), 2);
        assert_eq!(runtime.get_instruction_cursor(), 1);
    }

    #[test]
    fn jump_if_false_when_unit() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_unit().unwrap();
        runtime.push_instruction(Instruction::JumpIfFalse, Some(0)).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.push_jump_point(3).unwrap();
        runtime.set_instruction_cursor(1).unwrap();
        runtime.push_register(0).unwrap();

        runtime.jump_if_false(0).unwrap();

        assert!(runtime.get_register().is_empty());
        assert_eq!(runtime.get_data_len(), 1);
        assert_eq!(runtime.get_instruction_cursor(), 2);
    }

    #[test]
    fn jump_if_false_when_false() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_false().unwrap();
        runtime.push_instruction(Instruction::JumpIfFalse, Some(0)).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.push_jump_point(3).unwrap();
        runtime.set_instruction_cursor(1).unwrap();
        runtime.push_register(1).unwrap();

        runtime.jump_if_false(0).unwrap();

        assert!(runtime.get_register().is_empty());
        assert_eq!(runtime.get_data_len(), 2);
        assert_eq!(runtime.get_instruction_cursor(), 2);
    }
}
