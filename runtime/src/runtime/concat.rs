use crate::{next_two_raw_ref, GarnishLangRuntimeData, RuntimeError};

pub(crate) fn concat<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let (right_addr, left_addr) = next_two_raw_ref(this)?;

    this.add_concatenation(left_addr, right_addr).and_then(|v| this.push_register(v))?;

    Ok(None)
}
