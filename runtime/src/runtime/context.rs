use crate::{GarnishLangRuntimeData, GarnishLangRuntimeResult};

pub trait GarnishLangRuntimeContext<Data>
where
    Data: GarnishLangRuntimeData,
{
    fn resolve(&mut self, symbol: Data::Symbol, runtime: &mut Data) -> GarnishLangRuntimeResult<Data::Error, bool>;
    fn apply(&mut self, external_value: Data::Size, input_addr: Data::Size, runtime: &mut Data) -> GarnishLangRuntimeResult<Data::Error, bool>;
}

pub struct EmptyContext {}

impl<Data> GarnishLangRuntimeContext<Data> for EmptyContext
where
    Data: GarnishLangRuntimeData,
{
    fn resolve(&mut self, _: Data::Symbol, _: &mut Data) -> GarnishLangRuntimeResult<Data::Error, bool> {
        Ok(false)
    }

    fn apply(&mut self, _: Data::Size, _: Data::Size, _: &mut Data) -> GarnishLangRuntimeResult<Data::Error, bool> {
        Ok(false)
    }
}
