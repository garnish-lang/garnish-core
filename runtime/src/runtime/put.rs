use crate::runtime::error::state_error;
use crate::runtime::utilities::{next_ref, push_unit};
use garnish_lang_traits::{GarnishData, RuntimeError};

pub(crate) fn put<Data: GarnishData>(this: &mut Data, i: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    match i >= this.get_data_len() {
        true => state_error(format!(
            "Attempting to put reference to {:?} which is outside of data bounds {:?}.",
            i,
            this.get_data_len()
        ))?,
        false => this.push_register(i)?,
    }

    Ok(None)
}

pub(crate) fn put_value<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    match this.get_current_value() {
        None => push_unit(this)?,
        Some(i) => this.push_register(i)?,
    }

    Ok(None)
}

pub(crate) fn push_value<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let r = next_ref(this)?;

    this.push_value_stack(r)?;

    Ok(None)
}

pub(crate) fn upldate_value<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let r = next_ref(this)?;
    match this.get_current_value_mut() {
        None => state_error(format!("No inputs available to update for update value operation."))?,
        Some(v) => *v = r,
    }

    Ok(None)
}
