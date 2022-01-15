use crate::runtime::range::range_len;
use crate::{get_range, next_ref, push_unit, state_error, ExpressionDataType, GarnishLangRuntimeData, RuntimeError, TypeConstants};

pub(crate) fn make_list<Data: GarnishLangRuntimeData>(this: &mut Data, len: Data::Size) -> Result<(), RuntimeError<Data::Error>> {
    if len > this.get_register_len() {
        state_error(format!("Not enough register values to make list of length {:?}", len))?
    }

    this.start_list(len)?;

    let mut count = this.get_register_len() - len;
    let end = this.get_register_len();
    // look into getting this to work with a range value
    while count < end {
        let r = match this.get_register(count) {
            None => state_error(format!("No register value at {:?} when making list", count))?,
            Some(r) => r,
        };

        let is_associative = match this.get_data_type(r)? {
            ExpressionDataType::Pair => {
                let (left, _right) = this.get_pair(r)?;
                match this.get_data_type(left)? {
                    ExpressionDataType::Symbol => true,
                    _ => false,
                }
            }
            _ => false,
        };

        this.add_to_list(r, is_associative)?;

        count += Data::Size::one();
    }

    // remove used registers
    count = Data::Size::zero();
    while count < len {
        this.pop_register();
        count += Data::Size::one();
    }

    this.end_list().and_then(|r| this.push_register(r))?;

    Ok(())
}

pub(crate) fn access<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    let right_ref = next_ref(this)?;
    let left_ref = next_ref(this)?;

    match get_access_addr(this, right_ref, left_ref)? {
        None => push_unit(this)?,
        Some(i) => this.push_register(i)?,
    }

    Ok(())
}

pub(crate) fn get_access_addr<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    right: Data::Size,
    left: Data::Size,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    match this.get_data_type(right)? {
        ExpressionDataType::Integer => {
            let i = this.get_integer(right)?;
            access_with_integer(this, i, left)
        }
        ExpressionDataType::Symbol => {
            let sym = this.get_symbol(right)?;
            access_with_symbol(this, sym, left)
        }
        _ => Ok(None),
    }
}

