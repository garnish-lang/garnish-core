use crate::{next_ref, push_integer, push_unit, state_error, ExpressionDataType, GarnishLangRuntimeData, RuntimeError, TypeConstants};

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

pub(crate) fn access_left_internal<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    let r = next_ref(this)?;
    match this.get_data_type(r)? {
        ExpressionDataType::Pair => {
            let (left, _) = this.get_pair(r)?;
            this.push_register(left)?;
        }
        ExpressionDataType::Range => {
            let (start, _, start_ex, _) = this.get_range(r)?;
            match this.get_data_type(start)? {
                ExpressionDataType::Integer => {
                    let int = this.get_integer(start)?;
                    let addr = if start_ex {
                        this.add_integer(int + Data::Integer::one())?
                    } else {
                        start
                    };
                    this.push_register(addr)?;
                }
                _ => push_unit(this)?,
            }
        }
        _ => push_unit(this)?,
    }

    Ok(())
}

pub(crate) fn access_right_internal<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    let r = next_ref(this)?;
    match this.get_data_type(r)? {
        ExpressionDataType::Pair => {
            let (_, right) = this.get_pair(r)?;
            this.push_register(right)?;
        }
        ExpressionDataType::Range => {
            let (_, end, _, end_ex) = this.get_range(r)?;
            match this.get_data_type(end)? {
                ExpressionDataType::Integer => {
                    let int = this.get_integer(end)?;
                    let addr = if end_ex { this.add_integer(int - Data::Integer::one())? } else { end };
                    this.push_register(addr)?;
                }
                _ => push_unit(this)?,
            }
        }
        _ => push_unit(this)?,
    }

    Ok(())
}

pub(crate) fn access_length_internal<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    let r = next_ref(this)?;
    match this.get_data_type(r)? {
        ExpressionDataType::List => {
            let len = Data::size_to_integer(this.get_list_len(r)?);
            push_integer(this, len)?;
        }
        ExpressionDataType::CharList => {
            let len = Data::size_to_integer(this.get_char_list_len(r)?);
            push_integer(this, len)?;
        }
        ExpressionDataType::ByteList => {
            let len = Data::size_to_integer(this.get_byte_list_len(r)?);
            push_integer(this, len)?;
        }
        ExpressionDataType::Range => {
            let (start, end, start_ex, end_ex) = this.get_range(r)?;
            match (this.get_data_type(end)?, this.get_data_type(start)?) {
                (ExpressionDataType::Integer, ExpressionDataType::Integer) => {
                    let start_int = this.get_integer(start)?;
                    let end_int = this.get_integer(end)?;
                    let result = range_len::<Data>(start_int, end_int, start_ex, end_ex);

                    let addr = this.add_integer(result)?;
                    this.push_register(addr)?;
                }
                _ => push_unit(this)?,
            }
        }
        _ => push_unit(this)?,
    }

    Ok(())
}

pub(crate) fn get_access_addr<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    right: Data::Size,
    left: Data::Size,
) -> Result<Option<Data::Size>, Data::Error> {
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
            let (start, end, start_ex, end_ex) = this.get_range(left)?;
            match (this.get_data_type(start)?, this.get_data_type(end)?) {
                (ExpressionDataType::Integer, ExpressionDataType::Integer) => {
                    let start_int = this.get_integer(start)?;
                    let end_int = this.get_integer(end)?;
                    let index = this.get_integer(right)?;
                    let len = range_len::<Data>(start_int, end_int, start_ex, end_ex);

                    if index >= len {
                        return Ok(None);
                    } else {
                        let mut result = start_int + index;

                        if start_ex {
                            result += Data::Integer::one();
                        }

                        let addr = this.add_integer(result)?;
                        Ok(Some(addr))
                    }
                }
                (ExpressionDataType::Unit, ExpressionDataType::Integer) => {
                    let start_int = Data::Integer::zero();
                    let end_int = this.get_integer(end)?;
                    let index = this.get_integer(right)?;
                    let len = range_len::<Data>(start_int, end_int, start_ex, end_ex);

                    if index >= len {
                        return Ok(None);
                    } else {
                        let mut result = start_int + index;

                        if start_ex {
                            result += Data::Integer::one();
                        }

                        let addr = this.add_integer(result)?;
                        Ok(Some(addr))
                    }
                }
                (ExpressionDataType::Integer, ExpressionDataType::Unit) => {
                    let start_int = this.get_integer(start)?;
                    let end_int = Data::Integer::max();
                    let index = this.get_integer(right)?;
                    let len = range_len::<Data>(start_int, end_int, start_ex, end_ex);

                    if index >= len {
                        return Ok(None);
                    } else {
                        let mut result = start_int + index;

                        if start_ex {
                            result += Data::Integer::one();
                        }

                        let addr = this.add_integer(result)?;
                        Ok(Some(addr))
                    }
                }
                _ => Ok(None),
            }
        }
        _ => Ok(None),
    }
}

