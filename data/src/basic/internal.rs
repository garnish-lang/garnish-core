use crate::basic::storage::StorageBlock;
use crate::error::DataErrorType;
use crate::{BasicData, DataError};

use super::BasicGarnishData;

impl<T> BasicGarnishData<T>
where
    T: crate::basic::BasicDataCustom,
{
    pub(crate) fn get_from_data_block_ensure_index(&self, index: usize) -> Result<&BasicData<T>, DataError> {
        if index >= self.data_block().cursor {
            return Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(index)));
        }
        let true_index = self.data_block().start + index;
        Ok(&self.data()[true_index])
    }

    pub(crate) fn get_from_data_block_ensure_index_mut(&mut self, index: usize) -> Result<&mut BasicData<T>, DataError> {
        if index >= self.data_block().cursor {
            return Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(index)));
        }
        let true_index = self.data_block().start + index;
        Ok(&mut self.data_mut()[true_index])
    }

    pub(crate) fn get_from_instruction_block_ensure_index(&self, index: usize) -> Result<(garnish_lang_traits::Instruction, Option<usize>), DataError> {
        if index >= self.instruction_block().cursor {
            return Err(DataError::new("Invalid instruction index", DataErrorType::InvalidInstructionIndex(index)));
        }
        let true_index = self.instruction_block().start + index;
        match &self.data()[true_index] {
            BasicData::Instruction(instruction, data) => Ok((*instruction, Some(*data))),
            BasicData::InstructionRoot(instruction) => Ok((*instruction, None)),
            _ => Err(DataError::not_basic_type_error()),
        }
    }

    pub(crate) fn get_from_jump_table_block_ensure_index(&self, index: usize) -> Result<usize, DataError> {
        if index >= self.jump_table_block().cursor {
            return Err(DataError::new("Invalid jump table index", DataErrorType::InvalidJumpTableIndex(index)));
        }
        let true_index = self.jump_table_block().start + index;
        self.data()[true_index].as_jump_point()
    }

    pub(crate) fn get_from_jump_table_block_ensure_index_mut(&mut self, index: usize) -> Result<&mut usize, DataError> {
        if index >= self.jump_table_block().cursor {
            return Err(DataError::new("Invalid jump table index", DataErrorType::InvalidJumpTableIndex(index)));
        }
        let true_index = self.jump_table_block().start + index;
        self.data_mut()[true_index].as_jump_point_mut()
    }

    pub(crate) fn get_from_custom_data_block_ensure_index(&self, index: usize) -> Result<T, DataError> {
        if index >= self.custom_data_block().cursor {
            return Err(DataError::new("Invalid custom data index", DataErrorType::InvalidDataIndex(index)));
        }
        let true_index = self.custom_data_block().start + index;
        match &self.data()[true_index] {
            BasicData::Custom(value) => Ok(value.clone()),
            _ => Err(DataError::new("Not custom data", DataErrorType::NotType(garnish_lang_traits::GarnishDataType::Custom, self.data()[true_index].get_data_type()))),
        }
    }

    pub(crate) fn push_to_block(heap: &mut Vec<BasicData<T>>, block: &mut StorageBlock, data: BasicData<T>) -> usize {
        let index = block.cursor;
        heap[block.start + index] = data;
        block.cursor += 1;
        index
    }

    pub(crate) fn reallocate_heap(
        &mut self,
        new_instruction_size: usize,
        new_jump_table_size: usize,
        new_symbol_table_size: usize,
        new_data_size: usize,
        new_custom_data_size: usize,
    ) -> Result<(), DataError> {
        if new_instruction_size > self.instruction_block().settings.max_items() {
            return Err(DataError::new(
                "Instruction block size exceeds max items",
                DataErrorType::InstructionBlockExceededMaxItems(new_instruction_size, self.instruction_block().settings.max_items()),
            ));
        }
        if new_jump_table_size > self.jump_table_block().settings.max_items() {
            return Err(DataError::new(
                "Jump table block size exceeds max items",
                DataErrorType::JumpTableBlockExceededMaxItems(new_jump_table_size, self.jump_table_block().settings.max_items()),
            ));
        }
        if new_symbol_table_size > self.symbol_table_block().settings.max_items() {
            return Err(DataError::new(
                "Symbol table block size exceeds max items",
                DataErrorType::SymbolTableBlockExceededMaxItems(new_symbol_table_size, self.symbol_table_block().settings.max_items()),
            ));
        }
        if new_data_size > self.data_block().settings.max_items() {
            return Err(DataError::new(
                "Data block size exceeds max items",
                DataErrorType::DataBlockExceededMaxItems(new_data_size, self.data_block().settings.max_items()),
            ));
        }
        if new_custom_data_size > self.custom_data_block().settings.max_items() {
            return Err(DataError::new(
                "Custom data block size exceeds max items",
                DataErrorType::DataBlockExceededMaxItems(new_custom_data_size, self.custom_data_block().settings.max_items()),
            ));
        }

        let instruction_start = self.instruction_block().start;
        let instruction_cursor = self.instruction_block().cursor;
        let jump_table_start = self.jump_table_block().start;
        let jump_table_cursor = self.jump_table_block().cursor;
        let symbol_table_start = self.symbol_table_block().start;
        let symbol_table_cursor = self.symbol_table_block().cursor;
        let data_start = self.data_block().start;
        let data_cursor = self.data_block().cursor;
        let custom_data_start = self.custom_data_block().start;
        let custom_data_cursor = self.custom_data_block().cursor;

        let new_size = new_instruction_size + new_jump_table_size + new_symbol_table_size + new_data_size + new_custom_data_size;

        let mut new_heap = vec![BasicData::Empty; new_size];

        let mut current_block_start = 0;
        
        // Copy instruction block
        for i in 0..instruction_cursor {
            new_heap[current_block_start + i] = self.data()[instruction_start + i].clone();
        }
        self.instruction_block_mut().start = current_block_start;
        self.instruction_block_mut().size = new_instruction_size;
        current_block_start += new_instruction_size;

        // Copy jump table block
        for i in 0..jump_table_cursor {
            new_heap[current_block_start + i] = self.data()[jump_table_start + i].clone();
        }
        self.jump_table_block_mut().start = current_block_start;
        self.jump_table_block_mut().size = new_jump_table_size;
        current_block_start += new_jump_table_size;

        // Copy symbol table block
        for i in 0..symbol_table_cursor {
            new_heap[current_block_start + i] = self.data()[symbol_table_start + i].clone();
        }
        self.symbol_table_block_mut().start = current_block_start;
        self.symbol_table_block_mut().size = new_symbol_table_size;
        current_block_start += new_symbol_table_size;

        // Copy data block
        for i in 0..data_cursor {
            new_heap[current_block_start + i] = self.data()[data_start + i].clone();
        }
        self.data_block_mut().start = current_block_start;
        self.data_block_mut().size = new_data_size;
        current_block_start += new_data_size;

        // Copy custom data block
        for i in 0..custom_data_cursor {
            new_heap[current_block_start + i] = self.data()[custom_data_start + i].clone();
        }
        self.custom_data_block_mut().start = current_block_start;
        self.custom_data_block_mut().size = new_custom_data_size;

        *self.data_mut() = new_heap;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::basic::utilities::test_data;
    use crate::basic::BasicGarnishData;
    use crate::error::DataErrorType;
    use crate::{BasicData, DataError};

    #[test]
    fn get_from_data_block_ensure_index() {
        let mut data = test_data();

        data.push_to_data_block(BasicData::True).unwrap();

        let result = data.get_from_data_block_ensure_index(0).unwrap();

        assert_eq!(result, &BasicData::<()>::True);
    }

    #[test]
    fn get_from_data_block_ensure_index_index_out_of_bounds() {
        let mut data = test_data();

        data.push_to_data_block(BasicData::True).unwrap();

        let result = data.get_from_data_block_ensure_index(10);

        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(10))));
    }

    #[test]
    fn get_from_data_block_ensure_index_mut() {
        let mut data = test_data();

        data.push_to_data_block(BasicData::True).unwrap();

        let result = data.get_from_data_block_ensure_index_mut(0).unwrap();

        assert_eq!(result, &BasicData::<()>::True);

        *result = BasicData::False;

        let result_after = data.get_from_data_block_ensure_index(0).unwrap();
        assert_eq!(result_after, &BasicData::<()>::False);
    }

    #[test]
    fn get_from_data_block_ensure_index_mut_index_out_of_bounds() {
        let mut data = test_data();

        data.push_to_data_block(BasicData::True).unwrap();

        let result = data.get_from_data_block_ensure_index_mut(10);

        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(10))));
    }

    #[test]
    fn get_from_data_block_ensure_index_mut_can_modify() {
        let mut data = test_data();

        data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        data.push_to_data_block(BasicData::Number(200.into())).unwrap();

        let first = data.get_from_data_block_ensure_index_mut(0).unwrap();
        *first = BasicData::Number(300.into());

        assert_eq!(data.get_from_data_block_ensure_index(0).unwrap(), &BasicData::Number(300.into()));

        assert_eq!(data.get_from_data_block_ensure_index(1).unwrap(), &BasicData::Number(200.into()));
    }

    #[test]
    fn get_from_custom_data_block() {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
        struct TestCustom {
            value: String,
        }

        impl crate::basic::BasicDataCustom for TestCustom {}

        let mut data = BasicGarnishData::<TestCustom>::new_with_settings(
            crate::basic::storage::StorageSettings::new(0, usize::MAX, crate::basic::storage::ReallocationStrategy::FixedSize(10)),
            crate::basic::storage::StorageSettings::new(0, usize::MAX, crate::basic::storage::ReallocationStrategy::FixedSize(10)),
            crate::basic::storage::StorageSettings::new(0, usize::MAX, crate::basic::storage::ReallocationStrategy::FixedSize(10)),
            crate::basic::storage::StorageSettings::new(0, usize::MAX, crate::basic::storage::ReallocationStrategy::FixedSize(10)),
            crate::basic::storage::StorageSettings::new(10, usize::MAX, crate::basic::storage::ReallocationStrategy::FixedSize(10)),
        ).unwrap();

        let custom_value1 = TestCustom { value: "first".to_string() };
        let custom_value2 = TestCustom { value: "second".to_string() };

        let index1 = data.push_to_custom_data_block(custom_value1.clone()).unwrap();
        let index2 = data.push_to_custom_data_block(custom_value2.clone()).unwrap();

        assert_eq!(index1, 0);
        assert_eq!(index2, 1);

        let retrieved1 = data.get_from_custom_data_block(index1).unwrap();
        let retrieved2 = data.get_from_custom_data_block(index2).unwrap();

        assert_eq!(retrieved1, custom_value1);
        assert_eq!(retrieved2, custom_value2);

        // Test that None is returned for invalid index
        let invalid_result = data.get_from_custom_data_block(10);
        assert_eq!(invalid_result, None);
    }

    #[test]
    fn get_from_custom_data_block_index_out_of_bounds() {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
        struct TestCustom {
            value: String,
        }

        impl crate::basic::BasicDataCustom for TestCustom {}

        let data = BasicGarnishData::<TestCustom>::new_with_settings(
            crate::basic::storage::StorageSettings::new(0, usize::MAX, crate::basic::storage::ReallocationStrategy::FixedSize(10)),
            crate::basic::storage::StorageSettings::new(0, usize::MAX, crate::basic::storage::ReallocationStrategy::FixedSize(10)),
            crate::basic::storage::StorageSettings::new(0, usize::MAX, crate::basic::storage::ReallocationStrategy::FixedSize(10)),
            crate::basic::storage::StorageSettings::new(0, usize::MAX, crate::basic::storage::ReallocationStrategy::FixedSize(10)),
            crate::basic::storage::StorageSettings::new(10, usize::MAX, crate::basic::storage::ReallocationStrategy::FixedSize(10)),
        ).unwrap();

        let result = data.get_from_custom_data_block(10);

        assert_eq!(result, None);
    }
}
