use crate::runtime::utilities::next_two_raw_ref;
use garnish_lang_traits::{GarnishData, RuntimeError};

pub fn concat<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let (right_addr, left_addr) = next_two_raw_ref(this)?;

    this.add_concatenation(left_addr, right_addr).and_then(|v| this.push_register(v))?;

    Ok(None)
}
