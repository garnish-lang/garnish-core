use crate::{next_ref, next_two_raw_ref, push_number, push_unit, ExpressionDataType, GarnishLangRuntimeData, GarnishNumber, RuntimeError};

pub fn bitwise_not<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    todo!()
}

pub fn bitwise_and<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    todo!()
}

pub fn bitwise_or<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    todo!()
}

pub fn bitwise_xor<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    todo!()
}

pub fn bitwise_shift_left<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    todo!()
}

pub fn bitwise_shift_right<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, SimpleRuntimeData};
}
