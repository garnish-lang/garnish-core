use log::trace;

use crate::{error, ExpressionDataType, GarnishLangRuntime, GarnishLangRuntimeResult, NestInto};

use super::data::GarnishLangRuntimeData;

impl<Data> GarnishLangRuntime<Data>
where
    Data: GarnishLangRuntimeData,
{
    pub fn jump(&mut self, index: usize) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - Jump | Data - {:?}", index);

        self.data
            .set_instruction_cursor(
                self.data
                    .get_jump_point(index)
                    .ok_or(error(format!("No jump point at index {:?}", index)))?
                    - 1,
            )
            .nest_into()
    }

    pub fn jump_if_true(&mut self, index: usize) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - Execute Expression If True | Data - {:?}", index);
        let point = self
            .data
            .get_jump_point(index)
            .ok_or(error(format!("No jump point at index {:?}.", index)))?
            - 1;
        let d = self.next_ref()?;

        match self.data.get_data_type(d).nest_into()? {
            ExpressionDataType::False | ExpressionDataType::Unit => {
                trace!(
                    "Not jumping from value of type {:?} with addr {:?}",
                    self.data.get_data_type(d).nest_into()?,
                    self.get_data_len() - 1
                );
            }
            // all other values are considered true
            t => {
                trace!("Jumping from value of type {:?} with addr {:?}", t, self.get_data_len() - 1);
                self.data.set_instruction_cursor(point).nest_into()?
            }
        };

        Ok(())
    }

    pub fn jump_if_false(&mut self, index: usize) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - Execute Expression If False | Data - {:?}", index);
        let point = self
            .data
            .get_jump_point(index)
            .ok_or(error(format!("No jump point at index {:?}.", index)))?
            - 1;
        let d = self.next_ref()?;

        match self.data.get_data_type(d).nest_into()? {
            ExpressionDataType::False | ExpressionDataType::Unit => {
                trace!(
                    "Jumping from value of type {:?} with addr {:?}",
                    self.data.get_data_type(d).nest_into()?,
                    self.get_data_len() - 1
                );
                self.data.set_instruction_cursor(point).nest_into()?
            }
            t => {
                trace!("Not jumping from value of type {:?} with addr {:?}", t, self.get_data_len() - 1);
            }
        };

        Ok(())
    }

    pub fn end_expression(&mut self) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - End Expression");
        match self.data.pop_jump_path() {
            None => {
                // no more jumps, this should be the end of the entire execution
                let r = self.next_ref()?;
                self.data.advance_instruction_cursor().nest_into()?;
                self.data.set_result(Some(self.addr_of_raw_data(r)?)).nest_into()?;
            }
            Some(jump_point) => {
                self.data.set_instruction_cursor(jump_point).nest_into()?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{runtime::data::GarnishLangRuntimeData, ExpressionData, GarnishLangRuntime, Instruction};

    #[test]
    fn end_expression() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_instruction(Instruction::Put, Some(1)).unwrap();

        runtime.data.set_instruction_cursor(1).unwrap();
        runtime.data.push_register(1).unwrap();
        runtime.data.set_result(Some(1)).unwrap();

        runtime.end_expression().unwrap();

        assert_eq!(runtime.data.get_instruction_cursor(), 2);
        assert_eq!(runtime.data.get_integer(runtime.data.get_result().unwrap()).unwrap(), 10);
    }

    #[test]
    fn end_expression_last_register_is_result() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(30)).unwrap();
        runtime.add_instruction(Instruction::Put, Some(1)).unwrap();

        runtime.data.set_instruction_cursor(1).unwrap();
        runtime.data.push_register(2).unwrap();
        runtime.data.set_result(Some(1)).unwrap();

        runtime.end_expression().unwrap();

        assert_eq!(runtime.data.get_instruction_cursor(), 2);
        assert_eq!(runtime.data.get_integer(runtime.data.get_result().unwrap()).unwrap(), 20);
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

        runtime.data.push_register(1).unwrap();
        runtime.data.set_result(Some(1)).unwrap();
        runtime.data.push_jump_path(4).unwrap();
        runtime.data.set_instruction_cursor(2).unwrap();

        runtime.end_expression().unwrap();

        assert_eq!(runtime.data.get_instruction_cursor(), 4);
    }

    #[test]
    fn jump() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::symbol(&"false".to_string(), 0)).unwrap();
        runtime.add_instruction(Instruction::JumpIfFalse, Some(3)).unwrap();
        runtime.add_instruction(Instruction::JumpTo, Some(0)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.data.push_jump_point(4).unwrap();

        runtime.jump(0).unwrap();

        assert!(runtime.data.get_jump_path_vec().is_empty());
        assert_eq!(runtime.data.get_instruction_cursor(), 3);
    }

    #[test]
    fn jump_if_true_no_ref_is_err() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::boolean_true()).unwrap();
        runtime.add_instruction(Instruction::JumpIfTrue, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.data.push_jump_point(3).unwrap();

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

        runtime.data.push_jump_point(3).unwrap();

        let result = runtime.jump_if_false(0);

        assert!(result.is_err());
    }

    #[test]
    fn jump_if_true_when_true() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::boolean_true()).unwrap();
        runtime.add_instruction(Instruction::JumpIfTrue, Some(0)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.data.push_jump_point(3).unwrap();
        runtime.data.set_instruction_cursor(1).unwrap();
        runtime.data.push_register(1).unwrap();

        runtime.jump_if_true(0).unwrap();

        assert!(runtime.data.get_register().is_empty());
        assert_eq!(runtime.data.get_data_len(), 2);
        assert_eq!(runtime.data.get_instruction_cursor(), 2);
    }

    #[test]
    fn jump_if_true_when_unit() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::unit()).unwrap();
        runtime.add_instruction(Instruction::JumpIfTrue, Some(0)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.data.push_jump_point(3).unwrap();
        runtime.data.set_instruction_cursor(1).unwrap();
        runtime.data.push_register(1).unwrap();

        runtime.jump_if_true(0).unwrap();

        assert!(runtime.data.get_register().is_empty());
        assert_eq!(runtime.data.get_data_len(), 2);
        assert_eq!(runtime.data.get_instruction_cursor(), 1);
    }

    #[test]
    fn jump_if_true_when_false() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::boolean_false()).unwrap();
        runtime.add_instruction(Instruction::JumpIfTrue, Some(0)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.data.push_jump_point(3).unwrap();
        runtime.data.set_instruction_cursor(1).unwrap();
        runtime.data.push_register(1).unwrap();

        runtime.jump_if_true(0).unwrap();

        assert!(runtime.data.get_register().is_empty());
        assert_eq!(runtime.data.get_data_len(), 2);
        assert_eq!(runtime.data.get_instruction_cursor(), 1);
    }

    #[test]
    fn jump_if_false_when_true() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::boolean_true()).unwrap();
        runtime.add_instruction(Instruction::JumpIfFalse, Some(0)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.data.push_jump_point(3).unwrap();
        runtime.data.set_instruction_cursor(1).unwrap();
        runtime.data.push_register(1).unwrap();

        runtime.jump_if_false(0).unwrap();

        assert!(runtime.data.get_register().is_empty());
        assert_eq!(runtime.data.get_data_len(), 2);
        assert_eq!(runtime.data.get_instruction_cursor(), 1);
    }

    #[test]
    fn jump_if_false_when_unit() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::unit()).unwrap();
        runtime.add_instruction(Instruction::JumpIfFalse, Some(0)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.data.push_jump_point(3).unwrap();
        runtime.data.set_instruction_cursor(1).unwrap();
        runtime.data.push_register(1).unwrap();

        runtime.jump_if_false(0).unwrap();

        assert!(runtime.data.get_register().is_empty());
        assert_eq!(runtime.data.get_data_len(), 2);
        assert_eq!(runtime.data.get_instruction_cursor(), 2);
    }

    #[test]
    fn jump_if_false_when_false() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::boolean_false()).unwrap();
        runtime.add_instruction(Instruction::JumpIfFalse, Some(0)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.data.push_jump_point(3).unwrap();
        runtime.data.set_instruction_cursor(1).unwrap();
        runtime.data.push_register(1).unwrap();

        runtime.jump_if_false(0).unwrap();

        assert!(runtime.data.get_register().is_empty());
        assert_eq!(runtime.data.get_data_len(), 2);
        assert_eq!(runtime.data.get_instruction_cursor(), 2);
    }
}
