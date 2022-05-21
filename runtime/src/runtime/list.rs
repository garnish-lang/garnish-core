use crate::runtime::range::range_len;
use crate::{get_range, state_error, ExpressionDataType, GarnishLangRuntimeData, GarnishNumber, OrNumberError, RuntimeError, TypeConstants};

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

pub(crate) fn get_access_addr<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    right: Data::Size,
    left: Data::Size,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    match this.get_data_type(right)? {
        ExpressionDataType::Number => {
            let i = this.get_number(right)?;
            access_with_integer(this, i, left)
        }
        ExpressionDataType::Symbol => {
            let sym = this.get_symbol(right)?;
            access_with_symbol(this, sym, left)
        }
        _ => Err(RuntimeError::unsupported_types()),
    }
}

pub(crate) fn access_with_integer<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    index: Data::Number,
    value: Data::Size,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    match this.get_data_type(value)? {
        ExpressionDataType::List => index_list(this, value, index),
        ExpressionDataType::CharList => index_char_list(this, value, index),
        ExpressionDataType::ByteList => index_byte_list(this, value, index),
        ExpressionDataType::Range => {
            let (start, end) = this.get_range(value)?;
            match (this.get_data_type(start)?, this.get_data_type(end)?) {
                (ExpressionDataType::Number, ExpressionDataType::Number) => {
                    let start_int = this.get_number(start)?;
                    let end_int = this.get_number(end)?;
                    let len = range_len::<Data>(start_int, end_int)?;

                    if index >= len {
                        return Ok(None);
                    } else {
                        let result = start_int.plus(index).or_num_err()?;
                        let addr = this.add_number(result)?;
                        Ok(Some(addr))
                    }
                }
                _ => Ok(None),
            }
        }
        ExpressionDataType::Slice => {
            let (value, range) = this.get_slice(value)?;
            let (start, _, _) = get_range(this, range)?;
            let adjusted_index = start.plus(index).or_num_err()?;

            match this.get_data_type(value)? {
                ExpressionDataType::Link => index_link(this, value, adjusted_index),
                ExpressionDataType::List => index_list(this, value, adjusted_index),
                ExpressionDataType::CharList => index_char_list(this, value, adjusted_index),
                ExpressionDataType::ByteList => index_byte_list(this, value, adjusted_index),
                ExpressionDataType::Concatentation => {
                    Ok(iterate_concatenation_internal(
                        this,
                        value,
                        |this, index, addr| {
                            let list_len = this.get_list_len(addr)?;

                            // already know that index is greater than count
                            if adjusted_index < index.plus(Data::size_to_number(list_len)).or_num_err()? {
                                // item is in this list
                                let list_index = adjusted_index.subtract(index).or_num_err()?;
                                let list_r = this.get_list_item(addr, list_index)?;

                                return Ok(Some(list_r));
                            }

                            Ok(None)
                        },
                        |_this, index, addr| {
                            if index == adjusted_index {
                                return Ok(Some(addr));
                            }

                            Ok(None)
                        },
                    )?
                    .0)
                }
                t => state_error(format!("Invalid value for slice {:?}", t)),
            }
        }
        ExpressionDataType::Concatentation => {
            Ok(iterate_concatenation_internal(
                this,
                value,
                |this, current_index, addr| {
                    let list_len = this.get_list_len(addr)?;

                    // already know that index is greater than count
                    if index < current_index.plus(Data::size_to_number(list_len)).or_num_err()? {
                        // item is in this list
                        let list_index = index.subtract(current_index).or_num_err()?;
                        let list_r = this.get_list_item(addr, list_index)?;

                        return Ok(Some(list_r));
                    }

                    Ok(None)
                },
                |_this, current_index, addr| {
                    if current_index == index {
                        return Ok(Some(addr));
                    }

                    Ok(None)
                },
            )?
            .0)
        }
        ExpressionDataType::Link => index_link(this, value, index),
        _ => Err(RuntimeError::unsupported_types()),
    }
}

