use garnish_lang_traits::{ErrorType, GarnishDataType, GarnishContext, GarnishData, RuntimeError,};
use crate::runtime::list::get_access_addr;
use crate::runtime::utilities::push_unit;

pub fn resolve<Data: GarnishData, T: GarnishContext<Data>>(
    this: &mut Data,
    data: Data::Size,
    context: Option<&mut T>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    // check input
    match this.get_current_value() {
        None => (),
        Some(list_ref) => match get_access_addr(this, data.clone(), list_ref) {
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
                    return Ok(None);
                }
            },
        },
    }

    // check context
    match context {
        None => (),
        Some(c) => match this.get_data_type(data.clone())? {
            GarnishDataType::Symbol => {
                match c.resolve(this.get_symbol(data)?, this)? {
                    true => return Ok(None), // context resovled end look up
                    false => (),           // not resolved fall through
                }
            }
            _ => (), // not a symbol push unit below
        },
    }

    // default to unit
    push_unit(this)?;

    Ok(None)
}
