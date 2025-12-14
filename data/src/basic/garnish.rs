use crate::{
    BasicData, BasicDataCustom, ByteListIterator, CharListIterator, DataError, DataIndexIterator, NumberIterator, SizeIterator, SymbolListPartIterator, basic::{BasicGarnishData, BasicNumber, merge_to_symbol_list::merge_to_symbol_list}
};
use garnish_lang_traits::{GarnishData, GarnishDataType, SymbolListPart};

impl<T> GarnishData for BasicGarnishData<T>
where
    T: BasicDataCustom,
{
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
    type CharIterator = CharListIterator;
    type ByteIterator = ByteListIterator;
    type SymbolListPartIterator = SymbolListPartIterator;

    fn get_data_len(&self) -> Self::Size {
        self.data.len()
    }

    fn get_data_iter(&self) -> Self::DataIndexIterator {
        SizeIterator::new(0, self.data.len())
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

    fn get_data_type(&self, addr: Self::Size) -> Result<GarnishDataType, Self::Error> {
        self.get_data_ensure_index(addr).map(|data| data.get_data_type())
    }

    fn get_number(&self, addr: Self::Size) -> Result<Self::Number, Self::Error> {
        self.get_data_ensure_index(addr)?.as_number()
    }

    fn get_type(&self, addr: Self::Size) -> Result<GarnishDataType, Self::Error> {
        self.get_data_ensure_index(addr)?.as_type()
    }

    fn get_char(&self, addr: Self::Size) -> Result<Self::Char, Self::Error> {
        self.get_data_ensure_index(addr)?.as_char()
    }

    fn get_byte(&self, addr: Self::Size) -> Result<Self::Byte, Self::Error> {
        self.get_data_ensure_index(addr)?.as_byte()
    }

    fn get_symbol(&self, addr: Self::Size) -> Result<Self::Symbol, Self::Error> {
        self.get_data_ensure_index(addr)?.as_symbol()
    }

    fn get_expression(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        self.get_data_ensure_index(addr)?.as_expression()
    }

    fn get_external(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        self.get_data_ensure_index(addr)?.as_external()
    }

    fn get_pair(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
        self.get_data_ensure_index(addr)?.as_pair()
    }

    fn get_concatenation(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
        self.get_data_ensure_index(addr)?.as_concatenation()
    }

    fn get_range(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
        self.get_data_ensure_index(addr)?.as_range()
    }

    fn get_slice(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
        self.get_data_ensure_index(addr)?.as_slice()
    }

    fn get_partial(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
        self.get_data_ensure_index(addr)?.as_partial()
    }

    fn get_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        self.get_data_ensure_index(addr)?.as_list()
    }

    fn get_list_item(&self, list_addr: Self::Size, item_addr: Self::Number) -> Result<Option<Self::Size>, Self::Error> {
        todo!()
    }

    fn get_list_associations_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        todo!()
    }

    fn get_list_association(&self, list_addr: Self::Size, item_addr: Self::Number) -> Result<Option<Self::Size>, Self::Error> {
        todo!()
    }

    fn get_list_item_with_symbol(&self, list_addr: Self::Size, sym: Self::Symbol) -> Result<Option<Self::Size>, Self::Error> {
        todo!()
    }

    fn get_list_items_iter(&self, list_addr: Self::Size) -> Result<Self::ListIndexIterator, Self::Error> {
        todo!()
    }

    fn get_list_associations_iter(&self, list_addr: Self::Size) -> Result<Self::ListIndexIterator, Self::Error> {
        todo!()
    }

    fn get_char_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        todo!()
    }

    fn get_char_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Option<Self::Char>, Self::Error> {
        todo!()
    }

    fn get_char_list_iter(&self, list_addr: Self::Size) -> Result<Self::CharIterator, Self::Error> {
        todo!()
    }

    fn get_byte_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        todo!()
    }

    fn get_byte_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Option<Self::Byte>, Self::Error> {
        todo!()
    }

    fn get_byte_list_iter(&self, list_addr: Self::Size) -> Result<Self::ByteIterator, Self::Error> {
        todo!()
    }

    fn get_symbol_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        todo!()
    }

    fn get_symbol_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Option<SymbolListPart<Self::Symbol, Self::Number>>, Self::Error> {
        todo!()
    }

    fn get_symbol_list_iter(&self, list_addr: Self::Size) -> Result<Self::SymbolListPartIterator, Self::Error> {
        todo!()
    }

    fn get_list_item_iter(&self, list_addr: Self::Size) -> Result<Self::ListItemIterator, Self::Error> {
        todo!()
    }

    fn get_concatenation_iter(&self, addr: Self::Size) -> Result<Self::ConcatenationItemIterator, Self::Error> {
        todo!()
    }

    fn get_slice_iter(&self, addr: Self::Size) -> Result<Self::ListIndexIterator, Self::Error> {
        todo!()
    }

    fn get_list_slice_item_iter(&self, list_addr: Self::Size) -> Result<Self::ListItemIterator, Self::Error> {
        todo!()
    }

    fn get_concatenation_slice_iter(&self, addr: Self::Size) -> Result<Self::ConcatenationItemIterator, Self::Error> {
        todo!()
    }

    fn add_unit(&mut self) -> Result<Self::Size, Self::Error> {
        Ok(self.push_basic_data(BasicData::Unit))
    }

    fn add_true(&mut self) -> Result<Self::Size, Self::Error> {
        Ok(self.push_basic_data(BasicData::True))
    }

    fn add_false(&mut self) -> Result<Self::Size, Self::Error> {
        Ok(self.push_basic_data(BasicData::False))
    }

    fn add_number(&mut self, value: Self::Number) -> Result<Self::Size, Self::Error> {
        Ok(self.push_basic_data(BasicData::Number(value)))
    }

    fn add_type(&mut self, value: garnish_lang_traits::GarnishDataType) -> Result<Self::Size, Self::Error> {
        Ok(self.push_basic_data(BasicData::Type(value)))
    }

    fn add_char(&mut self, value: Self::Char) -> Result<Self::Size, Self::Error> {
        Ok(self.push_basic_data(BasicData::Char(value)))
    }

    fn add_byte(&mut self, value: Self::Byte) -> Result<Self::Size, Self::Error> {
        Ok(self.push_basic_data(BasicData::Byte(value)))
    }

    fn add_symbol(&mut self, value: Self::Symbol) -> Result<Self::Size, Self::Error> {
        Ok(self.push_basic_data(BasicData::Symbol(value)))
    }

    fn add_expression(&mut self, value: Self::Size) -> Result<Self::Size, Self::Error> {
        Ok(self.push_basic_data(BasicData::Expression(value)))
    }

    fn add_external(&mut self, value: Self::Size) -> Result<Self::Size, Self::Error> {
        Ok(self.push_basic_data(BasicData::External(value)))
    }

    fn add_pair(&mut self, value: (Self::Size, Self::Size)) -> Result<Self::Size, Self::Error> {
        Ok(self.push_basic_data(BasicData::Pair(value.0, value.1)))
    }

    fn add_concatenation(&mut self, left: Self::Size, right: Self::Size) -> Result<Self::Size, Self::Error> {
        Ok(self.push_basic_data(BasicData::Concatenation(left, right)))
    }

    fn add_range(&mut self, start: Self::Size, end: Self::Size) -> Result<Self::Size, Self::Error> {
        Ok(self.push_basic_data(BasicData::Range(start, end)))
    }

    fn add_slice(&mut self, list: Self::Size, range: Self::Size) -> Result<Self::Size, Self::Error> {
        Ok(self.push_basic_data(BasicData::Slice(list, range)))
    }

    fn add_partial(&mut self, reciever: Self::Size, input: Self::Size) -> Result<Self::Size, Self::Error> {
        Ok(self.push_basic_data(BasicData::Partial(reciever, input)))
    }

    fn merge_to_symbol_list(&mut self, first: Self::Size, second: Self::Size) -> Result<Self::Size, Self::Error> {
        merge_to_symbol_list(self, first, second)
    }

    fn start_list(&mut self, len: Self::Size) -> Result<(), Self::Error> {
        self.push_basic_data(BasicData::List(len));
        Ok(())
    }

    fn add_to_list(&mut self, addr: Self::Size, is_associative: bool) -> Result<(), Self::Error> {
        self.push_basic_data(BasicData::ListItem(addr));
        Ok(())
    }

    fn end_list(&mut self) -> Result<Self::Size, Self::Error> {
        let mut current = self.get_data_len();

        while current > 0 {
            let item = self.get_basic_data(current);
            match item {
                Some(BasicData::List(_)) => {
                    break;
                }
                _ => {}
            }

            current -= 1;
        }

        Ok(current)
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
    use crate::{BasicData, BasicGarnishDataUnit, error::DataErrorType};

    use super::*;

    #[test]
    fn get_data_len() {
        let data = BasicGarnishDataUnit::new();
        assert_eq!(data.get_data_len(), 0);
    }

    #[test]
    fn get_data_len_with_items() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Unit);
        data.push_basic_data(BasicData::True);
        assert_eq!(data.get_data_len(), 2);
    }

    #[test]
    fn get_data_iter() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Unit);
        data.push_basic_data(BasicData::True);

        let mut iter = data.get_data_iter();
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn add_unit() {
        let mut data = BasicGarnishDataUnit::new();
        data.add_unit().unwrap();

        assert_eq!(data, BasicGarnishDataUnit::new_full(vec![BasicData::Unit]));
    }

    #[test]
    fn add_true() {
        let mut data = BasicGarnishDataUnit::new();
        data.add_true().unwrap();

        assert_eq!(data, BasicGarnishDataUnit::new_full(vec![BasicData::True]));
    }

    #[test]
    fn add_false() {
        let mut data = BasicGarnishDataUnit::new();
        data.add_false().unwrap();

        assert_eq!(data, BasicGarnishDataUnit::new_full(vec![BasicData::False]));
    }

    #[test]
    fn add_number() {
        let mut data = BasicGarnishDataUnit::new();
        data.add_number(100.into()).unwrap();

        assert_eq!(data, BasicGarnishDataUnit::new_full(vec![BasicData::Number(100.into())]));
    }

    #[test]
    fn add_type() {
        let mut data = BasicGarnishDataUnit::new();
        data.add_type(GarnishDataType::Number).unwrap();

        assert_eq!(data, BasicGarnishDataUnit::new_full(vec![BasicData::Type(GarnishDataType::Number)]));
    }

    #[test]
    fn add_char() {
        let mut data = BasicGarnishDataUnit::new();
        data.add_char('a').unwrap();

        assert_eq!(data, BasicGarnishDataUnit::new_full(vec![BasicData::Char('a')]));
    }

    #[test]
    fn add_byte() {
        let mut data = BasicGarnishDataUnit::new();
        data.add_byte(100).unwrap();

        assert_eq!(data, BasicGarnishDataUnit::new_full(vec![BasicData::Byte(100)]));
    }

    #[test]
    fn add_symbol() {
        let mut data = BasicGarnishDataUnit::new();
        data.add_symbol(100).unwrap();

        assert_eq!(data, BasicGarnishDataUnit::new_full(vec![BasicData::Symbol(100)]));
    }

    #[test]
    fn add_expression() {
        let mut data = BasicGarnishDataUnit::new();
        data.add_expression(100).unwrap();

        assert_eq!(data, BasicGarnishDataUnit::new_full(vec![BasicData::Expression(100)]));
    }

    #[test]
    fn add_external() {
        let mut data = BasicGarnishDataUnit::new();
        data.add_external(100).unwrap();

        assert_eq!(data, BasicGarnishDataUnit::new_full(vec![BasicData::External(100)]));
    }

    #[test]
    fn add_pair() {
        let mut data = BasicGarnishDataUnit::new();
        data.add_pair((100, 200)).unwrap();

        assert_eq!(data, BasicGarnishDataUnit::new_full(vec![BasicData::Pair(100, 200)]));
    }

    #[test]
    fn add_concatenation() {
        let mut data = BasicGarnishDataUnit::new();
        data.add_concatenation(100, 200).unwrap();

        assert_eq!(data, BasicGarnishDataUnit::new_full(vec![BasicData::Concatenation(100, 200)]));
    }

    #[test]
    fn add_range() {
        let mut data = BasicGarnishDataUnit::new();
        data.add_range(100, 200).unwrap();

        assert_eq!(data, BasicGarnishDataUnit::new_full(vec![BasicData::Range(100, 200)]));
    }

    #[test]
    fn add_slice() {
        let mut data = BasicGarnishDataUnit::new();
        data.add_slice(100, 200).unwrap();

        assert_eq!(data, BasicGarnishDataUnit::new_full(vec![BasicData::Slice(100, 200)]));
    }

    #[test]
    fn add_partial() {
        let mut data = BasicGarnishDataUnit::new();
        data.add_partial(100, 200).unwrap();

        assert_eq!(data, BasicGarnishDataUnit::new_full(vec![BasicData::Partial(100, 200)]));
    }


    #[test]
    fn get_data_type_ok() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Number(100.into()));
        let result = data.get_data_type(0);
        assert_eq!(result, Ok(GarnishDataType::Number));
    }

    #[test]
    fn get_data_type_error() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Number(100.into()));
        let result = data.get_data_type(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn get_number_ok() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Number(100.into()));
        let result = data.get_number(0);
        assert_eq!(result, Ok(100.into()));
    }

    #[test]
    fn get_number_invalid_index() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Number(100.into()));
        let result = data.get_number(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn get_number_not_number() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Symbol(100));
        let result = data.get_number(0);
        assert_eq!(
            result,
            Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Number)))
        );
    }

    #[test]
    fn get_type_ok() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Type(GarnishDataType::Number));
        let result = data.get_type(0);
        assert_eq!(result, Ok(GarnishDataType::Number));
    }

    #[test]
    fn get_type_error() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Type(GarnishDataType::Number));
        let result = data.get_type(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn get_type_not_type() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Symbol(100));
        let result = data.get_type(0);
        assert_eq!(result, Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Type))));
    }

    #[test]
    fn get_char_ok() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Char('a'));
        let result = data.get_char(0);
        assert_eq!(result, Ok('a'));
    }

    #[test]
    fn get_char_not_char() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Number(100.into()));
        let result = data.get_char(0);
        assert_eq!(result, Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Char))));
    }

    #[test]
    fn get_char_invalid_index() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Char('a'));
        let result = data.get_char(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn get_byte_ok() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Byte(100));
        let result = data.get_byte(0);
        assert_eq!(result, Ok(100));
    }

    #[test]
    fn get_byte_not_byte() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Number(100.into()));
        let result = data.get_byte(0);
        assert_eq!(result, Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Byte))));
    }

    #[test]
    fn get_byte_invalid_index() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Byte(100));
        let result = data.get_byte(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn get_symbol_ok() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Symbol(100));
        let result = data.get_symbol(0);
        assert_eq!(result, Ok(100));
    }

    #[test]
    fn get_symbol_not_symbol() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Number(100.into()));
        let result = data.get_symbol(0);
        assert_eq!(
            result,
            Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Symbol)))
        );
    }

    #[test]
    fn get_symbol_invalid_index() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Symbol(100));
        let result = data.get_symbol(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn get_expression_ok() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Expression(100));
        let result = data.get_expression(0);
        assert_eq!(result, Ok(100));
    }

    #[test]
    fn get_expression_not_expression() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Number(100.into()));
        let result = data.get_expression(0);
        assert_eq!(
            result,
            Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Expression)))
        );
    }

    #[test]
    fn get_expression_invalid_index() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Expression(100));
        let result = data.get_expression(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn get_external_ok() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::External(100));
        let result = data.get_external(0);
        assert_eq!(result, Ok(100));
    }

    #[test]
    fn get_external_not_external() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Number(100.into()));
        let result = data.get_external(0);
        assert_eq!(
            result,
            Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::External)))
        );
    }

    #[test]
    fn get_external_invalid_index() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::External(100));
        let result = data.get_external(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn get_pair_ok() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Pair(100, 200));
        let result = data.get_pair(0);
        assert_eq!(result, Ok((100, 200)));
    }

    #[test]
    fn get_pair_not_pair() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Number(100.into()));
        let result = data.get_pair(0);
        assert_eq!(result, Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Pair))));
    }

    #[test]
    fn get_pair_invalid_index() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Pair(100, 200));
        let result = data.get_pair(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn get_partial_ok() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Partial(100, 200));
        let result = data.get_partial(0);
        assert_eq!(result, Ok((100, 200)));
    }

    #[test]
    fn get_partial_not_partial() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Number(100.into()));
        let result = data.get_partial(0);
        assert_eq!(
            result,
            Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Partial)))
        );
    }

    #[test]
    fn get_partial_invalid_index() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Partial(100, 200));
        let result = data.get_partial(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn get_concatenation_ok() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Concatenation(100, 200));
        let result = data.get_concatenation(0);
        assert_eq!(result, Ok((100, 200)));
    }

    #[test]
    fn get_concatenation_not_concatenation() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Number(100.into()));
        let result = data.get_concatenation(0);
        assert_eq!(
            result,
            Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Concatenation)))
        );
    }

    #[test]
    fn get_concatenation_invalid_index() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Concatenation(100, 200));
        let result = data.get_concatenation(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn get_range_ok() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Range(100, 200));
        let result = data.get_range(0);
        assert_eq!(result, Ok((100, 200)));
    }

    #[test]
    fn get_range_not_range() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Number(100.into()));
        let result = data.get_range(0);
        assert_eq!(result, Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Range))));
    }

    #[test]
    fn get_range_invalid_index() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Range(100, 200));
        let result = data.get_range(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn get_slice_ok() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Slice(100, 200));
        let result = data.get_slice(0);
        assert_eq!(result, Ok((100, 200)));
    }

    #[test]
    fn get_slice_not_slice() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Number(100.into()));
        let result = data.get_slice(0);
        assert_eq!(result, Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Slice))));
    }

    #[test]
    fn get_slice_invalid_index() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Slice(100, 200));
        let result = data.get_slice(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn create_list() {
        let mut data = BasicGarnishDataUnit::new();
        let v1 = data.push_basic_data(BasicData::Number(100.into()));
        let v2 = data.push_basic_data(BasicData::Number(200.into()));
        let v3 = data.push_basic_data(BasicData::Number(300.into()));
        data.start_list(3).unwrap();
        data.add_to_list(v1, false).unwrap();
        data.add_to_list(v2, false).unwrap();
        data.add_to_list(v3, false).unwrap();
        let list = data.end_list().unwrap();

        assert_eq!(list, 3);
        assert_eq!(data, BasicGarnishDataUnit::new_full(vec![
            BasicData::Number(100.into()),
            BasicData::Number(200.into()),
            BasicData::Number(300.into()),
            BasicData::List(3),
            BasicData::ListItem(v1),
            BasicData::ListItem(v2),
            BasicData::ListItem(v3),
        ]));
    }

    #[test]
    fn get_list_len_ok() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::List(100));
        let result = data.get_list_len(0);
        assert_eq!(result, Ok(100));
    }

    #[test]
    fn get_list_len_invalid_index() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::List(100));
        let result = data.get_list_len(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn get_list_len_not_list() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Number(100.into()));
        let result = data.get_list_len(0);
        assert_eq!(result, Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::List))));
    }
}
