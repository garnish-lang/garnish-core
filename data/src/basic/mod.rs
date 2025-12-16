mod data;
mod garnish;
mod merge_to_symbol_list;
mod storage;

use std::fmt::Debug;
use std::usize;

pub use data::{BasicData, BasicDataUnitCustom};

use crate::basic::storage::{ReallocationStrategy, StorageBlock, StorageSettings};
use crate::data::SimpleNumber;

use crate::{DataError, error::DataErrorType};

pub type BasicNumber = SimpleNumber;

pub trait BasicDataCustom: Clone + Debug {}

impl BasicDataCustom for () {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicGarnishData<T>
where
    T: BasicDataCustom,
{
    data: Vec<BasicData<T>>,
    instruction_block: StorageBlock,
    jump_table_block: StorageBlock,
    data_block: StorageBlock,
}

pub type BasicGarnishDataUnit = BasicGarnishData<()>;

impl<T> BasicGarnishData<T>
where
    T: BasicDataCustom,
{
    pub fn new() -> Self {
        Self::new_with_settings(StorageSettings::default(), StorageSettings::default(), StorageSettings::default())
    }

    pub fn new_with_settings(instruction_settings: StorageSettings, jump_table_settings: StorageSettings, data_settings: StorageSettings) -> Self {
        let total_size = instruction_settings.initial_size() + jump_table_settings.initial_size() + data_settings.initial_size();
        let data = vec![BasicData::Empty; total_size];
        Self {
            data,
            instruction_block: StorageBlock::new(instruction_settings.initial_size(), instruction_settings.clone()),
            jump_table_block: StorageBlock::new(jump_table_settings.initial_size(), jump_table_settings.clone()),
            data_block: StorageBlock::new(data_settings.initial_size(), data_settings.clone()),
        }
    }

    pub fn push_to_instruction_block(&mut self, data: BasicData<T>) -> usize {
        if self.instruction_block.cursor >= self.instruction_block.size {
            let new_size = match self.instruction_block.settings.reallocation_strategy() {
                ReallocationStrategy::FixedSize(size) => self.instruction_block.size + size,
                ReallocationStrategy::Multiplicative(multiplier) => self.instruction_block.size * multiplier,
            };
            self.reallocate_heap(new_size, self.jump_table_block.size, self.data_block.size);
        }
        let index = self.instruction_block.start + self.instruction_block.cursor;
        self.data[index] = data;
        self.instruction_block.cursor += 1;
        index
    }

    pub fn push_to_jump_table_block(&mut self, data: BasicData<T>) -> usize {
        if self.jump_table_block.cursor >= self.jump_table_block.size {
            let new_size = match self.jump_table_block.settings.reallocation_strategy() {
                ReallocationStrategy::FixedSize(size) => self.jump_table_block.size + size,
                ReallocationStrategy::Multiplicative(multiplier) => self.jump_table_block.size * multiplier,
            };
            self.reallocate_heap(self.instruction_block.size, new_size, self.data_block.size);
        }
        let index = self.jump_table_block.start + self.jump_table_block.cursor;
        self.data[index] = data;
        self.jump_table_block.cursor += 1;
        index
    }

    pub fn push_to_data_block(&mut self, data: BasicData<T>) -> usize {
        if self.data_block.cursor >= self.data_block.size {
            let new_size = match self.data_block.settings.reallocation_strategy() {
                ReallocationStrategy::FixedSize(size) => self.data_block.size + size,
                ReallocationStrategy::Multiplicative(multiplier) => self.data_block.size * multiplier,
            };
            self.reallocate_heap(self.instruction_block.size, self.jump_table_block.size, new_size);
        }
        let index = self.data_block.start + self.data_block.cursor;
        self.data[index] = data;
        self.data_block.cursor += 1;
        index
    }

    pub fn data_size(&self) -> usize {
        self.data_block.cursor
    }

    pub fn instruction_size(&self) -> usize {
        self.instruction_block.cursor
    }

    pub fn jump_table_size(&self) -> usize {
        self.jump_table_block.cursor
    }

    pub fn allocated_data_size(&self) -> usize {
        self.data_block.size
    }

    pub fn allocated_instruction_size(&self) -> usize {
        self.instruction_block.size
    }

    pub fn allocated_jump_table_size(&self) -> usize {
        self.jump_table_block.size
    }

    pub fn get_basic_data(&self, index: usize) -> Option<&BasicData<T>> {
        self.data.get(index)
    }

    pub fn get_basic_data_mut(&mut self, index: usize) -> Option<&mut BasicData<T>> {
        self.data.get_mut(index)
    }

    pub(crate) fn get_from_data_block_ensure_index(&self, index: usize) -> Result<&BasicData<T>, DataError> {
        if index >= self.data_block.cursor {
            return Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(index)));
        }
        let true_index = self.data_block.start + index;
        Ok(&self.data[true_index])
    }

    pub fn add_char_list_from_string(&mut self, s: impl AsRef<str>) -> Result<usize, DataError> {
        let index = self.push_to_data_block(BasicData::CharList(s.as_ref().len()));
        for c in s.as_ref().chars() {
            self.push_to_data_block(BasicData::Char(c));
        }
        Ok(index)
    }

    pub fn add_byte_list_from_vec(&mut self, v: impl AsRef<[u8]>) -> Result<usize, DataError> {
        let index = self.push_to_data_block(BasicData::ByteList(v.as_ref().len()));
        for b in v.as_ref() {
            self.push_to_data_block(BasicData::Byte(*b));
        }
        Ok(index)
    }

    fn reallocate_heap(&mut self, new_instruction_size: usize, new_jump_table_size: usize, new_data_size: usize) {
        let ordered= [
            (&mut self.instruction_block, new_instruction_size),
            (&mut self.jump_table_block, new_jump_table_size),
            (&mut self.data_block, new_data_size),
        ];

        let new_size = new_instruction_size + new_jump_table_size + new_data_size;

        let mut new_heap = vec![BasicData::Empty; new_size];

        let mut current_block_start = 0;
        for (block, new_size) in ordered {
            block.start = current_block_start;
            block.size = new_size;

            for i in 0..block.cursor {
                new_heap[block.start + i] = self.data[block.start + i].clone();
            }
            current_block_start += new_size;
        }

        self.data = new_heap;
    }
}