pub(crate) fn access_with_integer<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    index: Data::Integer,
    value: Data::Size,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    match this.get_data_type(value)? {
        ExpressionDataType::List => {
            if index < Data::Integer::zero() {
                Ok(None)
            } else {
                let i = index;
                if i >= Data::size_to_integer(this.get_list_len(value)?) {
                    Ok(None)
                } else {
                    Ok(Some(this.get_list_item(value, i)?))
                }
            }
        }
        ExpressionDataType::CharList => {
            if index < Data::Integer::zero() {
                Ok(None)
            } else {
                let i = index;
                if i >= Data::size_to_integer(this.get_char_list_len(value)?) {
                    Ok(None)
                } else {
                    let c = this.get_char_list_item(value, i)?;
                    let addr = this.add_char(c)?;
                    Ok(Some(addr))
                }
            }
        }
        ExpressionDataType::ByteList => {
            if index < Data::Integer::zero() {
                Ok(None)
            } else {
                let i = index;
                if i >= Data::size_to_integer(this.get_byte_list_len(value)?) {
                    Ok(None)
                } else {
                    let c = this.get_byte_list_item(value, i)?;
                    let addr = this.add_byte(c)?;
                    Ok(Some(addr))
                }
            }
        }
        ExpressionDataType::Range => {
            let (start, end) = this.get_range(value)?;
            match (this.get_data_type(start)?, this.get_data_type(end)?) {
                (ExpressionDataType::Integer, ExpressionDataType::Integer) => {
                    let start_int = this.get_integer(start)?;
                    let end_int = this.get_integer(end)?;
                    let len = range_len::<Data>(start_int, end_int);

                    if index >= len {
                        return Ok(None);
                    } else {
                        let result = start_int + index;
                        let addr = this.add_integer(result)?;
                        Ok(Some(addr))
                    }
                }
                _ => Ok(None),
            }
        }
        ExpressionDataType::Slice => {
            let (value, range) = this.get_slice(value)?;
            let (start, end, len) = get_range(this, range)?;

            if index >= len {
                return Ok(None);
            }

            let mut item: Option<Data::Size> = None;
            let mut count = Data::Integer::zero() - start;

            // keep track of starting point to any pushed values later
            let start_register = this.get_register_len();

            this.push_register(value)?;

            while this.get_register_len() > start_register {
                match this.pop_register() {
                    None => state_error(format!("Popping more registers than placed during linking indexing."))?,
                    Some(r) => match this.get_data_type(r)? {
                        ExpressionDataType::Link => {
                            let (val, linked, is_append) = this.get_link(r)?;

                            match this.get_data_type(linked)? {
                                ExpressionDataType::Unit => {
                                    // linked of type unit means, only push value
                                    this.push_register(val)?;
                                }
                                ExpressionDataType::Link => {
                                    if is_append {
                                        // linked value is previous, gets checked first, pushed last
                                        this.push_register(val)?;
                                        this.push_register(linked)?;
                                    } else {
                                        // linked value is next, gets resolved last, pushed first
                                        this.push_register(linked)?;
                                        this.push_register(val)?;
                                    }
                                }
                                t => state_error(format!("Invalid linked type {:?}", t))?,
                            }
                        }
                        ExpressionDataType::Slice => {
                            let (val, range) = this.get_slice(r)?;
                            let (start, end, len) = get_range(this, range)?;

                            count -= start;
                            this.push_register(val)?;
                        }
                        ExpressionDataType::List => {
                            let list_len = Data::size_to_integer(this.get_list_len(r)?);
                            if count + list_len >= index {
                                // item is in list
                                let list_index = index - count;
                                item = Some(this.get_list_item(r, list_index)?);
                                break;
                            } else {
                                // not in list, advance count by list len and continue to next item
                                count += list_len;
                            }
                        }
                        _ => {
                            if count == index {
                                item = Some(r);
                                break;
                            } else {
                                count += Data::Integer::one();
                            }
                        }
                    },
                }
            }

            // remove remaining registers added during operation
            while this.get_register_len() > start_register {
                this.pop_register();
            }

            Ok(item)
        }
        ExpressionDataType::Link => {
            let mut item: Option<Data::Size> = None;
            let mut count = Data::Integer::zero();
            // keep track of starting point to any pushed values later
            let start_register = this.get_register_len();

            this.push_register(value)?;

            while this.get_register_len() > start_register {
                match this.pop_register() {
                    None => state_error(format!("Popping more registers than placed during linking indexing."))?,
                    Some(r) => match this.get_data_type(r)? {
                        ExpressionDataType::Link => {
                            let (val, linked, is_append) = this.get_link(r)?;

                            match this.get_data_type(linked)? {
                                ExpressionDataType::Unit => {
                                    // linked of type unit means, only push value
                                    this.push_register(val)?;
                                }
                                ExpressionDataType::Link => {
                                    if is_append {
                                        // linked value is previous, gets checked first, pushed last
                                        this.push_register(val)?;
                                        this.push_register(linked)?;
                                    } else {
                                        // linked value is next, gets resolved last, pushed first
                                        this.push_register(linked)?;
                                        this.push_register(val)?;
                                    }
                                }
                                t => state_error(format!("Invalid linked type {:?}", t))?,
                            }
                        }
                        ExpressionDataType::Slice => {
                            let (val, range) = this.get_slice(r)?;
                            let (start, end, len) = get_range(this, range)?;

                            count -= start;
                            this.push_register(val)?;
                        }
                        ExpressionDataType::List => {
                            let list_len = Data::size_to_integer(this.get_list_len(r)?);
                            if count + list_len >= index {
                                // item is in list
                                let list_index = index - count;
                                item = Some(this.get_list_item(r, list_index)?);
                                break;
                            } else {
                                // not in list, advance count by list len and continue to next item
                                count += list_len;
                            }
                        }
                        _ => {
                            if count == index {
                                item = Some(r);
                                break;
                            } else {
                                count += Data::Integer::one();
                            }
                        }
                    },
                }
            }

            // remove remaining registers added during operation
            while this.get_register_len() > start_register {
                this.pop_register();
            }

            Ok(item)
        }
        _ => Ok(None),
    }
}

