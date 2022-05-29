use crate::{next_two_raw_ref, push_pair, GarnishLangRuntimeData, RuntimeError};

pub(crate) fn make_pair<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    let (right_addr, left_addr) = next_two_raw_ref(this)?;

    push_pair(this, left_addr, right_addr)
}