pub(crate) fn iterate_concatenation_internal<Data: GarnishLangRuntimeData, ListCheckFn, CheckFn>(
    this: &mut Data,
    addr: Data::Size,
    mut list_check_fn: ListCheckFn,
    mut check_fn: CheckFn,
) -> Result<(Option<Data::Size>, Data::Size), RuntimeError<Data::Error>>
where
    ListCheckFn: FnMut(&mut Data, Data::Number, Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>,
    CheckFn: FnMut(&mut Data, Data::Number, Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>,
{
    let (current, next) = this.get_concatentation(addr)?;
    let start_register = this.get_register_len();
    let mut index = Data::Size::zero();

    this.push_register(next)?;
    this.push_register(current)?;

    let mut result = None;

    while this.get_register_len() > start_register {
        match this.pop_register() {
            None => state_error(format!("Popping more registers than placed during concatenation indexing."))?,
            Some(r) => {
                let mut temp_result = None;
                match this.get_data_type(r)? {
                    ExpressionDataType::Concatentation => {
                        let (current, next) = this.get_concatentation(r)?;
                        this.push_register(next)?;
                        this.push_register(current)?;
                    }
                    ExpressionDataType::List => {
                        temp_result = list_check_fn(this, Data::size_to_number(index), r)?;
                        let list_len = this.get_list_len(r)?;
                        index = index + list_len;
                    }
                    _ => {
                        temp_result = check_fn(this, Data::size_to_number(index), r)?;
                        index += Data::Size::one();
                    }
                }

                match temp_result {
                    Some(_) => {
                        result = temp_result;
                        break;
                    }
                    _ => (), // continue
                }
            }
        }
    }

    // clear borrowed registers
    while this.get_register_len() > start_register {
        this.pop_register();
    }

    Ok((result, index))
}

fn index_list<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    list: Data::Size,
    index: Data::Number,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    if index < Data::Number::zero() {
        Ok(None)
    } else {
        if index >= Data::size_to_number(this.get_list_len(list)?) {
            Ok(None)
        } else {
            Ok(Some(this.get_list_item(list, index)?))
        }
    }
}

fn index_char_list<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    list: Data::Size,
    index: Data::Number,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    if index < Data::Number::zero() {
        Ok(None)
    } else {
        if index >= Data::size_to_number(this.get_char_list_len(list)?) {
            Ok(None)
        } else {
            let c = this.get_char_list_item(list, index)?;
            let addr = this.add_char(c)?;
            Ok(Some(addr))
        }
    }
}

fn index_byte_list<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    list: Data::Size,
    index: Data::Number,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    if index < Data::Number::zero() {
        Ok(None)
    } else {
        if index >= Data::size_to_number(this.get_byte_list_len(list)?) {
            Ok(None)
        } else {
            let c = this.get_byte_list_item(list, index)?;
            let addr = this.add_byte(c)?;
            Ok(Some(addr))
        }
    }
}

pub(crate) fn iterate_link_internal<Data: GarnishLangRuntimeData, Callback>(
    this: &mut Data,
    link: Data::Size,
    mut func: Callback,
) -> Result<(), RuntimeError<Data::Error>>
where
    Callback: FnMut(&mut Data, Data::Size, Data::Number) -> Result<bool, RuntimeError<Data::Error>>,
{
    let mut current_index = Data::Number::zero();
    // keep track of starting point to any pushed values later
    let start_register = this.get_register_len();

    this.push_register(link)?;

    while this.get_register_len() > start_register {
        match this.pop_register() {
            None => state_error(format!("Popping more registers than placed during linking indexing."))?,
            Some(r) => match this.get_data_type(r)? {
                // flatten all links, pushing their vals to the register
                // val of link should never be another link
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
                _ => {
                    if func(this, r, current_index)? {
                        break;
                    } else {
                        current_index = current_index.increment().or_num_err()?;
                    }
                }
            },
        }
    }

    while this.get_register_len() > start_register {
        this.pop_register();
    }

    Ok(())
}

pub(crate) fn iterate_link_internal_rev<Data: GarnishLangRuntimeData, Callback>(
    this: &mut Data,
    link: Data::Size,
    mut func: Callback,
) -> Result<(), RuntimeError<Data::Error>>
where
    Callback: FnMut(&mut Data, Data::Size, Data::Number) -> Result<bool, RuntimeError<Data::Error>>,
{
    let mut current_index = Data::Number::zero();
    // keep track of starting point to any pushed values later
    let start_register = this.get_register_len();

    this.push_register(link)?;

    while this.get_register_len() > start_register {
        match this.pop_register() {
            None => state_error(format!("Popping more registers than placed during linking indexing."))?,
            Some(r) => match this.get_data_type(r)? {
                // flatten all links, pushing their vals to the register
                // val of link should never be another link
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
                                this.push_register(linked)?;
                                this.push_register(val)?;
                            } else {
                                // linked value is next, gets resolved last, pushed first
                                this.push_register(val)?;
                                this.push_register(linked)?;
                            }
                        }
                        t => state_error(format!("Invalid linked type {:?}", t))?,
                    }
                }
                _ => {
                    if func(this, r, current_index)? {
                        break;
                    } else {
                        current_index = current_index.increment().or_num_err()?;
                    }
                }
            },
        }
    }

    while this.get_register_len() > start_register {
        this.pop_register();
    }

    Ok(())
}

