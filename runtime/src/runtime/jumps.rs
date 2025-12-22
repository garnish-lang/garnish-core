use log::trace;

use crate::runtime::error::state_error;
use crate::runtime::utilities::next_ref;
use garnish_lang_traits::{GarnishDataType, GarnishData, RuntimeError};

pub fn jump<Data: GarnishData>(this: &mut Data, index: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    match this.get_from_jump_table(index.clone()) {
        None => state_error(format!("No jump point at index {:?}", index))?,
        Some(point) => {
            trace!("Jumping to point {:?}", point);
            return Ok(Some(point));
        }
    }

    Ok(None)
}

pub fn jump_if_true<Data: GarnishData>(
    this: &mut Data,
    index: Data::Size,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let point = match this.get_from_jump_table(index.clone()) {
        None => state_error(format!("No jump point at index {:?}", index))?,
        Some(point) => point,
    };

    let d = next_ref(this)?;

    match this.get_data_type(d.clone())? {
        GarnishDataType::False | GarnishDataType::Unit => {
            trace!("Not jumping from value of type {:?} with addr {:?}", this.get_data_type(d.clone())?, d);
            Ok(None)
        }
        // all other values are considered true
        t => {
            trace!("Jumping from value of type {:?} with addr {:?} to point {:?}", t, d, point);
            Ok(Some(point))
        }
    }
}

pub fn jump_if_false<Data: GarnishData>(
    this: &mut Data,
    index: Data::Size,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let point = match this.get_from_jump_table(index.clone()) {
        None => state_error(format!("No jump point at index {:?}", index))?,
        Some(point) => point,
    };

    let d = next_ref(this)?;

    match this.get_data_type(d.clone())? {
        GarnishDataType::False | GarnishDataType::Unit => {
            trace!(
                "Jumping from value of type {:?} with addr {:?} to point {:?}",
                this.get_data_type(d.clone())?,
                d,
                point
            );
            Ok(Some(point))
        }
        t => {
            trace!("Not jumping from value of type {:?} with addr {:?}", t, d);
            Ok(None)
        }
    }
}

pub fn end_expression<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    // store return value
    let r = next_ref(this)?;

    match this.pop_frame()? {
        None => {
            // no more jumps, this should be the end of the entire execution
            trace!(
                "No remaining return points. Pushing {:?} to values. Setting cursor to instruction length {:?}.",
                r,
                this.get_instruction_len()
            );

            // set value to ended expressions return value
            match this.get_current_value_mut() {
                None => state_error(format!("No inputs available to update during end expression operation."))?,
                Some(v) => *v = r,
            }

            Ok(Some(this.get_instruction_len()))
        }
        Some(jump_point) => {
            trace!("Setting cursor to {:?}", jump_point);
            this.pop_value_stack();
            this.push_register(r)?;

            Ok(Some(jump_point))
        }
    }
}
