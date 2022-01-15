use crate::runtime::range::range_len;
use crate::{next_ref, push_unit, state_error, ExpressionDataType, GarnishLangRuntimeData, RuntimeError, TypeConstants};

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
    match (this.get_data_type(left)?, this.get_data_type(right)?) {
        (ExpressionDataType::List, ExpressionDataType::Symbol) => {
            let sym_val = this.get_symbol(right)?;

            Ok(this.get_list_item_with_symbol(left, sym_val)?)
        }
        (ExpressionDataType::List, ExpressionDataType::Integer) => {
            let i = this.get_integer(right)?;

            if i < Data::Integer::zero() {
                Ok(None)
            } else {
                let i = i;
                if i >= Data::size_to_integer(this.get_list_len(left)?) {
                    Ok(None)
                } else {
                    Ok(Some(this.get_list_item(left, i)?))
                }
            }
        }
        (ExpressionDataType::CharList, ExpressionDataType::Integer) => {
            let i = this.get_integer(right)?;

            if i < Data::Integer::zero() {
                Ok(None)
            } else {
                let i = i;
                if i >= Data::size_to_integer(this.get_char_list_len(left)?) {
                    Ok(None)
                } else {
                    let c = this.get_char_list_item(left, i)?;
                    let addr = this.add_char(c)?;
                    Ok(Some(addr))
                }
            }
        }
        (ExpressionDataType::ByteList, ExpressionDataType::Integer) => {
            let i = this.get_integer(right)?;

            if i < Data::Integer::zero() {
                Ok(None)
            } else {
                let i = i;
                if i >= Data::size_to_integer(this.get_byte_list_len(left)?) {
                    Ok(None)
                } else {
                    let c = this.get_byte_list_item(left, i)?;
                    let addr = this.add_byte(c)?;
                    Ok(Some(addr))
                }
            }
        }
        (ExpressionDataType::Range, ExpressionDataType::Integer) => {
            let (start, end) = this.get_range(left)?;
            match (this.get_data_type(start)?, this.get_data_type(end)?) {
                (ExpressionDataType::Integer, ExpressionDataType::Integer) => {
                    let start_int = this.get_integer(start)?;
                    let end_int = this.get_integer(end)?;
                    let index = this.get_integer(right)?;
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
        (ExpressionDataType::Slice, r) => {
            let (value, range) = this.get_slice(left)?;
            match r {
                ExpressionDataType::Symbol => {
                    let sym_val = this.get_symbol(right)?;

                    Ok(this.get_list_item_with_symbol(value, sym_val)?)
                }
                ExpressionDataType::Integer => {
                    let (start, end) = this.get_range(range)?;
                    let (start, end) = match (this.get_data_type(start)?, this.get_data_type(end)?) {
                        (ExpressionDataType::Integer, ExpressionDataType::Integer) => {
                            (this.get_integer(start)?, this.get_integer(end)?)
                        }
                        (s, e) => state_error(format!("Invalid range values {:?} {:?}", s, e))?,
                    };

                    let index = this.get_integer(right)?;
                    if index > end {
                        return Ok(None);
                    }

                    match this.get_data_type(value)? {
                        ExpressionDataType::List => {
                            let i = start + index;

                            if i < Data::Integer::zero() {
                                Ok(None)
                            } else {
                                let i = i;
                                if i >= Data::size_to_integer(this.get_list_len(value)?) {
                                    Ok(None)
                                } else {
                                    Ok(Some(this.get_list_item(value, i)?))
                                }
                            }
                        }
                        _ => Ok(None)
                    }
                }
                _ => Ok(None),
            }
        }
        (ExpressionDataType::Link, r) => {
            // let (mut value, mut linked, is_append) = this.get_link(left)?;

            match r {
                ExpressionDataType::Integer => {
                    let mut link = left;
                    let mut value: Option<Data::Size> = None;

                    let index = this.get_integer(right)?;
                    let mut count = Data::Integer::zero();
                    // keep track of starting point to any pushed values later
                    let mut start_register = this.get_register_len();

                    loop {
                        match this.get_data_type(link)? {
                            ExpressionDataType::Link => {
                                let (next_val, next_linked, is_append) = this.get_link(link)?;
                                if is_append {
                                    // if append link, need to push to registers to get in proper order
                                    this.push_register(link)?;
                                    link = next_linked;
                                } else {
                                    // all prepend links are in order of visitation, check immediatly
                                    if count == index {
                                        value = Some(next_val);
                                        break;
                                    } else {
                                        // adavance to next value
                                        link = next_linked;
                                        count += Data::Integer::one();
                                    }
                                }
                            }
                            ExpressionDataType::Unit => {
                                // end iteration of link
                                // to check any pushed registers
                                break;
                            }
                            t => state_error(format!("Invalid linked type {:?}", t))?
                        }
                    }

                    // registers now contain append links in correct order
                    // pop back to starting point and perform same check
                    while this.get_register_len() > start_register {
                        match this.pop_register() {
                            None => state_error(format!("Popping more registers than placed during linking indexing."))?,
                            Some(r) => {
                                if count == index {
                                    let (next_val, _, _) = this.get_link(r)?;
                                    value = Some(next_val);
                                    // no break, this branch should only happen once because of equality
                                    // continue to clean up remaining registers
                                }

                                count += Data::Integer::one();
                            }
                        }
                    }

                    Ok(value)
                }
                ExpressionDataType::Symbol => {
                    let (mut value, mut linked, is_append) = this.get_link(left)?;
                    let sym = this.get_symbol(right)?;

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
                _ => Ok(None)
            }
        }
        _ => Ok(None)
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
    use crate::testing_utilites::{add_integer_list, add_links, add_list};
    use crate::{runtime::GarnishRuntime, GarnishLangRuntimeData, SimpleRuntimeData};

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
}

#[cfg(test)]
mod link {
    use crate::{GarnishLangRuntimeData, GarnishRuntime, SimpleRuntimeData, symbol_value};
    use crate::testing_utilites::add_links;

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
}
