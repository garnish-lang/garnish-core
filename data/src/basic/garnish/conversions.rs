use crate::{BasicDataCustom, BasicGarnishData, DataError};

impl<T> BasicGarnishData<T> where T: BasicDataCustom {
    pub(crate) fn convert_basic_value_at_to_char_list(&mut self, from: usize) -> Result<usize, DataError> {
        todo!()
    }

    pub(crate) fn convert_basic_value_at_to_byte_list(&mut self, from: usize) -> Result<usize, DataError> {
        todo!()
    }

    pub(crate) fn convert_basic_value_at_to_number(&mut self, from: usize) -> Result<usize, DataError> {
        todo!()
    }

    pub(crate) fn convert_basic_value_at_to_byte(&mut self, from: usize) -> Result<u8, DataError> {
        todo!()
    }

    pub(crate) fn convert_basic_value_at_to_char(&mut self, from: usize) -> Result<char, DataError> {
        todo!()
    }

    pub(crate) fn convert_basic_value_at_to_symbol(&mut self, from: usize) -> Result<u64, DataError> {
        todo!()
    }
}