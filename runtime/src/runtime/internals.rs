use crate::runtime::range::range_len;
use crate::{next_ref, push_integer, push_unit, state_error, ExpressionDataType, GarnishLangRuntimeData, RuntimeError, TypeConstants};

pub(crate) fn access_left_internal<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    let r = next_ref(this)?;
    match this.get_data_type(r)? {
        ExpressionDataType::Pair => {
            let (left, _) = this.get_pair(r)?;
            this.push_register(left)?;
        }
        ExpressionDataType::Range => {
            let (start, _) = this.get_range(r)?;
            match this.get_data_type(start)? {
                ExpressionDataType::Number => {
                    this.push_register(start)?;
                }
                _ => push_unit(this)?,
            }
        }
        ExpressionDataType::Slice => {
            let (value, _) = this.get_slice(r)?;
            this.push_register(value)?;
        }
        ExpressionDataType::Link => {
            let (value, ..) = this.get_link(r)?;
            this.push_register(value)?;
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
            let (_, end) = this.get_range(r)?;
            match this.get_data_type(end)? {
                ExpressionDataType::Number => {
                    this.push_register(end)?;
                }
                _ => push_unit(this)?,
            }
        }
        ExpressionDataType::Slice => {
            let (_, range) = this.get_slice(r)?;
            this.push_register(range)?;
        }
        ExpressionDataType::Link => {
            let (_, linked, _) = this.get_link(r)?;
            this.push_register(linked)?;
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
            let (start, end) = this.get_range(r)?;
            match (this.get_data_type(end)?, this.get_data_type(start)?) {
                (ExpressionDataType::Number, ExpressionDataType::Number) => {
                    let start_int = this.get_integer(start)?;
                    let end_int = this.get_integer(end)?;
                    let result = range_len::<Data>(start_int, end_int)?;

                    let addr = this.add_integer(result)?;
                    this.push_register(addr)?;
                }
                _ => push_unit(this)?,
            }
        }
        ExpressionDataType::Slice => {
            let (_, range_addr) = this.get_slice(r)?;
            let (start, end) = this.get_range(range_addr)?;
            match (this.get_data_type(start)?, this.get_data_type(end)?) {
                (ExpressionDataType::Number, ExpressionDataType::Number) => {
                    let start = this.get_integer(start)?;
                    let end = this.get_integer(end)?;
                    let addr = this.add_integer(range_len::<Data>(start, end)?)?;
                    this.push_register(addr)?;
                }
                (s, e) => state_error(format!("Non integer values used for range {:?} {:?}", s, e))?,
            }
        }
        ExpressionDataType::Link => {
            let count = link_len(this, r)?;
            let addr = this.add_integer(count)?;
            this.push_register(addr)?;
        }
        _ => push_unit(this)?,
    }

    Ok(())
}

pub(crate) fn link_len<Data: GarnishLangRuntimeData>(this: &Data, addr: Data::Size) -> Result<Data::Number, RuntimeError<Data::Error>> {
    Ok(Data::size_to_integer(link_len_size(this, addr)?))
}

pub(crate) fn link_len_size<Data: GarnishLangRuntimeData>(this: &Data, addr: Data::Size) -> Result<Data::Size, RuntimeError<Data::Error>> {
    let (value, mut linked, _) = this.get_link(addr)?;
    let mut count = match this.get_data_type(value)? {
        ExpressionDataType::Link => state_error(format!("Linked found as value of link at addr {:?}", value))?,
        _ => Data::Size::one(),
    };

    // order doesn't matter, just loop through and count
    loop {
        match this.get_data_type(linked)? {
            ExpressionDataType::Link => {
                let (next_val, next, _) = this.get_link(linked)?;
                linked = next;
                count += match this.get_data_type(next_val)? {
                    ExpressionDataType::Link => state_error(format!("Linked found as value of link at addr {:?}", next_val))?,
                    _ => Data::Size::one(),
                };
            }
            ExpressionDataType::Unit => break,
            l => state_error(format!("Invalid linked type {:?}", l))?,
        }
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

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
}

#[cfg(test)]
mod ranges {
    use crate::{runtime::GarnishRuntime, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

    #[test]
    fn range_start() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let i3 = runtime.add_range(i1, i2).unwrap();

        runtime.push_instruction(Instruction::AccessLeftInternal, None).unwrap();

        runtime.push_register(i3).unwrap();

        runtime.access_left_internal().unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), 10);
    }

    #[test]
    fn range_end() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let i3 = runtime.add_range(i1, i2).unwrap();

        runtime.push_instruction(Instruction::AccessRightInternal, None).unwrap();

        runtime.push_register(i3).unwrap();

        runtime.access_right_internal().unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), 20);
    }

    #[test]
    fn range_len() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let i3 = runtime.add_range(i1, i2).unwrap();

        runtime.push_instruction(Instruction::AccessLengthInternal, None).unwrap();

        runtime.push_register(i3).unwrap();

        runtime.access_length_internal().unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), 11);
    }
}

