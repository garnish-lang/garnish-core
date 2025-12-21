mod data;
mod dump;
mod garnish;
mod internal;
mod merge_to_symbol_list;
mod object;
mod search;
mod storage;

use std::cmp::Ordering;
use std::fmt::Debug;
use std::usize;

pub use data::{BasicData, BasicDataUnitCustom};
pub use garnish::BasicDataFactory;

use crate::basic::search::search_for_associative_item;
use crate::basic::storage::{StorageBlock, StorageSettings};
use crate::data::SimpleNumber;

use crate::DataError;

pub type BasicNumber = SimpleNumber;

pub trait BasicDataCustom: Clone + Debug {}

impl BasicDataCustom for () {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicGarnishData<T = ()>
where
    T: BasicDataCustom,
{
    current_value: Option<usize>,
    current_register: Option<usize>,
    instruction_pointer: usize,
    current_jump_path: Option<usize>,
    data: Vec<BasicData<T>>,
    instruction_block: StorageBlock,
    jump_table_block: StorageBlock,
    symbol_table_block: StorageBlock,
    data_block: StorageBlock,
}

pub type BasicGarnishDataUnit = BasicGarnishData<()>;

impl<T> BasicGarnishData<T>
where
    T: BasicDataCustom,
{
    pub fn new() -> Result<Self, DataError> {
        Self::new_with_settings(
            StorageSettings::default(),
            StorageSettings::default(),
            StorageSettings::default(),
            StorageSettings::default(),
        )
    }

    pub fn new_with_settings(
        instruction_settings: StorageSettings,
        jump_table_settings: StorageSettings,
        symbol_table_settings: StorageSettings,
        data_settings: StorageSettings,
    ) -> Result<Self, DataError> {
        let mut this = Self {
            current_value: None,
            current_register: None,
            instruction_pointer: 0,
            current_jump_path: None,
            data: Vec::new(),
            instruction_block: StorageBlock::new(instruction_settings.initial_size(), instruction_settings.clone()),
            jump_table_block: StorageBlock::new(jump_table_settings.initial_size(), jump_table_settings.clone()),
            symbol_table_block: StorageBlock::new(symbol_table_settings.initial_size(), symbol_table_settings.clone()),
            data_block: StorageBlock::new(data_settings.initial_size(), data_settings.clone()),
        };

        this.reallocate_heap(
            instruction_settings.initial_size,
            jump_table_settings.initial_size,
            symbol_table_settings.initial_size,
            data_settings.initial_size,
        )?; // temp unwrap

        Ok(this)
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

    pub fn symbol_table_size(&self) -> usize {
        self.symbol_table_block.cursor
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

    pub fn allocated_symbol_table_size(&self) -> usize {
        self.symbol_table_block.size
    }

    pub fn get_basic_data(&self, index: usize) -> Option<&BasicData<T>> {
        self.data.get(index)
    }

    pub fn get_basic_data_mut(&mut self, index: usize) -> Option<&mut BasicData<T>> {
        self.data.get_mut(index)
    }

    pub fn push_to_instruction_block(&mut self, data: BasicData<T>) -> Result<usize, DataError> {
        if self.instruction_block.cursor >= self.instruction_block.size {
            self.reallocate_heap(
                self.instruction_block.next_size(),
                self.jump_table_block.size,
                self.symbol_table_block.size,
                self.data_block.size,
            )?;
        }
        Ok(Self::push_to_block(&mut self.data, &mut self.instruction_block, data))
    }

    pub fn push_to_jump_table_block(&mut self, data: BasicData<T>) -> Result<usize, DataError> {
        if self.jump_table_block.cursor >= self.jump_table_block.size {
            self.reallocate_heap(
                self.instruction_block.size,
                self.jump_table_block.next_size(),
                self.symbol_table_block.size,
                self.data_block.size,
            )?;
        }
        Ok(Self::push_to_block(&mut self.data, &mut self.jump_table_block, data))
    }

    pub fn push_to_symbol_table_block(&mut self, symbol: u64, value: usize) -> Result<(), DataError> {
        if self.symbol_table_block.cursor >= self.symbol_table_block.size {
            self.reallocate_heap(
                self.instruction_block.size,
                self.jump_table_block.size,
                self.symbol_table_block.next_size(),
                self.data_block.size,
            )?;
        }
        Self::push_to_block(&mut self.data, &mut self.symbol_table_block, BasicData::AssociativeItem(symbol, value));
        let sort_range = &mut self.data[self.symbol_table_block.start..self.symbol_table_block.start + self.symbol_table_block.cursor];
        sort_range.sort_by(|a, b| match (a, b) {
            (BasicData::AssociativeItem(sym1, _), BasicData::AssociativeItem(sym2, _)) => sym1.cmp(sym2),
            (BasicData::AssociativeItem(_, _), _) => Ordering::Less,
            (_, BasicData::AssociativeItem(_, _)) => Ordering::Greater,
            _ => Ordering::Equal,
        });

        Ok(())
    }

    pub fn push_to_data_block(&mut self, data: BasicData<T>) -> Result<usize, DataError> {
        if self.data_block.cursor >= self.data_block.size {
            self.reallocate_heap(
                self.instruction_block.size,
                self.jump_table_block.size,
                self.symbol_table_block.size,
                self.data_block.next_size(),
            )?;
        }
        Ok(Self::push_to_block(&mut self.data, &mut self.data_block, data))
    }

    pub fn get_string_for_data_at(&self, index: usize) -> Result<String, DataError> {
        self.string_from_basic_data_at(index)
    }

    pub fn get_symbol_string(&self, symbol: u64) -> Result<Option<String>, DataError> {
        let search_slice = &self.data[self.symbol_table_block.start..self.symbol_table_block.start + self.symbol_table_block.cursor];
        match search_for_associative_item(search_slice, symbol)? {
            Some(index) => match self.get_from_data_block_ensure_index(index)?.as_char_list()? {
                _ => Ok(Some(self.string_from_basic_data_at(index)?)),
            },
            None => Ok(None),
        }
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
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
        ).unwrap()
    }

    pub fn instruction_test_data() -> BasicGarnishDataUnit {
        BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
        ).unwrap()
    }

    pub fn jump_table_test_data() -> BasicGarnishDataUnit {
        BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
        ).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use garnish_lang_traits::GarnishDataType;

    use crate::{
        BasicData, BasicGarnishData, BasicGarnishDataUnit, DataError,
        basic::{
            object::BasicObject,
            storage::{ReallocationStrategy, StorageBlock, StorageSettings},
            utilities::test_data,
        },
        error::DataErrorType,
    };

    #[test]
    fn test_basic_garnish_data() {
        BasicGarnishData::<()>::new();
    }

    #[test]
    fn created_with_sizes_according_to_settings() {
        let data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
        );

        let expected_data = vec![BasicData::Empty; 40];

        let mut expected_instruction_block = StorageBlock::new(10, StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)));
        expected_instruction_block.start = 0;

        let mut expected_jump_table_block = StorageBlock::new(10, StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)));
        expected_jump_table_block.start = 10;

        let mut expected_symbol_table_block = StorageBlock::new(10, StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)));
        expected_symbol_table_block.start = 20;

        let mut expected_data_block = StorageBlock::new(10, StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)));
        expected_data_block.start = 30;

        assert_eq!(
            data,
            Ok(BasicGarnishDataUnit {
                instruction_pointer: 0,
                current_value: None,
                current_register: None,
                current_jump_path: None,
                data: expected_data,
                instruction_block: expected_instruction_block,
                jump_table_block: expected_jump_table_block,
                symbol_table_block: expected_symbol_table_block,
                data_block: expected_data_block,
            })
        );
    }

    #[test]
    fn push_to_data_block() {
        let mut data = test_data();
        let index = data.push_to_data_block(BasicData::Unit).unwrap();
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
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
        ).unwrap();
        let index = data.push_to_instruction_block(BasicData::Unit).unwrap();
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
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
        ).unwrap();
        let index = data.push_to_jump_table_block(BasicData::Unit).unwrap();
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
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, 20, ReallocationStrategy::FixedSize(10)),
        ).unwrap();
        for _ in 0..15 {
            data.push_to_data_block(BasicData::Unit).unwrap();
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
            StorageSettings::new(10, 20, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
        ).unwrap();
        for _ in 0..15 {
            data.push_to_instruction_block(BasicData::Unit).unwrap();
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
            StorageSettings::new(10, 20, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
        ).unwrap();
        for _ in 0..15 {
            data.push_to_jump_table_block(BasicData::Unit).unwrap();
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
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
        ).unwrap();

        data.push_to_instruction_block(BasicData::Unit).unwrap();
        data.push_to_jump_table_block(BasicData::True).unwrap();
        data.push_to_symbol_table_block(100, 123).unwrap();
        data.push_to_data_block(BasicData::Number(100.into())).unwrap();

        let mut expected_data = vec![BasicData::Empty; 40];
        expected_data[0] = BasicData::Unit;
        expected_data[10] = BasicData::True;
        expected_data[20] = BasicData::AssociativeItem(100, 123);
        expected_data[30] = BasicData::Number(100.into());

        assert_eq!(data.instruction_size(), 1);
        assert_eq!(data.jump_table_size(), 1);
        assert_eq!(data.symbol_table_size(), 1);
        assert_eq!(data.data_size(), 1);
        assert_eq!(data.allocated_instruction_size(), 10);
        assert_eq!(data.allocated_jump_table_size(), 10);
        assert_eq!(data.allocated_symbol_table_size(), 10);
        assert_eq!(data.allocated_data_size(), 10);
        assert_eq!(data.data, expected_data);
    }

    #[test]
    fn reallocations_happen_correctly_pushing_bottom_to_top() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
        ).unwrap();

        data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        data.push_to_symbol_table_block(100, 0).unwrap();
        data.push_to_jump_table_block(BasicData::True).unwrap();
        data.push_to_instruction_block(BasicData::Unit).unwrap();

        let mut expected_data = vec![BasicData::Empty; 40];
        expected_data[0] = BasicData::Unit;
        expected_data[10] = BasicData::True;
        expected_data[20] = BasicData::AssociativeItem(100, 0);
        expected_data[30] = BasicData::Number(100.into());

        assert_eq!(data.instruction_size(), 1);
        assert_eq!(data.jump_table_size(), 1);
        assert_eq!(data.symbol_table_size(), 1);
        assert_eq!(data.data_size(), 1);
        assert_eq!(data.allocated_instruction_size(), 10);
        assert_eq!(data.allocated_jump_table_size(), 10);
        assert_eq!(data.allocated_symbol_table_size(), 10);
        assert_eq!(data.allocated_data_size(), 10);
        assert_eq!(data.data, expected_data);
    }

    #[test]
    fn push_to_data_block_exceeds_max_items() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, 10, ReallocationStrategy::FixedSize(10)),
        ).unwrap();

        for _ in 0..10 {
            data.push_to_data_block(BasicData::Unit).unwrap();
        }

        let result = data.push_to_data_block(BasicData::Unit);

        assert_eq!(
            result,
            Err(DataError::new(
                "Data block size exceeds max items",
                DataErrorType::DataBlockExceededMaxItems(20, 10)
            ))
        );
    }

    #[test]
    fn push_to_instruction_block_exceeds_max_items() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(10, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
        ).unwrap();

        for _ in 0..10 {
            data.push_to_instruction_block(BasicData::Unit).unwrap();
        }

        let result = data.push_to_instruction_block(BasicData::Unit);

        assert_eq!(
            result,
            Err(DataError::new(
                "Instruction block size exceeds max items",
                DataErrorType::InstructionBlockExceededMaxItems(20, 10)
            ))
        );
    }

    #[test]
    fn push_to_jump_table_block_exceeds_max_items() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
        ).unwrap();

        for _ in 0..10 {
            data.push_to_jump_table_block(BasicData::Unit).unwrap();
        }

        let result = data.push_to_jump_table_block(BasicData::Unit);

        assert_eq!(
            result,
            Err(DataError::new(
                "Jump table block size exceeds max items",
                DataErrorType::JumpTableBlockExceededMaxItems(20, 10)
            ))
        );
    }

    #[test]
    fn push_to_symbol_table_block() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
        ).unwrap();
        data.push_to_symbol_table_block(100, 0).unwrap();
        assert_eq!(data.symbol_table_block.cursor, 1);
        assert_eq!(data.symbol_table_block.size, 10);
    }

    #[test]
    fn symbol_table_block_resizes_when_pushed_past_max_size() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, 20, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
        ).unwrap();
        for _ in 0..15 {
            data.push_to_symbol_table_block(100, 0).unwrap();
        }

        let mut expected_data = vec![BasicData::<()>::Empty; 40];
        for i in 20..35 {
            expected_data[i] = BasicData::AssociativeItem(100, 0);
        }

        assert_eq!(data.symbol_table_size(), 15);
        assert_eq!(data.allocated_symbol_table_size(), 20);
    }

    #[test]
    fn push_to_symbol_table_block_exceeds_max_items() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
        ).unwrap();

        for _ in 0..10 {
            data.push_to_symbol_table_block(100, 0).unwrap();
        }

        let result = data.push_to_symbol_table_block(100, 0);

        assert_eq!(
            result,
            Err(DataError::new(
                "Symbol table block size exceeds max items",
                DataErrorType::SymbolTableBlockExceededMaxItems(20, 10)
            ))
        );
    }

    #[test]
    fn push_to_symbol_table_block_is_sorted() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
        ).unwrap();
        data.push_to_symbol_table_block(300, 0).unwrap();
        data.push_to_symbol_table_block(600, 0).unwrap();
        data.push_to_symbol_table_block(100, 0).unwrap();
        data.push_to_symbol_table_block(500, 0).unwrap();
        data.push_to_symbol_table_block(200, 0).unwrap();
        data.push_to_symbol_table_block(400, 0).unwrap();

        let mut expected_data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
        ).unwrap();

        expected_data.data[0] = BasicData::AssociativeItem(100, 0);
        expected_data.data[1] = BasicData::AssociativeItem(200, 0);
        expected_data.data[2] = BasicData::AssociativeItem(300, 0);
        expected_data.data[3] = BasicData::AssociativeItem(400, 0);
        expected_data.data[4] = BasicData::AssociativeItem(500, 0);
        expected_data.data[5] = BasicData::AssociativeItem(600, 0);
        expected_data.symbol_table_block.cursor = 6;
        assert_eq!(data, expected_data);
    }

    #[test]
    fn get_symbol_string() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
        ).unwrap();
        let index = data.push_object_to_data_block(BasicObject::CharList("first symbol".to_string())).unwrap();
        data.push_to_symbol_table_block(100, index).unwrap();

        let index = data
            .push_object_to_data_block(BasicObject::CharList("second symbol".to_string()))
            .unwrap();
        data.push_to_symbol_table_block(50, index).unwrap();

        let index = data.push_object_to_data_block(BasicObject::CharList("third symbol".to_string())).unwrap();
        data.push_to_symbol_table_block(200, index).unwrap();

        let result = data.get_symbol_string(50).unwrap();

        assert_eq!(result, Some("second symbol".to_string()));
    }

    #[test]
    fn get_symbol_string_not_found() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
        ).unwrap();
        let index = data.push_object_to_data_block(BasicObject::CharList("first symbol".to_string())).unwrap();
        data.push_to_symbol_table_block(100, index).unwrap();

        let index = data
            .push_object_to_data_block(BasicObject::CharList("second symbol".to_string()))
            .unwrap();
        data.push_to_symbol_table_block(50, index).unwrap();

        let index = data.push_object_to_data_block(BasicObject::CharList("third symbol".to_string())).unwrap();
        data.push_to_symbol_table_block(200, index).unwrap();

        let result = data.get_symbol_string(150).unwrap();

        assert_eq!(result, None);
    }

    #[test]
    fn get_symbol_string_data_not_char_list() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
        ).unwrap();
        let index = data.push_object_to_data_block(BasicObject::Number(100.into())).unwrap();
        data.push_to_symbol_table_block(100, index).unwrap();

        let index = data
            .push_object_to_data_block(BasicObject::CharList("second symbol".to_string()))
            .unwrap();
        data.push_to_symbol_table_block(50, index).unwrap();

        let index = data.push_object_to_data_block(BasicObject::CharList("third symbol".to_string())).unwrap();
        data.push_to_symbol_table_block(200, index).unwrap();

        let result = data.get_symbol_string(100);

        assert_eq!(
            result,
            Err(DataError::new(
                "Not of type",
                DataErrorType::NotType(GarnishDataType::CharList, GarnishDataType::Number)
            ))
        );
    }
}
