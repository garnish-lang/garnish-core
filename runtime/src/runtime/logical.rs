use crate::runtime::utilities::{next_ref, next_two_raw_ref, push_boolean};
use garnish_lang_traits::{ExpressionDataType, GarnishLangRuntimeData, RuntimeError};

pub(crate) fn and<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let (left, right) = next_two_raw_ref(this)?;

    let result = match (is_true_value(this, left)?, is_true_value(this, right)?) {
        (true, true) => true,
        _ => false,
    };

    push_boolean(this, result)?;

    Ok(None)
}

pub fn or<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let (left, right) = next_two_raw_ref(this)?;

    let result = match (is_true_value(this, left)?, is_true_value(this, right)?) {
        (false, false) => false,
        _ => true,
    };

    push_boolean(this, result)?;

    Ok(None)
}

pub fn xor<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let (left, right) = next_two_raw_ref(this)?;

    let result = match (is_true_value(this, left)?, is_true_value(this, right)?) {
        (false, false) | (true, true) => false,
        _ => true,
    };

    push_boolean(this, result)?;

    Ok(None)
}

pub(crate) fn not<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let addr = next_ref(this)?;
    let result = is_true_value(this, addr)?;
    push_boolean(this, !result)?;

    Ok(None)
}

pub(crate) fn tis<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let addr = next_ref(this)?;
    let result = is_true_value(this, addr)?;
    push_boolean(this, result)?;

    Ok(None)
}

fn is_true_value<Data: GarnishLangRuntimeData>(this: &mut Data, addr: Data::Size) -> Result<bool, RuntimeError<Data::Error>> {
    Ok(match this.get_data_type(addr)? {
        ExpressionDataType::False | ExpressionDataType::Unit => false,
        _ => true,
    })
}
