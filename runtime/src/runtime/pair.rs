use crate::runtime::utilities::{next_two_raw_ref, push_pair};
use garnish_lang_traits::{GarnishLangRuntimeData, RuntimeError};

pub(crate) fn make_pair<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let (right_addr, left_addr) = next_two_raw_ref(this)?;

    push_pair(this, left_addr, right_addr)?;

    Ok(None)
}
