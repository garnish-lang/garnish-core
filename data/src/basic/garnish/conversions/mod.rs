mod bytes;
mod string;
mod symbol;
mod number;

use crate::{BasicData, BasicDataCustom, BasicGarnishData, DataError, basic::companion::BasicDataCompanion};

pub trait ConversionDelegate<T, Input, Companion>
where
    T: BasicDataCustom,
    Companion: BasicDataCompanion<T>,
{
    type Output;

    fn init(&mut self) -> Result<(), DataError>;
    fn push_char(&mut self, c: Input) -> Result<(), DataError>;
    fn get_data_at(&self, index: usize) -> Result<&BasicData<T>, DataError>;
    fn end(self) -> Result<Self::Output, DataError>;
    fn data(&self) -> &BasicGarnishData<T, Companion>;
}