pub(crate) fn index_link<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    link: Data::Size,
    index: Data::Number,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let mut item: Option<Data::Size> = None;
    iterate_link_internal(this, link, |_runtime, item_addr, current_index| {
        if current_index == index {
            item = Some(item_addr);
            Ok(true)
        } else {
            Ok(false)
        }
    })?;

    Ok(item)
}

fn access_with_symbol<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    sym: Data::Symbol,
    value: Data::Size,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    match this.get_data_type(value)? {
        ExpressionDataType::List => Ok(this.get_list_item_with_symbol(value, sym)?),
        ExpressionDataType::Slice => {
            let (value, range) = this.get_slice(value)?;
            let (start, end, _) = get_range(this, range)?;

            match this.get_data_type(value)? {
                ExpressionDataType::Link => sym_access_links_slices(this, start, value, sym, end),
                ExpressionDataType::List => {
                    // in order to limit to slice range need to check items manually
                    // can't push, in case any of the items are a Link or Slice
                    let mut i = start;
                    let mut item: Option<Data::Size> = None;
                    while i <= end {
                        let list_item = this.get_list_item(value, i)?;
                        match this.get_data_type(list_item)? {
                            ExpressionDataType::Pair => {
                                let (left, right) = this.get_pair(list_item)?;
                                match this.get_data_type(left)? {
                                    ExpressionDataType::Symbol => {
                                        if this.get_symbol(left)? == sym {
                                            item = Some(right);
                                            // found item break both loops
                                            break;
                                        }
                                    }
                                    _ => (),
                                }
                            }
                            _ => (),
                        }

                        i = i.increment().or_num_err()?;
                    }

                    Ok(item)
                }
                ExpressionDataType::Concatentation => {
                    Ok(iterate_concatenation_internal(
                        this,
                        value,
                        |this, index, addr| {
                            // check if current count is in desired range
                            let list_len = Data::size_to_number(this.get_list_len(addr)?);
                            let list_end = index.plus(list_len).or_num_err()?;

                            // start and end relative to current list
                            // start should be greater than 0 and less than end
                            // end should be less than len and greater than start
                            let adjusted_start = if index > start {
                                Data::Number::zero()
                            } else {
                                start.subtract(index).or_num_err()?
                            };
                            let adjusted_end = if end > list_end { list_len } else { end.subtract(index).or_num_err()? };

                            if adjusted_start < list_end && adjusted_end >= Data::Number::zero() {
                                let mut i = adjusted_start;
                                let mut item: Option<Data::Size> = None;
                                while i < adjusted_end {
                                    let list_item = this.get_list_item(addr, i)?;
                                    match this.get_data_type(list_item)? {
                                        ExpressionDataType::Pair => {
                                            let (left, right) = this.get_pair(list_item)?;
                                            match this.get_data_type(left)? {
                                                ExpressionDataType::Symbol => {
                                                    if this.get_symbol(left)? == sym {
                                                        item = Some(right);
                                                        break;
                                                    }
                                                }
                                                _ => (),
                                            }
                                        }
                                        _ => (),
                                    }

                                    i = i.increment().or_num_err()?;
                                }

                                Ok(item)
                            } else {
                                Ok(None)
                            }
                        },
                        |this, index, addr| {
                            if index > start && index <= end {
                                // in range
                                get_value_if_association(this, addr, sym)
                            } else {
                                Ok(None)
                            }
                        },
                    )?
                    .0)
                }
                t => state_error(format!("Invalid value for slice {:?}", t)),
            }
        }
        ExpressionDataType::Link => sym_access_links_slices(this, Data::Number::zero(), value, sym, Data::Number::max_value()),
        ExpressionDataType::Concatentation => Ok(iterate_concatenation_internal(
            this,
            value,
            |this, _index, addr| Ok(this.get_list_item_with_symbol(addr, sym)?),
            |this, _index, addr| get_value_if_association(this, addr, sym),
        )?
        .0),
        _ => Err(RuntimeError::unsupported_types()),
    }
}

