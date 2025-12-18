use std::cmp::Ordering;

use crate::{
    BasicData, BasicDataCustom, ByteListIterator, CharListIterator, DataError, DataIndexIterator, NumberIterator, SimpleNumber, SizeIterator, SymbolListPartIterator, basic::{BasicGarnishData, BasicNumber, merge_to_symbol_list::merge_to_symbol_list}, error::DataErrorType, symbol_value
};
use garnish_lang_traits::{Extents, GarnishData, GarnishDataType, SymbolListPart};

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
        self.data_block.cursor
    }

    fn get_data_iter(&self) -> Self::DataIndexIterator {
        SizeIterator::new(0, self.data_block.cursor)
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
        self.get_from_data_block_ensure_index(addr).map(|data| data.get_data_type())
    }

    fn get_number(&self, addr: Self::Size) -> Result<Self::Number, Self::Error> {
        self.get_from_data_block_ensure_index(addr)?.as_number()
    }

    fn get_type(&self, addr: Self::Size) -> Result<GarnishDataType, Self::Error> {
        self.get_from_data_block_ensure_index(addr)?.as_type()
    }

    fn get_char(&self, addr: Self::Size) -> Result<Self::Char, Self::Error> {
        self.get_from_data_block_ensure_index(addr)?.as_char()
    }

    fn get_byte(&self, addr: Self::Size) -> Result<Self::Byte, Self::Error> {
        self.get_from_data_block_ensure_index(addr)?.as_byte()
    }

    fn get_symbol(&self, addr: Self::Size) -> Result<Self::Symbol, Self::Error> {
        self.get_from_data_block_ensure_index(addr)?.as_symbol()
    }

    fn get_expression(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        self.get_from_data_block_ensure_index(addr)?.as_expression()
    }

    fn get_external(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        self.get_from_data_block_ensure_index(addr)?.as_external()
    }

    fn get_pair(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
        self.get_from_data_block_ensure_index(addr)?.as_pair()
    }

    fn get_concatenation(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
        self.get_from_data_block_ensure_index(addr)?.as_concatenation()
    }

    fn get_range(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
        self.get_from_data_block_ensure_index(addr)?.as_range()
    }

    fn get_slice(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
        self.get_from_data_block_ensure_index(addr)?.as_slice()
    }

    fn get_partial(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
        self.get_from_data_block_ensure_index(addr)?.as_partial()
    }

    fn get_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        Ok(self.get_from_data_block_ensure_index(addr)?.as_list()?.0)
    }

    fn get_list_item(&self, list_addr: Self::Size, item_index: Self::Number) -> Result<Option<Self::Size>, Self::Error> {
        let (len, _) = self.get_from_data_block_ensure_index(list_addr)?.as_list()?;

        let index: usize = item_index.into();

        if index >= len {
            return Err(DataError::new("Invalid list item index", DataErrorType::InvalidListItemIndex(index, len)));
        }
        
        Ok(Some(self.get_from_data_block_ensure_index(list_addr + 1 + index)?.as_list_item()?))
    }

    fn get_list_associations_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        todo!()
    }

    fn get_list_association(&self, list_addr: Self::Size, item_addr: Self::Number) -> Result<Option<Self::Size>, Self::Error> {
        todo!()
    }

    fn get_list_item_with_symbol(&self, list_index: Self::Size, sym: Self::Symbol) -> Result<Option<Self::Size>, Self::Error> {
        let (len, associations_len) = self.get_from_data_block_ensure_index(list_index)?.as_list()?;

        let association_start= list_index + len + 1;
        let association_range = association_start..association_start + associations_len;
        let association_slice = &self.data[association_range.clone()];

        let mut size = association_slice.len();
        let mut base = 0usize;

        while size > 1 {
            let half = size / 2;
            let mid = base + half;

            match &association_slice[mid] {
                BasicData::AssociativeItem(sym1, _) => match sym1.cmp(&sym) {
                    Ordering::Equal => return Ok(Some(association_slice[mid].as_associative_item()?.1)),
                    Ordering::Less => {}
                    Ordering::Greater => base = mid,
                }
                _ => todo!()
            }

            size -= half;
        }
        
        Ok(None)
    }

    fn get_list_items_iter(&self, list_addr: Self::Size, extents: Extents<Self::Number>) -> Result<Self::ListIndexIterator, Self::Error> {
        todo!()
    }

    fn get_list_associations_iter(&self, list_addr: Self::Size, extents: Extents<Self::Number>) -> Result<Self::ListIndexIterator, Self::Error> {
        todo!()
    }

    fn get_char_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        todo!()
    }

    fn get_char_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Option<Self::Char>, Self::Error> {
        todo!()
    }

    fn get_char_list_iter(&self, list_addr: Self::Size, extents: Extents<Self::Number>) -> Result<Self::CharIterator, Self::Error> {
        todo!()
    }

    fn get_byte_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        todo!()
    }

    fn get_byte_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Option<Self::Byte>, Self::Error> {
        todo!()
    }

    fn get_byte_list_iter(&self, list_addr: Self::Size, extents: Extents<Self::Number>) -> Result<Self::ByteIterator, Self::Error> {
        todo!()
    }

    fn get_symbol_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        todo!()
    }

    fn get_symbol_list_item(
        &self,
        addr: Self::Size,
        item_index: Self::Number,
    ) -> Result<Option<SymbolListPart<Self::Symbol, Self::Number>>, Self::Error> {
        todo!()
    }

    fn get_symbol_list_iter(&self, list_addr: Self::Size, extents: Extents<Self::Number>) -> Result<Self::SymbolListPartIterator, Self::Error> {
        todo!()
    }

    fn get_list_item_iter(&self, list_addr: Self::Size, extents: Extents<Self::Number>) -> Result<Self::ListItemIterator, Self::Error> {
        todo!()
    }

    fn get_concatenation_iter(&self, addr: Self::Size, extents: Extents<Self::Number>) -> Result<Self::ConcatenationItemIterator, Self::Error> {
        todo!()
    }

    fn add_unit(&mut self) -> Result<Self::Size, Self::Error> {
        self.push_to_data_block(BasicData::Unit)
    }

    fn add_true(&mut self) -> Result<Self::Size, Self::Error> {
        self.push_to_data_block(BasicData::True)
    }

    fn add_false(&mut self) -> Result<Self::Size, Self::Error> {
        self.push_to_data_block(BasicData::False)
    }

    fn add_number(&mut self, value: Self::Number) -> Result<Self::Size, Self::Error> {
        self.push_to_data_block(BasicData::Number(value))
    }

    fn add_type(&mut self, value: garnish_lang_traits::GarnishDataType) -> Result<Self::Size, Self::Error> {
        self.push_to_data_block(BasicData::Type(value))
    }

    fn add_char(&mut self, value: Self::Char) -> Result<Self::Size, Self::Error> {
        self.push_to_data_block(BasicData::Char(value))
    }

    fn add_byte(&mut self, value: Self::Byte) -> Result<Self::Size, Self::Error> {
        self.push_to_data_block(BasicData::Byte(value))
    }

    fn add_symbol(&mut self, value: Self::Symbol) -> Result<Self::Size, Self::Error> {
        self.push_to_data_block(BasicData::Symbol(value))
    }

    fn add_expression(&mut self, value: Self::Size) -> Result<Self::Size, Self::Error> {
        self.push_to_data_block(BasicData::Expression(value))
    }

    fn add_external(&mut self, value: Self::Size) -> Result<Self::Size, Self::Error> {
        self.push_to_data_block(BasicData::External(value))
    }

    fn add_pair(&mut self, value: (Self::Size, Self::Size)) -> Result<Self::Size, Self::Error> {
        self.push_to_data_block(BasicData::Pair(value.0, value.1))
    }

    fn add_concatenation(&mut self, left: Self::Size, right: Self::Size) -> Result<Self::Size, Self::Error> {
        self.push_to_data_block(BasicData::Concatenation(left, right))
    }

    fn add_range(&mut self, start: Self::Size, end: Self::Size) -> Result<Self::Size, Self::Error> {
        self.push_to_data_block(BasicData::Range(start, end))
    }

    fn add_slice(&mut self, list: Self::Size, range: Self::Size) -> Result<Self::Size, Self::Error> {
        self.push_to_data_block(BasicData::Slice(list, range))
    }

    fn add_partial(&mut self, reciever: Self::Size, input: Self::Size) -> Result<Self::Size, Self::Error> {
        self.push_to_data_block(BasicData::Partial(reciever, input))
    }

    fn merge_to_symbol_list(&mut self, first: Self::Size, second: Self::Size) -> Result<Self::Size, Self::Error> {
        merge_to_symbol_list(self, first, second)
    }

    fn start_list(&mut self, len: Self::Size) -> Result<Self::Size, Self::Error> {
        let allocation_size = len*2;
        let list_index = self.push_to_data_block(BasicData::UninitializedList(len, 0))?;
        for _ in 0..allocation_size {
            self.push_to_data_block(BasicData::Empty)?;
        }
        Ok(list_index)
    }

    fn add_to_list(&mut self, list_index: Self::Size, item_index: Self::Size) -> Result<Self::Size, Self::Error> {
        let (len, count) = self.get_from_data_block_ensure_index_mut(list_index)?.as_uninitialized_list_mut()?;
        if count >= len {
            return Err(DataError::new(
                "Exceeded initial list length",
                DataErrorType::ExceededInitialListLength(len.clone()),
            ));
        }

        let len = len.clone();
        let current_index = list_index + 1 + *count;
        *count += 1;

        let item = self.get_from_data_block_ensure_index_mut(current_index)?;
        *item = BasicData::ListItem(item_index);

        if let BasicData::Pair(left, right) = self.get_from_data_block_ensure_index(item_index)? {
            if let BasicData::Symbol(sym) = self.get_from_data_block_ensure_index(*left)? {
                let paired_index = current_index + len;
                let association_item = BasicData::<T>::AssociativeItem(sym.clone(), right.clone());
                let item = self.get_from_data_block_ensure_index_mut(paired_index)?;
                *item = association_item;
            }
        }

        Ok(list_index)
    }

    fn end_list(&mut self, list_index: Self::Size) -> Result<Self::Size, Self::Error> {
        let (len, count) = self.get_from_data_block_ensure_index_mut(list_index)?.as_uninitialized_list_mut()?;
        if count < len {
            return Err(DataError::new(
                "List not fully initialized",
                DataErrorType::NotFullyInitializedList(len.clone(), count.clone()),
            ));
        }

        let len = len.clone();
        let start = list_index + 1 + len;
        let associations_end = start + len;
        let associations_range = start..associations_end;
        let associations_slice = &mut self.data[associations_range];
        let mut associations_count = 0;
        
        for item in associations_slice.iter() {
            match item {
                BasicData::Empty => {}
                _ => associations_count += 1,
            }
        }
        
        associations_slice.sort_by(|a, b| match (a, b) {
            (BasicData::AssociativeItem(sym1, _), BasicData::AssociativeItem(sym2, _)) => sym1.cmp(sym2),
            (BasicData::AssociativeItem(_, _), _) => Ordering::Less,
            (_, BasicData::AssociativeItem(_, _)) => Ordering::Greater,
            _ => Ordering::Equal,
        });

        let list_item = BasicData::List(len, associations_count);

        let item = self.get_from_data_block_ensure_index_mut(list_index)?;
        *item = list_item;

        Ok(list_index)
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
        Ok(symbol_value(from))
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
    use crate::{
        BasicData, BasicGarnishDataUnit,
        basic::{object::BasicObject, storage::StorageSettings, utilities::test_data},
        error::DataErrorType,
    };

    use super::*;

    #[test]
    fn get_data_len() {
        let data = test_data();
        assert_eq!(data.get_data_len(), 0);
    }

    #[test]
    fn get_data_len_with_items() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Unit).unwrap();
        data.push_to_data_block(BasicData::True).unwrap();
        assert_eq!(data.get_data_len(), 2);
    }

    #[test]
    fn get_data_iter() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Unit).unwrap();
        data.push_to_data_block(BasicData::True).unwrap();

        let mut iter = data.get_data_iter();
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn add_unit() {
        let mut data = test_data();
        data.add_unit().unwrap();

        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::Unit;
        expected_data.data_block.cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_true() {
        let mut data = test_data();
        data.add_true().unwrap();

        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::True;
        expected_data.data_block.cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_false() {
        let mut data = test_data();
        data.add_false().unwrap();

        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::False;
        expected_data.data_block.cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_number() {
        let mut data = test_data();
        data.add_number(100.into()).unwrap();

        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::Number(100.into());
        expected_data.data_block.cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_type() {
        let mut data = test_data();
        data.add_type(GarnishDataType::Number).unwrap();

        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::Type(GarnishDataType::Number);
        expected_data.data_block.cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_char() {
        let mut data = test_data();
        data.add_char('a').unwrap();

        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::Char('a');
        expected_data.data_block.cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_byte() {
        let mut data = test_data();
        data.add_byte(100).unwrap();

        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::Byte(100);
        expected_data.data_block.cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_symbol() {
        let mut data = test_data();
        data.add_symbol(100).unwrap();

        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::Symbol(100);
        expected_data.data_block.cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_expression() {
        let mut data = test_data();
        data.add_expression(100).unwrap();

        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::Expression(100);
        expected_data.data_block.cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_external() {
        let mut data = test_data();
        data.add_external(100).unwrap();

        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::External(100);
        expected_data.data_block.cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_pair() {
        let mut data = test_data();
        data.add_pair((100, 200)).unwrap();

        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::Pair(100, 200);
        expected_data.data_block.cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_concatenation() {
        let mut data = test_data();
        data.add_concatenation(100, 200).unwrap();

        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::Concatenation(100, 200);
        expected_data.data_block.cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_range() {
        let mut data = test_data();
        data.add_range(100, 200).unwrap();

        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::Range(100, 200);
        expected_data.data_block.cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_slice() {
        let mut data = test_data();
        data.add_slice(100, 200).unwrap();

        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::Slice(100, 200);
        expected_data.data_block.cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_partial() {
        let mut data = test_data();
        data.add_partial(100, 200).unwrap();

        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::Partial(100, 200);
        expected_data.data_block.cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn get_data_type_ok() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let result = data.get_data_type(0);
        assert_eq!(result, Ok(GarnishDataType::Number));
    }

    #[test]
    fn get_data_type_error() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let result = data.get_data_type(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn get_number_ok() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let result = data.get_number(0);
        assert_eq!(result, Ok(100.into()));
    }

    #[test]
    fn get_number_invalid_index() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let result = data.get_number(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn get_number_not_number() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Symbol(100)).unwrap();
        let result = data.get_number(0);
        assert_eq!(
            result,
            Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Number)))
        );
    }

    #[test]
    fn get_type_ok() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Type(GarnishDataType::Number)).unwrap();
        let result = data.get_type(0);
        assert_eq!(result, Ok(GarnishDataType::Number));
    }

    #[test]
    fn get_type_error() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Type(GarnishDataType::Number)).unwrap();
        let result = data.get_type(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn get_type_not_type() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Symbol(100)).unwrap();
        let result = data.get_type(0);
        assert_eq!(result, Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Type))));
    }

    #[test]
    fn get_char_ok() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Char('a')).unwrap();
        let result = data.get_char(0);
        assert_eq!(result, Ok('a'));
    }

    #[test]
    fn get_char_not_char() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let result = data.get_char(0);
        assert_eq!(result, Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Char))));
    }

    #[test]
    fn get_char_invalid_index() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Char('a')).unwrap();
        let result = data.get_char(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn get_byte_ok() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Byte(100)).unwrap();
        let result = data.get_byte(0);
        assert_eq!(result, Ok(100));
    }

    #[test]
    fn get_byte_not_byte() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let result = data.get_byte(0);
        assert_eq!(result, Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Byte))));
    }

    #[test]
    fn get_byte_invalid_index() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Byte(100)).unwrap();
        let result = data.get_byte(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn get_symbol_ok() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Symbol(100)).unwrap();
        let result = data.get_symbol(0);
        assert_eq!(result, Ok(100));
    }

    #[test]
    fn get_symbol_not_symbol() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let result = data.get_symbol(0);
        assert_eq!(
            result,
            Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Symbol)))
        );
    }

    #[test]
    fn get_symbol_invalid_index() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Symbol(100)).unwrap();
        let result = data.get_symbol(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn get_expression_ok() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Expression(100)).unwrap();
        let result = data.get_expression(0);
        assert_eq!(result, Ok(100));
    }

    #[test]
    fn get_expression_not_expression() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let result = data.get_expression(0);
        assert_eq!(
            result,
            Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Expression)))
        );
    }

    #[test]
    fn get_expression_invalid_index() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Expression(100)).unwrap();
        let result = data.get_expression(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn get_external_ok() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::External(100)).unwrap();
        let result = data.get_external(0);
        assert_eq!(result, Ok(100));
    }

    #[test]
    fn get_external_not_external() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let result = data.get_external(0);
        assert_eq!(
            result,
            Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::External)))
        );
    }

    #[test]
    fn get_external_invalid_index() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::External(100)).unwrap();
        let result = data.get_external(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn get_pair_ok() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Pair(100, 200)).unwrap();
        let result = data.get_pair(0);
        assert_eq!(result, Ok((100, 200)));
    }

    #[test]
    fn get_pair_not_pair() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let result = data.get_pair(0);
        assert_eq!(result, Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Pair))));
    }

    #[test]
    fn get_pair_invalid_index() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Pair(100, 200)).unwrap();
        let result = data.get_pair(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn get_partial_ok() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Partial(100, 200)).unwrap();
        let result = data.get_partial(0);
        assert_eq!(result, Ok((100, 200)));
    }

    #[test]
    fn get_partial_not_partial() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let result = data.get_partial(0);
        assert_eq!(
            result,
            Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Partial)))
        );
    }

    #[test]
    fn get_partial_invalid_index() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Partial(100, 200)).unwrap();
        let result = data.get_partial(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn get_concatenation_ok() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Concatenation(100, 200)).unwrap();
        let result = data.get_concatenation(0);
        assert_eq!(result, Ok((100, 200)));
    }

    #[test]
    fn get_concatenation_not_concatenation() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let result = data.get_concatenation(0);
        assert_eq!(
            result,
            Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Concatenation)))
        );
    }

    #[test]
    fn get_concatenation_invalid_index() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Concatenation(100, 200)).unwrap();
        let result = data.get_concatenation(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn get_range_ok() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Range(100, 200)).unwrap();
        let result = data.get_range(0);
        assert_eq!(result, Ok((100, 200)));
    }

    #[test]
    fn get_range_not_range() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let result = data.get_range(0);
        assert_eq!(result, Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Range))));
    }

    #[test]
    fn get_range_invalid_index() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Range(100, 200)).unwrap();
        let result = data.get_range(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn get_slice_ok() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Slice(100, 200)).unwrap();
        let result = data.get_slice(0);
        assert_eq!(result, Ok((100, 200)));
    }

    #[test]
    fn get_slice_not_slice() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let result = data.get_slice(0);
        assert_eq!(result, Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Slice))));
    }

    #[test]
    fn get_slice_invalid_index() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Slice(100, 200)).unwrap();
        let result = data.get_slice(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn start_list_ok() {
        let mut data = test_data();
        let list_index = data.start_list(3).unwrap();

        assert_eq!(list_index, 0);
        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::UninitializedList(3, 0);
        expected_data.data_block.cursor = 7;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_list_list_ok() {
        let mut data = test_data();
        let v1 = data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let list_index = data.start_list(3).unwrap();
        let list_index = data.add_to_list(list_index, v1).unwrap();

        assert_eq!(list_index, 1);
        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::Number(100.into());
        expected_data.data[1] = BasicData::UninitializedList(3, 1);
        expected_data.data[2] = BasicData::ListItem(v1);
        expected_data.data_block.cursor = 8;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_list_list_invalid_list_index() {
        let mut data = test_data();
        let v1 = data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        data.start_list(3).unwrap();
        let result = data.add_to_list(10, v1);

        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(10))));
    }

    #[test]
    fn add_list_item_past_initial_length() {
        let mut data = test_data();
        let v1 = data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let list_index = data.start_list(3).unwrap();
        let list_index = data.add_to_list(list_index, v1).unwrap();
        let list_index = data.add_to_list(list_index, v1).unwrap();
        let list_index = data.add_to_list(list_index, v1).unwrap();
        let result = data.add_to_list(list_index, v1);

        assert_eq!(
            result,
            Err(DataError::new(
                "Exceeded initial list length",
                DataErrorType::ExceededInitialListLength(3)
            ))
        );
    }

    #[test]
    fn add_list_with_non_list() {
        let mut data = test_data();
        let v1 = data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        data.start_list(3).unwrap();
        let result = data.add_to_list(v1, v1);

        assert_eq!(result, Err(DataError::not_basic_type_error()));
    }

    #[test]
    fn create_list() {
        let mut data = test_data();
        let v1 = data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let v2 = data.push_to_data_block(BasicData::Number(200.into())).unwrap();
        let v3 = data.push_to_data_block(BasicData::Number(300.into())).unwrap();
        let mut list_index = data.start_list(3).unwrap();
        list_index = data.add_to_list(list_index, v1).unwrap();
        list_index = data.add_to_list(list_index, v2).unwrap();
        list_index = data.add_to_list(list_index, v3).unwrap();
        let list_index = data.end_list(list_index).unwrap();

        assert_eq!(list_index, 3);
        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::Number(100.into());
        expected_data.data[1] = BasicData::Number(200.into());
        expected_data.data[2] = BasicData::Number(300.into());
        expected_data.data[3] = BasicData::List(3, 0);
        expected_data.data[4] = BasicData::ListItem(v1);
        expected_data.data[5] = BasicData::ListItem(v2);
        expected_data.data[6] = BasicData::ListItem(v3);
        expected_data.data_block.cursor = 10;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn create_list_with_associations() {
        let mut data = test_data();
        let v1 = data.push_object_to_data_block(BasicObject::Pair(
            Box::new(BasicObject::Symbol(20)),
            Box::new(BasicObject::Number(100.into())),
        )).unwrap();
        let v2 = data.push_object_to_data_block(BasicObject::Pair(
            Box::new(BasicObject::Symbol(30)),
            Box::new(BasicObject::Number(200.into())),
        )).unwrap();
        let v3 = data.push_object_to_data_block(BasicObject::Pair(
            Box::new(BasicObject::Symbol(10)),
            Box::new(BasicObject::Number(300.into())),
        )).unwrap();

        let mut list_index = data.start_list(3).unwrap();
        list_index = data.add_to_list(list_index, v1).unwrap();
        list_index = data.add_to_list(list_index, v2).unwrap();
        list_index = data.add_to_list(list_index, v3).unwrap();
        let list_index = data.end_list(list_index).unwrap();

        let mut expected_data = test_data();
        expected_data.data.resize(20, BasicData::Empty);

        expected_data.data[0] = BasicData::Symbol(20);
        expected_data.data[1] = BasicData::Number(100.into());
        expected_data.data[2] = BasicData::Pair(0, 1);
        expected_data.data[3] = BasicData::Symbol(30);
        expected_data.data[4] = BasicData::Number(200.into());
        expected_data.data[5] = BasicData::Pair(3, 4);
        expected_data.data[6] = BasicData::Symbol(10);
        expected_data.data[7] = BasicData::Number(300.into());
        expected_data.data[8] = BasicData::Pair(6, 7);
        expected_data.data[9] = BasicData::List(3, 3);
        expected_data.data[10] = BasicData::ListItem(2);
        expected_data.data[11] = BasicData::ListItem(5);
        expected_data.data[12] = BasicData::ListItem(8);
        expected_data.data[13] = BasicData::AssociativeItem(10, 7);
        expected_data.data[14] = BasicData::AssociativeItem(20, 1);
        expected_data.data[15] = BasicData::AssociativeItem(30, 4);
        expected_data.data_block.cursor = 16;
        expected_data.data_block.size = 20;

        assert_eq!(list_index, 9);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn create_list_with_some_associations() {
        let mut data = test_data();
        let v1 = data.push_object_to_data_block(BasicObject::Number(50.into())).unwrap();
        let v2 = data.push_object_to_data_block(BasicObject::Pair(
            Box::new(BasicObject::Symbol(30)),
            Box::new(BasicObject::Number(100.into())),
        )).unwrap();
        let v3 = data.push_object_to_data_block(BasicObject::Pair(
            Box::new(BasicObject::Symbol(20)),
            Box::new(BasicObject::Number(200.into())),
        )).unwrap();
        let v4 = data.push_object_to_data_block(BasicObject::Number(300.into())).unwrap();

        let mut list_index = data.start_list(4).unwrap();
        list_index = data.add_to_list(list_index, v1).unwrap();
        list_index = data.add_to_list(list_index, v2).unwrap();
        list_index = data.add_to_list(list_index, v3).unwrap();
        list_index = data.add_to_list(list_index, v4).unwrap();
        let list_index = data.end_list(list_index).unwrap();

        let mut expected_data = test_data();
        expected_data.data.resize(20, BasicData::Empty);

        expected_data.data[0] = BasicData::Number(50.into());
        expected_data.data[1] = BasicData::Symbol(30);
        expected_data.data[2] = BasicData::Number(100.into());
        expected_data.data[3] = BasicData::Pair(1, 2);
        expected_data.data[4] = BasicData::Symbol(20);
        expected_data.data[5] = BasicData::Number(200.into());
        expected_data.data[6] = BasicData::Pair(4, 5);
        expected_data.data[7] = BasicData::Number(300.into());
        expected_data.data[8] = BasicData::List(4, 2);
        expected_data.data[9] = BasicData::ListItem(0);
        expected_data.data[10] = BasicData::ListItem(3);
        expected_data.data[11] = BasicData::ListItem(6);
        expected_data.data[12] = BasicData::ListItem(7);
        expected_data.data[13] = BasicData::AssociativeItem(20, 5);
        expected_data.data[14] = BasicData::AssociativeItem(30, 2);
        expected_data.data_block.cursor = 17;
        expected_data.data_block.size = 20;

        assert_eq!(list_index, 8);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn get_list_len_ok() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::List(100, 0)).unwrap();
        let result = data.get_list_len(0);
        assert_eq!(result, Ok(100));
    }

    #[test]
    fn get_list_len_invalid_index() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::List(100, 0)).unwrap();
        let result = data.get_list_len(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }

    #[test]
    fn get_list_len_not_list() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let result = data.get_list_len(0);
        assert_eq!(result, Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::List))));
    }

    #[test]
    fn get_list_item() {
        let mut data = test_data();
        data.push_object_to_data_block(BasicObject::List(vec![
            Box::new(BasicObject::Number(100.into())),
            Box::new(BasicObject::Number(200.into())),
            Box::new(BasicObject::Number(300.into())),
        ])).unwrap();
        let result = data.get_list_item(3, 1.into()).unwrap();
        assert_eq!(result, Some(1));
    }

    #[test]
    fn get_list_item_with_float_index() {
        let mut data = test_data();
        data.push_object_to_data_block(BasicObject::List(vec![
            Box::new(BasicObject::Number(100.into())),
            Box::new(BasicObject::Number(200.into())),
            Box::new(BasicObject::Number(300.into())),
        ])).unwrap();
        let result = data.get_list_item(3, 1.5.into()).unwrap();
        assert_eq!(result, Some(1));
    }

    #[test]
    fn get_list_item_invalid_index() {
        let mut data = test_data();
        data.push_object_to_data_block(BasicObject::List(vec![
            Box::new(BasicObject::Number(100.into())),
            Box::new(BasicObject::Number(200.into())),
            Box::new(BasicObject::Number(300.into())),
        ])).unwrap();
        let result = data.get_list_item(3, 4.into());
        assert_eq!(result, Err(DataError::new("Invalid list item index", DataErrorType::InvalidListItemIndex(4, 3))));
    }

    #[test]
    fn get_list_item_not_list() {
        let mut data = test_data();
        data.push_object_to_data_block(BasicObject::Number(100.into())).unwrap();
        let result = data.get_list_item(0, 1.into());
        assert_eq!(result, Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::List))));
    }

    #[test]
    fn get_list_item_with_symbol_ok() {
        let mut data = test_data();
        data.push_object_to_data_block(BasicObject::List(vec![
            Box::new(BasicObject::Pair(Box::new(BasicObject::Symbol(100)), Box::new(BasicObject::Number(200.into())))),
            Box::new(BasicObject::Pair(Box::new(BasicObject::Symbol(200)), Box::new(BasicObject::Number(300.into())))),
            Box::new(BasicObject::Pair(Box::new(BasicObject::Symbol(300)), Box::new(BasicObject::Number(400.into())))),
        ])).unwrap();
        let result = data.get_list_item_with_symbol(9, 200);
        assert_eq!(result, Ok(Some(4)));
    }

    #[test]
    fn get_list_item_with_symbol_invalid_symbol() {
        let mut data = test_data();
        data.push_object_to_data_block(BasicObject::List(vec![
            Box::new(BasicObject::Pair(Box::new(BasicObject::Symbol(100)), Box::new(BasicObject::Number(200.into())))),
            Box::new(BasicObject::Pair(Box::new(BasicObject::Symbol(200)), Box::new(BasicObject::Number(300.into())))),
            Box::new(BasicObject::Pair(Box::new(BasicObject::Symbol(300)), Box::new(BasicObject::Number(400.into())))),
        ])).unwrap();
        let result = data.get_list_item_with_symbol(9, 500);
        assert_eq!(result, Ok(None));
    }

    #[test]
    fn get_list_item_with_symbol_not_list() {
        let mut data = test_data();
        data.push_object_to_data_block(BasicObject::Number(100.into())).unwrap();
        let result = data.get_list_item_with_symbol(0, 100);
        assert_eq!(result, Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::List))));
    }

    #[test]
    fn get_list_item_with_symbol_invalid_index() {
        let mut data = test_data();
        data.push_object_to_data_block(BasicObject::List(vec![
            Box::new(BasicObject::Pair(Box::new(BasicObject::Symbol(100)), Box::new(BasicObject::Number(200.into())))),
            Box::new(BasicObject::Pair(Box::new(BasicObject::Symbol(200)), Box::new(BasicObject::Number(300.into())))),
            Box::new(BasicObject::Pair(Box::new(BasicObject::Symbol(300)), Box::new(BasicObject::Number(400.into())))),
        ])).unwrap();
        let result = data.get_list_item_with_symbol(100, 100);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(100))));
    }

    #[test]
    fn parse_symbol() {
        let result = BasicGarnishDataUnit::parse_symbol("my_symbol").unwrap();
        assert_eq!(result, 8904929874702161741);
    }
}
