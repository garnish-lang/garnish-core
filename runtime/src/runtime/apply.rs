use crate::runtime::utilities::*;
use crate::{error, ExpressionDataType, GarnishLangRuntimeData, GarnishLangRuntimeResult, NestInto, TypeConstants};
use log::trace;

use super::context::GarnishLangRuntimeContext;

pub(crate) fn apply<Data: GarnishLangRuntimeData, T: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut T>) -> GarnishLangRuntimeResult<Data::Error> {
    trace!("Instruction - Apply");
    apply_internal(this, context)
}

pub(crate) fn reapply<Data: GarnishLangRuntimeData>(this: &mut Data, index: Data::Size) -> GarnishLangRuntimeResult<Data::Error> {
    trace!("Instruction - Reapply | Data - {:?}", index);

    let (right_addr, left_addr) = next_two_raw_ref(this)?;

    // only execute if left side is a true like value
    match this.get_data_type(left_addr).nest_into()? {
        ExpressionDataType::Unit | ExpressionDataType::False => Ok(()),
        _ => {
            let point = this.get_jump_point(index).ok_or(error(format!("No jump point at index {:?}", index)))?;

            this.set_instruction_cursor(point - Data::Size::one()).nest_into()?;
            this.pop_value_stack()
                .ok_or(error(format!("Failed to pop input during reapply operation.")))?;
            this.push_value_stack(right_addr).nest_into()
        }
    }
}

pub(crate) fn empty_apply<Data: GarnishLangRuntimeData, T: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut T>) -> GarnishLangRuntimeResult<Data::Error> {
    trace!("Instruction - Empty Apply");
    push_unit(this)?;

    apply_internal(this, context)
}

