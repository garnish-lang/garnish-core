use log::trace;
use crate::runtime::list::get_access_addr;
use crate::runtime::utilities::*;
use crate::{state_error, ErrorType, ExpressionDataType, GarnishLangRuntimeData, GarnishNumber, RuntimeError, TypeConstants};
use garnish_traits::Instruction;

use super::context::GarnishLangRuntimeContext;

pub(crate) fn apply<Data: GarnishLangRuntimeData, T: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut T>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    apply_internal(this, Instruction::Apply, context)
}

pub(crate) fn reapply<Data: GarnishLangRuntimeData>(this: &mut Data, index: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let (right_addr, left_addr) = next_two_raw_ref(this)?;

    let mut next_instruction = this.get_instruction_cursor() + Data::Size::one();
    // only execute if left side is a true like value
    match this.get_data_type(left_addr)? {
        ExpressionDataType::Unit | ExpressionDataType::False => {
            trace!("Left is false, not reapplying");
            trace!("Next instruction will be {:?}", this.get_instruction_cursor());
            match this.get_current_value() {
                None => push_unit(this)?,
                Some(i) => this.push_register(i)?,
            }
        },
        _ => {
            let point = match this.get_jump_point(index) {
                None => state_error(format!("No jump point at index {:?}", index))?,
                Some(i) => i,
            };

            trace!("Reapplying jumping to instruction at point {:?}", point);

            next_instruction = point;
            match this.pop_value_stack() {
                None => state_error(format!("Failed to pop input during reapply operation."))?,
                Some(v) => {
                    trace!("Popped from value stack value {:?}", v)
                },
            }

            trace!("Pushing to value stack value {:?}", right_addr);
            this.push_value_stack(right_addr)?;
        }
    }

    Ok(Some(next_instruction))
}

pub(crate) fn empty_apply<Data: GarnishLangRuntimeData, T: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut T>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    push_unit(this)?;
    apply_internal(this, Instruction::EmptyApply, context)
}

pub(crate) fn apply_internal<Data: GarnishLangRuntimeData, T: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    instruction: Instruction,
    context: Option<&mut T>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let right_addr = next_ref(this)?;
    let left_addr = next_ref(this)?;

    // currently, apply is responsible for advancing the instruction cursor itself
    // assume default, apply expression will update to its value
    let mut next_instruction = this.get_instruction_cursor() + Data::Size::one();

    match (this.get_data_type(left_addr)?, this.get_data_type(right_addr)?) {
        (ExpressionDataType::Expression, _) => {
            let expression_index = this.get_expression(left_addr)?;

            // Expression stores index of expression table, look up actual instruction index
            let n = match this.get_jump_point(expression_index) {
                None => state_error(format!("No jump point at index {:?}", expression_index))?,
                Some(i) => i,
            };

            trace!("Next instruction will be {:?}", n);

            next_instruction = n;

            this.push_jump_path(this.get_instruction_cursor() + Data::Size::one())?;

            trace!("Pushing point to jump path {:?} + 1", this.get_instruction_cursor());

            trace!("Pushing to value stack {:?}", right_addr);
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
                        _ => Err(e)?,
                    },
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
                    },
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
        | (ExpressionDataType::Concatenation, ExpressionDataType::Range)
        | (ExpressionDataType::CharList, ExpressionDataType::Range)
        | (ExpressionDataType::ByteList, ExpressionDataType::Range) => {
            // create slice
            let addr = this.add_slice(left_addr, right_addr)?;
            this.push_register(addr)?;
        }
        (ExpressionDataType::List, ExpressionDataType::Number)
        | (ExpressionDataType::List, ExpressionDataType::Symbol)
        | (ExpressionDataType::CharList, ExpressionDataType::Number)
        | (ExpressionDataType::CharList, ExpressionDataType::Symbol)
        | (ExpressionDataType::ByteList, ExpressionDataType::Number)
        | (ExpressionDataType::ByteList, ExpressionDataType::Symbol)
        | (ExpressionDataType::Range, ExpressionDataType::Number)
        | (ExpressionDataType::Range, ExpressionDataType::Symbol)
        | (ExpressionDataType::Concatenation, ExpressionDataType::Number)
        | (ExpressionDataType::Concatenation, ExpressionDataType::Symbol)
        | (ExpressionDataType::Slice, ExpressionDataType::Number)
        | (ExpressionDataType::Slice, ExpressionDataType::Symbol) => match get_access_addr(this, right_addr, left_addr)? {
            None => push_unit(this)?,
            Some(i) => this.push_register(i)?,
        },
        (l, r) => match context {
            None => push_unit(this)?,
            Some(c) => {
                if !c.defer_op(this, instruction, (l, left_addr), (r, right_addr))? {
                    push_unit(this)?
                }
            }
        },
    }

    Ok(Some(next_instruction))
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
