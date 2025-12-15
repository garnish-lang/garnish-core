mod data;
mod garnish;
mod merge_to_symbol_list;
mod storage;

use std::usize;

pub use data::{BasicData, BasicDataUnitCustom};

use crate::basic::storage::StorageSettings;
use crate::data::SimpleNumber;

pub type BasicNumber = SimpleNumber;

use crate::{DataError, error::DataErrorType};

pub trait BasicDataCustom: Clone {}

impl BasicDataCustom for () {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicGarnishData<T>
where
    T: BasicDataCustom,
{
    data_cursor: usize,
    data: Vec<BasicData<T>>,
    storage_settings: StorageSettings,
}

pub type BasicGarnishDataUnit = BasicGarnishData<()>;

impl<T> BasicGarnishData<T>
where
    T: BasicDataCustom,
{
    pub fn new() -> Self {
        Self::new_full(vec![], StorageSettings::default())
    }

    pub fn new_full(mut data: Vec<BasicData<T>>, storage_settings: StorageSettings) -> Self {
        let data_cursor = data.len();
        Self::fill_data(&mut data, &storage_settings);
        Self {
            data_cursor,
            data,
            storage_settings,
        }
    }

    fn fill_data(data: &mut Vec<BasicData<T>>, settings: &StorageSettings) {
        data.resize(settings.initial_size(), BasicData::Empty);
    }

    pub fn push_basic_data(&mut self, data: BasicData<T>) -> usize {
        if self.data_cursor >= self.data.len() {
            todo!()
        }
        let index = self.data_cursor;
        self.data[self.data_cursor] = data;
        self.data_cursor += 1;
        index
    }

    pub fn get_basic_data(&self, index: usize) -> Option<&BasicData<T>> {
        self.data.get(index)
    }

    pub fn get_basic_data_mut(&mut self, index: usize) -> Option<&mut BasicData<T>> {
        self.data.get_mut(index)
    }

    pub(crate) fn get_data_ensure_index(&self, index: usize) -> Result<&BasicData<T>, DataError> {
        if index >= self.data_cursor {
            return Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(index)));
        }
        Ok(&self.data[index])
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
mod utilities {
    use crate::{
        BasicData, BasicGarnishDataUnit, basic::storage::{ReallocationStrategy, StorageSettings}
    };

    pub fn test_basic_data() -> BasicGarnishDataUnit {
        BasicGarnishDataUnit::new_full(vec![], StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)))
    }

    pub fn test_basic_data_with_data(data: Vec<BasicData<()>>) -> BasicGarnishDataUnit {
        BasicGarnishDataUnit::new_full(data, StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)))
    }
}

#[cfg(test)]
mod tests {
    use crate::basic::utilities::test_basic_data;

    use super::*;

    #[test]
    fn test_basic_garnish_data() {
        BasicGarnishData::<()>::new();
    }

    #[test]
    fn add_basic_data() {
        let mut data = test_basic_data();

        let index = data.push_basic_data(BasicData::Unit);
        assert_eq!(index, 0);

        let index = data.push_basic_data(BasicData::True);
        assert_eq!(index, 1);

        assert_eq!(
            data.data,
            vec![
                BasicData::Unit,
                BasicData::True,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty
            ]
        );
    }

    #[test]
    fn get_basic_data() {
        let mut data = test_basic_data();
        let index1 = data.push_basic_data(BasicData::Unit);
        let index2 = data.push_basic_data(BasicData::True);

        assert_eq!(data.get_basic_data(index1), Some(&BasicData::Unit));
        assert_eq!(data.get_basic_data(index2), Some(&BasicData::True));
    }

    #[test]
    fn get_basic_data_mut() {
        let mut data = test_basic_data();
        let index1 = data.push_basic_data(BasicData::Unit);
        let index2 = data.push_basic_data(BasicData::True);

        assert_eq!(data.get_basic_data_mut(index1), Some(&mut BasicData::Unit));
        assert_eq!(data.get_basic_data_mut(index2), Some(&mut BasicData::True));
    }

    #[test]
    fn get_data_ensure_index_ok() {
        let mut data = test_basic_data();
        data.push_basic_data(BasicData::Number(100.into()));
        let result = data.get_data_ensure_index(0);
        assert_eq!(result, Ok(&BasicData::Number(100.into())));
    }

    #[test]
    fn get_data_ensure_index_error() {
        let mut data = test_basic_data();
        data.push_basic_data(BasicData::Number(100.into()));
        let result = data.get_data_ensure_index(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn add_char_list_from_string() {
        let mut data = test_basic_data();
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
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
            ]
        );
    }

    #[test]
    fn add_char_list_from_string_empty() {
        let mut data = test_basic_data();
        let index = data.add_char_list_from_string("").unwrap();

        assert_eq!(index, 0);
        assert_eq!(
            data.data,
            vec![
                BasicData::CharList(0),
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
            ]
        );
    }

    #[test]
    fn add_char_list_from_string_single_char() {
        let mut data = test_basic_data();
        let index = data.add_char_list_from_string("a").unwrap();

        assert_eq!(index, 0);
        assert_eq!(
            data.data,
            vec![
                BasicData::CharList(1),
                BasicData::Char('a'),
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
            ]
        );
    }

    #[test]
    fn add_byte_list_from_vec() {
        let mut data = test_basic_data();
        let index = data.add_byte_list_from_vec(vec![100, 150, 200]).unwrap();

        assert_eq!(index, 0);
        assert_eq!(
            data.data,
            vec![
                BasicData::ByteList(3),
                BasicData::Byte(100),
                BasicData::Byte(150),
                BasicData::Byte(200),
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
            ]
        );
    }

    #[test]
    fn add_byte_list_from_vec_empty() {
        let mut data = test_basic_data();
        let index = data.add_byte_list_from_vec(vec![]).unwrap();

        assert_eq!(index, 0);
        assert_eq!(
            data.data,
            vec![
                BasicData::ByteList(0),
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
            ]
        );
    }

    #[test]
    fn add_byte_list_from_vec_single_byte() {
        let mut data = test_basic_data();
        let index = data.add_byte_list_from_vec(vec![42]).unwrap();

        assert_eq!(index, 0);
        assert_eq!(
            data.data,
            vec![
                BasicData::ByteList(1),
                BasicData::Byte(42),
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
            ]
        );
    }

    #[test]
    fn add_byte_list_from_vec_slice() {
        let mut data = test_basic_data();
        let bytes = [10, 20, 30, 40];
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
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
            ]
        );
    }

    #[test]
    fn created_with_initial_size() {
        let data = test_basic_data();

        assert_eq!(
            data.data,
            vec![
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
            ]
        );
    }
}