pub(crate) fn apply_internal<Data: GarnishLangRuntimeData, T: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut T>,
) -> GarnishLangRuntimeResult<Data::Error> {
    let right_addr = next_ref(this)?;
    let left_addr = next_ref(this)?;

    match this.get_data_type(left_addr).nest_into()? {
        ExpressionDataType::Expression => {
            let expression_index = this.get_expression(left_addr).nest_into()?;

            let next_instruction = this
                .get_jump_point(expression_index)
                .ok_or(error(format!("No jump point at index {:?}", expression_index)))?;

            // Expression stores index of expression table, look up actual instruction index

            this.push_jump_path(this.get_instruction_cursor()).nest_into()?;
            this.set_instruction_cursor(next_instruction - Data::Size::one()).nest_into()?;
            this.push_value_stack(right_addr).nest_into()
        }
        ExpressionDataType::External => {
            let external_value = this.get_external(left_addr).nest_into()?;

            match context {
                None => push_unit(this),
                Some(c) => match c.apply(external_value, right_addr, this)? {
                    true => Ok(()),
                    false => push_unit(this),
                },
            }
        }
        t => Err(error(format!("Data type {:?} not supported on left side of apply operation.", t))),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        runtime::{
            context::{EmptyContext, GarnishLangRuntimeContext},
            GarnishRuntime,
        }, ExpressionDataType, GarnishLangRuntimeData, GarnishLangRuntimeResult, Instruction, NestInto, SimpleRuntimeData,
    };

    #[test]
    fn apply() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_integer(10).unwrap();
        let exp1 = runtime.add_expression(0).unwrap();
        let int2 = runtime.add_integer(20).unwrap();

        // 1
        runtime.push_instruction(Instruction::Put, Some(int1)).unwrap();
        runtime.push_instruction(Instruction::PutValue, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::EndExpression, None).unwrap();

        // 5
        runtime.push_instruction(Instruction::Put, Some(exp1)).unwrap();
        runtime.push_instruction(Instruction::Put, Some(int2)).unwrap();
        runtime.push_instruction(Instruction::Apply, None).unwrap();

        runtime.push_jump_point(1).unwrap();

        runtime.push_register(exp1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.set_instruction_cursor(7).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_value(0).unwrap(), 5);
        assert_eq!(runtime.get_instruction_cursor(), 0);
        assert_eq!(runtime.get_jump_path(0).unwrap(), 7);
    }

    #[test]
    fn apply_no_references_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_integer(10).unwrap();
        runtime.add_expression(0).unwrap();
        runtime.add_integer(20).unwrap();

        // 1
        runtime.push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.push_instruction(Instruction::PutValue, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::EndExpression, None).unwrap();

        // 5
        runtime.push_instruction(Instruction::Put, Some(2)).unwrap();
        runtime.push_instruction(Instruction::Put, Some(3)).unwrap();
        runtime.push_instruction(Instruction::Apply, None).unwrap();

        runtime.push_jump_point(1).unwrap();

        runtime.set_instruction_cursor(7).unwrap();

        let result = runtime.apply::<EmptyContext>(None);

        assert!(result.is_err());
    }

    #[test]
    fn empty_apply() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_integer(10).unwrap();
        let exp1 = runtime.add_expression(0).unwrap();

        // 1
        runtime.push_instruction(Instruction::Put, Some(int1)).unwrap();
        runtime.push_instruction(Instruction::PutValue, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::EndExpression, None).unwrap();

        // 5
        runtime.push_instruction(Instruction::Put, Some(exp1)).unwrap();
        runtime.push_instruction(Instruction::EmptyApply, None).unwrap();

        runtime.push_jump_point(1).unwrap();

        runtime.push_register(exp1).unwrap();

        runtime.set_instruction_cursor(6).unwrap();

        runtime.empty_apply::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_value(0).unwrap(), 0);
        assert_eq!(runtime.get_instruction_cursor(), 0);
        assert_eq!(runtime.get_jump_path(0).unwrap(), 6);
    }

    #[test]
    fn empty_apply_no_references_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_integer(10).unwrap();
        runtime.add_expression(0).unwrap();

        // 1
        runtime.push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.push_instruction(Instruction::PutValue, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::EndExpression, None).unwrap();

        // 5
        runtime.push_instruction(Instruction::Put, Some(2)).unwrap();
        runtime.push_instruction(Instruction::EmptyApply, None).unwrap();

        runtime.push_jump_point(1).unwrap();

        runtime.set_instruction_cursor(6).unwrap();

        let result = runtime.empty_apply::<EmptyContext>(None);

        assert!(result.is_err());
    }

    #[test]
    fn reapply_if_true() {
        let mut runtime = SimpleRuntimeData::new();

        let true1 = runtime.add_true().unwrap();
        let _exp1 = runtime.add_expression(0).unwrap();
        let int1 = runtime.add_integer(20).unwrap();
        let _int2 = runtime.add_integer(30).unwrap();
        let int3 = runtime.add_integer(40).unwrap();

        // 1
        runtime.push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.push_instruction(Instruction::Put, Some(2)).unwrap();
        runtime.push_instruction(Instruction::Apply, None).unwrap();

        // 4
        runtime.push_instruction(Instruction::Put, Some(0)).unwrap();
        runtime.push_instruction(Instruction::PutValue, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PutValue, None).unwrap();
        runtime.push_instruction(Instruction::Reapply, Some(0)).unwrap();
        runtime.push_instruction(Instruction::EndExpression, None).unwrap();

        runtime.push_jump_point(4).unwrap();

        runtime.push_register(true1).unwrap();
        runtime.push_register(int3).unwrap();

        runtime.push_value_stack(int1).unwrap();
        runtime.push_jump_path(9).unwrap();

        runtime.set_instruction_cursor(8).unwrap();

        runtime.reapply(0).unwrap();

        assert_eq!(runtime.get_value_stack_len(), 1);
        assert_eq!(runtime.get_value(0).unwrap(), int3);
        assert_eq!(runtime.get_instruction_cursor(), 3);
        assert_eq!(runtime.get_jump_path(0).unwrap(), 9);
    }

    #[test]
    fn reapply_if_false() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_false().unwrap();
        runtime.add_expression(0).unwrap();
        runtime.add_integer(20).unwrap();
        runtime.add_integer(30).unwrap();
        runtime.add_integer(40).unwrap();

        // 1
        runtime.push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.push_instruction(Instruction::Put, Some(2)).unwrap();
        runtime.push_instruction(Instruction::Apply, None).unwrap();

        // 4
        runtime.push_instruction(Instruction::Put, Some(0)).unwrap();
        runtime.push_instruction(Instruction::PutValue, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PutValue, None).unwrap();
        runtime.push_instruction(Instruction::Reapply, Some(0)).unwrap();
        runtime.push_instruction(Instruction::EndExpression, None).unwrap();

        runtime.push_jump_point(4).unwrap();

        runtime.push_register(1).unwrap();
        runtime.push_register(4).unwrap();

        runtime.push_value_stack(2).unwrap();
        runtime.push_jump_path(9).unwrap();

        runtime.set_instruction_cursor(8).unwrap();

        runtime.reapply(0).unwrap();

        assert_eq!(runtime.get_value_stack_len(), 1);
        assert_eq!(runtime.get_value(0).unwrap(), 2);
        assert_eq!(runtime.get_instruction_cursor(), 8);
        assert_eq!(runtime.get_jump_path(0).unwrap(), 9);
    }

    #[test]
    fn apply_from_context() {
        let mut runtime = SimpleRuntimeData::new();

        let ext1 = runtime.add_external(3).unwrap();
        let int1 = runtime.add_integer(100).unwrap();

        runtime.push_instruction(Instruction::Resolve, None).unwrap();

        runtime.push_register(ext1).unwrap();
        runtime.push_register(int1).unwrap();

        struct MyContext {
            new_addr: usize
        }

        impl GarnishLangRuntimeContext<SimpleRuntimeData> for MyContext {
            fn resolve(&mut self, _: u64, _: &mut SimpleRuntimeData) -> GarnishLangRuntimeResult<String, bool> {
                Ok(false)
            }

            fn apply(&mut self, external_value: usize, input_addr: usize, runtime: &mut SimpleRuntimeData) -> GarnishLangRuntimeResult<String, bool> {
                assert_eq!(external_value, 3);

                let value = match runtime.get_data_type(input_addr).nest_into()? {
                    ExpressionDataType::Integer => {
                        runtime.get_integer(input_addr).nest_into()?
                    },
                    _ => return Ok(false),
                };

                self.new_addr = runtime.add_integer(value * 2).nest_into()?;
                runtime.push_register(self.new_addr).nest_into()?;

                Ok(true)
            }
        }

        let mut context = MyContext { new_addr: 0 };

        runtime.apply(Some(&mut context)).unwrap();

        assert_eq!(runtime.get_integer(context.new_addr).unwrap(), 200);
        assert_eq!(runtime.get_register().get(0).unwrap(), &context.new_addr);
    }
}
