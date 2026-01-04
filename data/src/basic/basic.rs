use std::cmp::Ordering;
use std::fmt::Debug;
use std::usize;

use garnish_lang_traits::GarnishDataType;
use garnish_lang_traits::Instruction;

use crate::ConversionDelegate;
use crate::basic::clone::CloneDelegate;
use crate::basic::companion::BasicDataCompanion;
use crate::basic::ordering::OrderingDelegate;
use crate::basic::search::{search_for_associative_item, search_for_associative_item_index};
use crate::basic::storage::{StorageBlock, StorageSettings};
use crate::{BasicData, DataError, SimpleNumber};

pub type BasicNumber = SimpleNumber;

pub trait BasicDataCustom: Clone + Debug + PartialEq + Eq + PartialOrd {
    fn convert_custom_data_with_delegate<Companion>(_delegate: &mut impl ConversionDelegate<Self, char, Companion>, _value: Self) -> Result<(), DataError> 
    where
        Companion: BasicDataCompanion<Self>,
    {
        Ok(())
    }

    fn push_clone_items_for_custom_data<Companion>(_delegate: &mut impl OrderingDelegate<Companion>, _value: Self) -> Result<(), DataError> 
    where
        Companion: BasicDataCompanion<Self>,
    {
        Ok(())
    }

    fn create_cloned_custom_data<Companion>(_delegate: &mut impl CloneDelegate<Companion>, value: Self) -> Result<Self, DataError> 
    where
        Companion: BasicDataCompanion<Self>,
    {
        Ok(value.clone())
    }
}

impl BasicDataCustom for () {
    fn convert_custom_data_with_delegate<Companion>(delegate: &mut impl ConversionDelegate<Self, char, Companion>, _value: Self) -> Result<(), DataError> 
    where
        Companion: BasicDataCompanion<Self>,
    {
        delegate.push_char('(')?;
        delegate.push_char(')')?;
        Ok(())
    }
} 

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct NoOpCompanion {

}

impl NoOpCompanion {
    pub fn new() -> Self {
        Self {}
    }
}

impl BasicDataCompanion<()> for NoOpCompanion {
    fn resolve(_data: &mut BasicGarnishData<()>, _symbol: u64) -> Result<bool, DataError> {
        Ok(false)
    }

    fn apply(_data: &mut BasicGarnishData<()>, _external_value: usize, _input_addr: usize) -> Result<bool, DataError> {
        Ok(false)
    }

