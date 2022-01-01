use crate::{GarnishLangRuntimeData, RuntimeError};

pub trait GarnishLangRuntimeContext<Data>
where
    Data: GarnishLangRuntimeData,
{
    fn resolve(&mut self, symbol: Data::Symbol, runtime: &mut Data) -> Result<bool, RuntimeError<Data::Error>>;
    fn apply(&mut self, external_value: Data::Size, input_addr: Data::Size, runtime: &mut Data) -> Result<bool, RuntimeError<Data::Error>>;
}

pub struct EmptyContext {}

impl<Data> GarnishLangRuntimeContext<Data> for EmptyContext
where
    Data: GarnishLangRuntimeData,
{
    fn resolve(&mut self, _: Data::Symbol, _: &mut Data) -> Result<bool, RuntimeError<Data::Error>> {
        Ok(false)
    }

    fn apply(&mut self, _: Data::Size, _: Data::Size, _: &mut Data) -> Result<bool, RuntimeError<Data::Error>> {
        Ok(false)
    }
}
