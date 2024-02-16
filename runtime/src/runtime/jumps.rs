use log::trace;

use crate::{next_ref, state_error, ExpressionDataType, GarnishLangRuntimeData, RuntimeError};

pub(crate) fn jump<Data: GarnishLangRuntimeData>(this: &mut Data, index: Data::Size) -> Result<(), RuntimeError<Data::Error>> {
    match this.get_jump_point(index) {
        None => state_error(format!("No jump point at index {:?}", index))?,
        Some(point) => {
            trace!("Jumping to point {:?}", point);
            this.set_instruction_cursor(point)?;
        }
    }

    Ok(())
}

pub(crate) fn jump_if_true<Data: GarnishLangRuntimeData>(this: &mut Data, index: Data::Size) -> Result<(), RuntimeError<Data::Error>> {
    let point = match this.get_jump_point(index) {
        None => state_error(format!("No jump point at index {:?}", index))?,
        Some(point) => point,
    };

    let d = next_ref(this)?;

    match this.get_data_type(d)? {
        ExpressionDataType::False | ExpressionDataType::Unit => {
            trace!("Not jumping from value of type {:?} with addr {:?}", this.get_data_type(d)?, d);
        }
        // all other values are considered true
        t => {
            trace!("Jumping from value of type {:?} with addr {:?} to point {:?}", t, d, point);
            this.set_instruction_cursor(point)?
        }
    };

    Ok(())
}

pub(crate) fn jump_if_false<Data: GarnishLangRuntimeData>(this: &mut Data, index: Data::Size) -> Result<(), RuntimeError<Data::Error>> {
    let point = match this.get_jump_point(index) {
        None => state_error(format!("No jump point at index {:?}", index))?,
        Some(point) => point,
    };

    let d = next_ref(this)?;

    match this.get_data_type(d)? {
        ExpressionDataType::False | ExpressionDataType::Unit => {
            trace!("Jumping from value of type {:?} with addr {:?} to point {:?}", this.get_data_type(d)?, d, point);
            this.set_instruction_cursor(point)?
        }
        t => {
            trace!("Not jumping from value of type {:?} with addr {:?}", t, d);
        }
    };

    Ok(())
}

pub(crate) fn end_expression<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    match this.pop_jump_path() {
        None => {
            // no more jumps, this should be the end of the entire execution
            let r = next_ref(this)?;
            trace!(
                "No remaining return points. Pushing {:?} to values. Setting cursor to instruction length {:?}.",
                r,
                this.get_instruction_len()
            );
            this.set_instruction_cursor(this.get_instruction_len())?;

            match this.get_current_value_mut() {
                None => state_error(format!("No inputs available to update during end expression operation."))?,
                Some(v) => *v = r,
            }
        }
        Some(jump_point) => {
            trace!("Setting cursor to {:?}", jump_point);
            this.set_instruction_cursor(jump_point)?;
            this.pop_value_stack();
        }
    }

    Ok(())
}
