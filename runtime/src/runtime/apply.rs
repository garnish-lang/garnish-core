use crate::runtime::error::state_error;
use crate::runtime::utilities::*;
use garnish_lang_traits::{GarnishContext, GarnishData, GarnishDataType, GarnishNumber, Instruction, RuntimeError, TypeConstants};
use log::trace;
use crate::runtime::list::{access_with_integer, access_with_symbol};

pub(crate) fn apply<Data: GarnishData, T: GarnishContext<Data>>(
    this: &mut Data,
    context: Option<&mut T>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    apply_internal(this, Instruction::Apply, context)
}

pub(crate) fn reapply<Data: GarnishData>(this: &mut Data, index: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let value_addr = next_ref(this)?;

    let next_instruction = match this.get_jump_point(index.clone()) {
        None => state_error(format!("No jump point at index {:?}", index))?,
        Some(i) => i,
    };

    trace!("Reapplying jumping to instruction at point {:?}", next_instruction);

    match this.pop_value_stack() {
        None => state_error(format!("Failed to pop input during reapply operation."))?,
        Some(v) => {
            trace!("Popped from value stack value {:?}", v)
        }
    }

    trace!("Pushing to value stack value {:?}", value_addr);
    this.push_value_stack(value_addr)?;

    Ok(Some(next_instruction))
}

pub(crate) fn empty_apply<Data: GarnishData, T: GarnishContext<Data>>(
    this: &mut Data,
    context: Option<&mut T>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    push_unit(this)?;
    apply_internal(this, Instruction::EmptyApply, context)
}

fn apply_internal<Data: GarnishData, T: GarnishContext<Data>>(
    this: &mut Data,
    instruction: Instruction,
    context: Option<&mut T>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let right_addr = next_ref(this)?;
    let left_addr = next_ref(this)?;

    // currently, apply is responsible for advancing the instruction cursor itself
    // assume default, apply expression will update to its value
    let mut next_instruction = this.get_instruction_cursor() + Data::Size::one();

    match (this.get_data_type(left_addr.clone())?, this.get_data_type(right_addr.clone())?) {
        (GarnishDataType::Expression, _) => {
            let expression_index = this.get_expression(left_addr)?;

            // Expression stores index of expression table, look up actual instruction index
            let n = match this.get_jump_point(expression_index.clone()) {
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
        (GarnishDataType::External, _) => {
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
        (GarnishDataType::Range, GarnishDataType::Range) => {
            let addr = narrow_range(this, left_addr, right_addr)?;
            this.push_register(addr)?;
        }
        (GarnishDataType::Slice, GarnishDataType::Range) => {
            // create new slice by narrowing this give range
            let (value, slice_range) = this.get_slice(left_addr)?;
            let range_addr = narrow_range(this, slice_range, right_addr)?;
            let addr = this.add_slice(value, range_addr)?;
            this.push_register(addr)?;
        }
        (GarnishDataType::List, GarnishDataType::Number) => {
            let num = this.get_number(right_addr)?;
            match access_with_integer(this, num, left_addr)? {
                None => push_unit(this)?,
                Some(i) => this.push_register(i)?
            }
        }
        (GarnishDataType::List, GarnishDataType::Symbol) => {
            let sym = this.get_symbol(right_addr)?;
            match access_with_symbol(this, sym, left_addr)? {
                None => push_unit(this)?,
                Some(i) => this.push_register(i)?
            }
        }
        (GarnishDataType::List, GarnishDataType::Range)
        | (GarnishDataType::Concatenation, GarnishDataType::Range)
        | (GarnishDataType::CharList, GarnishDataType::Range)
        | (GarnishDataType::ByteList, GarnishDataType::Range) => {
            // create slice
            let addr = this.add_slice(left_addr, right_addr)?;
            this.push_register(addr)?;
        }
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

pub(crate) fn narrow_range<Data: GarnishData>(
    this: &mut Data,
    to_narrow: Data::Size,
    by: Data::Size,
) -> Result<Data::Size, RuntimeError<Data::Error>> {
    let (start, end) = this.get_range(by)?;
    let (old_start, _) = this.get_range(to_narrow)?;

    match (
        this.get_data_type(start.clone())?,
        this.get_data_type(end.clone())?,
        this.get_data_type(old_start.clone())?,
    ) {
        (GarnishDataType::Number, GarnishDataType::Number, GarnishDataType::Number) => {
            let (start_int, end_int, old_start_int) = (this.get_number(start)?, this.get_number(end)?, this.get_number(old_start)?);

            match (old_start_int.plus(start_int.clone()), end_int.subtract(start_int)) {
                (Some(new_start), Some(adjusted_end)) => {
                    // end is always len away from start
                    // offset end by same amount as start
                    match new_start.clone().plus(adjusted_end.clone()) {
                        Some(new_end) => {
                            let start_addr = this.add_number(new_start.clone())?;
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
    use crate::runtime::tests::MockGarnishData;
    use garnish_lang_traits::{GarnishDataType, NO_CONTEXT};
    use crate::runtime::apply::apply;

    struct StackData {
        addrs: Vec<i32>,
    }

    impl Default for StackData {
        fn default() -> Self {
            StackData { addrs: vec![] }
        }
    }

    #[test]
    fn apply_integer_to_list() {
        let mut mock_data = MockGarnishData::default_with_data(StackData { addrs: vec![10, 20] });

        mock_data.stub_get_instruction_cursor = |_| 0;
        mock_data.stub_pop_register = |data| Ok(Some(data.addrs.pop().unwrap()));
        mock_data.stub_get_data_type = |_, i| Ok(if i == 10 { GarnishDataType::List } else { GarnishDataType::Number });
        mock_data.stub_get_number = |_, i| {
            assert_eq!(i, 20);
            Ok(0)
        };
        mock_data.stub_get_list_len = |_, _| Ok(1);
        mock_data.stub_get_list_item = |_, list, i| {
            assert_eq!(list, 10);
            assert_eq!(i, 0);
            Ok(30)
        };
        mock_data.stub_push_register = |_, i| {
            assert_eq!(i, 30);
            Ok(())
        };

        let result = apply(&mut mock_data, NO_CONTEXT).unwrap();

        assert_eq!(result, Some(1));
    }

    #[test]
    fn apply_symbol_to_list() {
        let mut mock_data = MockGarnishData::default_with_data(StackData { addrs: vec![10, 20] });

        mock_data.stub_get_instruction_cursor = |_| 0;
        mock_data.stub_pop_register = |data| Ok(Some(data.addrs.pop().unwrap()));
        mock_data.stub_get_data_type = |_, i| Ok(if i == 10 { GarnishDataType::List } else { GarnishDataType::Symbol });
        mock_data.stub_get_symbol = |_, i| {
            assert_eq!(i, 20);
            Ok(0)
        };
        mock_data.stub_get_list_item_with_symbol = |_, list, i| {
            assert_eq!(list, 10);
            assert_eq!(i, 0);
            Ok(Some(30))
        };
        mock_data.stub_push_register = |_, i| {
            assert_eq!(i, 30);
            Ok(())
        };

        let result = apply(&mut mock_data, NO_CONTEXT).unwrap();

        assert_eq!(result, Some(1));
    }
}
