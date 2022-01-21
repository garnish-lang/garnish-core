use super::context::GarnishLangRuntimeContext;
use crate::runtime::list::get_access_addr;
use crate::runtime::utilities::*;
use crate::{state_error, ExpressionDataType, GarnishLangRuntimeData, GarnishNumber, ErrorType, RuntimeError, TypeConstants, Instruction};

pub(crate) fn apply<Data: GarnishLangRuntimeData, T: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut T>,
) -> Result<(), RuntimeError<Data::Error>> {
    apply_internal(this, Instruction::Apply, context)
}

pub(crate) fn reapply<Data: GarnishLangRuntimeData>(this: &mut Data, index: Data::Size) -> Result<(), RuntimeError<Data::Error>> {
    let (right_addr, left_addr) = next_two_raw_ref(this)?;

    // only execute if left side is a true like value
    match this.get_data_type(left_addr)? {
        ExpressionDataType::Unit | ExpressionDataType::False => Ok(()),
        _ => {
            let point = match this.get_jump_point(index) {
                None => state_error(format!("No jump point at index {:?}", index))?,
                Some(i) => i,
            };

            this.set_instruction_cursor(point)?;
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
    push_unit(this)?;
    apply_internal(this, Instruction::EmptyApply, context)
}

pub(crate) fn apply_internal<Data: GarnishLangRuntimeData, T: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    instruction: Instruction,
    context: Option<&mut T>,
) -> Result<(), RuntimeError<Data::Error>> {
    let right_addr = next_ref(this)?;
    let left_addr = next_ref(this)?;

    // currently, apply is responsible for advancing the instruction cursor itself
    // assume default, apply expression will update to its value
    let mut next_instruction = this.get_instruction_cursor() + Data::Size::one();

    match (this.get_data_type(left_addr)?, this.get_data_type(right_addr)?) {
        (ExpressionDataType::Expression, _) => {
            let expression_index = this.get_expression(left_addr)?;

            // Expression stores index of expression table, look up actual instruction index
            next_instruction = match this.get_jump_point(expression_index) {
                None => state_error(format!("No jump point at index {:?}", expression_index))?,
                Some(i) => i,
            };

            this.push_jump_path(this.get_instruction_cursor() + Data::Size::one())?;
            this.push_value_stack(right_addr)?;
        }
        (ExpressionDataType::External, _) => {
            let external_value = this.get_external(left_addr)?;

            match context {
                None => {
                    push_unit(this)?;
                }
                Some(c) => match c.apply(external_value, right_addr, this)? {
                    true => (),
                    false => {
                        push_unit(this)?;
                    }
                },
            };
        }
        (ExpressionDataType::List, ExpressionDataType::List) => {
            let len = this.get_list_len(right_addr)?;

            let mut count = Data::Size::zero();

            this.start_list(len)?;

            while count < len {
                let i = Data::size_to_number(count);
                let mut item = this.get_list_item(right_addr, i)?;

                let (is_pair_mapping, sym_addr) = match this.get_data_type(item)? {
                    ExpressionDataType::Pair => {
                        let (left, right) = this.get_pair(item)?;
                        match this.get_data_type(left)? {
                            ExpressionDataType::Symbol => {
                                // update item to be mapping pair's right
                                item = right;
                                (true, left)
                            }
                            _ => (false, Data::Size::zero()),
                        }
                    }
                    _ => (false, Data::Size::zero()),
                };

                let (value, is_associative) = match get_access_addr(this, item, left_addr) {
                    Err(e) => match e.get_type() {
                        ErrorType::UnsupportedOpTypes => (this.add_unit()?, false),
                        _ => Err(e)?
                    }
                    Ok(i) => match i {
                        Some(addr) => match this.get_data_type(addr)? {
                            ExpressionDataType::Pair => {
                                let (left, _right) = this.get_pair(addr)?;
                                match this.get_data_type(left)? {
                                    ExpressionDataType::Symbol => (addr, true),
                                    _ => (addr, false),
                                }
                            }
                            _ => (addr, false),
                        },
                        // make sure there is a unit value to use
                        None => (this.add_unit()?, false),
                    }
                };

                if is_pair_mapping {
                    let value = this.add_pair((sym_addr, value))?;
                    this.add_to_list(value, true)?;
                } else {
                    this.add_to_list(value, is_associative)?;
                }

                count += Data::Size::one();
            }

            this.end_list().and_then(|r| this.push_register(r))?;
        }
        (ExpressionDataType::Range, ExpressionDataType::Range) => {
            let addr = narrow_range(this, left_addr, right_addr)?;
            this.push_register(addr)?;
        }
        (ExpressionDataType::Slice, ExpressionDataType::Range) => {
            // create new slice by narrowing this give range
            let (value, slice_range) = this.get_slice(left_addr)?;
            let range_addr = narrow_range(this, slice_range, right_addr)?;
            let addr = this.add_slice(value, range_addr)?;
            this.push_register(addr)?;
        }
        (ExpressionDataType::List, ExpressionDataType::Range)
        | (ExpressionDataType::CharList, ExpressionDataType::Range)
        | (ExpressionDataType::ByteList, ExpressionDataType::Range)
        | (ExpressionDataType::Link, ExpressionDataType::Range) => {
            // create slice
            let addr = this.add_slice(left_addr, right_addr)?;
            this.push_register(addr)?;
        }
        (ExpressionDataType::List, _) | (ExpressionDataType::CharList, _) | (ExpressionDataType::ByteList, _) | (ExpressionDataType::Range, _) => {
            match get_access_addr(this, right_addr, left_addr)? {
                None => push_unit(this)?,
                Some(i) => this.push_register(i)?,
            }
        }
        (l, r) => match context {
            None => push_unit(this)?,
            Some(c) => if !c.defer_op(this,instruction, (l, left_addr), (r, right_addr))? {
                push_unit(this)?
            }
        }
    }

    this.set_instruction_cursor(next_instruction)?;

    Ok(())
}

pub(crate) fn narrow_range<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    to_narrow: Data::Size,
    by: Data::Size,
) -> Result<Data::Size, RuntimeError<Data::Error>> {
    let (start, end) = this.get_range(by)?;
    let (old_start, _) = this.get_range(to_narrow)?;

    match (this.get_data_type(start)?, this.get_data_type(end)?, this.get_data_type(old_start)?) {
        (ExpressionDataType::Number, ExpressionDataType::Number, ExpressionDataType::Number) => {
            let (start_int, end_int, old_start_int) = (this.get_number(start)?, this.get_number(end)?, this.get_number(old_start)?);

            match (old_start_int.plus(start_int), end_int.subtract(start_int)) {
                (Some(new_start), Some(adjusted_end)) => {
                    // end is always len away from start
                    // offset end by same amount as start
                    match new_start.plus(adjusted_end) {
                        Some(new_end) => {
                            let start_addr = this.add_number(new_start)?;
                            let end_addr = this.add_number(new_end)?;
                            let range_addr = this.add_range(start_addr, end_addr)?;

                            Ok(range_addr)
                        }
                        _ => state_error(format!("Could not narrow range."))?,
                    }
                }
                _ => state_error(format!("Could not narrow range."))?,
            }
        }
        (s1, e1, s2) => state_error(format!(
            "Attempting to create slice from slice with an invalid range. Slice range starting with {:?}. Range {:?} {:?}",
            s2, s1, e1
        ))?,
    }
}