#[cfg(test)]
mod slices {
    use crate::testing_utilites::add_list;
    use crate::{runtime::GarnishRuntime, GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn left_internal_gives_value() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_list(&mut runtime, 10);
        let d2 = runtime.add_integer(1).unwrap();
        let d3 = runtime.add_integer(4).unwrap();
        let d4 = runtime.add_range(d2, d3).unwrap();
        let d5 = runtime.add_slice(d1, d4).unwrap();

        runtime.push_register(d5).unwrap();

        runtime.access_left_internal().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), d1);
    }

    #[test]
    fn right_internal_gives_range() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_list(&mut runtime, 10);
        let d2 = runtime.add_integer(1).unwrap();
        let d3 = runtime.add_integer(4).unwrap();
        let d4 = runtime.add_range(d2, d3).unwrap();
        let d5 = runtime.add_slice(d1, d4).unwrap();

        runtime.push_register(d5).unwrap();

        runtime.access_right_internal().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), d4);
    }

    #[test]
    fn len() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_list(&mut runtime, 10);
        let d2 = runtime.add_integer(1).unwrap();
        let d3 = runtime.add_integer(4).unwrap();
        let d4 = runtime.add_range(d2, d3).unwrap();
        let d5 = runtime.add_slice(d1, d4).unwrap();

        runtime.push_register(d5).unwrap();

        runtime.access_length_internal().unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), 4);
    }
}

#[cfg(test)]
mod links {
    use crate::testing_utilites::{add_links_with_start, add_list, add_range};
    use crate::{runtime::GarnishRuntime, GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn left_internal_gives_value() {
        let mut runtime = SimpleRuntimeData::new();

        let unit = runtime.add_unit().unwrap();
        let d1 = runtime.add_integer(10).unwrap();
        let d2 = runtime.add_link(d1, unit, true).unwrap();
        let d3 = runtime.add_integer(20).unwrap();
        let d4 = runtime.add_link(d3, d2, true).unwrap();

        runtime.push_register(d4).unwrap();

        runtime.access_left_internal().unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), 20);
    }

    #[test]
    fn right_internal_gives_next_link() {
        let mut runtime = SimpleRuntimeData::new();

        let unit = runtime.add_unit().unwrap();
        let d1 = runtime.add_integer(10).unwrap();
        let d2 = runtime.add_link(d1, unit, true).unwrap();
        let d3 = runtime.add_integer(20).unwrap();
        let d4 = runtime.add_link(d3, d2, true).unwrap();

        runtime.push_register(d4).unwrap();

        runtime.access_right_internal().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), d2);
    }

    #[test]
    fn len() {
        let mut runtime = SimpleRuntimeData::new();

        let list = add_links_with_start(&mut runtime, 10, true, 15);

        runtime.push_register(list).unwrap();

        runtime.access_length_internal().unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), 10);
    }

    #[test]
    fn len_with_slice_and_list_chains() {
        let mut runtime = SimpleRuntimeData::new();

        let unit = runtime.add_unit().unwrap();
        let d1 = add_list(&mut runtime, 10);
        let d2 = runtime.add_link(d1, unit, true).unwrap();

        let d3 = add_list(&mut runtime, 5);
        let d4 = add_range(&mut runtime, 1, 3);
        let d5 = runtime.add_slice(d3, d4).unwrap();

        let d6 = runtime.add_link(d5, d2, true).unwrap();
        let d7 = runtime.add_integer(100).unwrap();
        let d8 = runtime.add_link(d7, d6, true).unwrap();

        runtime.push_register(d8).unwrap();

        runtime.access_length_internal().unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), 3);
    }
}