pub(crate) fn get_value_if_association<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    addr: Data::Size,
    sym: Data::Symbol,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    match this.get_data_type(addr)? {
        ExpressionDataType::Pair => {
            let (left, right) = this.get_pair(addr)?;
            match this.get_data_type(left)? {
                ExpressionDataType::Symbol => {
                    if this.get_symbol(left)? == sym {
                        return Ok(Some(right));
                    }
                }
                _ => (),
            }
        }
        _ => (),
    }

    Ok(None)
}

pub(crate) fn is_value_association<Data: GarnishLangRuntimeData>(this: &Data, addr: Data::Size) -> Result<bool, RuntimeError<Data::Error>> {
    Ok(match this.get_data_type(addr)? {
        ExpressionDataType::Pair => {
            let (left, _right) = this.get_pair(addr)?;
            match this.get_data_type(left)? {
                ExpressionDataType::Symbol => true,
                _ => false,
            }
        }
        _ => false,
    })
}

fn sym_access_links_slices<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    start_count: Data::Number,
    start_value: Data::Size,
    sym: Data::Symbol,
    limit: Data::Number,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let mut item: Option<Data::Size> = None;
    let mut skip = start_count;
    let mut count = Data::Number::zero();
    // keep track of starting point to any pushed values later
    let start_register = this.get_register_len();

    this.push_register(start_value)?;

    while this.get_register_len() > start_register {
        if count > limit {
            break;
        }

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
                ExpressionDataType::Pair => {
                    // skip values
                    if skip > Data::Number::zero() {
                        skip = skip.decrement().or_num_err()?;
                        continue;
                    }

                    let (left, right) = this.get_pair(r)?;
                    match this.get_data_type(left)? {
                        ExpressionDataType::Symbol => {
                            // if this sym equals search sym, return 'right' value
                            // else, continue checking
                            if this.get_symbol(left)? == sym {
                                item = Some(right);
                                break;
                            } else {
                                count = count.increment().or_num_err()?;
                            }
                        }
                        // not an associations, continue on
                        _ => {
                            count = count.increment().or_num_err()?;
                        }
                    }
                }
                // nothing to do for any other values
                _ => {
                    // skip values
                    if skip > Data::Number::zero() {
                        skip = skip.decrement().or_num_err()?;
                        continue;
                    }

                    count = count.increment().or_num_err()?;
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

#[cfg(test)]
mod tests {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, Instruction, SimpleNumber, SimpleRuntimeData, NO_CONTEXT};

    #[test]
    fn make_list() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_number(10.into()).unwrap();
        let i2 = runtime.add_number(20.into()).unwrap();
        let i3 = runtime.add_number(20.into()).unwrap();
        let start = runtime.get_data_len();

        runtime.push_register(i1).unwrap();
        runtime.push_register(i2).unwrap();
        runtime.push_register(i3).unwrap();

        runtime.push_instruction(Instruction::MakeList, Some(3)).unwrap();

        runtime.make_list(3).unwrap();

        assert_eq!(runtime.get_list_len(start).unwrap(), 3);
        assert_eq!(runtime.get_list_item(start, 0.into()).unwrap(), i1);
        assert_eq!(runtime.get_list_item(start, 1.into()).unwrap(), i2);
        assert_eq!(runtime.get_list_item(start, 2.into()).unwrap(), i3);
    }

    #[test]
    fn make_list_no_refs_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_number(10.into()).unwrap();
        runtime.add_number(20.into()).unwrap();
        runtime.add_number(20.into()).unwrap();

        runtime.push_instruction(Instruction::MakeList, Some(3)).unwrap();

        let result = runtime.make_list(3);

        assert!(result.is_err());
    }

    #[test]
    fn make_list_with_associations() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol(1).unwrap();
        let i2 = runtime.add_number(10.into()).unwrap();
        let i3 = runtime.add_symbol(2).unwrap();
        let i4 = runtime.add_number(20.into()).unwrap();
        let i5 = runtime.add_symbol(3).unwrap();
        let i6 = runtime.add_number(30.into()).unwrap();
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
        assert_eq!(runtime.get_list_item(start, 0.into()).unwrap(), i7);
        assert_eq!(runtime.get_list_item(start, 1.into()).unwrap(), i8);
        assert_eq!(runtime.get_list_item(start, 2.into()).unwrap(), i9);

        assert_eq!(runtime.get_list_associations_len(start).unwrap(), 3);
        assert_eq!(runtime.get_list_association(start, 0.into()).unwrap(), i7);
        assert_eq!(runtime.get_list_association(start, 1.into()).unwrap(), i8);
        assert_eq!(runtime.get_list_association(start, 2.into()).unwrap(), i9);
    }

    #[test]
    fn apply() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol(1).unwrap();
        let i2 = runtime.add_number(10.into()).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_symbol(1).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i4).unwrap();
        runtime.push_register(i5).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), i2);
    }

    #[test]
    fn apply_with_integer() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol(1).unwrap();
        let i2 = runtime.add_number(10.into()).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_number(0.into()).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i4).unwrap();
        runtime.push_register(i5).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), i3);
    }

    #[test]
    fn apply_char_list_with_integer() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.start_char_list().unwrap();
        runtime.add_to_char_list('a').unwrap();
        runtime.add_to_char_list('b').unwrap();
        runtime.add_to_char_list('c').unwrap();
        let d1 = runtime.end_char_list().unwrap();
        let d2 = runtime.add_number(2.into()).unwrap();
        let start = runtime.get_data_len();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), start);
        assert_eq!(runtime.get_char(start).unwrap(), 'c');
    }

    #[test]
    fn apply_byte_list_with_integer() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.start_byte_list().unwrap();
        runtime.add_to_byte_list(10).unwrap();
        runtime.add_to_byte_list(20).unwrap();
        runtime.add_to_byte_list(30).unwrap();
        let d1 = runtime.end_byte_list().unwrap();
        let d2 = runtime.add_number(2.into()).unwrap();
        let start = runtime.get_data_len();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), start);
        assert_eq!(runtime.get_byte(start).unwrap(), 30.into());
    }

    #[test]
    fn apply_with_integer_out_of_bounds_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol(1).unwrap();
        let i2 = runtime.add_number(10.into()).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_number(10.into()).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i4).unwrap();
        runtime.push_register(i5).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn apply_with_number_negative_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol(1).unwrap();
        let i2 = runtime.add_number(10.into()).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_number(SimpleNumber::Integer(-1)).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i4).unwrap();
        runtime.push_register(i5).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn apply_non_list_on_left_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_number(10.into()).unwrap();
        let i2 = runtime.add_symbol(1).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i1).unwrap();
        runtime.push_register(i2).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn apply_non_symbol_on_right_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol(1).unwrap();
        let i2 = runtime.add_number(10.into()).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_expression(10).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i4).unwrap();
        runtime.push_register(i5).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn apply_no_refs_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol(1).unwrap();
        let i2 = runtime.add_number(10.into()).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let _i4 = runtime.end_list().unwrap();
        let _i5 = runtime.add_symbol(1).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        let result = runtime.apply(NO_CONTEXT);

        assert!(result.is_err());
    }

    #[test]
    fn apply_with_non_existent_key() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol(1).unwrap();
        let i2 = runtime.add_number(10.into()).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_symbol(2).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i4).unwrap();
        runtime.push_register(i5).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }
}