#[cfg(test)]
mod tests {
    use crate::simple::{symbol_value, DataError};
    use crate::{
        runtime::{
            context::{EmptyContext, GarnishLangRuntimeContext},
            GarnishRuntime,
        },
        ExpressionDataType, GarnishLangRuntimeData, Instruction, RuntimeError, SimpleRuntimeData,
    };
    use crate::testing_utilites::{DeferOpTestContext, DEFERRED_VALUE};

    #[test]
    fn deferred() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10).unwrap();
        let int2 = runtime.add_number(20).unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        let mut context = DeferOpTestContext::new();
        runtime.apply(Some(&mut context)).unwrap();

        assert_eq!(runtime.get_external(runtime.get_register(0).unwrap()).unwrap(), DEFERRED_VALUE);
    }

    #[test]
    fn apply() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10).unwrap();
        let exp1 = runtime.add_expression(0).unwrap();
        let int2 = runtime.add_number(20).unwrap();

        // 1
        let i1 = runtime.push_instruction(Instruction::Put, Some(int1)).unwrap();
        runtime.push_instruction(Instruction::PutValue, None).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();
        runtime.push_instruction(Instruction::EndExpression, None).unwrap();

        // 5
        runtime.push_instruction(Instruction::Put, Some(exp1)).unwrap();
        runtime.push_instruction(Instruction::Put, Some(int2)).unwrap();
        let i2 = runtime.push_instruction(Instruction::Apply, None).unwrap();
        let i3 = runtime.push_instruction(Instruction::EndExpression, None).unwrap();

        runtime.push_jump_point(i1).unwrap();

        runtime.push_register(exp1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.set_instruction_cursor(i2).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_value(0).unwrap(), int2);
        assert_eq!(runtime.get_instruction_cursor(), i1);
        assert_eq!(runtime.get_jump_path(0).unwrap(), i3);
    }

    #[test]
    fn apply_integer_to_list() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_number(10).unwrap();
        let d2 = runtime.add_number(20).unwrap();
        let d3 = runtime.add_number(30).unwrap();
        runtime.start_list(3).unwrap();
        runtime.add_to_list(d1, false).unwrap();
        runtime.add_to_list(d2, false).unwrap();
        runtime.add_to_list(d3, false).unwrap();
        let d4 = runtime.end_list().unwrap();
        let d5 = runtime.add_number(2).unwrap();

        runtime.push_instruction(Instruction::Add, None).unwrap();
        let i1 = runtime.push_instruction(Instruction::Apply, None).unwrap();
        let i2 = runtime.push_instruction(Instruction::Add, None).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();

        runtime.set_instruction_cursor(i1).unwrap();

        runtime.push_register(d4).unwrap();
        runtime.push_register(d5).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_number(runtime.get_register(0).unwrap()).unwrap(), 30);
        assert_eq!(runtime.get_instruction_cursor(), i2);
    }

    #[test]
    fn apply_integer_to_char_list() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.start_char_list().unwrap();
        runtime.add_to_char_list('a').unwrap();
        runtime.add_to_char_list('b').unwrap();
        runtime.add_to_char_list('c').unwrap();
        let d1 = runtime.end_char_list().unwrap();
        let d2 = runtime.add_number(2).unwrap();
        let start = runtime.get_data_len();

        runtime.push_instruction(Instruction::Add, None).unwrap();
        let i1 = runtime.push_instruction(Instruction::Apply, None).unwrap();
        let i2 = runtime.push_instruction(Instruction::Add, None).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();

        runtime.set_instruction_cursor(i1).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), start);
        assert_eq!(runtime.get_char(start).unwrap(), 'c');
        assert_eq!(runtime.get_instruction_cursor(), i2);
    }

    #[test]
    fn apply_integer_to_byte_list() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.start_byte_list().unwrap();
        runtime.add_to_byte_list(10).unwrap();
        runtime.add_to_byte_list(20).unwrap();
        runtime.add_to_byte_list(30).unwrap();
        let d1 = runtime.end_byte_list().unwrap();
        let d2 = runtime.add_number(2).unwrap();
        let start = runtime.get_data_len();

        runtime.push_instruction(Instruction::Add, None).unwrap();
        let i1 = runtime.push_instruction(Instruction::Apply, None).unwrap();
        let i2 = runtime.push_instruction(Instruction::Add, None).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();

        runtime.set_instruction_cursor(i1).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), start);
        assert_eq!(runtime.get_byte(start).unwrap(), 30);
        assert_eq!(runtime.get_instruction_cursor(), i2);
    }

    #[test]
    fn range_with_integer() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_number(10).unwrap();
        let i2 = runtime.add_number(20).unwrap();
        let i3 = runtime.add_range(i1, i2).unwrap();
        let i4 = runtime.add_number(5).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i3).unwrap();
        runtime.push_register(i4).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_number(runtime.get_register(0).unwrap()).unwrap(), 15);
    }

    #[test]
    fn apply_symbol_to_list() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol(SimpleRuntimeData::parse_symbol("val1").unwrap()).unwrap();
        let i2 = runtime.add_number(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();

        let i4 = runtime.add_symbol(SimpleRuntimeData::parse_symbol("val2").unwrap()).unwrap();
        let i5 = runtime.add_number(20).unwrap();
        let i6 = runtime.add_pair((i4, i5)).unwrap();

        let i7 = runtime.add_symbol(SimpleRuntimeData::parse_symbol("val3").unwrap()).unwrap();
        let i8 = runtime.add_number(30).unwrap();
        let i9 = runtime.add_pair((i7, i8)).unwrap();

        runtime.start_list(3).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        runtime.add_to_list(i6, true).unwrap();
        runtime.add_to_list(i9, true).unwrap();
        let i10 = runtime.end_list().unwrap();

        let i11 = runtime.add_symbol(SimpleRuntimeData::parse_symbol("val2").unwrap()).unwrap();

        runtime.push_register(i10).unwrap();
        runtime.push_register(i11).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_number(runtime.get_register(0).unwrap()).unwrap(), 20);
    }

    #[test]
    fn apply_list_of_sym_or_integer_to_list() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol(SimpleRuntimeData::parse_symbol("val1").unwrap()).unwrap();
        let i2 = runtime.add_number(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();

        let i4 = runtime.add_symbol(SimpleRuntimeData::parse_symbol("val2").unwrap()).unwrap();
        let i5 = runtime.add_number(20).unwrap();
        let i6 = runtime.add_pair((i4, i5)).unwrap();

        let i7 = runtime.add_symbol(SimpleRuntimeData::parse_symbol("val3").unwrap()).unwrap();
        let i8 = runtime.add_number(30).unwrap();
        let i9 = runtime.add_pair((i7, i8)).unwrap();

        runtime.start_list(3).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        runtime.add_to_list(i6, true).unwrap();
        runtime.add_to_list(i9, true).unwrap();
        let i10 = runtime.end_list().unwrap();

        let i11 = runtime.add_number(2).unwrap(); // integer access
        let i12 = runtime.add_symbol(SimpleRuntimeData::parse_symbol("val2").unwrap()).unwrap(); // symbol access
        let i13 = runtime.add_number(5).unwrap(); // access out of bounds
        let i14 = runtime.add_expression(10).unwrap(); // invalid access type

        let i15 = runtime.add_symbol(SimpleRuntimeData::parse_symbol("new_key").unwrap()).unwrap();
        let i16 = runtime.add_symbol(SimpleRuntimeData::parse_symbol("val1").unwrap()).unwrap();
        let i17 = runtime.add_pair((i15, i16)).unwrap(); // pair mapping
        runtime.start_list(2).unwrap();
        runtime.add_to_list(i11, false).unwrap();
        runtime.add_to_list(i12, false).unwrap();
        runtime.add_to_list(i13, false).unwrap();
        runtime.add_to_list(i14, false).unwrap();
        runtime.add_to_list(i17, true).unwrap();
        let i18 = runtime.end_list().unwrap();

        runtime.push_register(i10).unwrap();
        runtime.push_register(i18).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        let addr = runtime.pop_register().unwrap();
        let len = runtime.get_list_len(addr).unwrap();
        let association_len = runtime.get_list_associations_len(addr).unwrap();
        assert_eq!(len, 5);
        assert_eq!(association_len, 2);

        let pair_addr = runtime.get_list_item(addr, 0).unwrap();
        assert_eq!(runtime.get_data_type(pair_addr).unwrap(), ExpressionDataType::Pair);

        let (pair_left, pair_right) = runtime.get_pair(pair_addr).unwrap();
        assert_eq!(runtime.get_data_type(pair_left).unwrap(), ExpressionDataType::Symbol);
        assert_eq!(runtime.get_symbol(pair_left).unwrap(), symbol_value("val3"));
        assert_eq!(runtime.get_data_type(pair_right).unwrap(), ExpressionDataType::Number);
        assert_eq!(runtime.get_number(pair_right).unwrap(), 30);

        let association_value1 = runtime.get_list_item_with_symbol(addr, symbol_value("val3")).unwrap().unwrap();
        assert_eq!(runtime.get_number(association_value1).unwrap(), 30);

        let int_addr = runtime.get_list_item(addr, 1).unwrap();
        assert_eq!(runtime.get_data_type(int_addr).unwrap(), ExpressionDataType::Number);
        assert_eq!(runtime.get_number(int_addr).unwrap(), 20);

        let unit1 = runtime.get_list_item(addr, 2).unwrap();
        let unit2 = runtime.get_list_item(addr, 3).unwrap();

        assert_eq!(runtime.get_data_type(unit1).unwrap(), ExpressionDataType::Unit);
        assert_eq!(runtime.get_data_type(unit2).unwrap(), ExpressionDataType::Unit);

        let map_pair_addr = runtime.get_list_item(addr, 4).unwrap();
        assert_eq!(runtime.get_data_type(map_pair_addr).unwrap(), ExpressionDataType::Pair);

        let (map_pair_left, map_pair_right) = runtime.get_pair(map_pair_addr).unwrap();
        assert_eq!(runtime.get_data_type(map_pair_left).unwrap(), ExpressionDataType::Symbol);
        assert_eq!(runtime.get_symbol(map_pair_left).unwrap(), symbol_value("new_key"));
        assert_eq!(runtime.get_data_type(map_pair_right).unwrap(), ExpressionDataType::Number);
        assert_eq!(runtime.get_number(map_pair_right).unwrap(), 10);
    }

    #[test]
    fn apply_with_unsupported_left_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10).unwrap();
        let int2 = runtime.add_number(20).unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), 0usize);
    }

    #[test]
    fn apply_no_references_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_number(10).unwrap();
        runtime.add_expression(0).unwrap();
        runtime.add_number(20).unwrap();

        // 1
        runtime.push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.push_instruction(Instruction::PutValue, None).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();
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

        let int1 = runtime.add_number(10).unwrap();
        let exp1 = runtime.add_expression(0).unwrap();

        // 1
        let i1 = runtime.push_instruction(Instruction::Put, Some(int1)).unwrap();
        runtime.push_instruction(Instruction::PutValue, None).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();
        runtime.push_instruction(Instruction::EndExpression, None).unwrap();

        // 5
        runtime.push_instruction(Instruction::Put, Some(exp1)).unwrap();
        let i2 = runtime.push_instruction(Instruction::EmptyApply, None).unwrap();
        let i3 = runtime.push_instruction(Instruction::EndExpression, None).unwrap();

        runtime.push_jump_point(i1).unwrap();

        runtime.push_register(exp1).unwrap();

        runtime.set_instruction_cursor(i2).unwrap();

        runtime.empty_apply::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_value(0).unwrap()).unwrap(), ExpressionDataType::Unit);
        assert_eq!(runtime.get_instruction_cursor(), i1);
        assert_eq!(runtime.get_jump_path(0).unwrap(), i3);
    }

    #[test]
    fn empty_apply_no_references_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_number(10).unwrap();
        runtime.add_expression(0).unwrap();

        // 1
        runtime.push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.push_instruction(Instruction::PutValue, None).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();
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
        let int1 = runtime.add_number(20).unwrap();
        let _int2 = runtime.add_number(30).unwrap();
        let int3 = runtime.add_number(40).unwrap();

        // 1
        runtime.push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.push_instruction(Instruction::Put, Some(2)).unwrap();
        runtime.push_instruction(Instruction::Apply, None).unwrap();

        // 4
        let i1 = runtime.push_instruction(Instruction::Put, Some(0)).unwrap();
        runtime.push_instruction(Instruction::PutValue, None).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();
        runtime.push_instruction(Instruction::PutValue, None).unwrap();
        let i2 = runtime.push_instruction(Instruction::Reapply, Some(0)).unwrap();
        runtime.push_instruction(Instruction::EndExpression, None).unwrap();

        runtime.push_jump_point(i1).unwrap();

        runtime.push_register(true1).unwrap();
        runtime.push_register(int3).unwrap();

        runtime.push_value_stack(int1).unwrap();

        runtime.set_instruction_cursor(i2).unwrap();

        runtime.reapply(0).unwrap();

        assert_eq!(runtime.get_value_stack_len(), 1);
        assert_eq!(runtime.get_value(0).unwrap(), int3);
        assert_eq!(runtime.get_instruction_cursor(), i1);
    }

    #[test]
    fn reapply_if_false() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_false().unwrap();
        runtime.add_expression(0).unwrap();
        runtime.add_number(20).unwrap();
        runtime.add_number(30).unwrap();
        runtime.add_number(40).unwrap();

        // 1
        runtime.push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.push_instruction(Instruction::Put, Some(2)).unwrap();
        runtime.push_instruction(Instruction::Apply, None).unwrap();

        // 4
        runtime.push_instruction(Instruction::Put, Some(0)).unwrap();
        runtime.push_instruction(Instruction::PutValue, None).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();
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
        let int1 = runtime.add_number(100).unwrap();

        runtime.push_instruction(Instruction::Add, None).unwrap();
        let i1 = runtime.push_instruction(Instruction::Apply, None).unwrap();
        let i2 = runtime.push_instruction(Instruction::Add, None).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();

        runtime.push_register(ext1).unwrap();
        runtime.push_register(int1).unwrap();

        runtime.set_instruction_cursor(i1).unwrap();

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
                    ExpressionDataType::Number => runtime.get_number(input_addr)?,
                    _ => return Ok(false),
                };

                self.new_addr = runtime.add_number(value * 2)?;
                runtime.push_register(self.new_addr)?;

                Ok(true)
            }
        }

        let mut context = MyContext { new_addr: 0 };

        runtime.apply(Some(&mut context)).unwrap();

        assert_eq!(runtime.get_number(context.new_addr).unwrap(), 200);
        assert_eq!(runtime.get_register(0).unwrap(), context.new_addr);
        assert_eq!(runtime.get_instruction_cursor(), i2);
    }
}

