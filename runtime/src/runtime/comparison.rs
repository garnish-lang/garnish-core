
use crate::{next_two_raw_ref, push_number, push_unit, ExpressionDataType, GarnishLangRuntimeData, GarnishNumber, RuntimeError, next_ref};

pub fn less_than<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    todo!()
}

pub fn less_than_or_equal<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    todo!()
}

pub fn greater_than<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    todo!()
}

pub fn greater_than_or_equal<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, SimpleRuntimeData};
}