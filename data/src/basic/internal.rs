use crate::basic::storage::StorageBlock;
use crate::error::DataErrorType;
use crate::{BasicData, DataError};

use super::BasicGarnishData;

impl<T> BasicGarnishData<T>
where
    T: crate::basic::BasicDataCustom,
{
    pub(crate) fn get_from_data_block_ensure_index(&self, index: usize) -> Result<&BasicData<T>, DataError> {
        if index >= self.data_block.cursor {
            return Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(index)));
        }
        let true_index = self.data_block.start + index;
        Ok(&self.data[true_index])
    }

    pub(crate) fn get_from_data_block_ensure_index_mut(&mut self, index: usize) -> Result<&mut BasicData<T>, DataError> {
        if index >= self.data_block.cursor {
            return Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(index)));
        }
        let true_index = self.data_block.start + index;
        Ok(&mut self.data[true_index])
    }

    pub(crate) fn get_from_instruction_block_ensure_index(&self, index: usize) -> Result<&BasicData<T>, DataError> {
        if index >= self.instruction_block.cursor {
            return Err(DataError::new("Invalid instruction index", DataErrorType::InvalidInstructionIndex(index)));
        }
        let true_index = self.instruction_block.start + index;
        Ok(&self.data[true_index])
    }

    pub(crate) fn get_from_jump_table_block_ensure_index(&self, index: usize) -> Result<&BasicData<T>, DataError> {
        if index >= self.jump_table_block.cursor {
            return Err(DataError::new("Invalid jump table index", DataErrorType::InvalidJumpTableIndex(index)));
        }
        let true_index = self.jump_table_block.start + index;
        Ok(&self.data[true_index])
    }

    pub(crate) fn get_from_jump_table_block_ensure_index_mut(&mut self, index: usize) -> Result<&mut BasicData<T>, DataError> {
        if index >= self.jump_table_block.cursor {
            return Err(DataError::new("Invalid jump table index", DataErrorType::InvalidJumpTableIndex(index)));
        }
        let true_index = self.jump_table_block.start + index;
        Ok(&mut self.data[true_index])
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
    ) -> Result<(), DataError> {
        if new_instruction_size > self.instruction_block.settings.max_items() {
            return Err(DataError::new(
                "Instruction block size exceeds max items",
                DataErrorType::InstructionBlockExceededMaxItems(new_instruction_size, self.instruction_block.settings.max_items()),
            ));
        }
        if new_jump_table_size > self.jump_table_block.settings.max_items() {
            return Err(DataError::new(
                "Jump table block size exceeds max items",
                DataErrorType::JumpTableBlockExceededMaxItems(new_jump_table_size, self.jump_table_block.settings.max_items()),
            ));
        }
        if new_symbol_table_size > self.symbol_table_block.settings.max_items() {
            return Err(DataError::new(
                "Symbol table block size exceeds max items",
                DataErrorType::SymbolTableBlockExceededMaxItems(new_symbol_table_size, self.symbol_table_block.settings.max_items()),
            ));
        }
        if new_data_size > self.data_block.settings.max_items() {
            return Err(DataError::new(
                "Data block size exceeds max items",
                DataErrorType::DataBlockExceededMaxItems(new_data_size, self.data_block.settings.max_items()),
            ));
        }

        let ordered = [
            (&mut self.instruction_block, new_instruction_size),
            (&mut self.jump_table_block, new_jump_table_size),
            (&mut self.symbol_table_block, new_symbol_table_size),
            (&mut self.data_block, new_data_size),
        ];

        let new_size = new_instruction_size + new_jump_table_size + new_symbol_table_size + new_data_size;

        let mut new_heap = vec![BasicData::Empty; new_size];

        let mut current_block_start = 0;
        for (block, new_size) in ordered {
            for i in 0..block.cursor {
                new_heap[current_block_start + i] = self.data[block.start + i].clone();
            }

            block.start = current_block_start;
            block.size = new_size;
            current_block_start += new_size;
        }

        self.data = new_heap;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::basic::utilities::test_data;
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
}
