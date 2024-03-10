use garnish_lang_traits::{RuntimeError, GarnishDataType, GarnishData, GarnishNumber};
use crate::runtime::utilities::{next_two_raw_ref, push_unit};
use crate::runtime::error::OrNumberError;

pub(crate) fn make_range<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    make_range_internal(this, false, false)
}

pub(crate) fn make_start_exclusive_range<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    make_range_internal(this, true, false)
}

pub(crate) fn make_end_exclusive_range<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    make_range_internal(this, false, true)
}

pub(crate) fn make_exclusive_range<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    make_range_internal(this, true, true)
}

pub(crate) fn range_len<Data: GarnishData>(start: Data::Number, end: Data::Number) -> Result<Data::Number, RuntimeError<Data::Error>> {
    end.subtract(start).or_num_err()?.increment().or_num_err()
}

fn make_range_internal<Data: GarnishData>(
    this: &mut Data,
    start_exclusive: bool,
    end_exclusive: bool,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let (right_addr, left_addr) = next_two_raw_ref(this)?;
    let types = (this.get_data_type(left_addr.clone())?, this.get_data_type(right_addr.clone())?);

    match types {
        (GarnishDataType::Number, GarnishDataType::Number) => {
            let left_addr = if start_exclusive {
                this.add_number(this.get_number(left_addr)?.increment().or_num_err()?)?
            } else {
                left_addr
            };

            let right_addr = if end_exclusive {
                this.add_number(this.get_number(right_addr)?.decrement().or_num_err()?)?
            } else {
                right_addr
            };

            let addr = this.add_range(left_addr, right_addr)?;
            this.push_register(addr)?;
        }
        _ => {
            push_unit(this)?;
        }
    }

    Ok(None)
}