fn access_with_symbol<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    sym: Data::Symbol,
    value: Data::Size,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    match this.get_data_type(value)? {
        ExpressionDataType::List => Ok(this.get_list_item_with_symbol(value, sym)?),
        ExpressionDataType::Slice => {
            let (value, _) = this.get_slice(value)?;
            Ok(this.get_list_item_with_symbol(value, sym)?)
        }
        ExpressionDataType::Link => {
            let (mut value, mut linked, _) = this.get_link(value)?;
            loop {
                match this.get_data_type(value)? {
                    ExpressionDataType::Pair => {
                        let (left, right) = this.get_pair(value)?;
                        match this.get_data_type(left)? {
                            ExpressionDataType::Symbol => {
                                let value_sym = this.get_symbol(left)?;
                                if value_sym == sym {
                                    value = right;
                                    break;
                                } else {
                                    let (next_val, next_linked, _) = this.get_link(linked)?;
                                    value = next_val;
                                    linked = next_linked;
                                }
                            }
                            _ => {
                                let (next_val, next_linked, _) = this.get_link(linked)?;
                                value = next_val;
                                linked = next_linked;
                            }
                        }
                    }
                    _ => {
                        let (next_val, next_linked, _) = this.get_link(linked)?;
                        value = next_val;
                        linked = next_linked;
                    }
                }
            }

            Ok(Some(value))
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

    #[test]
    fn make_list() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let i3 = runtime.add_integer(20).unwrap();
        let start = runtime.get_data_len();

        runtime.push_register(i1).unwrap();
        runtime.push_register(i2).unwrap();
        runtime.push_register(i3).unwrap();

        runtime.push_instruction(Instruction::MakeList, Some(3)).unwrap();

        runtime.make_list(3).unwrap();

        assert_eq!(runtime.get_list_len(start).unwrap(), 3);
        assert_eq!(runtime.get_list_item(start, 0).unwrap(), i1);
        assert_eq!(runtime.get_list_item(start, 1).unwrap(), i2);
        assert_eq!(runtime.get_list_item(start, 2).unwrap(), i3);
    }

    #[test]
    fn make_list_no_refs_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_integer(10).unwrap();
        runtime.add_integer(20).unwrap();
        runtime.add_integer(20).unwrap();

        runtime.push_instruction(Instruction::MakeList, Some(3)).unwrap();

        let result = runtime.make_list(3);

        assert!(result.is_err());
    }

    #[test]
    fn make_list_with_associations() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_symbol("two").unwrap();
        let i4 = runtime.add_integer(20).unwrap();
        let i5 = runtime.add_symbol("three").unwrap();
        let i6 = runtime.add_integer(30).unwrap();
        // 6
        let i7 = runtime.add_pair((i1, i2)).unwrap();
        let i8 = runtime.add_pair((i3, i4)).unwrap();
        let i9 = runtime.add_pair((i5, i6)).unwrap();

        let start = runtime.get_data_len();

        runtime.push_register(i7).unwrap();
        runtime.push_register(i8).unwrap();
        runtime.push_register(i9).unwrap();

        runtime.push_instruction(Instruction::MakeList, Some(3)).unwrap();

        runtime.make_list(3).unwrap();

        assert_eq!(runtime.get_list_len(start).unwrap(), 3);
        assert_eq!(runtime.get_list_item(start, 0).unwrap(), i7);
        assert_eq!(runtime.get_list_item(start, 1).unwrap(), i8);
        assert_eq!(runtime.get_list_item(start, 2).unwrap(), i9);

        assert_eq!(runtime.get_list_associations_len(start).unwrap(), 3);
        assert_eq!(runtime.get_list_association(start, 0).unwrap(), i7);
        assert_eq!(runtime.get_list_association(start, 1).unwrap(), i8);
        assert_eq!(runtime.get_list_association(start, 2).unwrap(), i9);
    }

    #[test]
    fn access() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_symbol("one").unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i4).unwrap();
        runtime.push_register(i5).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), i2);
    }

    #[test]
    fn access_with_integer() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_integer(0).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i4).unwrap();
        runtime.push_register(i5).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), i3);
    }

    #[test]
    fn access_char_list_with_integer() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.start_char_list().unwrap();
        runtime.add_to_char_list('a').unwrap();
        runtime.add_to_char_list('b').unwrap();
        runtime.add_to_char_list('c').unwrap();
        let d1 = runtime.end_char_list().unwrap();
        let d2 = runtime.add_integer(2).unwrap();
        let start = runtime.get_data_len();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), start);
        assert_eq!(runtime.get_char(start).unwrap(), 'c');
    }

    #[test]
    fn access_byte_list_with_integer() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.start_byte_list().unwrap();
        runtime.add_to_byte_list(10).unwrap();
        runtime.add_to_byte_list(20).unwrap();
        runtime.add_to_byte_list(30).unwrap();
        let d1 = runtime.end_byte_list().unwrap();
        let d2 = runtime.add_integer(2).unwrap();
        let start = runtime.get_data_len();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), start);
        assert_eq!(runtime.get_byte(start).unwrap(), 30);
    }

    #[test]
    fn access_with_integer_out_of_bounds_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_integer(10).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i4).unwrap();
        runtime.push_register(i5).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn access_with_number_negative_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_integer(-1).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i4).unwrap();
        runtime.push_register(i5).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn access_non_list_on_left_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_symbol("one").unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i1).unwrap();
        runtime.push_register(i2).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn access_non_symbol_on_right_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_expression(10).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i4).unwrap();
        runtime.push_register(i5).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn access_no_refs_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let _i4 = runtime.end_list().unwrap();
        let _i5 = runtime.add_symbol("one").unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        let result = runtime.access();

        assert!(result.is_err());
    }

    #[test]
    fn access_with_non_existent_key() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_symbol("two").unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i4).unwrap();
        runtime.push_register(i5).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }
}

