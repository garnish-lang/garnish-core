use crate::{GarnishLangRuntimeData, GarnishLangRuntimeResult};

pub trait GarnishLangRuntimeContext<Data>
where
    Data: GarnishLangRuntimeData,
{
    fn resolve(&mut self, symbol_addr: usize, runtime: &mut Data) -> GarnishLangRuntimeResult<Data::Error, bool>;
    fn apply(&mut self, external_value: usize, input_addr: usize, runtime: &mut Data) -> GarnishLangRuntimeResult<Data::Error, bool>;
}

pub struct EmptyContext {}

impl<Data> GarnishLangRuntimeContext<Data> for EmptyContext
where
    Data: GarnishLangRuntimeData,
{
    fn resolve(&mut self, _: usize, _: &mut Data) -> GarnishLangRuntimeResult<Data::Error, bool> {
        Ok(false)
    }

    fn apply(&mut self, _: usize, _: usize, _: &mut Data) -> GarnishLangRuntimeResult<Data::Error, bool> {
        Ok(false)
    }
}
