use crate::runtime::error::state_error;
use garnish_lang_traits::{GarnishData, RuntimeError};

pub(crate) fn start_side_effect<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    match this.get_current_value() {
        None => {
            let r = this.add_unit()?;
            this.push_value_stack(r)?;
        }
        Some(r) => this.push_value_stack(r)?,
    }

    Ok(None)
}

pub(crate) fn end_side_effect<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    match this.pop_value_stack() {
        Some(_) => (),
        None => state_error("Could not pop value at end of side effect.".to_string())?,
    }

    match this.pop_register()? {
        Some(_) => Ok(None),
        None => state_error("Could not pop register at end of side effect.".to_string()),
    }
}
