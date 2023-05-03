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
                ExpressionDataType::List => index_list(this, value, adjusted_index),
                ExpressionDataType::CharList => index_char_list(this, value, adjusted_index),
                ExpressionDataType::ByteList => index_byte_list(this, value, adjusted_index),
                ExpressionDataType::Concatenation => {
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
        ExpressionDataType::Concatenation => index_concatenation_for(this, value, index),
        _ => Err(RuntimeError::unsupported_types()),
    }
}

pub(crate) fn index_concatenation_for<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    addr: Data::Size,
    index: Data::Number,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    Ok(iterate_concatenation_internal(
        this,
        addr,
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

pub(crate) fn iterate_concatenation_internal<Data: GarnishLangRuntimeData, ListCheckFn, CheckFn>(
    this: &mut Data,
    addr: Data::Size,
    mut _list_check_fn: ListCheckFn,
    mut check_fn: CheckFn,
) -> Result<(Option<Data::Size>, Data::Size), RuntimeError<Data::Error>>
where
    ListCheckFn: FnMut(&mut Data, Data::Number, Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>,
    CheckFn: FnMut(&mut Data, Data::Number, Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>,
{
    let (current, next) = this.get_concatenation(addr)?;
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
                    ExpressionDataType::Concatenation => {
                        let (current, next) = this.get_concatenation(r)?;
                        this.push_register(next)?;
                        this.push_register(current)?;
                    }
                    ExpressionDataType::List => {
                        // temp_result = list_check_fn(this, Data::size_to_number(index), r)?;
                        // let list_len = this.get_list_len(r)?;
                        // index = index + list_len;
                        let len = this.get_list_len(r)?;
                        let mut i = Data::Size::zero();

                        while i < len {
                            let sub_index = Data::size_to_number(i);
                            let item = this.get_list_item(r, sub_index)?;
                            temp_result = check_fn(this, (Data::size_to_number(index)).plus(sub_index).or_num_err()?, item)?;
                            match temp_result {
                                Some(_) => break,
                                None => i = i + Data::Size::one()
                            }
                        }
                        index = index + len;
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
                ExpressionDataType::Concatenation => {
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
        ExpressionDataType::Concatenation => Ok(iterate_concatenation_internal(
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
