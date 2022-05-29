// use log::trace;

use crate::runtime::list::iterate_link_internal;
use crate::runtime::range::range_len;
use crate::{state_error, ExpressionDataType, GarnishLangRuntimeData, GarnishNumber, OrNumberError, RuntimeError, TypeConstants};

pub(crate) fn next_ref<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<Data::Size, RuntimeError<Data::Error>> {
    match this.pop_register() {
        None => state_error(format!("No references in register."))?,
        Some(i) => Ok(i),
    }
}

pub(crate) fn next_two_raw_ref<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(Data::Size, Data::Size), RuntimeError<Data::Error>> {
    let first_ref = next_ref(this)?;
    let second_ref = next_ref(this)?;

    Ok((first_ref, second_ref))
}

pub(crate) fn get_range<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    addr: Data::Size,
) -> Result<(Data::Number, Data::Number, Data::Number), RuntimeError<Data::Error>> {
    let (start, end) = this.get_range(addr)?;
    let (start, end) = match (this.get_data_type(start)?, this.get_data_type(end)?) {
        (ExpressionDataType::Number, ExpressionDataType::Number) => (this.get_number(start)?, this.get_number(end)?),
        (s, e) => state_error(format!("Invalid range values {:?} {:?}", s, e))?,
    };

    Ok((start, end, range_len::<Data>(start, end)?))
}

// push utilities

pub(crate) fn push_unit<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    this.add_unit().and_then(|v| this.push_register(v))?;
    Ok(())
}

pub(crate) fn push_number<Data: GarnishLangRuntimeData>(this: &mut Data, value: Data::Number) -> Result<(), RuntimeError<Data::Error>> {
    this.add_number(value).and_then(|v| this.push_register(v))?;
    Ok(())
}

pub(crate) fn push_boolean<Data: GarnishLangRuntimeData>(this: &mut Data, value: bool) -> Result<(), RuntimeError<Data::Error>> {
    match value {
        true => this.add_true(),
        false => this.add_false(),
    }
    .and_then(|v| this.push_register(v))?;

    Ok(())
}

pub(crate) fn push_pair<Data: GarnishLangRuntimeData>(this: &mut Data, left: Data::Size, right: Data::Size) -> Result<(), RuntimeError<Data::Error>> {
    this.add_pair((left, right)).and_then(|v| this.push_register(v))?;
    Ok(())
}

// public utilities

// modify so 'this' doesn't have to be mutable
pub fn iterate_link<Data: GarnishLangRuntimeData, Callback>(
    this: &mut Data,
    link: Data::Size,
    func: Callback,
) -> Result<(), RuntimeError<Data::Error>>
where
    Callback: FnMut(&mut Data, Data::Size, Data::Number) -> Result<bool, RuntimeError<Data::Error>>,
{
    iterate_link_internal(this, link, func)
}

pub fn link_count<Data: GarnishLangRuntimeData>(this: &mut Data, link: Data::Size) -> Result<Data::Number, RuntimeError<Data::Error>> {
    let mut count = Data::Number::zero();

    iterate_link_internal(this, link, |_, _, _| {
        count = count.increment().or_num_err()?;
        Ok(false)
    })?;

    Ok(count)
}
