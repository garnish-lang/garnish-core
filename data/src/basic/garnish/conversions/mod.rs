mod string;

use crate::{BasicData, BasicDataCustom, BasicGarnishData, DataError};

pub trait ConversionDelegate<T>
where
    T: BasicDataCustom,
{
    type Output;
    fn init(&mut self) -> Result<(), DataError>;
    fn push_char(&mut self, c: char) -> Result<(), DataError>;
    fn get_data_at(&self, index: usize) -> Result<&BasicData<T>, DataError>;
    fn end(self) -> Result<Self::Output, DataError>;
    fn data(&self) -> &BasicGarnishData<T>;
}
