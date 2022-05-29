use crate::{next_ref, push_unit, state_error, GarnishLangRuntimeData, RuntimeError};

pub(crate) fn put<Data: GarnishLangRuntimeData>(this: &mut Data, i: Data::Size) -> Result<(), RuntimeError<Data::Error>> {
    match i >= this.get_data_len() {
        true => state_error(format!(
            "Attempting to put reference to {:?} which is outside of data bounds {:?}.",
            i,
            this.get_data_len()
        ))?,
        false => this.push_register(i)?,
    }

    Ok(())
}

pub(crate) fn put_input<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    match this.get_current_value() {
        None => push_unit(this)?,
        Some(i) => this.push_register(i)?,
    }

    Ok(())
}

pub(crate) fn push_input<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    let r = next_ref(this)?;

    this.push_value_stack(r)?;

    Ok(())
}

pub(crate) fn push_result<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    let r = next_ref(this)?;
    match this.get_current_value_mut() {
        None => state_error(format!("No inputs availble to update for update value operation."))?,
        Some(v) => *v = r,
    }

    Ok(())
}