    fn defer_op(_data: &mut BasicGarnishData<()>, _operation: Instruction, _left: (GarnishDataType, usize), _right: (GarnishDataType, usize)) -> Result<bool, DataError> {
        Ok(false)
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicGarnishData<T = (), Companion = NoOpCompanion>
where 
    T: BasicDataCustom,
    Companion: BasicDataCompanion<T>
{
    current_value: Option<usize>,
    current_register: Option<usize>,
    instruction_pointer: usize,
    current_frame: Option<usize>,
    data_retention_count: usize,
    data: Vec<BasicData<T>>,
    instruction_block: StorageBlock,
    jump_table_block: StorageBlock,
    symbol_table_block: StorageBlock,
    expression_symbol_block: StorageBlock,
    data_block: StorageBlock,
    custom_data_block: StorageBlock,
    pub(crate) companion: Companion, 
}

pub type BasicGarnishDataUnit = BasicGarnishData<()>;

impl<T, Companion> BasicGarnishData<T, Companion>
where
    T: BasicDataCustom,
    Companion: BasicDataCompanion<T>,
{
    pub fn new(companion: Companion) -> Result<Self, DataError> {
        Self::new_with_settings(
            StorageSettings::default(),
            StorageSettings::default(),
            StorageSettings::default(),
            StorageSettings::default(),
            StorageSettings::default(),
            StorageSettings::default(),
            companion,
        )
    }

    pub fn new_with_settings(
        instruction_settings: StorageSettings,
        jump_table_settings: StorageSettings,
        symbol_table_settings: StorageSettings,
        expression_symbol_settings: StorageSettings,
        data_settings: StorageSettings,
        custom_data_settings: StorageSettings,
        companion: Companion,
    ) -> Result<Self, DataError> {
        let mut this = Self {
            current_value: None,
            current_register: None,
            instruction_pointer: 0,
            current_frame: None,
            data_retention_count: 0,
            data: Vec::new(),
            instruction_block: StorageBlock::new(instruction_settings.initial_size(), instruction_settings.clone()),
            jump_table_block: StorageBlock::new(jump_table_settings.initial_size(), jump_table_settings.clone()),
            symbol_table_block: StorageBlock::new(symbol_table_settings.initial_size(), symbol_table_settings.clone()),
            expression_symbol_block: StorageBlock::new(expression_symbol_settings.initial_size(), expression_symbol_settings.clone()),
            data_block: StorageBlock::new(data_settings.initial_size(), data_settings.clone()),
            custom_data_block: StorageBlock::new(custom_data_settings.initial_size(), custom_data_settings.clone()),
            companion,
        };

        this.reallocate_heap(
            instruction_settings.initial_size,
            jump_table_settings.initial_size,
            symbol_table_settings.initial_size,
            expression_symbol_settings.initial_size,
            data_settings.initial_size,
            custom_data_settings.initial_size,
        )?;

        Ok(this)
    }

    pub fn companion(&self) -> &Companion {
        &self.companion
    }

    pub fn companion_mut(&mut self) -> &mut Companion {
        &mut self.companion
    }

    pub fn set_companion(&mut self, companion: Companion) {
        self.companion = companion;
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

    pub fn expression_symbol_block_size(&self) -> usize {
        self.expression_symbol_block.cursor
    }

    pub fn custom_data_size(&self) -> usize {
        self.custom_data_block.cursor
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

    pub fn allocated_expression_symbol_block_size(&self) -> usize {
        self.expression_symbol_block.size
    }

    pub fn allocated_custom_data_size(&self) -> usize {
        self.custom_data_block.size
    }

    pub fn total_allocated_size(&self) -> usize {
        self.instruction_block.size + self.jump_table_block.size + self.symbol_table_block.size + self.expression_symbol_block.size + self.data_block.size + self.custom_data_block.size
    }

    pub(crate) fn get_basic_data(&self, index: usize) -> Option<&BasicData<T>> {
        self.data.get(index)
    }

    pub fn data_retention_count(&self) -> usize {
        self.data_retention_count
    }

    pub fn retain_all_current_data(&mut self) {
        self.data_retention_count = self.data_block().cursor;
    }

    pub fn set_data_retention_count(&mut self, count: usize) {
        self.data_retention_count = count;
    }

    pub fn optimize(&mut self, additional_data_retentions: &[usize]) -> Result<Vec<usize>, DataError> {
        self.optimize_data_block_and_retain(additional_data_retentions)
    }

    pub fn clone_data(&mut self, index: usize) -> Result<usize, DataError> {
        let index_stack_start = self.create_index_stack(index)?;
        self.clone_index_stack(index_stack_start, 0)
    }

    pub fn push_to_instruction_block(&mut self, instruction: Instruction, data: Option<usize>) -> Result<usize, DataError> {
        if self.instruction_block.cursor >= self.instruction_block.size {
            self.reallocate_heap(
                self.instruction_block.next_size(),
                self.jump_table_block.size,
                self.symbol_table_block.size,
                self.expression_symbol_block.size,
                self.data_block.size,
                self.custom_data_block.size,
            )?;
        }
        let instruction_data = match data {
            Some(data_index) => BasicData::InstructionWithData(instruction, data_index),
            None => BasicData::Instruction(instruction),
        };
        Ok(Self::push_to_block(&mut self.data, &mut self.instruction_block, instruction_data))
    }

    pub fn push_to_jump_table_block(&mut self, index: usize) -> Result<usize, DataError> {
        if self.jump_table_block.cursor >= self.jump_table_block.size {
            self.reallocate_heap(
                self.instruction_block.size,
                self.jump_table_block.next_size(),
                self.symbol_table_block.size,
                self.expression_symbol_block.size,
                self.data_block.size,
                self.custom_data_block.size,
            )?;
        }
        Ok(Self::push_to_block(&mut self.data, &mut self.jump_table_block, BasicData::JumpPoint(index)))
    }

    pub fn push_to_symbol_table_block(&mut self, symbol: u64, value: usize) -> Result<(), DataError> {
        if self.symbol_table_block.cursor >= self.symbol_table_block.size {
            self.reallocate_heap(
                self.instruction_block.size,
                self.jump_table_block.size,
                self.symbol_table_block.next_size(),
                self.expression_symbol_block.size,
                self.data_block.size,
                self.custom_data_block.size,
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

    pub fn push_to_expression_symbol_block(&mut self, symbol: u64, value: usize) -> Result<(), DataError> {
        if self.expression_symbol_block.cursor >= self.expression_symbol_block.size {
            self.reallocate_heap(
                self.instruction_block.size,
                self.jump_table_block.size,
                self.symbol_table_block.size,
                self.expression_symbol_block.next_size(),
                self.data_block.size,
                self.custom_data_block.size,
            )?;
        }
        Self::push_to_block(&mut self.data, &mut self.expression_symbol_block, BasicData::AssociativeItem(symbol, value));
        let sort_range = &mut self.data[self.expression_symbol_block.start..self.expression_symbol_block.start + self.expression_symbol_block.cursor];
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
                self.expression_symbol_block.size,
                self.data_block.next_size(),
                self.custom_data_block.size,
            )?;
        }
        Ok(Self::push_to_block(&mut self.data, &mut self.data_block, data))
    }

    pub fn push_to_custom_data_block(&mut self, value: T) -> Result<usize, DataError> {
        if self.custom_data_block.cursor >= self.custom_data_block.size {
            self.reallocate_heap(
                self.instruction_block.size,
                self.jump_table_block.size,
                self.symbol_table_block.size,
                self.expression_symbol_block.size,
                self.data_block.size,
                self.custom_data_block.next_size(),
            )?;
        }
        Ok(Self::push_to_block(&mut self.data, &mut self.custom_data_block, BasicData::Custom(value)))
    }

    pub fn get_from_custom_data_block(&self, index: usize) -> Option<T> {
        self.get_from_custom_data_block_ensure_index(index).ok()
    }

    pub fn get_string_for_data_at(&self, index: usize) -> Result<String, DataError> {
        self.string_from_basic_data_at(index)
    }

    pub fn get_symbol_string(&self, symbol: u64) -> Result<Option<String>, DataError> {
        let search_slice = &self.data()[self.symbol_table_block().start..self.symbol_table_block().start + self.symbol_table_block().cursor];
        match search_for_associative_item(search_slice, symbol)? {
            Some(item) => {
                let (_, index) = item.as_associative_item()?;
                let list_length = self.get_from_data_block_ensure_index(index)?.as_char_list()?;
                let start = self.data_block().start + index + 1;
                let slice = &self.data()[start..start + list_length];
                let result = slice.iter().map(|data| data.as_char().unwrap()).collect::<String>();
                Ok(Some(result))
            }
            None => Ok(None),
        }
    }

    pub fn get_symbol_expression(&self, symbol: u64) -> Result<Option<usize>, DataError> {
        let search_slice = &self.data()[self.expression_symbol_block().start..self.expression_symbol_block().start + self.expression_symbol_block().cursor];
        match search_for_associative_item(search_slice, symbol)? {
            Some(item) => Ok(Some(item.as_associative_item()?.1)),
            None => Ok(None),
        }
    }

    pub fn get_expression_string(&self, expression_index: usize) -> Result<Option<String>, DataError> {
        let expression_slice = &self.data()[self.expression_symbol_block().start..self.expression_symbol_block().start + self.expression_symbol_block().cursor];
        
        for item in expression_slice {
            let (symbol, value) = item.as_associative_item()?;
            if value == expression_index {
                return self.get_symbol_string(symbol);
            }
        }
        
        Ok(None)
    }

    pub fn get_associative_item_with_symbol(&self, list_index: usize, symbol: u64) -> Result<Option<&BasicData<T>>, DataError> {
        let (len, associations_len) = self.get_from_data_block_ensure_index(list_index)?.as_list()?;

        let association_start = self.data_block().start + list_index + len + 1;
        let association_range = association_start..association_start + associations_len;
        let association_slice = &self.data()[association_range];

        match search_for_associative_item_index(association_slice, symbol)? {
            Some(index) => Ok(Some(&self.data()[association_start + index])),
            None => Ok(None),
        }
    }

    pub fn get_associative_item_with_symbol_mut(&mut self, list_index: usize, symbol: u64) -> Result<Option<&mut BasicData<T>>, DataError> {
        let (len, associations_len) = self.get_from_data_block_ensure_index(list_index)?.as_list()?;

        let association_start = self.data_block().start + list_index + len + 1;
        let association_range = association_start..association_start + associations_len;
        
        let search_index = {
            let association_slice = &self.data()[association_range.clone()];
            search_for_associative_item_index(association_slice, symbol)?
        };

        match search_index {
            Some(index) => {
                let true_index = association_start + index;
                Ok(Some(&mut self.data_mut()[true_index]))
            },
            None => Ok(None),
        }
    }

    pub(crate) fn data(&self) -> &Vec<BasicData<T>> {
        &self.data
    }

    pub(crate) fn data_mut(&mut self) -> &mut Vec<BasicData<T>> {
        &mut self.data
    }

    pub(crate) fn data_block(&self) -> &StorageBlock {
        &self.data_block
    }

    pub(crate) fn instruction_block(&self) -> &StorageBlock {
        &self.instruction_block
    }

    pub(crate) fn jump_table_block(&self) -> &StorageBlock {
        &self.jump_table_block
    }

    pub(crate) fn symbol_table_block(&self) -> &StorageBlock {
        &self.symbol_table_block
    }

    pub(crate) fn expression_symbol_block(&self) -> &StorageBlock {
        &self.expression_symbol_block
    }

    pub(crate) fn instruction_block_mut(&mut self) -> &mut StorageBlock {
        &mut self.instruction_block
    }

    pub(crate) fn jump_table_block_mut(&mut self) -> &mut StorageBlock {
        &mut self.jump_table_block
    }

    pub(crate) fn symbol_table_block_mut(&mut self) -> &mut StorageBlock {
        &mut self.symbol_table_block
    }

    pub(crate) fn expression_symbol_block_mut(&mut self) -> &mut StorageBlock {
        &mut self.expression_symbol_block
    }

    pub(crate) fn custom_data_block(&self) -> &StorageBlock {
        &self.custom_data_block
    }

    pub(crate) fn data_block_mut(&mut self) -> &mut StorageBlock {
        &mut self.data_block
    }

    pub(crate) fn custom_data_block_mut(&mut self) -> &mut StorageBlock {
        &mut self.custom_data_block
    }

    pub(crate) fn current_value(&self) -> Option<usize> {
        self.current_value
    }

    pub(crate) fn set_current_value(&mut self, value: Option<usize>) {
        self.current_value = value;
    }

    pub(crate) fn current_register(&self) -> Option<usize> {
        self.current_register
    }

    pub(crate) fn set_current_register(&mut self, value: Option<usize>) {
        self.current_register = value;
    }

    pub(crate) fn instruction_pointer(&self) -> usize {
        self.instruction_pointer
    }

    pub(crate) fn set_instruction_pointer(&mut self, value: usize) {
        self.instruction_pointer = value;
    }

    pub(crate) fn current_frame(&self) -> Option<usize> {
        self.current_frame
    }

    pub(crate) fn set_current_frame(&mut self, value: Option<usize>) {
        self.current_frame = value;
    }

}

#[cfg(test)]
pub mod utilities {
    use crate::{
        BasicGarnishDataUnit, NoOpCompanion, basic::storage::{ReallocationStrategy, StorageSettings}
    };

    pub fn test_data() -> BasicGarnishDataUnit {
        BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::default(),
        ).unwrap()
    }

    pub fn instruction_test_data() -> BasicGarnishDataUnit {
        BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::default(),
        ).unwrap()
    }

    pub fn jump_table_test_data() -> BasicGarnishDataUnit {
        BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
        ).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use garnish_lang_traits::{GarnishDataType, Instruction};

    use crate::{
        BasicData, BasicDataCustom, BasicGarnishData, BasicGarnishDataUnit, DataError, NoOpCompanion, basic::{
            basic::utilities::test_data, object::BasicObject, storage::{ReallocationStrategy, StorageBlock, StorageSettings}
            
        }, error::DataErrorType
    };

    #[test]
    fn test_basic_garnish_data() {
        BasicGarnishData::<(), NoOpCompanion>::new(NoOpCompanion::new()).unwrap();
    }

    #[test]
    fn created_with_sizes_according_to_settings() {
        let data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
        );

        let expected_data = vec![BasicData::Empty; 50];

        let mut expected_instruction_block = StorageBlock::new(10, StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)));
        expected_instruction_block.start = 0;

        let mut expected_jump_table_block = StorageBlock::new(10, StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)));
        expected_jump_table_block.start = 10;

        let mut expected_symbol_table_block = StorageBlock::new(10, StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)));
        expected_symbol_table_block.start = 20;

        let mut expected_expression_symbol_block = StorageBlock::new(10, StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)));
        expected_expression_symbol_block.start = 30;

        let mut expected_data_block = StorageBlock::new(10, StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)));
        expected_data_block.start = 40;

        let mut expected_custom_data_block = StorageBlock::new(0, StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)));
        expected_custom_data_block.start = 50;

        assert_eq!(
            data,
            Ok(BasicGarnishDataUnit {
                instruction_pointer: 0,
                current_value: None,
                current_register: None,
                current_frame: None,
                data_retention_count: 0,
                data: expected_data,
                instruction_block: expected_instruction_block,
                jump_table_block: expected_jump_table_block,
                symbol_table_block: expected_symbol_table_block,
                expression_symbol_block: expected_expression_symbol_block,
                data_block: expected_data_block,
                custom_data_block: expected_custom_data_block,
                companion: crate::basic::NoOpCompanion::default(),
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
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
        ).unwrap();
        let index = data.push_to_instruction_block(Instruction::Add, None).unwrap();
        assert_eq!(index, 0);
        assert_eq!(data.instruction_block().cursor, 1);
        assert_eq!(data.instruction_block().size, 10);
        assert_eq!(
            data.data()[0],
            BasicData::Instruction(Instruction::Add)
        );
    }

    #[test]
    fn push_to_jump_table_block() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
        ).unwrap();
        let index = data.push_to_jump_table_block(100).unwrap();
        assert_eq!(index, 0);
        assert_eq!(data.jump_table_block().cursor, 1);
        assert_eq!(data.jump_table_block().size, 10);
        assert_eq!(
            data.data()[0],
            BasicData::JumpPoint(100)
        );
    }

    #[test]
    fn data_block_resizes_when_pushed_past_max_size() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, 20, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
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
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
        ).unwrap();
        for _ in 0..15 {
            data.push_to_instruction_block(Instruction::Add, None).unwrap();
        }

        let mut expected_data = vec![BasicData::Instruction(Instruction::Add); 15];
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
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
        ).unwrap();
        for i in 0..15 {
            data.push_to_jump_table_block(i).unwrap();
        }

        let mut expected_data: Vec<BasicData<()>> = (0..15).map(|i| BasicData::JumpPoint(i)).collect();
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
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
        ).unwrap();

        data.push_to_instruction_block(Instruction::Add, None).unwrap();
        data.push_to_jump_table_block(200).unwrap();
        data.push_to_symbol_table_block(100, 123).unwrap();
        data.push_to_expression_symbol_block(50, 0).unwrap();
        data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        data.push_to_custom_data_block(()).unwrap();

        let mut expected_data = vec![BasicData::Empty; 60];
        expected_data[0] = BasicData::Instruction(Instruction::Add);
        expected_data[10] = BasicData::JumpPoint(200);
        expected_data[20] = BasicData::AssociativeItem(100, 123);
        expected_data[30] = BasicData::AssociativeItem(50, 0);
        expected_data[40] = BasicData::Number(100.into());
        expected_data[50] = BasicData::Custom(());

        assert_eq!(data.instruction_size(), 1);
        assert_eq!(data.jump_table_size(), 1);
        assert_eq!(data.symbol_table_size(), 1);
        assert_eq!(data.data_size(), 1);
        assert_eq!(data.allocated_instruction_size(), 10);
        assert_eq!(data.allocated_jump_table_size(), 10);
        assert_eq!(data.allocated_symbol_table_size(), 10);
        assert_eq!(data.allocated_data_size(), 10);
        assert_eq!(data.allocated_expression_symbol_block_size(), 10);
        assert_eq!(data.allocated_custom_data_size(), 10);
        assert_eq!(data.data, expected_data);
    }

    #[test]
    fn reallocations_happen_correctly_pushing_bottom_to_top() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
        ).unwrap();

        data.push_to_custom_data_block(()).unwrap();
        data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        data.push_to_expression_symbol_block(50, 0).unwrap();
        data.push_to_symbol_table_block(100, 0).unwrap();
        data.push_to_jump_table_block(200).unwrap();
        data.push_to_instruction_block(Instruction::Add, None).unwrap();

        let mut expected_data = vec![BasicData::Empty; 60];
        expected_data[0] = BasicData::Instruction(Instruction::Add);
        expected_data[10] = BasicData::JumpPoint(200);
        expected_data[20] = BasicData::AssociativeItem(100, 0);
        expected_data[30] = BasicData::AssociativeItem(50, 0);
        expected_data[40] = BasicData::Number(100.into());
        expected_data[50] = BasicData::Custom(());

        assert_eq!(data.instruction_size(), 1);
        assert_eq!(data.jump_table_size(), 1);
        assert_eq!(data.symbol_table_size(), 1);
        assert_eq!(data.data_size(), 1);
        assert_eq!(data.allocated_instruction_size(), 10);
        assert_eq!(data.allocated_jump_table_size(), 10);
        assert_eq!(data.allocated_symbol_table_size(), 10);
        assert_eq!(data.allocated_data_size(), 10);
        assert_eq!(data.allocated_expression_symbol_block_size(), 10);
        assert_eq!(data.allocated_custom_data_size(), 10);
        assert_eq!(data.data, expected_data);
    }

    #[test]
    fn push_to_data_block_exceeds_max_items() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
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
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
        ).unwrap();

        for _ in 0..10 {
            data.push_to_instruction_block(Instruction::Add, None).unwrap();
        }

        let result = data.push_to_instruction_block(Instruction::Add, None);

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
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
        ).unwrap();

        for i in 0..10 {
            data.push_to_jump_table_block(i).unwrap();
        }

        let result = data.push_to_jump_table_block(10);

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
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
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
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
        ).unwrap();
        for _ in 0..15 {
            data.push_to_symbol_table_block(100, 0).unwrap();
        }

        let mut expected_data = vec![BasicData::<()>::Empty; 50];
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
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
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
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
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
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
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
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
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
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
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
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
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

    #[test]
    fn push_to_expression_symbol_block() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
        ).unwrap();
        data.push_to_expression_symbol_block(100, 0).unwrap();
        assert_eq!(data.expression_symbol_block.cursor, 1);
        assert_eq!(data.expression_symbol_block.size, 10);
    }

    #[test]
    fn expression_symbol_block_resizes_when_pushed_past_max_size() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, 20, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
        ).unwrap();
        for _ in 0..15 {
            data.push_to_expression_symbol_block(100, 0).unwrap();
        }

        let mut expected_data = vec![BasicData::<()>::Empty; 50];
        for i in 30..45 {
            expected_data[i] = BasicData::AssociativeItem(100, 0);
        }

        assert_eq!(data.expression_symbol_block_size(), 15);
        assert_eq!(data.allocated_expression_symbol_block_size(), 20);
    }

    #[test]
    fn push_to_expression_symbol_block_exceeds_max_items() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
        ).unwrap();

        for _ in 0..10 {
            data.push_to_expression_symbol_block(100, 0).unwrap();
        }

        let result = data.push_to_expression_symbol_block(100, 0);

        assert_eq!(
            result,
            Err(DataError::new(
                "Expression symbol block size exceeds max items",
                DataErrorType::ExpressionSymbolBlockExceededMaxItems(20, 10)
            ))
        );
    }

    #[test]
    fn push_to_expression_symbol_block_is_sorted() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 0, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
        ).unwrap();
        data.push_to_expression_symbol_block(300, 0).unwrap();
        data.push_to_expression_symbol_block(600, 0).unwrap();
        data.push_to_expression_symbol_block(100, 0).unwrap();
        data.push_to_expression_symbol_block(500, 0).unwrap();
        data.push_to_expression_symbol_block(200, 0).unwrap();
        data.push_to_expression_symbol_block(400, 0).unwrap();

        let mut expected_data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, 10, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 0, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, 10, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
        ).unwrap();

        expected_data.data[0] = BasicData::AssociativeItem(100, 0);
        expected_data.data[1] = BasicData::AssociativeItem(200, 0);
        expected_data.data[2] = BasicData::AssociativeItem(300, 0);
        expected_data.data[3] = BasicData::AssociativeItem(400, 0);
        expected_data.data[4] = BasicData::AssociativeItem(500, 0);
        expected_data.data[5] = BasicData::AssociativeItem(600, 0);
        expected_data.expression_symbol_block.cursor = 6;
        assert_eq!(data, expected_data);
    }

    #[test]
    fn get_symbol_expression() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
        ).unwrap();
        let index = data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        data.push_to_expression_symbol_block(100, index).unwrap();

        let index = data.push_to_data_block(BasicData::Number(200.into())).unwrap();
        data.push_to_expression_symbol_block(50, index).unwrap();

        let index = data.push_to_data_block(BasicData::Number(300.into())).unwrap();
        data.push_to_expression_symbol_block(200, index).unwrap();

        let result = data.get_symbol_expression(50).unwrap();

        assert_eq!(result, Some(1));
    }

    #[test]
    fn get_symbol_expression_not_found() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
        ).unwrap();
        let index = data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        data.push_to_expression_symbol_block(100, index).unwrap();

        let index = data.push_to_data_block(BasicData::Number(200.into())).unwrap();
        data.push_to_expression_symbol_block(50, index).unwrap();

        let index = data.push_to_data_block(BasicData::Number(300.into())).unwrap();
        data.push_to_expression_symbol_block(200, index).unwrap();

        let result = data.get_symbol_expression(150).unwrap();

        assert_eq!(result, None);
    }

    #[test]
    fn get_expression_string() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
        ).unwrap();
        
        // Create expression data
        let expr_index = data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        
        // Map symbol 50 to expression index
        data.push_to_expression_symbol_block(50, expr_index).unwrap();
        
        // Map symbol 50 to string "my expression"
        let str_index = data.push_object_to_data_block(BasicObject::CharList("my expression".to_string())).unwrap();
        data.push_to_symbol_table_block(50, str_index).unwrap();

        let result = data.get_expression_string(expr_index).unwrap();

        assert_eq!(result, Some("my expression".to_string()));
    }

    #[test]
    fn get_expression_string_not_found_expression() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
        ).unwrap();
        
        // Create expression data but don't map it
        let expr_index = data.push_to_data_block(BasicData::Number(100.into())).unwrap();

        let result = data.get_expression_string(expr_index).unwrap();

        assert_eq!(result, None);
    }

    #[test]
    fn get_expression_string_not_found_symbol() {
        let mut data = BasicGarnishDataUnit::new_with_settings(
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            NoOpCompanion::new(),
        ).unwrap();
        
        // Create expression data
        let expr_index = data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        
        // Map symbol 50 to expression index
        data.push_to_expression_symbol_block(50, expr_index).unwrap();
        
        // Don't map symbol 50 to a string

        let result = data.get_expression_string(expr_index).unwrap();

        assert_eq!(result, None);
    }

    #[test]
    fn push_to_custom_data_block() {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
        struct TestCustom {
            value: String,
        }

        impl BasicDataCustom for TestCustom {}

        #[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd)]
        struct TestCustomCompanion;

        impl crate::basic::companion::BasicDataCompanion<TestCustom> for TestCustomCompanion {
            fn resolve(_data: &mut BasicGarnishData<TestCustom, Self>, _symbol: u64) -> Result<bool, DataError> {
                Ok(false)
            }

            fn apply(_data: &mut BasicGarnishData<TestCustom, Self>, _external_value: usize, _input_addr: usize) -> Result<bool, DataError> {
                Ok(false)
            }

            fn defer_op(_data: &mut BasicGarnishData<TestCustom, Self>, _operation: Instruction, _left: (GarnishDataType, usize), _right: (GarnishDataType, usize)) -> Result<bool, DataError> {
                Ok(false)
            }
        }

        let mut data = BasicGarnishData::<TestCustom, TestCustomCompanion>::new_with_settings(
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            TestCustomCompanion {},
        ).unwrap();

        let custom_value = TestCustom { value: "test value".to_string() };
        let index = data.push_to_custom_data_block(custom_value.clone()).unwrap();

        assert_eq!(index, 0);
        assert_eq!(data.custom_data_size(), 1);
        assert_eq!(data.allocated_custom_data_size(), 10);
    }
}
