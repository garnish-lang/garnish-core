use crate::derive::Data;
use garnish_lang::simple::{SimpleDataType, SimpleGarnishData};
use garnish_lang::{GarnishData, delegate_garnish_data};

pub struct DataWrapper {
    data: SimpleGarnishData,
}

#[delegate_garnish_data(delegate_field = data, delegate_field_type = SimpleGarnishData)]
impl GarnishData for DataWrapper {
    fn resolve(&mut self, _symbol: Self::Symbol) -> Result<bool, Self::Error> {
        Ok(false)
    }
}

pub struct DataWrapperWithGenerics {
    data: SimpleGarnishData<Data>,
}

#[delegate_garnish_data(delegate_field = data, delegate_field_type = SimpleGarnishData<Data>)]
impl GarnishData for DataWrapperWithGenerics {
    fn resolve(&mut self, _symbol: Self::Symbol) -> Result<bool, Self::Error> {
        Ok(false)
    }
}
pub struct DataWrapperWithUnresolvedGenerics<T>
where
    T: SimpleDataType,
{
    data: SimpleGarnishData<T>,
}

#[delegate_garnish_data(delegate_field = data, delegate_field_type = SimpleGarnishData<T>)]
impl<T> GarnishData for DataWrapperWithUnresolvedGenerics<T> where T: SimpleDataType {
    fn resolve(&mut self, _symbol: Self::Symbol) -> Result<bool, Self::Error> {
        Ok(false)
    }
}
