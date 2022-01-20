use crate::{GarnishLangRuntimeData, RuntimeError};

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
}

pub struct EmptyContext {}

impl<Data> GarnishLangRuntimeContext<Data> for EmptyContext where Data: GarnishLangRuntimeData {}
