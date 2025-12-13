use garnish_lang_traits::GarnishData;
use crate::{DataError, DataIndexIterator, NumberIterator, SizeIterator, basic::{BasicGarnishData, BasicNumber}};

impl<T> GarnishData for BasicGarnishData<T> {
    type Error = DataError;
    type Symbol = u64;
    type Byte = u8;
    type Char = char;
    type Number = BasicNumber;
    type Size = usize;
    type SizeIterator = SizeIterator;
    type NumberIterator = NumberIterator;
    type InstructionIterator = SizeIterator;
    type DataIndexIterator = SizeIterator;
    type ValueIndexIterator = SizeIterator;
    type RegisterIndexIterator = SizeIterator;
    type JumpTableIndexIterator = SizeIterator;
    type JumpPathIndexIterator = SizeIterator;
    type ListIndexIterator = NumberIterator;
    type ListItemIterator = DataIndexIterator;
    type ConcatenationItemIterator = DataIndexIterator;
    
    fn get_data_len(&self) -> Self::Size {
        todo!()
    }
    
    fn get_data_iter(&self) -> Self::DataIndexIterator {
        todo!()
    }
    
    fn get_value_stack_len(&self) -> Self::Size {
        todo!()
    }
    
    fn push_value_stack(&mut self, addr: Self::Size) -> Result<(), Self::Error> {
        todo!()
    }
    
    fn pop_value_stack(&mut self) -> Option<Self::Size> {
        todo!()
    }
    
    fn get_value(&self, addr: Self::Size) -> Option<Self::Size> {
        todo!()
    }
    
    fn get_value_mut(&mut self, addr: Self::Size) -> Option<&mut Self::Size> {
        todo!()
    }
    
    fn get_current_value(&self) -> Option<Self::Size> {
        todo!()
    }
    
    fn get_current_value_mut(&mut self) -> Option<&mut Self::Size> {
        todo!()
    }
    
    fn get_value_iter(&self) -> Self::ValueIndexIterator {
        todo!()
    }
    
    fn get_data_type(&self, addr: Self::Size) -> Result<garnish_lang_traits::GarnishDataType, Self::Error> {
        todo!()
    }
    
    fn get_number(&self, addr: Self::Size) -> Result<Self::Number, Self::Error> {
        todo!()
    }
    
    fn get_type(&self, addr: Self::Size) -> Result<garnish_lang_traits::GarnishDataType, Self::Error> {
        todo!()
    }
    
    fn get_char(&self, addr: Self::Size) -> Result<Self::Char, Self::Error> {
        todo!()
    }
    
    fn get_byte(&self, addr: Self::Size) -> Result<Self::Byte, Self::Error> {
        todo!()
    }
    
    fn get_symbol(&self, addr: Self::Size) -> Result<Self::Symbol, Self::Error> {
        todo!()
    }
    
