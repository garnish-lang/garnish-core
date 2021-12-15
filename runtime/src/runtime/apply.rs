use log::trace;

use crate::{error, ExpressionDataType, GarnishLangRuntime, GarnishLangRuntimeResult};

use super::{context::GarnishLangRuntimeContext, data::GarnishLangRuntimeDataPool};

impl<Data> GarnishLangRuntime<Data>
where
    Data: GarnishLangRuntimeDataPool,
{
    pub fn apply<T: GarnishLangRuntimeContext>(&mut self, context: Option<&mut T>) -> GarnishLangRuntimeResult {
        trace!("Instruction - Apply");
        self.apply_internal(context)
    }

    pub fn reapply(&mut self, index: usize) -> GarnishLangRuntimeResult {
        trace!("Instruction - Reapply | Data - {:?}", index);

        let right_addr = self.next_ref()?;
        let point = self.data.get_jump_point(index)?;

        self.data.set_instruction_cursor(point - 1)?;
        self.data.pop_input()?;
        self.data.push_input(right_addr)
    }

    pub fn empty_apply<T: GarnishLangRuntimeContext>(&mut self, context: Option<&mut T>) -> GarnishLangRuntimeResult {
        trace!("Instruction - Empty Apply");
        self.push_unit()?;

        self.apply_internal(context)
    }

    fn apply_internal<T: GarnishLangRuntimeContext>(&mut self, context: Option<&mut T>) -> GarnishLangRuntimeResult {
        let right_addr = self.next_ref()?;
        let left_addr = self.next_ref()?;

        match self.data.get_data_type(left_addr)? {
            ExpressionDataType::Expression => {
                let expression_index = self.data.get_expression(left_addr)?;

                let next_instruction = self.data.get_jump_point(expression_index)?;

                // Expression stores index of expression table, look up actual instruction index

                self.data.push_jump_path(self.data.get_instruction_cursor()?)?;
                self.data.set_instruction_cursor(next_instruction - 1)?;
                self.data.push_input(right_addr)
            }
            ExpressionDataType::External => {
                let external_value = self.data.get_expression(left_addr)?;

                match context {
                    None => self.push_unit(),
                    Some(c) => match c.apply(external_value, right_addr, self)? {
                        true => Ok(()),
                        false => self.push_unit(),
                    },
                }
            }
            t => Err(error(format!("Data type {:?} not supported on left side of apply operation.", t))),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        runtime::{
            context::{EmptyContext, GarnishLangRuntimeContext},
            data::GarnishLangRuntimeDataPool,
        },
        ExpressionData, ExpressionDataType, GarnishLangRuntime, GarnishLangRuntimeResult, Instruction,
    };

    #[test]
    fn apply() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::expression(0)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        // 1
        runtime.add_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.add_instruction(Instruction::PutInput, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::EndExpression, None).unwrap();

        // 5
        runtime.add_instruction(Instruction::Put, Some(2)).unwrap();
        runtime.add_instruction(Instruction::Put, Some(3)).unwrap();
        runtime.add_instruction(Instruction::Apply, None).unwrap();

        runtime.data.push_jump_point(1).unwrap();

        runtime.data.push_register(2).unwrap();
        runtime.data.push_register(3).unwrap();

        runtime.data.set_instruction_cursor(7).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.data.get_input(0).unwrap(), 3);
        assert_eq!(runtime.data.get_instruction_cursor().unwrap(), 0);
        assert_eq!(runtime.data.get_jump_path(0).unwrap(), 7);
    }

    #[test]
    fn apply_no_references_is_err() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::expression(0)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        // 1
        runtime.add_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.add_instruction(Instruction::PutInput, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::EndExpression, None).unwrap();

        // 5
        runtime.add_instruction(Instruction::Put, Some(2)).unwrap();
        runtime.add_instruction(Instruction::Put, Some(3)).unwrap();
        runtime.add_instruction(Instruction::Apply, None).unwrap();

        runtime.data.push_jump_point(1).unwrap();

        runtime.set_instruction_cursor(7).unwrap();

        let result = runtime.apply::<EmptyContext>(None);

        assert!(result.is_err());
    }

    #[test]
    fn empty_apply() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::expression(0)).unwrap();

        // 1
        runtime.add_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.add_instruction(Instruction::PutInput, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::EndExpression, None).unwrap();

        // 5
        runtime.add_instruction(Instruction::Put, Some(2)).unwrap();
        runtime.add_instruction(Instruction::EmptyApply, None).unwrap();

        runtime.data.push_jump_point(1).unwrap();

        runtime.data.push_register(2).unwrap();

        runtime.data.set_instruction_cursor(6).unwrap();

        runtime.empty_apply::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.data.get_input(0).unwrap(), 3);
        assert_eq!(runtime.data.get_instruction_cursor().unwrap(), 0);
        assert_eq!(runtime.data.get_jump_path(0).unwrap(), 6);
    }

    #[test]
    fn empty_apply_no_references_is_err() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::expression(0)).unwrap();

        // 1
        runtime.add_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.add_instruction(Instruction::PutInput, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::EndExpression, None).unwrap();

        // 5
        runtime.add_instruction(Instruction::Put, Some(2)).unwrap();
        runtime.add_instruction(Instruction::EmptyApply, None).unwrap();

        runtime.data.push_jump_point(1).unwrap();

        runtime.data.set_instruction_cursor(6).unwrap();

        let result = runtime.empty_apply::<EmptyContext>(None);

        assert!(result.is_err());
    }

    #[test]
    fn reapply() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::expression(0)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(30)).unwrap();
        runtime.add_data(ExpressionData::integer(40)).unwrap();

        // 1
        runtime.add_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.add_instruction(Instruction::Put, Some(2)).unwrap();
        runtime.add_instruction(Instruction::Apply, None).unwrap();

        // 4
        runtime.add_instruction(Instruction::Put, Some(0)).unwrap();
        runtime.add_instruction(Instruction::PutInput, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PutResult, None).unwrap();
        runtime.add_instruction(Instruction::Reapply, Some(0)).unwrap();
        runtime.add_instruction(Instruction::EndExpression, None).unwrap();

        runtime.data.push_jump_point(4).unwrap();

        runtime.data.push_register(4).unwrap();

        runtime.data.push_input(2).unwrap();
        runtime.data.set_result(Some(4)).unwrap();
        runtime.data.push_jump_path(9).unwrap();

        runtime.data.set_instruction_cursor(8).unwrap();

        runtime.reapply(0).unwrap();

        assert_eq!(runtime.data.get_input_count(), 1);
        assert_eq!(runtime.data.get_input(0).unwrap(), 4);
        assert_eq!(runtime.data.get_instruction_cursor().unwrap(), 3);
        assert_eq!(runtime.data.get_jump_path(0).unwrap(), 9);
    }

    #[test]
    fn apply_from_context() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::external(3)).unwrap();
        runtime.add_data(ExpressionData::integer(100)).unwrap();

        runtime.add_instruction(Instruction::Resolve, None).unwrap();

        runtime.data.push_register(1).unwrap();
        runtime.data.push_register(2).unwrap();

        struct MyContext {}

        impl GarnishLangRuntimeContext for MyContext {
            fn resolve<Data: GarnishLangRuntimeDataPool>(&mut self, _: usize, _: &mut GarnishLangRuntime<Data>) -> GarnishLangRuntimeResult<bool> {
                Ok(false)
            }

            fn apply<Data: GarnishLangRuntimeDataPool>(
                &mut self,
                external_value: usize,
                input_addr: usize,
                runtime: &mut GarnishLangRuntime<Data>,
            ) -> GarnishLangRuntimeResult<bool> {
                assert_eq!(external_value, 3);

                let value = match runtime.data.get_data_type(input_addr)? {
                    ExpressionDataType::Integer => runtime.data.get_integer(input_addr)?,
                    _ => return Ok(false),
                };

                runtime.push_integer(value * 2)?;
                Ok(true)
            }
        }

        let mut context = MyContext {};

        runtime.apply(Some(&mut context)).unwrap();

        assert_eq!(runtime.data.get_integer(3).unwrap(), 200);
        assert_eq!(runtime.data.get_register().get(0).unwrap(), &3);
    }
}
