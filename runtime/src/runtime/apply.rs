use crate::runtime::list::get_access_addr;
use crate::runtime::utilities::*;
use crate::{state_error, ExpressionDataType, GarnishLangRuntimeData, RuntimeError, TypeConstants};
use log::trace;

use super::context::GarnishLangRuntimeContext;

pub(crate) fn apply<Data: GarnishLangRuntimeData, T: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut T>,
) -> Result<(), RuntimeError<Data::Error>> {
    trace!("Instruction - Apply");
    apply_internal(this, context)
}

pub(crate) fn reapply<Data: GarnishLangRuntimeData>(this: &mut Data, index: Data::Size) -> Result<(), RuntimeError<Data::Error>> {
    trace!("Instruction - Reapply | Data - {:?}", index);

    let (right_addr, left_addr) = next_two_raw_ref(this)?;

    // only execute if left side is a true like value
    match this.get_data_type(left_addr)? {
        ExpressionDataType::Unit | ExpressionDataType::False => Ok(()),
        _ => {
            let point = match this.get_jump_point(index) {
                None => state_error(format!("No jump point at index {:?}", index))?,
                Some(i) => i,
            };

            this.set_instruction_cursor(point - Data::Size::one())?;
            match this.pop_value_stack() {
                None => state_error(format!("Failed to pop input during reapply operation."))?,
                Some(_) => (),
            }
            Ok(this.push_value_stack(right_addr)?)
        }
    }
}

pub(crate) fn empty_apply<Data: GarnishLangRuntimeData, T: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut T>,
) -> Result<(), RuntimeError<Data::Error>> {
    trace!("Instruction - Empty Apply");
    push_unit(this)?;

    apply_internal(this, context)
}

pub(crate) fn apply_internal<Data: GarnishLangRuntimeData, T: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut T>,
) -> Result<(), RuntimeError<Data::Error>> {
    let right_addr = next_ref(this)?;
    let left_addr = next_ref(this)?;

    match this.get_data_type(left_addr)? {
        ExpressionDataType::Expression => {
            let expression_index = this.get_expression(left_addr)?;

            let next_instruction = match this.get_jump_point(expression_index) {
                None => state_error(format!("No jump point at index {:?}", expression_index))?,
                Some(i) => i,
            };

            // Expression stores index of expression table, look up actual instruction index

            this.push_jump_path(this.get_instruction_cursor())?;
            this.set_instruction_cursor(next_instruction - Data::Size::one())?;
            Ok(this.push_value_stack(right_addr)?)
        }
        ExpressionDataType::External => {
            let external_value = this.get_external(left_addr)?;

            match context {
                None => push_unit(this),
                Some(c) => match c.apply(external_value, right_addr, this)? {
                    true => Ok(()),
                    false => push_unit(this),
                },
            }
        }
        ExpressionDataType::List => {
            match this.get_data_type(right_addr)? {
                ExpressionDataType::List => {
                    let len = this.get_list_len(right_addr)?;

                    // lease stack to avoid creating a partial list
                    // and finding out all items aren't either an integer or symbol
                    let lease = this.lease_tmp_stack()?;

                    let mut count = len;

                    while count > Data::Size::zero() {
                        let i = Data::size_to_integer(count - Data::Size::one());
                        let item = this.get_list_item(right_addr, i)?;
                        let value = match get_access_addr(this, item, left_addr)? {
                            Some(addr) => addr,
                            // make sure there is a unit value to use
                            None => this.add_unit()?,
                        };

                        this.push_tmp_stack(lease, value)?;

                        count -= Data::Size::one();
                    }

                    this.start_list(len)?;
                    while let Some(i) = this.pop_tmp_stack(lease)? {
                        this.add_to_list(i, false)?;
                    }

                    this.end_list().and_then(|r| this.push_register(r))?;

                    this.release_tmp_stack(lease)?;
                }
                _ => match get_access_addr(this, right_addr, left_addr)? {
                    None => push_unit(this)?,
                    Some(i) => this.push_register(i)?,
                },
            }

            Ok(())
        }
        _ => push_unit(this),
    }
}