#[cfg(test)]
mod ranges {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, Instruction, SimpleRuntimeData, NO_CONTEXT};

    #[test]
    fn apply_with_integer() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_number(10.into()).unwrap();
        let i2 = runtime.add_number(20.into()).unwrap();
        let i3 = runtime.add_range(i1, i2).unwrap();
        let i4 = runtime.add_number(5.into()).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i3).unwrap();
        runtime.push_register(i4).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_number(runtime.get_register(0).unwrap()).unwrap(), 15.into());
    }

    #[test]
    fn apply_with_integer_out_of_range() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_number(10.into()).unwrap();
        let i2 = runtime.add_number(20.into()).unwrap();
        let i3 = runtime.add_range(i1, i2).unwrap();
        let i4 = runtime.add_number(30.into()).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i3).unwrap();
        runtime.push_register(i4).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }
}

#[cfg(test)]
mod slice {
    use crate::testing_utilites::{add_integer_list, add_links, add_list, add_pair, add_range};
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, SimpleDataRuntimeNC, SimpleRuntimeData, NO_CONTEXT};

    #[test]
    fn index_slice_of_list() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_integer_list(&mut runtime, 10);
        let d2 = runtime.add_number(1.into()).unwrap();
        let d3 = runtime.add_number(4.into()).unwrap();
        let d4 = runtime.add_range(d2, d3).unwrap();
        let d5 = runtime.add_slice(d1, d4).unwrap();
        let d6 = runtime.add_number(2.into()).unwrap();

        runtime.push_register(d5).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_number(runtime.get_register(0).unwrap()).unwrap(), 40.into());
    }

    #[test]
    fn index_slice_of_char_list() {
        let mut runtime = SimpleRuntimeData::new();

        let chars = SimpleDataRuntimeNC::parse_char_list("abcde").unwrap();

        runtime.start_char_list().unwrap();
        for c in chars {
            runtime.add_to_char_list(c).unwrap();
        }
        let list = runtime.end_char_list().unwrap();

        let range = add_range(&mut runtime, 1, 3);
        let slice = runtime.add_slice(list, range).unwrap();
        let d6 = runtime.add_number(2.into()).unwrap();

        runtime.push_register(slice).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_char(runtime.get_register(0).unwrap()).unwrap(), 'd');
    }

    #[test]
    fn index_slice_of_byte_list() {
        let mut runtime = SimpleRuntimeData::new();

        let bytes = SimpleDataRuntimeNC::parse_byte_list("abcde").unwrap();

        runtime.start_byte_list().unwrap();
        for b in bytes {
            runtime.add_to_byte_list(b).unwrap();
        }
        let list = runtime.end_byte_list().unwrap();

        let range = add_range(&mut runtime, 1, 3);
        let slice = runtime.add_slice(list, range).unwrap();
        let d6 = runtime.add_number(2.into()).unwrap();

        runtime.push_register(slice).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_byte(runtime.get_register(0).unwrap()).unwrap(), 'd' as u8);
    }

    #[test]
    fn sym_index_slice_of_list() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_list(&mut runtime, 10);
        let d2 = runtime.add_number(1.into()).unwrap();
        let d3 = runtime.add_number(4.into()).unwrap();
        let d4 = runtime.add_range(d2, d3).unwrap();
        let d5 = runtime.add_slice(d1, d4).unwrap();
        let d6 = runtime.add_symbol(SimpleDataRuntimeNC::parse_symbol("val4").unwrap()).unwrap();

        runtime.push_register(d5).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_number(runtime.get_register(0).unwrap()).unwrap(), 50.into());
    }

    #[test]
    fn sym_index_slice_of_list_sym_not_in_slice_before() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_list(&mut runtime, 10);
        let d2 = runtime.add_number(2.into()).unwrap();
        let d3 = runtime.add_number(4.into()).unwrap();
        let d4 = runtime.add_range(d2, d3).unwrap();
        let d5 = runtime.add_slice(d1, d4).unwrap();
        let d6 = runtime.add_symbol(1).unwrap();

        runtime.push_register(d5).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn sym_index_slice_of_list_sym_not_in_slice() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_list(&mut runtime, 10);
        let d2 = runtime.add_number(2.into()).unwrap();
        let d3 = runtime.add_number(4.into()).unwrap();
        let d4 = runtime.add_range(d2, d3).unwrap();
        let d5 = runtime.add_slice(d1, d4).unwrap();
        let d6 = runtime.add_symbol(8).unwrap();

        runtime.push_register(d5).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn index_slice_of_links() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_links(&mut runtime, 10, true);
        let d2 = runtime.add_number(2.into()).unwrap();
        let d3 = runtime.add_number(8.into()).unwrap();
        let d4 = runtime.add_range(d2, d3).unwrap();
        let d5 = runtime.add_slice(d1, d4).unwrap();
        let d6 = runtime.add_number(2.into()).unwrap();

        runtime.push_register(d5).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let (left, right) = runtime.get_pair(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(runtime.get_symbol(left).unwrap(), SimpleDataRuntimeNC::parse_symbol("val4").unwrap());
        assert_eq!(runtime.get_number(right).unwrap(), 5.into());
    }

    #[test]
    fn sym_index_slice_of_links() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_links(&mut runtime, 10, true);
        let d2 = runtime.add_number(2.into()).unwrap();
        let d3 = runtime.add_number(5.into()).unwrap();
        let d4 = runtime.add_range(d2, d3).unwrap();
        let d5 = runtime.add_slice(d1, d4).unwrap();
        let d6 = runtime.add_symbol(SimpleDataRuntimeNC::parse_symbol("val4").unwrap()).unwrap();

        runtime.push_register(d5).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_number(runtime.get_register(0).unwrap()).unwrap(), 5.into());
    }

    #[test]
    fn sym_index_slice_of_links_not_found() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_links(&mut runtime, 10, true);
        let d2 = runtime.add_number(2.into()).unwrap();
        let d3 = runtime.add_number(5.into()).unwrap();
        let d4 = runtime.add_range(d2, d3).unwrap();
        let d5 = runtime.add_slice(d1, d4).unwrap();
        let d6 = runtime.add_symbol(8).unwrap();

        runtime.push_register(d5).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn sym_index_slice_of_links_mixed() {
        let mut runtime = SimpleRuntimeData::new();

        let unit = runtime.add_unit().unwrap();
        let d1 = runtime.add_number(100.into()).unwrap();
        let d2 = add_pair(&mut runtime, "pair", 200);
        let d3 = runtime.add_number(300.into()).unwrap();
        let d4 = runtime.add_pair((d1, d3)).unwrap();

        let link1 = runtime.add_link(d1, unit, true).unwrap();
        let link2 = runtime.add_link(d4, link1, true).unwrap();
        let link3 = runtime.add_link(d2, link2, true).unwrap();
        let link4 = runtime.add_link(d3, link3, true).unwrap();
        let link5 = runtime.add_link(d4, link4, true).unwrap();

        let start = runtime.add_number(1.into()).unwrap();
        let end = runtime.add_number(4.into()).unwrap();
        let range = runtime.add_range(start, end).unwrap();
        let slice = runtime.add_slice(link5, range).unwrap();
        let sym = runtime.add_symbol(SimpleDataRuntimeNC::parse_symbol("pair").unwrap()).unwrap();

        runtime.push_register(slice).unwrap();
        runtime.push_register(sym).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_number(runtime.get_register(0).unwrap()).unwrap(), 200.into());
    }
}