#[cfg(test)]
mod ranges {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

    #[test]
    fn access_with_integer() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let i3 = runtime.add_range(i1, i2).unwrap();
        let i4 = runtime.add_integer(5).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i3).unwrap();
        runtime.push_register(i4).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), 15);
    }

    #[test]
    fn access_with_integer_out_of_range() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let i3 = runtime.add_range(i1, i2).unwrap();
        let i4 = runtime.add_integer(30).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i3).unwrap();
        runtime.push_register(i4).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }
}

#[cfg(test)]
mod slice {
    use crate::testing_utilites::{add_integer_list, add_links, add_list, add_list_with_start};
    use crate::{runtime::GarnishRuntime, symbol_value, GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn index_slice_of_list() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_integer_list(&mut runtime, 10);
        let d2 = runtime.add_integer(1).unwrap();
        let d3 = runtime.add_integer(4).unwrap();
        let d4 = runtime.add_range(d2, d3).unwrap();
        let d5 = runtime.add_slice(d1, d4).unwrap();
        let d6 = runtime.add_integer(2).unwrap();

        runtime.push_register(d5).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), 40);
    }

    #[test]
    fn sym_index_slice_of_list() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_list(&mut runtime, 10);
        let d2 = runtime.add_integer(1).unwrap();
        let d3 = runtime.add_integer(4).unwrap();
        let d4 = runtime.add_range(d2, d3).unwrap();
        let d5 = runtime.add_slice(d1, d4).unwrap();
        let d6 = runtime.add_symbol("val4").unwrap();

        runtime.push_register(d5).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), 50);
    }

    #[test]
    fn index_slice_of_links() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_links(&mut runtime, 10, true);
        let d2 = runtime.add_integer(2).unwrap();
        let d3 = runtime.add_integer(8).unwrap();
        let d4 = runtime.add_range(d2, d3).unwrap();
        let d5 = runtime.add_slice(d1, d4).unwrap();
        let d6 = runtime.add_integer(2).unwrap();

        runtime.push_register(d5).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.access().unwrap();

        let (left, right) = runtime.get_pair(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(runtime.get_symbol(left).unwrap(), symbol_value("val4"));
        assert_eq!(runtime.get_integer(right).unwrap(), 50);
    }

    #[test]
    fn index_slice_of_links_of_list() {
        let mut runtime = SimpleRuntimeData::new();

        let unit = runtime.add_unit().unwrap();
        let d1 = add_list_with_start(&mut runtime, 5, 10);
        let d2 = add_list_with_start(&mut runtime, 5, 20);

        let d3 = runtime.add_link(d1, unit, true).unwrap();
        let d4 = runtime.add_link(d2, d3, true).unwrap();

        let d5 = runtime.add_integer(2).unwrap();
        let d6 = runtime.add_integer(8).unwrap();
        let d7 = runtime.add_range(d5, d6).unwrap();
        let d8 = runtime.add_slice(d4, d7).unwrap();
        let d9 = runtime.add_integer(2).unwrap();

        runtime.push_register(d8).unwrap();
        runtime.push_register(d9).unwrap();

        runtime.access().unwrap();

        let (left, right) = runtime.get_pair(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(runtime.get_symbol(left).unwrap(), symbol_value("val14"));
        assert_eq!(runtime.get_integer(right).unwrap(), 14);
    }
}

