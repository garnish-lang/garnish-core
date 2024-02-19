use crate::{state_error, GarnishLangRuntimeData, RuntimeError};

pub(crate) fn start_side_effect<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    match this.get_current_value() {
        None => {
            let r = this.add_unit()?;
            this.push_value_stack(r)?;
        }
        Some(r) => this.push_value_stack(r)?,
    }

    Ok(None)
}

pub(crate) fn end_side_effect<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    match this.pop_value_stack() {
        Some(_) => (),
        None => state_error("Could not pop value at end of side effect.".to_string())?,
    }

    match this.pop_register() {
        Some(_) => Ok(None),
        None => state_error("Could not pop register at end of side effect.".to_string()),
    }
}
