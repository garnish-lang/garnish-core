use garnish_lang::{delegate_garnish_data, GarnishData};
use garnish_lang::simple::SimpleGarnishData;

pub struct DataWrapper {
    data: SimpleGarnishData,
}

#[delegate_garnish_data(delegate_field = data, delegate_field_type = SimpleGarnishData)]
impl GarnishData for DataWrapper {
    fn resolve(&mut self, _symbol: Self::Symbol) -> Result<bool, Self::Error> {
        Ok(false)
    }
}