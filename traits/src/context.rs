use crate::{ExpressionDataType, GarnishLangRuntimeData, Instruction, RuntimeError};

pub trait GarnishLangRuntimeContext<Data>
    where
        Data: GarnishLangRuntimeData,
{
    fn resolve(&mut self, _symbol: Data::Symbol, _runtime: &mut Data) -> Result<bool, RuntimeError<Data::Error>> {
        Ok(false)
    }

    fn apply(&mut self, _external_value: Data::Size, _input_addr: Data::Size, _runtime: &mut Data) -> Result<bool, RuntimeError<Data::Error>> {
        Ok(false)
    }

    fn defer_op(
        &mut self,
        _runtime: &mut Data,
        _operation: Instruction,
        _left: (ExpressionDataType, Data::Size),
        _right: (ExpressionDataType, Data::Size),
    ) -> Result<bool, RuntimeError<Data::Error>> {
        Ok(false)
    }
}

pub struct EmptyContext {}

pub const EMPTY_CONTEXT: EmptyContext = EmptyContext {};
pub const NO_CONTEXT: Option<&mut EmptyContext> = None;

impl<Data> GarnishLangRuntimeContext<Data> for EmptyContext where Data: GarnishLangRuntimeData {}
