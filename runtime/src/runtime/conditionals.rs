use log::trace;

use crate::{ExpressionDataType, GarnishLangRuntime, GarnishLangRuntimeResult, Instruction};

impl GarnishLangRuntime {
    pub fn jump_if_true(&mut self, index: usize) -> GarnishLangRuntimeResult {
        trace!("Instruction - Execute Expression If True | Data - {:?}", index);
        let point = self.get_jump_point(index)? - 1;
        let i = self.addr_of_raw_data(self.data.len() - 1)?;
        let d = self.get_data_internal(i)?;
        let remove_data = match self.get_instruction(self.instruction_cursor + 1) {
            None => true,
            Some(instruction) => match instruction.instruction {
                Instruction::JumpIfFalse => false,
                _ => true,
            },
        };

        match d.get_type() {
            crate::ExpressionDataType::False | ExpressionDataType::Unit => {
                trace!("Not jumping from value of type {:?} with addr {:?}", d.get_type(), self.data.len() - 1);
            }
            // all other values are considered true
            _ => {
                trace!("Jumping from value of type {:?} with addr {:?}", d.get_type(), self.data.len() - 1);
                self.instruction_cursor = point;
            }
        };

        if remove_data {
            self.data.pop();
        }

        Ok(())
    }

    pub fn jump_if_false(&mut self, index: usize) -> GarnishLangRuntimeResult {
        trace!("Instruction - Execute Expression If False | Data - {:?}", index);
        let point = self.get_jump_point(index)? - 1;
        let i = self.addr_of_raw_data(self.data.len() - 1)?;
        let d = self.get_data_internal(i)?;
        let remove_data = match self.get_instruction(self.instruction_cursor + 1) {
            None => true,
            Some(instruction) => match instruction.instruction {
                Instruction::JumpIfTrue => false,
                _ => true,
            },
        };

        match d.get_type() {
            ExpressionDataType::False | ExpressionDataType::Unit => {
                trace!("Jumping from value of type {:?} with addr {:?}", d.get_type(), self.data.len() - 1);
                self.instruction_cursor = point;
            }
            _ => {
                trace!("Not jumping from value of type {:?} with addr {:?}", d.get_type(), self.data.len() - 1);
            }
        };

        if remove_data {
            self.data.pop();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{ExpressionData, GarnishLangRuntime, Instruction};

    #[test]
    fn jump_if_true_when_true() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::boolean_true()).unwrap();
        runtime.add_instruction(Instruction::JumpIfTrue, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.add_expression(3).unwrap();

        runtime.jump_if_true(0).unwrap();

        assert!(runtime.data.is_empty());
        assert_eq!(runtime.instruction_cursor, 2);
    }

    #[test]
    fn jump_if_true_when_unit() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::unit()).unwrap();
        runtime.add_instruction(Instruction::JumpIfTrue, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.add_expression(3).unwrap();

        runtime.jump_if_true(0).unwrap();

        assert!(runtime.data.is_empty());
        assert_eq!(runtime.instruction_cursor, 1);
    }

    #[test]
    fn jump_if_true_when_false() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::boolean_false()).unwrap();
        runtime.add_instruction(Instruction::JumpIfTrue, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.add_expression(3).unwrap();

        runtime.jump_if_true(0).unwrap();

        assert!(runtime.data.is_empty());
        assert_eq!(runtime.instruction_cursor, 1);
    }

    #[test]
    fn jump_if_false_when_true() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::boolean_true()).unwrap();
        runtime.add_instruction(Instruction::JumpIfFalse, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.add_expression(3).unwrap();

        runtime.jump_if_false(0).unwrap();

        assert!(runtime.data.is_empty());
        assert_eq!(runtime.instruction_cursor, 1);
    }

    #[test]
    fn jump_if_false_when_unit() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::unit()).unwrap();
        runtime.add_instruction(Instruction::JumpIfFalse, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.add_expression(3).unwrap();

        runtime.jump_if_false(0).unwrap();

        assert!(runtime.data.is_empty());
        assert_eq!(runtime.instruction_cursor, 2);
    }

    #[test]
    fn jump_if_false_when_false() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::boolean_false()).unwrap();
        runtime.add_instruction(Instruction::JumpIfFalse, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.add_expression(3).unwrap();

        runtime.jump_if_false(0).unwrap();

        assert!(runtime.data.is_empty());
        assert_eq!(runtime.instruction_cursor, 2);
    }

    #[test]
    fn conditional_execute_double_check_removes_data_after_last_true_false() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::boolean_false()).unwrap();
        runtime.add_instruction(Instruction::JumpIfTrue, Some(3)).unwrap();
        runtime.add_instruction(Instruction::JumpIfFalse, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.add_expression(4).unwrap();

        runtime.jump_if_true(0).unwrap();

        assert_eq!(runtime.data.len(), 1);

        runtime.jump_if_false(0).unwrap();

        assert!(runtime.data.is_empty());
        assert_eq!(runtime.instruction_cursor, 3);
    }

    #[test]
    fn conditional_execute_double_check_removes_data_after_last_false_true() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::boolean_true()).unwrap();
        runtime.add_instruction(Instruction::JumpIfFalse, Some(3)).unwrap();
        runtime.add_instruction(Instruction::JumpIfTrue, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.add_expression(4).unwrap();

        runtime.jump_if_false(0).unwrap();

        assert_eq!(runtime.data.len(), 1);

        runtime.jump_if_true(0).unwrap();

        assert!(runtime.data.is_empty());
        assert_eq!(runtime.instruction_cursor, 3);
    }
}
