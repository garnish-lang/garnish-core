use garnish_lang_traits::{ErrorType, GarnishDataType, GarnishData, RuntimeError};
use crate::runtime::list::get_access_addr;
use crate::runtime::utilities::push_unit;

pub fn resolve<Data: GarnishData>(
    this: &mut Data,
    data: Data::Size,
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
    match this.get_data_type(data.clone())? {
        GarnishDataType::Symbol => {
            match this.resolve(this.get_symbol(data)?)? {
                true => return Ok(None), // context resolved end look up
                false => (),           // not resolved fall through
            }
        }
        _ => (), // not a symbol push unit below
    }

    // default to unit
    push_unit(this)?;

    Ok(None)
}

#[cfg(test)]
mod tests {
    use garnish_lang_traits::{GarnishData, GarnishDataType};
    use crate::ops::resolve;
    use crate::runtime::tests::MockGarnishData;

    #[test]
    fn calls_data_resolve() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::List, GarnishDataType::Symbol]);
        mock_data.data.registers.push(10);
        mock_data.data.registers.push(20);
        
        mock_data.stub_get_current_value = |_| None;
        mock_data.stub_get_symbol = |_, _| Ok(200);
        mock_data.stub_resolve = |data, symbol| {
            data.registers.push(symbol as i32);
            Ok(true)
        };

        resolve(&mut mock_data, 1).unwrap();
        
        assert_eq!(mock_data.pop_register().unwrap(), Some(200));
    }
}