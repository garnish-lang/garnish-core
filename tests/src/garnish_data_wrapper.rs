use garnish_lang::{data_wrapper_proc, GarnishData};
use garnish_lang::simple::SimpleGarnishData;

pub struct DataWrapper {
    data: SimpleGarnishData,
}

#[data_wrapper_proc(delegate_field = data, delegate_field_type = SimpleGarnishData)]
impl GarnishData for DataWrapper {
    fn resolve(&mut self, _symbol: Self::Symbol) -> Result<bool, Self::Error> {
        Ok(false)
    }
}