#[cfg(test)]
mod link {
    use crate::testing_utilites::{add_links, add_list_with_start, add_range};
    use crate::{symbol_value, GarnishLangRuntimeData, GarnishRuntime, SimpleRuntimeData};

    #[test]
    fn index_prepend_link_with_integer() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_links(&mut runtime, 10, false);
        let d2 = runtime.add_integer(3).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.access().unwrap();

        let (left, right) = runtime.get_pair(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(runtime.get_symbol(left).unwrap(), symbol_value("val3"));
        assert_eq!(runtime.get_integer(right).unwrap(), 40);
    }

    #[test]
    fn index_append_link_with_integer() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_links(&mut runtime, 10, true);
        let d2 = runtime.add_integer(3).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.access().unwrap();

        let (left, right) = runtime.get_pair(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(runtime.get_symbol(left).unwrap(), symbol_value("val3"));
        assert_eq!(runtime.get_integer(right).unwrap(), 40);
    }

    #[test]
    fn index_prepend_link_with_symbol() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_links(&mut runtime, 10, false);
        let d2 = runtime.add_symbol("val2").unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), 30);
    }

    #[test]
    fn index_append_link_with_symbol() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_links(&mut runtime, 10, true);
        let d2 = runtime.add_symbol("val2").unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), 30);
    }

    #[test]
    fn index_prepend_link_of_lists_with_integer() {
        let mut runtime = SimpleRuntimeData::new();

        let unit = runtime.add_unit().unwrap();
        let d1 = add_list_with_start(&mut runtime, 5, 50);
        let d2 = runtime.add_link(d1, unit, false).unwrap();
        let d3 = add_list_with_start(&mut runtime, 5, 100);
        let d4 = runtime.add_link(d3, d2, false).unwrap();
        let d5 = runtime.add_integer(7).unwrap();

        runtime.push_register(d4).unwrap();
        runtime.push_register(d5).unwrap();

        runtime.access().unwrap();

        let (left, right) = runtime.get_pair(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(runtime.get_symbol(left).unwrap(), symbol_value("val52"));
        assert_eq!(runtime.get_integer(right).unwrap(), 52);
    }

    #[test]
    fn index_append_link_of_lists_with_integer() {
        let mut runtime = SimpleRuntimeData::new();

        let unit = runtime.add_unit().unwrap();
        let d1 = add_list_with_start(&mut runtime, 5, 50);
        let d2 = runtime.add_link(d1, unit, true).unwrap();
        let d3 = add_list_with_start(&mut runtime, 5, 100);
        let d4 = runtime.add_link(d3, d2, true).unwrap();
        let d5 = runtime.add_integer(7).unwrap();

        runtime.push_register(d4).unwrap();
        runtime.push_register(d5).unwrap();

        runtime.access().unwrap();

        let (left, right) = runtime.get_pair(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(runtime.get_symbol(left).unwrap(), symbol_value("val102"));
        assert_eq!(runtime.get_integer(right).unwrap(), 102);
    }

    #[test]
    fn index_link_of_slices_of_list() {
        let mut runtime = SimpleRuntimeData::new();

        let unit = runtime.add_unit().unwrap();
        let d1 = add_list_with_start(&mut runtime, 20, 10);

        let d2 = add_range(&mut runtime, 2, 8);
        let d3 = runtime.add_slice(d1, d2).unwrap();

        let d4 = add_range(&mut runtime, 14, 19);
        let d5 = runtime.add_slice(d1, d4).unwrap();

        let d6 = runtime.add_link(d3, unit, true).unwrap();
        let d7 = runtime.add_link(d5, d6, true).unwrap();

        // resulting list should look like this
        // 0   1   2   3   4   5   6   7   8   9   10  11  12
        // 12, 13, 14, 15, 16, 17, 18, 24, 25, 26, 27, 28, 29

        let d8 = runtime.add_integer(4).unwrap();

        runtime.push_register(d7).unwrap();
        runtime.push_register(d8).unwrap();

        runtime.access().unwrap();

        let (left, right) = runtime.get_pair(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(runtime.get_symbol(left).unwrap(), symbol_value("val16"));
        assert_eq!(runtime.get_integer(right).unwrap(), 16);
    }
}