    fn get_expression(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn get_external(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn get_pair(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
        todo!()
    }
    
    fn get_concatenation(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
        todo!()
    }
    
    fn get_range(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
        todo!()
    }
    
    fn get_slice(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
        todo!()
    }
    
    fn get_partial(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
        todo!()
    }
    
    fn get_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn get_list_item(&self, list_addr: Self::Size, item_addr: Self::Number) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn get_list_associations_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn get_list_association(&self, list_addr: Self::Size, item_addr: Self::Number) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn get_list_item_with_symbol(&self, list_addr: Self::Size, sym: Self::Symbol) -> Result<Option<Self::Size>, Self::Error> {
        todo!()
    }
    
    fn get_list_items_iter(&self, list_addr: Self::Size) -> Self::ListIndexIterator {
        todo!()
    }
    
    fn get_list_associations_iter(&self, list_addr: Self::Size) -> Self::ListIndexIterator {
        todo!()
    }
    
    fn get_char_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn get_char_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Self::Char, Self::Error> {
        todo!()
    }
    
    fn get_char_list_iter(&self, list_addr: Self::Size) -> Self::ListIndexIterator {
        todo!()
    }
    
    fn get_byte_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn get_byte_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Self::Byte, Self::Error> {
        todo!()
    }
    
    fn get_byte_list_iter(&self, list_addr: Self::Size) -> Self::ListIndexIterator {
        todo!()
    }
    
    fn get_symbol_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn get_symbol_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Self::Symbol, Self::Error> {
        todo!()
    }
    
    fn get_symbol_list_iter(&self, list_addr: Self::Size) -> Self::ListIndexIterator {
        todo!()
    }
    
    fn get_list_item_iter(&self, list_addr: Self::Size) -> Self::ListItemIterator {
        todo!()
    }
    
    fn get_concatenation_iter(&self, addr: Self::Size) -> Self::ConcatenationItemIterator {
        todo!()
    }
    
    fn get_slice_iter(&self, addr: Self::Size) -> Self::ListIndexIterator {
        todo!()
    }
    
    fn get_list_slice_item_iter(&self, list_addr: Self::Size) -> Self::ListItemIterator {
        todo!()
    }
    
    fn get_concatenation_slice_iter(&self, addr: Self::Size) -> Self::ConcatenationItemIterator {
        todo!()
    }
    
    fn add_unit(&mut self) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn add_true(&mut self) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn add_false(&mut self) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn add_number(&mut self, value: Self::Number) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn add_type(&mut self, value: garnish_lang_traits::GarnishDataType) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn add_char(&mut self, value: Self::Char) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn add_byte(&mut self, value: Self::Byte) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn add_symbol(&mut self, value: Self::Symbol) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn add_expression(&mut self, value: Self::Size) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn add_external(&mut self, value: Self::Size) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn add_pair(&mut self, value: (Self::Size, Self::Size)) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn add_concatenation(&mut self, left: Self::Size, right: Self::Size) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn add_range(&mut self, start: Self::Size, end: Self::Size) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn add_slice(&mut self, list: Self::Size, range: Self::Size) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn add_partial(&mut self, list: Self::Size, range: Self::Size) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn merge_to_symbol_list(&mut self, first: Self::Size, second: Self::Size) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn start_list(&mut self, len: Self::Size) -> Result<(), Self::Error> {
        todo!()
    }
    
    fn add_to_list(&mut self, addr: Self::Size, is_associative: bool) -> Result<(), Self::Error> {
        todo!()
    }
    
    fn end_list(&mut self) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn start_char_list(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
    
    fn add_to_char_list(&mut self, c: Self::Char) -> Result<(), Self::Error> {
        todo!()
    }
    
    fn end_char_list(&mut self) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn start_byte_list(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
    
    fn add_to_byte_list(&mut self, c: Self::Byte) -> Result<(), Self::Error> {
        todo!()
    }
    
    fn end_byte_list(&mut self) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn get_register_len(&self) -> Self::Size {
        todo!()
    }
    
    fn push_register(&mut self, addr: Self::Size) -> Result<(), Self::Error> {
        todo!()
    }
    
    fn get_register(&self, addr: Self::Size) -> Option<Self::Size> {
        todo!()
    }
    
    fn pop_register(&mut self) -> Result<Option<Self::Size>, Self::Error> {
        todo!()
    }
    
    fn get_register_iter(&self) -> Self::RegisterIndexIterator {
        todo!()
    }
    
    fn get_instruction_len(&self) -> Self::Size {
        todo!()
    }
    
    fn push_instruction(&mut self, instruction: garnish_lang_traits::Instruction, data: Option<Self::Size>) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn get_instruction(&self, addr: Self::Size) -> Option<(garnish_lang_traits::Instruction, Option<Self::Size>)> {
        todo!()
    }
    
    fn get_instruction_iter(&self) -> Self::InstructionIterator {
        todo!()
    }
    
    fn get_instruction_cursor(&self) -> Self::Size {
        todo!()
    }
    
    fn set_instruction_cursor(&mut self, addr: Self::Size) -> Result<(), Self::Error> {
        todo!()
    }
    
    fn get_jump_table_len(&self) -> Self::Size {
        todo!()
    }
    
    fn push_jump_point(&mut self, index: Self::Size) -> Result<(), Self::Error> {
        todo!()
    }
    
    fn get_jump_point(&self, index: Self::Size) -> Option<Self::Size> {
        todo!()
    }
    
    fn get_jump_point_mut(&mut self, index: Self::Size) -> Option<&mut Self::Size> {
        todo!()
    }
    
    fn get_jump_table_iter(&self) -> Self::JumpTableIndexIterator {
        todo!()
    }
    
    fn push_jump_path(&mut self, index: Self::Size) -> Result<(), Self::Error> {
        todo!()
    }
    
    fn pop_jump_path(&mut self) -> Option<Self::Size> {
        todo!()
    }
    
    fn get_jump_path_iter(&self) -> Self::JumpPathIndexIterator {
        todo!()
    }
    
    fn size_to_number(from: Self::Size) -> Self::Number {
        todo!()
    }
    
    fn number_to_size(from: Self::Number) -> Option<Self::Size> {
        todo!()
    }
    
    fn number_to_char(from: Self::Number) -> Option<Self::Char> {
        todo!()
    }
    
    fn number_to_byte(from: Self::Number) -> Option<Self::Byte> {
        todo!()
    }
    
    fn char_to_number(from: Self::Char) -> Option<Self::Number> {
        todo!()
    }
    
    fn char_to_byte(from: Self::Char) -> Option<Self::Byte> {
        todo!()
    }
    
    fn byte_to_number(from: Self::Byte) -> Option<Self::Number> {
        todo!()
    }
    
    fn byte_to_char(from: Self::Byte) -> Option<Self::Char> {
        todo!()
    }
    
    fn add_char_list_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn add_byte_list_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn add_symbol_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn add_byte_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn add_number_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
        todo!()
    }
    
    fn parse_number(from: &str) -> Result<Self::Number, Self::Error> {
        todo!()
    }
    
    fn parse_symbol(from: &str) -> Result<Self::Symbol, Self::Error> {
        todo!()
    }
    
    fn parse_char(from: &str) -> Result<Self::Char, Self::Error> {
        todo!()
    }
    
    fn parse_byte(from: &str) -> Result<Self::Byte, Self::Error> {
        todo!()
    }
    
    fn parse_char_list(from: &str) -> Result<Vec<Self::Char>, Self::Error> {
        todo!()
    }
    
    fn parse_byte_list(from: &str) -> Result<Vec<Self::Byte>, Self::Error> {
        todo!()
    }
    
    fn make_size_iterator_range(min: Self::Size, max: Self::Size) -> Self::SizeIterator {
        todo!()
    }
    
    fn make_number_iterator_range(min: Self::Number, max: Self::Number) -> Self::NumberIterator {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_data_len() {
        let data = BasicGarnishData::new_unit();
        assert_eq!(data.get_data_len(), 0);
    }
}