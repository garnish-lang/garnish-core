use crate::runtime::utilities::{next_two_raw_ref, push_pair};
use garnish_lang_traits::{GarnishData, RuntimeError};

pub fn make_pair<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let (right_addr, left_addr) = next_two_raw_ref(this)?;

    push_pair(this, left_addr, right_addr)?;

    Ok(None)
}