#[cfg(test)]
mod utilities {
    use crate::{
        BasicGarnishDataUnit,
        basic::storage::{ReallocationStrategy, StorageSettings},
    };

    pub fn test_data() -> BasicGarnishDataUnit {
        BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, 10, ReallocationStrategy::FixedSize(10)),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::basic::{storage::ReallocationStrategy, utilities::test_data};

    use super::*;

    #[test]
    fn test_basic_garnish_data() {
        BasicGarnishData::<()>::new();
    }

    #[test]
    fn created_with_sizes_according_to_settings() {
        let data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(10, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, 10, ReallocationStrategy::FixedSize(10)),
        );

        let expected_data = vec![BasicData::Empty; 30];

        assert_eq!(
            data,
            BasicGarnishDataUnit {
                data: expected_data,
                instruction_block: StorageBlock::new(10, StorageSettings::new(10, 10, ReallocationStrategy::FixedSize(10))),
                jump_table_block: StorageBlock::new(10, StorageSettings::new(10, 10, ReallocationStrategy::FixedSize(10))),
                data_block: StorageBlock::new(10, StorageSettings::new(10, 10, ReallocationStrategy::FixedSize(10))),
            }
        );
    }

    #[test]
    fn push_to_data_block() {
        let mut data = test_data();
        let index = data.push_to_data_block(BasicData::Unit);
        assert_eq!(index, 0);
        assert_eq!(data.data_block.cursor, 1);
        assert_eq!(data.data_block.size, 10);
        assert_eq!(
            data.data,
            vec![
                BasicData::Unit,
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
    fn push_to_instruction_block() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(10, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
        );
        let index = data.push_to_instruction_block(BasicData::Unit);
        assert_eq!(index, 0);
        assert_eq!(data.instruction_block.cursor, 1);
        assert_eq!(data.instruction_block.size, 10);
        assert_eq!(
            data.data,
            vec![
                BasicData::Unit,
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
    fn push_to_jump_table_block() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
        );
        let index = data.push_to_jump_table_block(BasicData::Unit);
        assert_eq!(index, 0);
        assert_eq!(data.jump_table_block.cursor, 1);
        assert_eq!(data.jump_table_block.size, 10);
        assert_eq!(
            data.data,
            vec![
                BasicData::Unit,
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
    fn data_block_resizes_when_pushed_past_max_size() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, 10, ReallocationStrategy::FixedSize(10)),
        );
        for _ in 0..15 {
            data.push_to_data_block(BasicData::Unit);
        }

        let mut expected_data = vec![BasicData::Unit; 15];
        expected_data.resize(20, BasicData::Empty);

        assert_eq!(data.data_size(), 15);
        assert_eq!(data.allocated_data_size(), 20);
        assert_eq!(data.data, expected_data);
    }

    #[test]
    fn instruction_block_resizes_when_pushed_past_max_size() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(10, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
        );
        for _ in 0..15 {
            data.push_to_instruction_block(BasicData::Unit);
        }

        let mut expected_data = vec![BasicData::Unit; 15];
        expected_data.resize(20, BasicData::Empty);

        assert_eq!(data.instruction_size(), 15);
        assert_eq!(data.allocated_instruction_size(), 20);
        assert_eq!(data.data, expected_data);
    }

    #[test]
    fn jump_table_block_resizes_when_pushed_past_max_size() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
        );
        for _ in 0..15 {
            data.push_to_jump_table_block(BasicData::Unit);
        }

        let mut expected_data = vec![BasicData::Unit; 15];
        expected_data.resize(20, BasicData::Empty);

        assert_eq!(data.jump_table_size(), 15);
        assert_eq!(data.allocated_jump_table_size(), 20);
        assert_eq!(data.data, expected_data);
    }

    #[test]
    fn reallocations_happen_correctly_pushing_top_to_bottom() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
        );

        data.push_to_instruction_block(BasicData::Unit);
        data.push_to_jump_table_block(BasicData::True);
        data.push_to_data_block(BasicData::False);

        let mut expected_data = vec![BasicData::Empty; 30];
        expected_data[0] = BasicData::Unit;
        expected_data[10] = BasicData::True;
        expected_data[20] = BasicData::False;

        assert_eq!(data.instruction_size(), 1);
        assert_eq!(data.jump_table_size(), 1);
        assert_eq!(data.data_size(), 1);
        assert_eq!(data.allocated_instruction_size(), 10);
        assert_eq!(data.allocated_jump_table_size(), 10);
        assert_eq!(data.allocated_data_size(), 10);
        assert_eq!(data.data, expected_data);
    }

    #[test]
    fn get_from_data_block_ensure_index() {
        let mut data = test_data();

        let index = data.push_to_data_block(BasicData::True);

        let result = data.get_from_data_block_ensure_index(index).unwrap();

        assert_eq!(result, &BasicData::<()>::True);
    }

    #[test]
    fn get_from_data_block_ensure_index_index_out_of_bounds() {
        let mut data = test_data();

        data.push_to_data_block(BasicData::True);

        let result = data.get_from_data_block_ensure_index(10);

        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(10))));
    }
}
