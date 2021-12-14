use log::trace;

use crate::{error, ExpressionDataType, GarnishLangRuntime, GarnishLangRuntimeResult};

use super::data::GarnishLangRuntimeDataPool;

impl<Data> GarnishLangRuntime<Data>
where
    Data: GarnishLangRuntimeDataPool,
{
    pub fn jump(&mut self, index: usize) -> GarnishLangRuntimeResult {
        trace!("Instruction - Jump | Data - {:?}", index);
        self.instruction_cursor = self.get_jump_point(index)? - 1;

        Ok(())
    }

    pub fn jump_if_true(&mut self, index: usize) -> GarnishLangRuntimeResult {
        trace!("Instruction - Execute Expression If True | Data - {:?}", index);
        let point = self.get_jump_point(index)? - 1;
        let d = self.next_ref_data()?;

        match d.get_type() {
            ExpressionDataType::False | ExpressionDataType::Unit => {
                trace!("Not jumping from value of type {:?} with addr {:?}", d.get_type(), self.data.len() - 1);
            }
            // all other values are considered true
            _ => {
                trace!("Jumping from value of type {:?} with addr {:?}", d.get_type(), self.data.len() - 1);
                self.instruction_cursor = point;
            }
        };

        Ok(())
    }

    pub fn jump_if_false(&mut self, index: usize) -> GarnishLangRuntimeResult {
        trace!("Instruction - Execute Expression If False | Data - {:?}", index);
        let point = self.get_jump_point(index)? - 1;

        let d = self.next_ref_data()?;
        match d.get_type() {
            ExpressionDataType::False | ExpressionDataType::Unit => {
                trace!("Jumping from value of type {:?} with addr {:?}", d.get_type(), self.data.len() - 1);
                self.instruction_cursor = point;
            }
            _ => {
                trace!("Not jumping from value of type {:?} with addr {:?}", d.get_type(), self.data.len() - 1);
            }
        };

        Ok(())
    }

    pub fn end_expression(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - End Expression");
        match self.jump_path.pop() {
            None => {
                self.instruction_cursor += 1;
                self.current_result = Some(self.addr_of_raw_data(self.data.len() - 1)?);
            }
            Some(jump_point) => {
                self.instruction_cursor = jump_point;
            }
        }

        Ok(())
    }

    pub(crate) fn get_jump_point(&self, index: usize) -> GarnishLangRuntimeResult<usize> {
        match self.expression_table.get(index) {
            None => Err(error(format!("No jump point at position {:?}.", index))),
            Some(point) => Ok(*point),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ExpressionData, GarnishLangRuntime, Instruction};

    #[test]
    fn end_expression() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_instruction(Instruction::Put, Some(1)).unwrap();

        runtime.end_expression().unwrap();

        assert_eq!(runtime.instruction_cursor, 2);
        assert_eq!(runtime.get_result().unwrap().bytes, 10i64.to_le_bytes());
    }

    #[test]
    fn end_expression_with_path() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.add_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.add_instruction(Instruction::EndExpression, Some(0)).unwrap();
        runtime.add_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.add_instruction(Instruction::EmptyApply, None).unwrap();

        runtime.jump_path.push(4);
        runtime.set_instruction_cursor(2).unwrap();
        runtime.end_expression().unwrap();

        assert_eq!(runtime.instruction_cursor, 4);
    }

    #[test]
    fn jump() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::symbol(&"false".to_string(), 0)).unwrap();
        runtime.add_instruction(Instruction::JumpIfFalse, Some(3)).unwrap();
        runtime.add_instruction(Instruction::JumpTo, Some(0)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.add_expression(4).unwrap();

        runtime.jump(0).unwrap();

        assert!(runtime.jump_path.is_empty());
        assert_eq!(runtime.instruction_cursor, 3);
    }

    #[test]
    fn jump_if_true_no_ref_is_err() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::boolean_true()).unwrap();
        runtime.add_instruction(Instruction::JumpIfTrue, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.add_expression(3).unwrap();

        let result = runtime.jump_if_true(0);

        assert!(result.is_err());
    }

    #[test]
    fn jump_if_false_no_ref_is_error() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::boolean_true()).unwrap();
        runtime.add_instruction(Instruction::JumpIfFalse, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.add_expression(3).unwrap();

        let result = runtime.jump_if_false(0);

        assert!(result.is_err());
    }

    #[test]
    fn jump_if_true_when_true() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::boolean_true()).unwrap();
        runtime.add_instruction(Instruction::JumpIfTrue, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.add_expression(3).unwrap();

        runtime.reference_stack.push(1);

        runtime.jump_if_true(0).unwrap();

        assert!(runtime.reference_stack.is_empty());
        assert_eq!(runtime.get_data_len(), 2);
        assert_eq!(runtime.instruction_cursor, 2);
    }

    #[test]
    fn jump_if_true_when_unit() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::unit()).unwrap();
        runtime.add_instruction(Instruction::JumpIfTrue, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.add_expression(3).unwrap();

        runtime.reference_stack.push(1);

        runtime.jump_if_true(0).unwrap();

        assert!(runtime.reference_stack.is_empty());
        assert_eq!(runtime.get_data_len(), 2);
        assert_eq!(runtime.instruction_cursor, 1);
    }

    #[test]
    fn jump_if_true_when_false() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::boolean_false()).unwrap();
        runtime.add_instruction(Instruction::JumpIfTrue, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.add_expression(3).unwrap();

        runtime.reference_stack.push(1);

        runtime.jump_if_true(0).unwrap();

        assert!(runtime.reference_stack.is_empty());
        assert_eq!(runtime.get_data_len(), 2);
        assert_eq!(runtime.instruction_cursor, 1);
    }

    #[test]
    fn jump_if_false_when_true() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::boolean_true()).unwrap();
        runtime.add_instruction(Instruction::JumpIfFalse, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.add_expression(3).unwrap();

        runtime.reference_stack.push(1);

        runtime.jump_if_false(0).unwrap();

        assert!(runtime.reference_stack.is_empty());
        assert_eq!(runtime.get_data_len(), 2);
        assert_eq!(runtime.instruction_cursor, 1);
    }

    #[test]
    fn jump_if_false_when_unit() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::unit()).unwrap();
        runtime.add_instruction(Instruction::JumpIfFalse, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.add_expression(3).unwrap();

        runtime.reference_stack.push(1);

        runtime.jump_if_false(0).unwrap();

        assert!(runtime.reference_stack.is_empty());
        assert_eq!(runtime.get_data_len(), 2);
        assert_eq!(runtime.instruction_cursor, 2);
    }

    #[test]
    fn jump_if_false_when_false() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::boolean_false()).unwrap();
        runtime.add_instruction(Instruction::JumpIfFalse, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.add_expression(3).unwrap();

        runtime.reference_stack.push(1);

        runtime.jump_if_false(0).unwrap();

        assert!(runtime.reference_stack.is_empty());
        assert_eq!(runtime.get_data_len(), 2);
        assert_eq!(runtime.instruction_cursor, 2);
    }
}
