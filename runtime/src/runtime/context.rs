use crate::{GarnishLangRuntime, GarnishLangRuntimeResult};

pub trait GarnishLangRuntimeContext {
    fn resolve(&mut self, symbol_addr: usize, runtime: &mut GarnishLangRuntime) -> GarnishLangRuntimeResult<bool>;
    fn apply(&mut self, external_value: usize, input_addr: usize, runtime: &mut GarnishLangRuntime) -> GarnishLangRuntimeResult<bool>;
}

pub struct EmptyContext {}

impl GarnishLangRuntimeContext for EmptyContext {
    fn resolve(&mut self, _: usize, _: &mut GarnishLangRuntime) -> GarnishLangRuntimeResult<bool> {
        Ok(false)
    }

    fn apply(&mut self, _: usize, _: usize, _: &mut GarnishLangRuntime) -> GarnishLangRuntimeResult<bool> {
        Ok(false)
    }
}
