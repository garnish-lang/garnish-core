use crate::runtime::error::state_error;
use crate::runtime::error::OrNumberError;
use crate::runtime::range::range_len;
use crate::runtime::utilities::get_range;
use garnish_lang_traits::helpers::{iterate_concatenation_mut, iterate_rev_concatenation_mut};
use garnish_lang_traits::{GarnishDataType, GarnishData, GarnishNumber, RuntimeError, TypeConstants};

pub(crate) fn make_list<Data: GarnishData>(this: &mut Data, len: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    if len > this.get_register_len() {
        state_error(format!("Not enough register values to make list of length {:?}", len))?
    }

    this.start_list(len.clone())?;

    let mut count = this.get_register_len() - len.clone();
    let end = this.get_register_len();
    // look into getting this to work with a range value
    while count < end {
        let r = match this.get_register(count.clone()) {
            None => state_error(format!("No register value at {:?} when making list", count))?,
            Some(r) => r,
        };

        let is_associative = match this.get_data_type(r.clone().clone())? {
            GarnishDataType::Pair => {
                let (left, _right) = this.get_pair(r.clone())?;
                match this.get_data_type(left)? {
                    GarnishDataType::Symbol => true,
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
        this.pop_register()?;
        count += Data::Size::one();
    }

    this.end_list().and_then(|r| this.push_register(r))?;

    Ok(None)
}

pub(crate) fn get_access_addr<Data: GarnishData>(
    this: &mut Data,
    right: Data::Size,
    left: Data::Size,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    match this.get_data_type(right.clone())? {
        GarnishDataType::Number => {
            let i = this.get_number(right)?;
            access_with_integer(this, i, left)
        }
        GarnishDataType::Symbol => {
            let sym = this.get_symbol(right)?;
            access_with_symbol(this, sym, left)
        }
        _ => Err(RuntimeError::unsupported_types()),
    }
}

pub(crate) fn access_with_integer<Data: GarnishData>(
    this: &mut Data,
    index: Data::Number,
    value: Data::Size,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    match this.get_data_type(value.clone())? {
        GarnishDataType::List => index_list(this, value, index),
        GarnishDataType::CharList => index_char_list(this, value, index),
        GarnishDataType::ByteList => index_byte_list(this, value, index),
        GarnishDataType::Range => {
            let (start, end) = this.get_range(value)?;
            match (this.get_data_type(start.clone())?, this.get_data_type(end.clone())?) {
                (GarnishDataType::Number, GarnishDataType::Number) => {
                    let start_int = this.get_number(start)?;
                    let end_int = this.get_number(end)?;
                    let len = range_len::<Data>(start_int.clone(), end_int)?;

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
        GarnishDataType::Slice => {
            let (value, range) = this.get_slice(value)?;
            let (start, _, _) = get_range(this, range)?;
            let adjusted_index = start.plus(index).or_num_err()?;

            match this.get_data_type(value.clone())? {
                GarnishDataType::List => index_list(this, value, adjusted_index),
                GarnishDataType::CharList => index_char_list(this, value, adjusted_index),
                GarnishDataType::ByteList => index_byte_list(this, value, adjusted_index),
                GarnishDataType::Concatenation => Ok(iterate_concatenation_mut(this, value, |_this, index, addr| {
                    if index == adjusted_index {
                        return Ok(Some(addr));
                    }

                    Ok(None)
                })?
                .0),
                t => state_error(format!("Invalid value for slice {:?}", t)),
            }
        }
        GarnishDataType::Concatenation => index_concatenation_for(this, value, index),
        _ => Err(RuntimeError::unsupported_types()),
    }
}

pub(crate) fn index_concatenation_for<Data: GarnishData>(
    this: &mut Data,
    addr: Data::Size,
    index: Data::Number,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    Ok(iterate_concatenation_mut(this, addr, |_this, current_index, addr| {
        if current_index == index {
            return Ok(Some(addr));
        }

        Ok(None)
    })?
    .0)
}

fn index_list<Data: GarnishData>(
    this: &mut Data,
    list: Data::Size,
    index: Data::Number,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    if index < Data::Number::zero() {
        Ok(None)
    } else {
        if index >= Data::size_to_number(this.get_list_len(list.clone())?) {
            Ok(None)
        } else {
            Ok(Some(this.get_list_item(list, index)?))
        }
    }
}

fn index_char_list<Data: GarnishData>(
    this: &mut Data,
    list: Data::Size,
    index: Data::Number,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    if index < Data::Number::zero() {
        Ok(None)
    } else {
        if index >= Data::size_to_number(this.get_char_list_len(list.clone())?) {
            Ok(None)
        } else {
            let c = this.get_char_list_item(list, index)?;
            let addr = this.add_char(c)?;
            Ok(Some(addr))
        }
    }
}

fn index_byte_list<Data: GarnishData>(
    this: &mut Data,
    list: Data::Size,
    index: Data::Number,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    if index < Data::Number::zero() {
        Ok(None)
    } else {
        if index >= Data::size_to_number(this.get_byte_list_len(list.clone())?) {
            Ok(None)
        } else {
            let c = this.get_byte_list_item(list, index)?;
            let addr = this.add_byte(c)?;
            Ok(Some(addr))
        }
    }
}

pub(crate) fn access_with_symbol<Data: GarnishData>(
    this: &mut Data,
    sym: Data::Symbol,
    value: Data::Size,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    match this.get_data_type(value.clone())? {
        GarnishDataType::List => Ok(this.get_list_item_with_symbol(value, sym.clone())?),
        GarnishDataType::Slice => {
            let (value, range) = this.get_slice(value)?;
            let (start, end, _) = get_range(this, range)?;

            match this.get_data_type(value.clone())? {
                GarnishDataType::List => {
                    // in order to limit to slice range need to check items manually
                    // can't push, in case any of the items are a Link or Slice
                    let mut i = start;
                    let mut item: Option<Data::Size> = None;
                    let length = Data::size_to_number(this.get_list_len(value.clone())?);

                    let end = if end >= length {
                        length.subtract(Data::Number::one()).or_num_err()?
                    } else {
                        end
                    };

                    // need the latest value, being the value closest to the end for symbol access
                    // check entire concatenation, reassigning found each time we find a match
                    while i <= end {
                        let list_item = this.get_list_item(value.clone(), i.clone())?;
                        match this.get_data_type(list_item.clone())? {
                            GarnishDataType::Pair => {
                                let (left, right) = this.get_pair(list_item)?;
                                match this.get_data_type(left.clone())? {
                                    GarnishDataType::Symbol => {
                                        if this.get_symbol(left)? == sym {
                                            item = Some(right);
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
                GarnishDataType::Concatenation => {
                    let mut found = None;
                    iterate_concatenation_mut(this, value, |this, index, addr| {
                        if index > start && index <= end {
                            // in range
                            // need the latest value, being the value closest to the end for symbol access
                            // check entire concatenation, reassigning found each time we find a match
                            let item = get_value_if_association(this, addr, sym.clone())?;
                            match item {
                                None => {}
                                Some(i) => {
                                    found = Some(i);
                                }
                            }
                        }

                        Ok(None)
                    })?;

                    Ok(found)
                }
                t => state_error(format!("Invalid value for slice {:?}", t)),
            }
        }
        GarnishDataType::Concatenation => {
            Ok(iterate_rev_concatenation_mut(this, value, |this, _index, addr| get_value_if_association(this, addr, sym.clone()))?.0)
        }
        _ => Err(RuntimeError::unsupported_types()),
    }
}

pub(crate) fn get_value_if_association<Data: GarnishData>(
    this: &mut Data,
    addr: Data::Size,
    sym: Data::Symbol,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    match this.get_data_type(addr.clone())? {
        GarnishDataType::Pair => {
            let (left, right) = this.get_pair(addr)?;
            match this.get_data_type(left.clone())? {
                GarnishDataType::Symbol => {
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

pub(crate) fn is_value_association<Data: GarnishData>(this: &Data, addr: Data::Size) -> Result<bool, RuntimeError<Data::Error>> {
    Ok(match this.get_data_type(addr.clone())? {
        GarnishDataType::Pair => {
            let (left, _right) = this.get_pair(addr)?;
            match this.get_data_type(left)? {
                GarnishDataType::Symbol => true,
                _ => false,
            }
        }
        _ => false,
    })
}