#[cfg(test)]
mod link {
    use crate::testing_utilites::add_links;
    use crate::{GarnishLangRuntimeData, GarnishRuntime, SimpleDataRuntimeNC, SimpleRuntimeData, NO_CONTEXT};

    #[test]
    fn index_prepend_link_with_integer() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_links(&mut runtime, 10, false);
        let d2 = runtime.add_number(3.into()).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let (left, right) = runtime.get_pair(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(runtime.get_number(right).unwrap(), 4.into());
        assert_eq!(runtime.get_symbol(left).unwrap(), SimpleDataRuntimeNC::parse_symbol("val3").unwrap());
    }

    #[test]
    fn index_append_link_with_integer() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_links(&mut runtime, 10, true);
        let d2 = runtime.add_number(3.into()).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let (left, right) = runtime.get_pair(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(runtime.get_number(right).unwrap(), 4.into());
        assert_eq!(runtime.get_symbol(left).unwrap(), SimpleDataRuntimeNC::parse_symbol("val3").unwrap());
    }

    #[test]
    fn index_prepend_link_with_symbol() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_links(&mut runtime, 10, false);
        let d2 = runtime.add_symbol(SimpleDataRuntimeNC::parse_symbol("val2").unwrap()).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_number(runtime.get_register(0).unwrap()).unwrap(), 3.into());
    }

    #[test]
    fn index_append_link_with_symbol() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_links(&mut runtime, 10, true);
        let d2 = runtime.add_symbol(SimpleDataRuntimeNC::parse_symbol("val2").unwrap()).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_number(runtime.get_register(0).unwrap()).unwrap(), 3.into());
    }
}

