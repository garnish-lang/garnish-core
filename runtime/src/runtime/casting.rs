use crate::runtime::range::range_len;
use crate::{get_range, next_ref, push_unit, state_error, ExpressionDataType, GarnishLangRuntimeData, RuntimeError, TypeConstants};

pub(crate) fn type_cast<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

}