#[cfg(test)]
mod slices {
    use crate::testing_utilites::{add_list, add_range};
    use crate::{
        runtime::{context::EmptyContext, GarnishRuntime},
        GarnishLangRuntimeData, SimpleRuntimeData,
    };

    #[test]
    fn create_with_list() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_list(&mut runtime, 10);
        let d2 = add_range(&mut runtime, 1, 5);

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        let (list, range) = runtime.get_slice(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(list, d1);
        assert_eq!(range, d2);
    }

    #[test]
    fn create_with_slice() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_list(&mut runtime, 10);
        let d2 = add_range(&mut runtime, 1, 8);
        let d3 = runtime.add_slice(d1, d2).unwrap();
        let d4 = add_range(&mut runtime, 2, 4);

        runtime.push_register(d3).unwrap();
        runtime.push_register(d4).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        let (list, range) = runtime.get_slice(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(list, d1);

        let (start, end) = runtime.get_range(range).unwrap();
        assert_eq!(runtime.get_number(start).unwrap(), 3);
        assert_eq!(runtime.get_number(end).unwrap(), 5);
    }

    #[test]
    fn create_with_char_list() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.start_char_list().unwrap();
        runtime.add_to_char_list('a').unwrap();
        runtime.add_to_char_list('b').unwrap();
        runtime.add_to_char_list('c').unwrap();
        let d1 = runtime.end_char_list().unwrap();
        let d2 = add_range(&mut runtime, 1, 5);

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        let (list, range) = runtime.get_slice(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(list, d1);
        assert_eq!(range, d2);
    }