#[cfg(test)]
mod tests {
    use crate::simple::DataError;
    use crate::{
        runtime::{
            context::{EmptyContext, GarnishLangRuntimeContext},
            GarnishRuntime,
        },
        symbol_value, ExpressionDataType, GarnishLangRuntimeData, Instruction, RuntimeError, SimpleRuntimeData,
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
    fn apply_integer_to_list() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let i3 = runtime.add_integer(30).unwrap();
        runtime.start_list(3).unwrap();
        runtime.add_to_list(i1, false).unwrap();
        runtime.add_to_list(i2, false).unwrap();
        runtime.add_to_list(i3, false).unwrap();
        let i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_integer(2).unwrap();

        runtime.push_register(i4).unwrap();
        runtime.push_register(i5).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), 30);
    }

    #[test]
    fn apply_symbol_to_list() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("val1").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();

        let i4 = runtime.add_symbol("val2").unwrap();
        let i5 = runtime.add_integer(20).unwrap();
        let i6 = runtime.add_pair((i4, i5)).unwrap();

        let i7 = runtime.add_symbol("val3").unwrap();
        let i8 = runtime.add_integer(30).unwrap();
        let i9 = runtime.add_pair((i7, i8)).unwrap();

        runtime.start_list(3).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        runtime.add_to_list(i6, true).unwrap();
        runtime.add_to_list(i9, true).unwrap();
        let i10 = runtime.end_list().unwrap();

        let i11 = runtime.add_symbol("val2").unwrap();

        runtime.push_register(i10).unwrap();
        runtime.push_register(i11).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), 20);
    }

    #[test]
    fn apply_list_of_sym_or_integer_to_list() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("val1").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();

        let i4 = runtime.add_symbol("val2").unwrap();
        let i5 = runtime.add_integer(20).unwrap();
        let i6 = runtime.add_pair((i4, i5)).unwrap();

        let i7 = runtime.add_symbol("val3").unwrap();
        let i8 = runtime.add_integer(30).unwrap();
        let i9 = runtime.add_pair((i7, i8)).unwrap();

        runtime.start_list(3).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        runtime.add_to_list(i6, true).unwrap();
        runtime.add_to_list(i9, true).unwrap();
        let i10 = runtime.end_list().unwrap();

        let i11 = runtime.add_integer(2).unwrap();
        let i12 = runtime.add_symbol("val2").unwrap();
        let i13 = runtime.add_integer(5).unwrap();
        let i14 = runtime.add_expression(10).unwrap();
        runtime.start_list(2).unwrap();
        runtime.add_to_list(i11, false).unwrap();
        runtime.add_to_list(i12, false).unwrap();
        runtime.add_to_list(i13, false).unwrap();
        runtime.add_to_list(i14, false).unwrap();
        let i15 = runtime.end_list().unwrap();

        runtime.push_register(i10).unwrap();
        runtime.push_register(i15).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        let addr = runtime.pop_register().unwrap();
        let len = runtime.get_list_len(addr).unwrap();
        assert_eq!(len, 4);

        let pair_addr = runtime.get_list_item(addr, 0).unwrap();
        assert_eq!(runtime.get_data_type(pair_addr).unwrap(), ExpressionDataType::Pair);

        let (pair_left, pair_right) = runtime.get_pair(pair_addr).unwrap();
        assert_eq!(runtime.get_data_type(pair_left).unwrap(), ExpressionDataType::Symbol);
        assert_eq!(runtime.get_symbol(pair_left).unwrap(), symbol_value("val3"));
        assert_eq!(runtime.get_data_type(pair_right).unwrap(), ExpressionDataType::Integer);
        assert_eq!(runtime.get_integer(pair_right).unwrap(), 30);

        let int_addr = runtime.get_list_item(addr, 1).unwrap();
        assert_eq!(runtime.get_data_type(int_addr).unwrap(), ExpressionDataType::Integer);
        assert_eq!(runtime.get_integer(int_addr).unwrap(), 20);

        let unit1 = runtime.get_list_item(addr, 2).unwrap();
        let unit2 = runtime.get_list_item(addr, 3).unwrap();

        assert_eq!(runtime.get_data_type(unit1).unwrap(), ExpressionDataType::Unit);
        assert_eq!(runtime.get_data_type(unit2).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn apply_with_unsupported_left_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_integer(10).unwrap();
        let int2 = runtime.add_integer(20).unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), 0usize);
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
            new_addr: usize,
        }

        impl GarnishLangRuntimeContext<SimpleRuntimeData> for MyContext {
            fn resolve(&mut self, _: u64, _: &mut SimpleRuntimeData) -> Result<bool, RuntimeError<DataError>> {
                Ok(false)
            }

            fn apply(&mut self, external_value: usize, input_addr: usize, runtime: &mut SimpleRuntimeData) -> Result<bool, RuntimeError<DataError>> {
                assert_eq!(external_value, 3);

                let value = match runtime.get_data_type(input_addr)? {
                    ExpressionDataType::Integer => runtime.get_integer(input_addr)?,
                    _ => return Ok(false),
                };

                self.new_addr = runtime.add_integer(value * 2)?;
                runtime.push_register(self.new_addr)?;

                Ok(true)
            }
        }

        let mut context = MyContext { new_addr: 0 };

        runtime.apply(Some(&mut context)).unwrap();

        assert_eq!(runtime.get_integer(context.new_addr).unwrap(), 200);
        assert_eq!(runtime.get_register(0).unwrap(), context.new_addr);
    }
}