#[cfg(test)]
mod concatenation {
    use crate::testing_utilites::{add_concatenation_with_start, add_integer_list_with_start, add_list_with_start, add_range};
    use crate::{ExpressionDataType, GarnishLangRuntimeData, GarnishRuntime, SimpleDataRuntimeNC, SimpleRuntimeData, NO_CONTEXT};

    #[test]
    fn index_concat_of_items_with_number() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_concatenation_with_start(&mut runtime, 10, 20);
        let d2 = runtime.add_number(3.into()).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let (_left, right) = runtime.get_pair(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(runtime.get_number(right).unwrap(), 23.into());
    }

    #[test]
    fn index_concat_of_items_with_symbol() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_concatenation_with_start(&mut runtime, 10, 20);
        let d2 = runtime.add_symbol(SimpleDataRuntimeNC::parse_symbol("val23").unwrap()).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_number(runtime.get_register(0).unwrap()).unwrap(), 23.into());
    }

    #[test]
    fn index_concat_of_lists_with_number() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_integer_list_with_start(&mut runtime, 10, 20);
        let d2 = add_integer_list_with_start(&mut runtime, 10, 40);
        let d3 = runtime.add_concatenation(d1, d2).unwrap();
        let d4 = runtime.add_number(13.into()).unwrap();

        runtime.push_register(d3).unwrap();
        runtime.push_register(d4).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_number(runtime.get_register(0).unwrap()).unwrap(), 43.into());
    }

    #[test]
    fn index_concat_of_lists_with_symbol() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_list_with_start(&mut runtime, 10, 20);
        let d2 = add_list_with_start(&mut runtime, 10, 40);
        let d3 = runtime.add_concatenation(d1, d2).unwrap();
        let d4 = runtime.add_symbol(SimpleDataRuntimeNC::parse_symbol("val43").unwrap()).unwrap();

        runtime.push_register(d3).unwrap();
        runtime.push_register(d4).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_number(runtime.get_register(0).unwrap()).unwrap(), 43.into());
    }

    #[test]
    fn index_slice_of_concat_of_items_with_number() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_concatenation_with_start(&mut runtime, 10, 20);
        let d2 = add_range(&mut runtime, 2, 5);
        let d3 = runtime.add_slice(d1, d2).unwrap();
        let d4 = runtime.add_number(1.into()).unwrap();

        runtime.push_register(d3).unwrap();
        runtime.push_register(d4).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let (_left, right) = runtime.get_pair(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(runtime.get_number(right).unwrap(), 23.into());
    }

    #[test]
    fn index_slice_of_concat_of_items_with_symbol() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_concatenation_with_start(&mut runtime, 10, 20);
        let d2 = add_range(&mut runtime, 2, 5);
        let d3 = runtime.add_slice(d1, d2).unwrap();
        let d4 = runtime.add_symbol(SimpleDataRuntimeNC::parse_symbol("val23").unwrap()).unwrap();

        runtime.push_register(d3).unwrap();
        runtime.push_register(d4).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_number(runtime.get_register(0).unwrap()).unwrap(), 23.into());
    }

    #[test]
    fn index_slice_of_concat_of_lists_with_number() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_integer_list_with_start(&mut runtime, 10, 20);
        let d2 = add_integer_list_with_start(&mut runtime, 10, 40);
        let d3 = runtime.add_concatenation(d1, d2).unwrap();
        let d4 = add_range(&mut runtime, 12, 15);
        let d5 = runtime.add_slice(d3, d4).unwrap();
        let d6 = runtime.add_number(1.into()).unwrap();

        runtime.push_register(d5).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_number(runtime.get_register(0).unwrap()).unwrap(), 43.into());
    }

    #[test]
    fn index_slice_of_concat_of_lists_with_symbol() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_list_with_start(&mut runtime, 10, 20);
        let d2 = add_list_with_start(&mut runtime, 10, 40);
        let d3 = runtime.add_concatenation(d1, d2).unwrap();
        let d4 = add_range(&mut runtime, 12, 15);
        let d5 = runtime.add_slice(d3, d4).unwrap();
        let d6 = runtime.add_symbol(SimpleDataRuntimeNC::parse_symbol("val43").unwrap()).unwrap();

        runtime.push_register(d5).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_number(runtime.get_register(0).unwrap()).unwrap(), 43.into());
    }

    #[test]
    fn index_slice_of_concat_of_lists_with_symbol_range_across_lists() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_list_with_start(&mut runtime, 10, 20);
        let d2 = add_list_with_start(&mut runtime, 10, 40);
        let d3 = runtime.add_concatenation(d1, d2).unwrap();
        let d4 = add_range(&mut runtime, 8, 12);
        let d5 = runtime.add_slice(d3, d4).unwrap();
        let d6 = runtime.add_symbol(SimpleDataRuntimeNC::parse_symbol("val40").unwrap()).unwrap();

        runtime.push_register(d5).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_number(runtime.get_register(0).unwrap()).unwrap(), 40.into());
    }

    #[test]
    fn index_slice_of_concat_of_lists_with_symbol_out_of_bounds() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_list_with_start(&mut runtime, 10, 20);
        let d2 = add_list_with_start(&mut runtime, 10, 40);
        let d3 = runtime.add_concatenation(d1, d2).unwrap();
        let d4 = add_range(&mut runtime, 12, 15);
        let d5 = runtime.add_slice(d3, d4).unwrap();
        let d6 = runtime.add_symbol(SimpleDataRuntimeNC::parse_symbol("val23").unwrap()).unwrap();

        runtime.push_register(d5).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn index_slice_of_concat_of_lists_with_symbol_out_of_bounds_same_list() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_list_with_start(&mut runtime, 10, 20);
        let d2 = add_list_with_start(&mut runtime, 10, 40);
        let d3 = runtime.add_concatenation(d1, d2).unwrap();
        let d4 = add_range(&mut runtime, 12, 15);
        let d5 = runtime.add_slice(d3, d4).unwrap();
        let d6 = runtime.add_symbol(SimpleDataRuntimeNC::parse_symbol("val48").unwrap()).unwrap();

        runtime.push_register(d5).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }
}
