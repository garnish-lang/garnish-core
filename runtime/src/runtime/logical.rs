use crate::runtime::error::state_error;
use crate::runtime::utilities::{next_ref, next_two_raw_ref, push_boolean};
use garnish_lang_traits::{GarnishData, GarnishDataType, RuntimeError};

pub fn and<Data: GarnishData>(this: &mut Data, data: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let value = next_ref(this)?;

    match is_true_value(this, value)? {
        true => match this.get_jump_point(data.clone()) {
            Some(v) => Ok(Some(v)),
            None => state_error(format!("No jump point at index {:?}", data)),
        },
        false => {
            push_boolean(this, false)?;
            Ok(None)
        }
    }
}

pub fn or<Data: GarnishData>(this: &mut Data, data: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let value = next_ref(this)?;

    match is_true_value(this, value)? {
        true => {
            push_boolean(this, true)?;
            Ok(None)
        }
        false => match this.get_jump_point(data.clone()) {
            Some(v) => Ok(Some(v)),
            None => state_error(format!("No jump point at index {:?}", data)),
        }
    }
}

pub fn xor<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let (left, right) = next_two_raw_ref(this)?;

    let result = match (is_true_value(this, left)?, is_true_value(this, right)?) {
        (false, false) | (true, true) => false,
        _ => true,
    };

    push_boolean(this, result)?;

    Ok(None)
}

pub fn not<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let addr = next_ref(this)?;
    let result = is_true_value(this, addr)?;
    push_boolean(this, !result)?;

    Ok(None)
}

pub fn tis<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let addr = next_ref(this)?;
    let result = is_true_value(this, addr)?;
    push_boolean(this, result)?;

    Ok(None)
}

fn is_true_value<Data: GarnishData>(this: &mut Data, addr: Data::Size) -> Result<bool, RuntimeError<Data::Error>> {
    Ok(match this.get_data_type(addr)? {
        GarnishDataType::False | GarnishDataType::Unit => false,
        _ => true,
    })
}
