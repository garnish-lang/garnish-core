use crate::{GarnishLangRuntime, GarnishLangRuntimeResult};

use super::data::GarnishLangRuntimeDataPool;

pub trait GarnishLangRuntimeContext {
    fn resolve<Data: GarnishLangRuntimeDataPool>(
        &mut self,
        symbol_addr: usize,
        runtime: &mut GarnishLangRuntime<Data>,
    ) -> GarnishLangRuntimeResult<bool>;
    fn apply<Data: GarnishLangRuntimeDataPool>(
        &mut self,
        external_value: usize,
        input_addr: usize,
        runtime: &mut GarnishLangRuntime<Data>,
    ) -> GarnishLangRuntimeResult<bool>;
}

pub struct EmptyContext {}

impl GarnishLangRuntimeContext for EmptyContext {
    fn resolve<Data>(&mut self, _: usize, _: &mut GarnishLangRuntime<Data>) -> GarnishLangRuntimeResult<bool> {
        Ok(false)
    }

    fn apply<Data>(&mut self, _: usize, _: usize, _: &mut GarnishLangRuntime<Data>) -> GarnishLangRuntimeResult<bool> {
        Ok(false)
    }
}