fn range_len<Data: GarnishLangRuntimeData>(start: Data::Integer, end: Data::Integer, start_ex: bool, end_ex: bool) -> Data::Integer {
    // example range 10..20
    // dif              20 - 10 = 10
    // regular  len = 11, add 1
    // start ex len = 10
    // end ex   len = 10
    // ex       len = 9, sub 1

    let dif = end - start;
    match (start_ex, end_ex) {
        (true, true) => dif - Data::Integer::one(),
        (false, false) => dif + Data::Integer::one(),
        (true, false) | (false, true) => dif
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
    fn access_pair_left() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();

        runtime.push_instruction(Instruction::AccessLeftInternal, None).unwrap();

        runtime.push_register(i3).unwrap();

        runtime.access_left_internal().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), i1);
    }

    #[test]
    fn access_left_internal_incompatible_type_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();

        runtime.push_instruction(Instruction::AccessLeftInternal, None).unwrap();

        runtime.push_register(i1).unwrap();

        runtime.access_left_internal().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn access_pair_right() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();

        runtime.push_instruction(Instruction::AccessRightInternal, None).unwrap();

        runtime.push_register(i3).unwrap();

        runtime.access_right_internal().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), i2);
    }

    #[test]
    fn access_right_internal_incompatible_type_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();

        runtime.push_instruction(Instruction::AccessRightInternal, None).unwrap();

        runtime.push_register(i1).unwrap();

        runtime.access_right_internal().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn access_list_length() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let start = runtime.get_data_len();

        runtime.push_instruction(Instruction::AccessLengthInternal, None).unwrap();

        runtime.push_register(i4).unwrap();

        runtime.access_length_internal().unwrap();

        assert_eq!(runtime.get_integer(start).unwrap(), 1);
        assert_eq!(runtime.get_register(0).unwrap(), start);
    }

    #[test]
    fn access_char_list_length() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.start_char_list().unwrap();
        runtime.add_to_char_list('a').unwrap();
        runtime.add_to_char_list('b').unwrap();
        runtime.add_to_char_list('c').unwrap();
        let i4 = runtime.end_char_list().unwrap();
        let start = runtime.get_data_len();

        runtime.push_instruction(Instruction::AccessLengthInternal, None).unwrap();

        runtime.push_register(i4).unwrap();

        runtime.access_length_internal().unwrap();

        assert_eq!(runtime.get_integer(start).unwrap(), 3);
        assert_eq!(runtime.get_register(0).unwrap(), start);
    }

    #[test]
    fn access_byte_list_length() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.start_byte_list().unwrap();
        runtime.add_to_byte_list(10).unwrap();
        runtime.add_to_byte_list(20).unwrap();
        runtime.add_to_byte_list(30).unwrap();
        let d1 = runtime.end_byte_list().unwrap();
        let start = runtime.get_data_len();

        runtime.push_instruction(Instruction::AccessLengthInternal, None).unwrap();

        runtime.push_register(d1).unwrap();

        runtime.access_length_internal().unwrap();

        assert_eq!(runtime.get_integer(start).unwrap(), 3);
        assert_eq!(runtime.get_register(0).unwrap(), start);
    }

    #[test]
    fn access_length_internal_incompatible_type_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();

        runtime.push_instruction(Instruction::AccessLengthInternal, None).unwrap();

        runtime.push_register(i1).unwrap();

        runtime.access_length_internal().unwrap();

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
    fn range_start() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let i3 = runtime.add_range(i1, i2, false, false).unwrap();

        runtime.push_instruction(Instruction::AccessLeftInternal, None).unwrap();

        runtime.push_register(i3).unwrap();

        runtime.access_left_internal().unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), 10);
    }

    #[test]
    fn start_exclusive_range_start() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let i3 = runtime.add_range(i1, i2, true, false).unwrap();

        runtime.push_instruction(Instruction::AccessLeftInternal, None).unwrap();

        runtime.push_register(i3).unwrap();

        runtime.access_left_internal().unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), 11);
    }

    #[test]
    fn range_end() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let i3 = runtime.add_range(i1, i2, false, false).unwrap();

        runtime.push_instruction(Instruction::AccessRightInternal, None).unwrap();

        runtime.push_register(i3).unwrap();

        runtime.access_right_internal().unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), 20);
    }

    #[test]
    fn end_exclusive_range_end() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let i3 = runtime.add_range(i1, i2, false, true).unwrap();

        runtime.push_instruction(Instruction::AccessRightInternal, None).unwrap();

        runtime.push_register(i3).unwrap();

        runtime.access_right_internal().unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), 19);
    }

    #[test]
    fn range_length() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let i3 = runtime.add_range(i1, i2, false, false).unwrap();

        runtime.push_instruction(Instruction::AccessLengthInternal, None).unwrap();

        runtime.push_register(i3).unwrap();

        runtime.access_length_internal().unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), 11);
    }

    #[test]
    fn range_length_unit_if_start_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_unit().unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let i3 = runtime.add_range(i1, i2, false, false).unwrap();

        runtime.push_instruction(Instruction::AccessLengthInternal, None).unwrap();

        runtime.push_register(i3).unwrap();

        runtime.access_length_internal().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn range_length_unit_if_end_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(20).unwrap();
        let i2 = runtime.add_unit().unwrap();
        let i3 = runtime.add_range(i1, i2, false, false).unwrap();

        runtime.push_instruction(Instruction::AccessLengthInternal, None).unwrap();

        runtime.push_register(i3).unwrap();

        runtime.access_length_internal().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn start_exclusive_range_length() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let i3 = runtime.add_range(i1, i2, true, false).unwrap();

        runtime.push_instruction(Instruction::AccessLengthInternal, None).unwrap();

        runtime.push_register(i3).unwrap();

        runtime.access_length_internal().unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), 10);
    }

    #[test]
    fn end_exclusive_range_length() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let i3 = runtime.add_range(i1, i2, false, true).unwrap();

        runtime.push_instruction(Instruction::AccessLengthInternal, None).unwrap();

        runtime.push_register(i3).unwrap();

        runtime.access_length_internal().unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), 10);
    }

    #[test]
    fn exclusive_range_length() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let i3 = runtime.add_range(i1, i2, true, true).unwrap();

        runtime.push_instruction(Instruction::AccessLengthInternal, None).unwrap();

        runtime.push_register(i3).unwrap();

        runtime.access_length_internal().unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), 9);
    }

    #[test]
    fn access_with_integer() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let i3 = runtime.add_range(i1, i2, false, false).unwrap();
        let i4 = runtime.add_integer(5).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i3).unwrap();
        runtime.push_register(i4).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), 15);
    }

    #[test]
    fn access_start_exclusive_with_integer() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let i3 = runtime.add_range(i1, i2, true, false).unwrap();
        let i4 = runtime.add_integer(5).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i3).unwrap();
        runtime.push_register(i4).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), 16);
    }

    #[test]
    fn access_unit_start_first_is_zero() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_unit().unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let i3 = runtime.add_range(i1, i2, false, false).unwrap();
        let i4 = runtime.add_integer(0).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i3).unwrap();
        runtime.push_register(i4).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), 0);
    }

    #[test]
    fn access_unit_start_exclusive_first_is_one() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_unit().unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let i3 = runtime.add_range(i1, i2, true, false).unwrap();
        let i4 = runtime.add_integer(0).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i3).unwrap();
        runtime.push_register(i4).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), 1);
    }

    #[test]
    fn access_unit_end_max_is_max() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(100).unwrap();
        let i2 = runtime.add_unit().unwrap();
        let i3 = runtime.add_range(i1, i2, false, false).unwrap();
        let i4 = runtime.add_integer(i32::MAX - 100).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i3).unwrap();
        runtime.push_register(i4).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), i32::MAX);
    }

    #[test]
    fn access_unit_end_exclusive_minus_start_is_max() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(100).unwrap();
        let i2 = runtime.add_unit().unwrap();
        let i3 = runtime.add_range(i1, i2, false, true).unwrap();
        let i4 = runtime.add_integer(i32::MAX - 101).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i3).unwrap();
        runtime.push_register(i4).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), i32::MAX - 1);
    }

    #[test]
    fn access_start_exclusive_with_integer_zero() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let i3 = runtime.add_range(i1, i2, true, false).unwrap();
        let i4 = runtime.add_integer(0).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i3).unwrap();
        runtime.push_register(i4).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), 11);
    }

    #[test]
    fn access_end_exclusive_with_integer() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let i3 = runtime.add_range(i1, i2, false, true).unwrap();
        let i4 = runtime.add_integer(10).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i3).unwrap();
        runtime.push_register(i4).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }
}
