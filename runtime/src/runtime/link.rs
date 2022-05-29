use crate::{next_two_raw_ref, ExpressionDataType, GarnishLangRuntimeData, RuntimeError};

pub(crate) fn append_link<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    let (right, left) = next_two_raw_ref(this)?;
    link_internal(this, right, left, true)
}

pub(crate) fn prepend_link<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    let (right, left) = next_two_raw_ref(this)?;
    link_internal(this, left, right, false)
}

pub fn link_internal<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    value: Data::Size,
    link_to: Data::Size,
    is_append: bool,
) -> Result<(), RuntimeError<Data::Error>> {
    match this.get_data_type(link_to)? {
        ExpressionDataType::Link => {
            let value = match this.get_data_type(value)? {
                ExpressionDataType::Link => {
                    let (addr, ..) = this.get_link(value)?;
                    addr
                }
                _ => value,
            };

            // create new link with value and link_to as linked
            let addr = this.add_link(value, link_to, is_append)?;
            this.push_register(addr)?;
        }
        _ => {
            let unit = this.add_unit()?;
            // unit is next value
            let linked = this.add_link(link_to, unit, is_append)?;
            // linked is next value
            let addr = this.add_link(value, linked, is_append)?;
            this.push_register(addr)?;
        }
    }

    Ok(())
}
