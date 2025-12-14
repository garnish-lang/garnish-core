mod data;
mod garnish;
mod merge_to_symbol_list;

pub use data::{BasicData, BasicDataUnitCustom};

use crate::data::SimpleNumber;

pub type BasicNumber = SimpleNumber;

use crate::{DataError, error::DataErrorType};

pub trait BasicDataCustom: Clone {}

impl BasicDataCustom for () {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicGarnishData<T>
where T: BasicDataCustom {
    data: Vec<BasicData<T>>,
}

pub type BasicGarnishDataUnit = BasicGarnishData<()>;

impl<T> BasicGarnishData<T>
where T: BasicDataCustom {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn new_full(data: Vec<BasicData<T>>) -> Self {
        Self { data }
    }

    pub fn push_basic_data(&mut self, data: BasicData<T>) -> usize {
        let index = self.data.len();
        self.data.push(data);
        index
    }

    pub fn get_basic_data(&self, index: usize) -> Option<&BasicData<T>> {
        self.data.get(index)
    }

    pub fn get_basic_data_mut(&mut self, index: usize) -> Option<&mut BasicData<T>> {
        self.data.get_mut(index)
    }

    pub(crate) fn get_data_ensure_index(&self, index: usize) -> Result<&BasicData<T>, DataError> {
        match self.get_basic_data(index) {
            Some(data) => Ok(data),
            None => Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(index))),
        }
    }

    pub fn add_char_list_from_string(&mut self, s: impl AsRef<str>) -> Result<usize, DataError> {
        let index = self.push_basic_data(BasicData::CharList(s.as_ref().len()));
        for c in s.as_ref().chars() {
            self.push_basic_data(BasicData::Char(c));
        }
        Ok(index)
    }

    pub fn add_byte_list_from_vec(&mut self, v: impl AsRef<[u8]>) -> Result<usize, DataError> {
        let index = self.push_basic_data(BasicData::ByteList(v.as_ref().len()));
        for b in v.as_ref() {
            self.push_basic_data(BasicData::Byte(*b));
        }
        Ok(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_garnish_data() {
        BasicGarnishData::<()>::new();
    }

    #[test]
    fn add_basic_data() {
        let mut data = BasicGarnishDataUnit::new();

        let index = data.push_basic_data(BasicData::Unit);
        assert_eq!(index, 0);

        let index = data.push_basic_data(BasicData::True);
        assert_eq!(index, 1);
        
        assert_eq!(data.data, vec![BasicData::Unit, BasicData::True]);
    }

    #[test]
    fn get_basic_data() {
        let mut data = BasicGarnishDataUnit::new();
        let index1 = data.push_basic_data(BasicData::Unit);
        let index2 = data.push_basic_data(BasicData::True);

        assert_eq!(data.get_basic_data(index1), Some(&BasicData::Unit));
        assert_eq!(data.get_basic_data(index2), Some(&BasicData::True));
    }

    #[test]
    fn get_basic_data_mut() {
        let mut data = BasicGarnishDataUnit::new();
        let index1 = data.push_basic_data(BasicData::Unit);
        let index2 = data.push_basic_data(BasicData::True);

        assert_eq!(data.get_basic_data_mut(index1), Some(&mut BasicData::Unit));
        assert_eq!(data.get_basic_data_mut(index2), Some(&mut BasicData::True));
    }

    #[test]
    fn get_data_ensure_index_ok() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Number(100.into()));
        let result = data.get_data_ensure_index(0);
        assert_eq!(result, Ok(&BasicData::Number(100.into())));
    }

    #[test]
    fn get_data_ensure_index_error() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Number(100.into()));
        let result = data.get_data_ensure_index(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn add_char_list_from_string() {
        let mut data = BasicGarnishDataUnit::new();
        let index = data.add_char_list_from_string("hello").unwrap();

        assert_eq!(index, 0);
        assert_eq!(
            data.data,
            vec![
                BasicData::CharList(5),
                BasicData::Char('h'),
                BasicData::Char('e'),
                BasicData::Char('l'),
                BasicData::Char('l'),
                BasicData::Char('o'),
            ]
        );
    }

    #[test]
    fn add_char_list_from_string_empty() {
        let mut data = BasicGarnishDataUnit::new();
        let index = data.add_char_list_from_string("").unwrap();

        assert_eq!(index, 0);
        assert_eq!(data.data, vec![BasicData::CharList(0)]);
    }

    #[test]
    fn add_char_list_from_string_single_char() {
        let mut data = BasicGarnishDataUnit::new();
        let index = data.add_char_list_from_string("a").unwrap();

        assert_eq!(index, 0);
        assert_eq!(
            data.data,
            vec![
                BasicData::CharList(1),
                BasicData::Char('a'),
            ]
        );
    }

    #[test]
    fn add_byte_list_from_vec() {
        let mut data = BasicGarnishDataUnit::new();
        let index = data.add_byte_list_from_vec(vec![100, 150, 200]).unwrap();

        assert_eq!(index, 0);
        assert_eq!(
            data.data,
            vec![
                BasicData::ByteList(3),
                BasicData::Byte(100),
                BasicData::Byte(150),
                BasicData::Byte(200),
            ]
        );
    }

    #[test]
    fn add_byte_list_from_vec_empty() {
        let mut data = BasicGarnishDataUnit::new();
        let index = data.add_byte_list_from_vec(vec![]).unwrap();

        assert_eq!(index, 0);
        assert_eq!(data.data, vec![BasicData::ByteList(0)]);
    }

    #[test]
    fn add_byte_list_from_vec_single_byte() {
        let mut data = BasicGarnishDataUnit::new();
        let index = data.add_byte_list_from_vec(vec![42]).unwrap();

        assert_eq!(index, 0);
        assert_eq!(
            data.data,
            vec![
                BasicData::ByteList(1),
                BasicData::Byte(42),
            ]
        );
    }

    #[test]
    fn add_byte_list_from_vec_slice() {
        let mut data = BasicGarnishDataUnit::new();
        let bytes: [u8; 4] = [10, 20, 30, 40];
        let index = data.add_byte_list_from_vec(&bytes[..]).unwrap();

        assert_eq!(index, 0);
        assert_eq!(
            data.data,
            vec![
                BasicData::ByteList(4),
                BasicData::Byte(10),
                BasicData::Byte(20),
                BasicData::Byte(30),
                BasicData::Byte(40),
            ]
        );
    }
}