    #[test]
    fn create_with_byte_list() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.start_byte_list().unwrap();
        runtime.add_to_byte_list(10).unwrap();
        runtime.add_to_byte_list(20).unwrap();
        runtime.add_to_byte_list(30).unwrap();
        let d1 = runtime.end_byte_list().unwrap();
        let d2 = add_range(&mut runtime, 1, 5);

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        let (list, range) = runtime.get_slice(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(list, d1);
        assert_eq!(range, d2);
    }

    #[test]
    fn apply_range_to_range_narrows_it() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_range(&mut runtime, 5, 15);
        let d2 = add_range(&mut runtime, 1, 9);

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        let (start, end) = runtime.get_range(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(runtime.get_number(start).unwrap(), 6);
        assert_eq!(runtime.get_number(end).unwrap(), 14);
    }

    #[test]
    fn create_with_link() {
        let mut runtime = SimpleRuntimeData::new();

        let unit = runtime.add_unit().unwrap();
        let d1 = runtime.add_number(10).unwrap();
        let d2 = runtime.add_link(d1, unit, true).unwrap();

        let d3 = add_range(&mut runtime, 1, 5);

        runtime.push_register(d2).unwrap();
        runtime.push_register(d3).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        let (list, range) = runtime.get_slice(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(list, d2);
        assert_eq!(range, d3);
    }
}
