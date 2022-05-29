use crate::{
    push_unit, runtime::list::get_access_addr, ErrorType, ExpressionDataType, GarnishLangRuntimeContext, GarnishLangRuntimeData, RuntimeError,
};

pub fn resolve<Data: GarnishLangRuntimeData, T: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    data: Data::Size,
    context: Option<&mut T>,
) -> Result<(), RuntimeError<Data::Error>> {
    // check input
    match this.get_current_value() {
        None => (),
        Some(list_ref) => match get_access_addr(this, data, list_ref) {
            Err(e) => {
                // ignore unsupported op type, will be handled by below resolve
                if e.get_type() != ErrorType::UnsupportedOpTypes {
                    Err(e)?;
                }
            }
            Ok(v) => match v {
                None => (),
                Some(i) => {
                    this.push_register(i)?;
                    return Ok(());
                }
            },
        },
    }

    // check context
    match context {
        None => (),
        Some(c) => match this.get_data_type(data)? {
            ExpressionDataType::Symbol => {
                match c.resolve(this.get_symbol(data)?, this)? {
                    true => return Ok(()), // context resovled end look up
                    false => (),           // not resolved fall through
                }
            }
            _ => (), // not a symbol push unit below
        },
    }

    // default to unit
    push_unit(this)
}
