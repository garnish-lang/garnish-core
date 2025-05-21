use crate::runtime::error::state_error;
use crate::runtime::list::{access_with_integer, access_with_symbol};
use crate::runtime::utilities::*;
use garnish_lang_traits::{GarnishContext, GarnishData, GarnishDataType, GarnishNumber, Instruction, RuntimeError, TypeConstants};
use log::trace;

pub(crate) fn apply<Data: GarnishData, T: GarnishContext<Data>>(this: &mut Data, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    apply_internal(this, Instruction::Apply, context, true)
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

pub(crate) fn empty_apply<Data: GarnishData, T: GarnishContext<Data>>(this: &mut Data, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    push_unit(this)?;
    apply_internal(this, Instruction::EmptyApply, context, false)
}

fn apply_internal<Data: GarnishData, T: GarnishContext<Data>>(this: &mut Data, instruction: Instruction, context: Option<&mut T>, use_right: bool) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
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
        (GarnishDataType::Partial, _) => {
            let (expression, input) = this.get_partial(left_addr)?;
            match this.get_data_type(expression.clone())? {
                GarnishDataType::Expression => {
                    let value = if use_right {
                        this.add_concatenation(input, right_addr)?
                    } else {
                        input
                    };
                    this.push_value_stack(value)?;

                    let expression = this.get_expression(expression)?;
                    let n = match this.get_jump_point(expression.clone()) {
                        None => state_error(format!("No jump point at index {:?}", expression))?,
                        Some(i) => i,
                    };

                    next_instruction = n;
                    this.push_jump_path(this.get_instruction_cursor() + Data::Size::one())?;
                }
                _ => this.add_unit().and_then(|i| this.push_register(i))?,
            }
        }
        (GarnishDataType::Symbol, GarnishDataType::SymbolList) | (GarnishDataType::SymbolList, GarnishDataType::Symbol) | (GarnishDataType::SymbolList, GarnishDataType::SymbolList) => {
            this.merge_to_symbol_list(left_addr, right_addr).and_then(|i| this.push_register(i))?
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
        (GarnishDataType::SymbolList, GarnishDataType::Number) | (GarnishDataType::List, GarnishDataType::Number) => {
            let num = this.get_number(right_addr)?;
            match access_with_integer(this, num, left_addr)? {
                None => push_unit(this)?,
                Some(i) => this.push_register(i)?,
            }
        }
        (GarnishDataType::List, GarnishDataType::Symbol) => {
            let sym = this.get_symbol(right_addr)?;
            match access_with_symbol(this, sym, left_addr)? {
                None => push_unit(this)?,
                Some(i) => this.push_register(i)?,
            }
        }
        (GarnishDataType::List, GarnishDataType::SymbolList) => {
            let mut iter = this.get_symbol_list_iter(right_addr.clone());
            let mut current = left_addr.clone();
            while let Some(sym_index) = iter.next() {
                let sym = this.get_symbol_list_item(right_addr.clone(), sym_index)?;

                match access_with_symbol(this, sym, current)? {
                    None => {
                        current = this.add_unit()?;
                        break;
                    }
                    Some(i) => current = i,
                }
            }

            this.push_register(current)?;
        }
        (GarnishDataType::List, GarnishDataType::Range)
        | (GarnishDataType::Concatenation, GarnishDataType::Range)
        | (GarnishDataType::CharList, GarnishDataType::Range)
        | (GarnishDataType::ByteList, GarnishDataType::Range)
        | (GarnishDataType::SymbolList, GarnishDataType::Range) => {
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

pub(crate) fn narrow_range<Data: GarnishData>(this: &mut Data, to_narrow: Data::Size, by: Data::Size) -> Result<Data::Size, RuntimeError<Data::Error>> {
    let (start, end) = this.get_range(by)?;
    let (old_start, _) = this.get_range(to_narrow)?;

    match (this.get_data_type(start.clone())?, this.get_data_type(end.clone())?, this.get_data_type(old_start.clone())?) {
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
    use crate::runtime::apply::{apply, empty_apply};
    use crate::runtime::tests::{MockGarnishData, MockIterator};
    use garnish_lang_traits::{GarnishDataType, NO_CONTEXT};

    #[test]
    fn apply_integer_to_list() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::List, GarnishDataType::Number]);

        mock_data.stub_get_number = |_, i| {
            assert_eq!(i, 1);
            Ok(0)
        };
        mock_data.stub_get_list_len = |_, _| Ok(1);
        mock_data.stub_get_list_item = |_, list, i| {
            assert_eq!(list, 0);
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
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::List, GarnishDataType::Symbol]);

        mock_data.stub_get_symbol = |_, i| {
            assert_eq!(i, 1);
            Ok(0)
        };
        mock_data.stub_get_list_item_with_symbol = |_, list, i| {
            assert_eq!(list, 0);
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

    #[test]
    fn apply_symbol_list_to_list() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::List, GarnishDataType::List, GarnishDataType::SymbolList]);

        mock_data.stub_get_symbol_list_iter = |_, _| MockIterator::new(2);
        mock_data.stub_get_symbol_list_item = |_, list, i| {
            assert_eq!(list, 2);
            Ok((i + 1) as u32 * 100u32)
        };
        mock_data.stub_get_list_item_with_symbol = |_, list, i| {
            if i == 100 {
                assert_eq!(list, 1);
                Ok(Some(0))
            } else if i == 200 {
                assert_eq!(list, 0);
                Ok(Some(40))
            } else {
                assert!(false);
                Ok(None)
            }
        };
        mock_data.stub_push_register = |_, i| {
            assert_eq!(i, 40);
            Ok(())
        };

        let result = apply(&mut mock_data, NO_CONTEXT).unwrap();

        assert_eq!(result, Some(1));
    }

    #[test]
    fn extend_symbol_list_from_left() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Symbol, GarnishDataType::SymbolList]);

        mock_data.stub_merge_to_symbol_list = |_, first, second| {
            assert_eq!(first, 0);
            assert_eq!(second, 1);
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
    fn extend_symbol_list_from_right() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::SymbolList, GarnishDataType::Symbol]);

        mock_data.stub_merge_to_symbol_list = |_, first, second| {
            assert_eq!(first, 0);
            assert_eq!(second, 1);
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
    fn merge_symbol_lists() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::SymbolList, GarnishDataType::SymbolList]);

        mock_data.stub_merge_to_symbol_list = |_, first, second| {
            assert_eq!(first, 0);
            assert_eq!(second, 1);
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
    fn apply_symbol_list_with_number() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::SymbolList, GarnishDataType::Number]);

        mock_data.stub_get_number = |_, num| {
            assert_eq!(num, 1);
            Ok(1)
        };
        mock_data.stub_get_symbol_list_len = |_, _| Ok(2);
        mock_data.stub_get_symbol_list_item = |_, _, index| Ok((index + 1) as u32 * 10);
        mock_data.stub_add_symbol = |_, sym| {
            assert_eq!(sym, 20);
            Ok(5)
        };
        mock_data.stub_push_register = |_, i| {
            assert_eq!(i, 5);
            Ok(())
        };

        let result = apply(&mut mock_data, NO_CONTEXT).unwrap();

        assert_eq!(result, Some(1));
    }

    #[test]
    fn apply_partial_non_expression() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Number, GarnishDataType::Partial]);

        mock_data.stub_add_unit = |_| Ok(100);
        mock_data.stub_push_register = |_, i| {
            assert_eq!(i, 100);
            Ok(())
        };

        let result = apply(&mut mock_data, NO_CONTEXT).unwrap();

        assert_eq!(result, Some(1));
    }

    #[test]
    fn apply_partial_expression() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Expression, GarnishDataType::Number, GarnishDataType::Partial, GarnishDataType::Number]);

        mock_data.stub_get_partial = |_, i| {
            assert_eq!(i, 2);
            Ok((0, 1))
        };
        mock_data.stub_get_expression = |_, i| {
            assert_eq!(i, 0);
            Ok(200)
        };
        mock_data.stub_add_concatenation = |_, left, right| {
            assert_eq!(1, left);
            assert_eq!(3, right);
            Ok(100)
        };
        mock_data.stub_get_jump_point = |_, index| {
            assert_eq!(index, 200);
            Some(3000)
        };
        mock_data.stub_push_value_stack = |_, i| {
            assert_eq!(i, 100);
            Ok(())
        };
        mock_data.stub_set_instruction_cursor = |_, addr| {
            assert_eq!(addr, 300);
            Ok(())
        };
        mock_data.stub_push_register = |_, i| {
            assert_eq!(i, 100);
            Ok(())
        };
        mock_data.stub_get_instruction_cursor = |_| 123;
        mock_data.stub_push_jump_path = |_, i| {
            assert_eq!(i, 124);
            Ok(())
        };

        let result = apply(&mut mock_data, NO_CONTEXT).unwrap();

        assert_eq!(result, Some(3000));
    }

    #[test]
    fn empty_apply_partial_expression() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Expression, GarnishDataType::Number, GarnishDataType::Partial, GarnishDataType::Unit]);

        mock_data.stub_add_unit = |_| Ok(100);
        mock_data.stub_get_partial = |_, i| {
            assert_eq!(i, 2);
            Ok((0, 1))
        };
        mock_data.stub_get_expression = |_, i| {
            assert_eq!(i, 0);
            Ok(200)
        };
        mock_data.stub_get_jump_point = |_, index| {
            assert_eq!(index, 200);
            Some(3000)
        };
        mock_data.stub_push_value_stack = |_, i| {
            assert_eq!(i, 1);
            Ok(())
        };
        mock_data.stub_set_instruction_cursor = |_, addr| {
            assert_eq!(addr, 300);
            Ok(())
        };
        mock_data.stub_push_register = |_, i| {
            assert_eq!(i, 100);
            Ok(())
        };
        mock_data.stub_get_instruction_cursor = |_| 123;
        mock_data.stub_push_jump_path = |_, i| {
            assert_eq!(i, 124);
            Ok(())
        };

        let result = empty_apply(&mut mock_data, NO_CONTEXT).unwrap();

        assert_eq!(result, Some(3000));
    }
}

#[cfg(test)]
mod slice {
    use crate::runtime::apply::apply;
    use crate::runtime::tests::MockGarnishData;
    use garnish_lang_traits::{GarnishDataType, NO_CONTEXT};

    #[test]
    fn symbol_list() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::SymbolList, GarnishDataType::Range]);

        mock_data.stub_get_number = |_, num| {
            assert_eq!(num, 1);
            Ok(1)
        };
        mock_data.stub_get_symbol_list_len = |_, _| Ok(2);
        mock_data.stub_get_symbol_list_item = |_, _, index| Ok((index + 1) as u32 * 10);
        mock_data.stub_add_slice = |_, list, range| {
            assert_eq!(list, 0);
            assert_eq!(range, 1);
            Ok(5)
        };
        mock_data.stub_push_register = |_, i| {
            assert_eq!(i, 5);
            Ok(())
        };

        let result = apply(&mut mock_data, NO_CONTEXT).unwrap();

        assert_eq!(result, Some(1));
    }
}
