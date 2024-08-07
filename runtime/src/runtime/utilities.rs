// use log::trace;

use crate::runtime::range::range_len;
use garnish_lang_traits::{GarnishDataType, GarnishData, RuntimeError};
use crate::runtime::error::state_error;

pub(crate) fn next_ref<Data: GarnishData>(this: &mut Data) -> Result<Data::Size, RuntimeError<Data::Error>> {
    match this.pop_register()? {
        None => state_error(format!("No references in register."))?,
        Some(i) => Ok(i),
    }
}

pub(crate) fn next_two_raw_ref<Data: GarnishData>(this: &mut Data) -> Result<(Data::Size, Data::Size), RuntimeError<Data::Error>> {
    let first_ref = next_ref(this)?;
    let second_ref = next_ref(this)?;

    Ok((first_ref, second_ref))
}

pub(crate) fn get_range<Data: GarnishData>(
    this: &mut Data,
    addr: Data::Size,
) -> Result<(Data::Number, Data::Number, Data::Number), RuntimeError<Data::Error>> {
    let (start, end) = this.get_range(addr)?;
    let (start, end) = match (this.get_data_type(start.clone())?, this.get_data_type(end.clone())?) {
        (GarnishDataType::Number, GarnishDataType::Number) => (this.get_number(start)?, this.get_number(end)?),
        (s, e) => state_error(format!("Invalid range values {:?} {:?}", s, e))?,
    };

    Ok((start.clone(), end.clone(), range_len::<Data>(start, end)?))
}

// push utilities

pub(crate) fn push_unit<Data: GarnishData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    this.add_unit().and_then(|v| this.push_register(v))?;
    Ok(())
}

pub(crate) fn push_number<Data: GarnishData>(this: &mut Data, value: Data::Number) -> Result<(), RuntimeError<Data::Error>> {
    this.add_number(value).and_then(|v| this.push_register(v))?;
    Ok(())
}

pub(crate) fn push_boolean<Data: GarnishData>(this: &mut Data, value: bool) -> Result<(), RuntimeError<Data::Error>> {
    match value {
        true => this.add_true(),
        false => this.add_false(),
    }
    .and_then(|v| this.push_register(v))?;

    Ok(())
}

pub(crate) fn push_pair<Data: GarnishData>(this: &mut Data, left: Data::Size, right: Data::Size) -> Result<(), RuntimeError<Data::Error>> {
    this.add_pair((left, right)).and_then(|v| this.push_register(v))?;
    Ok(())
}
