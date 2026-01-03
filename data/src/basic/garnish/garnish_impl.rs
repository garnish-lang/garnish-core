use std::cmp::Ordering;

use crate::{
    BasicData, BasicDataCustom, ByteListIterator, CharListIterator, DataError, DataIndexIterator, NumberIterator, SizeIterator,
    SymbolListPartIterator,
    basic::{
        BasicGarnishData, BasicNumber,
        companion::BasicDataCompanion,
        garnish::{factory::BasicDataFactory, utils::extents_to_start_end},
        merge_to_symbol_list::merge_to_symbol_list,
        search::search_for_associative_item,
    },
    error::DataErrorType,
};
use garnish_lang_traits::{Extents, GarnishData, GarnishDataFactory, GarnishDataType, Instruction, SymbolListPart, TypeConstants};

impl<T, Companion> GarnishData for BasicGarnishData<T, Companion>
where
    T: BasicDataCustom,
    Companion: BasicDataCompanion<T>,
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
    type DataFactory = BasicDataFactory;

    fn get_data_len(&self) -> Self::Size {
        self.data_block().cursor
    }

    fn get_data_iter(&self) -> Self::DataIndexIterator {
        SizeIterator::new(0, self.data_block().cursor)
    }

    fn push_value_stack(&mut self, addr: Self::Size) -> Result<(), Self::Error> {
        let index = match self.current_value() {
            Some(previous) => self.push_to_data_block(BasicData::Value(previous, addr))?,
            None => self.push_to_data_block(BasicData::ValueRoot(addr))?,
        };
        self.set_current_value(Some(index));
        Ok(())
    }

    fn pop_value_stack(&mut self) -> Option<Self::Size> {
        match self.current_value() {
            None => return None,
            Some(index) => {
                let (previous_opt, value) = match self.get_from_data_block_ensure_index(index) {
                    Ok(BasicData::Value(previous, value)) => (Some(*previous), *value),
                    Ok(BasicData::ValueRoot(value)) => (None, *value),
                    _ => return None,
                };
                self.set_current_value(previous_opt);
                Some(value)
            }
        }
    }

    fn get_current_value(&self) -> Option<Self::Size> {
        match self.current_value() {
            None => None,
            Some(index) => {
                let data = match self.get_from_data_block_ensure_index(index) {
                    Ok(d) => d,
                    Err(_) => return None,
                };
                match data {
                    BasicData::Value(_previous, value) => Some(*value),
                    BasicData::ValueRoot(value) => Some(*value),
                    _ => None,
                }
            }
        }
    }

    fn get_current_value_mut(&mut self) -> Option<&mut Self::Size> {
        match self.current_value() {
            None => None,
            Some(index) => {
                let data = match self.get_from_data_block_ensure_index_mut(index) {
                    Ok(d) => d,
                    Err(_) => return None,
                };
                match data {
                    BasicData::Value(_previous, value) => Some(value),
                    BasicData::ValueRoot(value) => Some(value),
                    _ => None,
                }
            }
        }
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

    fn get_list_item_with_symbol(&self, list_index: Self::Size, sym: Self::Symbol) -> Result<Option<Self::Size>, Self::Error> {
        let (len, associations_len) = self.get_from_data_block_ensure_index(list_index)?.as_list()?;

        let association_start = self.data_block().start + list_index + len + 1;
        let association_range = association_start..association_start + associations_len;
        let association_slice = &self.data()[association_range.clone()];

        search_for_associative_item(association_slice, sym)
    }

    fn get_char_list_len(&self, list_index: Self::Size) -> Result<Self::Size, Self::Error> {
        Ok(self.get_from_data_block_ensure_index(list_index)?.as_char_list()?)
    }

    fn get_char_list_item(&self, list_index: Self::Size, item_index: Self::Number) -> Result<Option<Self::Char>, Self::Error> {
        let len = self.get_from_data_block_ensure_index(list_index)?.as_char_list()?;
        let index: usize = item_index.into();
        if index >= len {
            return Ok(None);
        }
        Ok(Some(self.get_from_data_block_ensure_index(list_index + 1 + index)?.as_char()?))
    }

    fn get_char_list_iter(&self, list_index: Self::Size, extents: Extents<Self::Number>) -> Result<Self::CharIterator, Self::Error> {
        let len = self.get_from_data_block_ensure_index(list_index)?.as_char_list()?;
        let (start, end) = extents_to_start_end(extents, self.data_block().start + list_index, len);

        Ok(CharListIterator::new(
            self.data()[start..end].iter().map(|c| c.as_char().unwrap()).collect(),
        ))
    }

    fn get_byte_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        Ok(self.get_from_data_block_ensure_index(addr)?.as_byte_list()?)
    }

    fn get_byte_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Option<Self::Byte>, Self::Error> {
        let len = self.get_from_data_block_ensure_index(addr)?.as_byte_list()?;
        let index: usize = item_index.into();
        if index >= len {
            return Ok(None);
        }
        Ok(Some(self.get_from_data_block_ensure_index(addr + 1 + index)?.as_byte()?))
    }

    fn get_byte_list_iter(&self, list_index: Self::Size, extents: Extents<Self::Number>) -> Result<Self::ByteIterator, Self::Error> {
        let len = self.get_from_data_block_ensure_index(list_index)?.as_byte_list()?;
        let (start, end) = extents_to_start_end(extents, self.data_block().start + list_index, len);

        Ok(ByteListIterator::new(
            self.data()[start..end].iter().map(|c| c.as_byte().unwrap()).collect(),
        ))
    }

    fn get_symbol_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        Ok(self.get_from_data_block_ensure_index(addr)?.as_symbol_list()?)
    }

    fn get_symbol_list_item(
        &self,
        addr: Self::Size,
        item_index: Self::Number,
    ) -> Result<Option<SymbolListPart<Self::Symbol, Self::Number>>, Self::Error> {
        let len = self.get_from_data_block_ensure_index(addr)?.as_symbol_list()?;
        let index: usize = item_index.into();
        if index >= len {
            return Ok(None);
        }
        Ok(Some(match self.get_from_data_block_ensure_index(addr + 1 + index)? {
            BasicData::Symbol(sym) => SymbolListPart::Symbol(sym.clone()),
            BasicData::Number(num) => SymbolListPart::Number(num.clone()),
            d => {
                return Err(DataError::new(
                    "Not a symbol list part",
                    DataErrorType::NotASymbolListPart(d.get_data_type()),
                ));
            }
        }))
    }

    fn get_symbol_list_iter(&self, list_index: Self::Size, extents: Extents<Self::Number>) -> Result<Self::SymbolListPartIterator, Self::Error> {
        let len = self.get_from_data_block_ensure_index(list_index)?.as_symbol_list()?;
        let start: usize = self.data_block().start + list_index + 1 + usize::from(extents.start()).min(len);
        let end: usize = self.data_block().start + list_index + 1 + (usize::from(extents.end())).min(len);

        Ok(SymbolListPartIterator::new(
            self.data()[start..end]
                .iter()
                .map(|c| match c {
                    BasicData::Symbol(sym) => Ok(SymbolListPart::Symbol(sym.clone())),
                    BasicData::Number(num) => Ok(SymbolListPart::Number(num.clone())),
                    d => {
                        return Err(DataError::new(
                            "Not a symbol list part",
                            DataErrorType::NotASymbolListPart(d.get_data_type()),
                        ));
                    }
                })
                .collect::<Result<Vec<SymbolListPart<u64, BasicNumber>>, DataError>>()?,
        ))
    }

    fn get_list_item_iter(&self, list_index: Self::Size, extents: Extents<Self::Number>) -> Result<Self::ListItemIterator, Self::Error> {
        let len = self.get_from_data_block_ensure_index(list_index)?.as_list()?.0;

        let (start, end) = extents_to_start_end(extents, self.data_block().start + list_index, len);
        let slice = &self.data()[start..end];
        let mut items = Vec::new();

        for item in slice.iter() {
            match item {
                BasicData::ListItem(index) => items.push(*index),
                _ => return Err(DataError::new("Not a list item", DataErrorType::NotAListItem(item.get_data_type()))),
            }
        }

        Ok(DataIndexIterator::new(items))
    }

    fn get_concatenation_iter(&self, index: Self::Size, extents: Extents<Self::Number>) -> Result<Self::ConcatenationItemIterator, Self::Error> {
        self.get_from_data_block_ensure_index(index)?.as_concatenation()?;

        let mut stack = vec![index];
        let mut items = vec![];

        while let Some(index) = stack.pop() {
            match self.get_from_data_block_ensure_index(index)? {
                BasicData::Concatenation(left, right) => {
                    stack.push(right.clone());
                    stack.push(left.clone());
                }
                BasicData::List(_len, _) => {
                    let list_iter = self.get_list_item_iter(index, Extents::new(0.into(), BasicNumber::max_value()))?;

                    for item in list_iter {
                        items.push(item);
                    }
                }
                _ => items.push(index),
            }
        }

        let len = items.len();
        let start: usize = usize::from(extents.start()).min(len);
        let end: usize = usize::from(extents.end()).min(len);

        Ok(DataIndexIterator::new(items[start..end].to_vec()))
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
        let allocation_size = len * 2;
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
        let start = self.data_block().start + list_index + 1 + len;
        let associations_end = start + len;
        let associations_range = start..associations_end;
        let associations_slice = &mut self.data_mut()[associations_range];
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

    fn get_register_len(&self) -> Self::Size {
        let mut count = 0;
        let mut current = self.current_register();
        while let Some(index) = current {
            match self.get_from_data_block_ensure_index(index) {
                Ok(BasicData::Register(previous, _value)) => {
                    current = Some(*previous);
                    count += 1;
                }
                Ok(BasicData::RegisterRoot(_value)) => {
                    current = None;
                    count += 1;
                }
                _ => break,
            }
        }

        count
    }

    fn push_register(&mut self, index: Self::Size) -> Result<(), Self::Error> {
        let register_index = match self.current_register() {
            Some(previous) => self.push_to_data_block(BasicData::Register(previous, index))?,
            None => self.push_to_data_block(BasicData::RegisterRoot(index))?,
        };
        self.set_current_register(Some(register_index));
        Ok(())
    }

    fn get_register(&self, index: Self::Size) -> Option<Self::Size> {
        let mut current = self.current_register();

        let mut list = Vec::new();
        while let Some(index) = current {
            match self.get_from_data_block_ensure_index(index) {
                Ok(BasicData::Register(previous, value)) => {
                    current = Some(*previous);
                    list.push(*value);
                }
                Ok(BasicData::RegisterRoot(value)) => {
                    current = None;
                    list.push(*value);
                }
                _ => break,
            }
        }

        list.reverse();

        match list.get(index).cloned() {
            Some(value) => Some(value.clone()),
            None => None,
        }
    }

    fn pop_register(&mut self) -> Result<Option<Self::Size>, Self::Error> {
        match self.current_register() {
            Some(index) => {
                let data = self.get_from_data_block_ensure_index(index)?;
                let (previous_opt, value) = match data {
                    BasicData::Register(previous, value) => (Some(*previous), *value),
                    BasicData::RegisterRoot(value) => (None, *value),
                    _ => return Err(DataError::not_basic_type_error()),
                };
                self.set_current_register(previous_opt);
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    fn get_instruction_len(&self) -> Self::Size {
        self.instruction_block().cursor
    }

    fn push_instruction(&mut self, instruction: Instruction, data: Option<Self::Size>) -> Result<Self::Size, Self::Error> {
        self.push_to_instruction_block(instruction, data)
    }

    fn get_instruction(&self, addr: Self::Size) -> Option<(Instruction, Option<Self::Size>)> {
        match self.get_from_instruction_block_ensure_index(addr) {
            Ok(instruction) => Some(instruction),
            Err(_) => None,
        }
    }

    fn get_instruction_iter(&self) -> Self::InstructionIterator {
        SizeIterator::new(0, self.instruction_block().cursor)
    }

    fn get_instruction_cursor(&self) -> Self::Size {
        self.instruction_pointer()
    }

    fn set_instruction_cursor(&mut self, addr: Self::Size) -> Result<(), Self::Error> {
        self.set_instruction_pointer(addr);
        Ok(())
    }

    fn get_jump_table_len(&self) -> Self::Size {
        self.jump_table_block().cursor
    }

    fn push_to_jump_table(&mut self, index: Self::Size) -> Result<(), Self::Error> {
        self.push_to_jump_table_block(index)?;
        Ok(())
    }

    fn get_from_jump_table(&self, index: Self::Size) -> Option<Self::Size> {
        match self.get_from_jump_table_block_ensure_index(index) {
            Ok(point) => Some(point),
            Err(_) => None,
        }
    }

    fn get_from_jump_table_mut(&mut self, index: Self::Size) -> Option<&mut Self::Size> {
        match self.get_from_jump_table_block_ensure_index_mut(index) {
            Ok(point) => Some(point),
            Err(_) => None,
        }
    }

    fn push_frame(&mut self, index: Self::Size) -> Result<(), Self::Error> {
        self.push_to_data_block(BasicData::JumpPoint(index))?;
        let frame_data = match (self.current_frame(), self.current_register()) {
            (Some(frame), Some(register)) => BasicData::Frame(frame, register),
            (Some(frame), None) => BasicData::FrameIndex(frame),
            (None, Some(register)) => BasicData::FrameRegister(register),
            (None, None) => BasicData::FrameRoot,
        };
        let frame_index = self.push_to_data_block(frame_data)?;
        self.set_current_frame(Some(frame_index));
        Ok(())
    }

    fn pop_frame(&mut self) -> Result<Option<Self::Size>, Self::Error> {
        Ok(match self.current_frame() {
            Some(index) => {
                let return_index = self.get_from_data_block_ensure_index(index - 1).and_then(|data| data.as_jump_point())?;
                let data = self.get_from_data_block_ensure_index(index)?;
                let (previous_opt, register_opt) = match data {
                    BasicData::Frame(previous, register) => (Some(*previous), Some(*register)),
                    BasicData::FrameIndex(previous) => (Some(*previous), None),
                    BasicData::FrameRegister(register) => (None, Some(*register)),
                    BasicData::FrameRoot => (None, None),
                    _ => return Err(DataError::not_basic_type_error()),
                };
                self.set_current_frame(previous_opt);
                self.set_current_register(register_opt);
                Some(return_index)
            }
            None => None,
        })
    }

    fn add_char_list_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
        self.convert_basic_data_at_to_char_list(from)
    }

    fn add_byte_list_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
        let bytes = self.convert_basic_data_at_to_bytes(from)?;
        self.push_to_data_block(BasicData::ByteList(bytes.len()))?;
        for byte in bytes {
            self.push_to_data_block(BasicData::Byte(byte))?;
        }
        Ok(from)
    }

    fn add_symbol_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
        let sym = self.convert_basic_data_at_to_symbol(from)?;
        self.push_to_data_block(BasicData::Symbol(sym))
    }

    fn add_number_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
        match self.convert_basic_data_at_to_number(from)? {
            Some(number) => self.add_number(number),
            None => self.add_unit(),
        }
    }

    fn parse_add_char_list(&mut self, from: &str) -> Result<Self::Size, Self::Error> {
        let chars = Self::DataFactory::parse_char_list(from)?;
        let len = chars.len();
        let list_index = self.push_to_data_block(BasicData::CharList(len))?;
        for c in chars {
            self.push_to_data_block(BasicData::Char(c))?;
        }
        Ok(list_index)
    }

    fn parse_add_byte_list(&mut self, from: &str) -> Result<Self::Size, Self::Error> {
        let bytes = Self::DataFactory::parse_byte_list(from)?;
        let len = bytes.len();
        let list_index = self.push_to_data_block(BasicData::ByteList(len))?;
        for b in bytes {
            self.push_to_data_block(BasicData::Byte(b))?;
        }
        Ok(list_index)
    }

    fn parse_add_symbol(&mut self, from: &str) -> Result<Self::Size, Self::Error> {
        let symbol = Self::DataFactory::parse_symbol(from)?;
        let symbol_index = self.push_to_data_block(BasicData::Symbol(symbol))?;
        let list_index = self.push_to_data_block(BasicData::CharList(from.len()))?;
        for c in from.chars() {
            self.push_to_data_block(BasicData::Char(c))?;
        }
        self.push_to_symbol_table_block(symbol, list_index)?;
        Ok(symbol_index)
    }

    fn apply(&mut self, external_value: Self::Size, input_addr: Self::Size) -> Result<bool, Self::Error> {
        Companion::apply(self, external_value, input_addr)
    }

    fn defer_op(
            &mut self,
            operation: Instruction,
            left: (GarnishDataType, Self::Size),
            right: (GarnishDataType, Self::Size),
        ) -> Result<bool, Self::Error> {
        Companion::defer_op(self, operation, left, right)
    }

    fn resolve(&mut self, symbol: Self::Symbol) -> Result<bool, Self::Error> {
        Companion::resolve(self, symbol)
    }
}

#[cfg(test)]
mod tests {
    use garnish_lang_traits::Instruction;

    use crate::{
        BasicData, BasicDataCustom, basic::{
            companion::BasicDataCompanion,
            storage::{ReallocationStrategy, StorageSettings}, utilities::{instruction_test_data, jump_table_test_data, test_data}
        }, basic_object, error::DataErrorType, DataError
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
        expected_data.data_mut()[0] = BasicData::Unit;
        expected_data.data_block_mut().cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_true() {
        let mut data = test_data();
        data.add_true().unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut()[0] = BasicData::True;
        expected_data.data_block_mut().cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_false() {
        let mut data = test_data();
        data.add_false().unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut()[0] = BasicData::False;
        expected_data.data_block_mut().cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_number() {
        let mut data = test_data();
        data.add_number(100.into()).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut()[0] = BasicData::Number(100.into());
        expected_data.data_block_mut().cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_type() {
        let mut data = test_data();
        data.add_type(GarnishDataType::Number).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut()[0] = BasicData::Type(GarnishDataType::Number);
        expected_data.data_block_mut().cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_char() {
        let mut data = test_data();
        data.add_char('a').unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut()[0] = BasicData::Char('a');
        expected_data.data_block_mut().cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_byte() {
        let mut data = test_data();
        data.add_byte(100).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut()[0] = BasicData::Byte(100);
        expected_data.data_block_mut().cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_symbol() {
        let mut data = test_data();
        data.add_symbol(100).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut()[0] = BasicData::Symbol(100);
        expected_data.data_block_mut().cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_expression() {
        let mut data = test_data();
        data.add_expression(100).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut()[0] = BasicData::Expression(100);
        expected_data.data_block_mut().cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_external() {
        let mut data = test_data();
        data.add_external(100).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut()[0] = BasicData::External(100);
        expected_data.data_block_mut().cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_pair() {
        let mut data = test_data();
        data.add_pair((100, 200)).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut()[0] = BasicData::Pair(100, 200);
        expected_data.data_block_mut().cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_concatenation() {
        let mut data = test_data();
        data.add_concatenation(100, 200).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut()[0] = BasicData::Concatenation(100, 200);
        expected_data.data_block_mut().cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_range() {
        let mut data = test_data();
        data.add_range(100, 200).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut()[0] = BasicData::Range(100, 200);
        expected_data.data_block_mut().cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_slice() {
        let mut data = test_data();
        data.add_slice(100, 200).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut()[0] = BasicData::Slice(100, 200);
        expected_data.data_block_mut().cursor = 1;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_partial() {
        let mut data = test_data();
        data.add_partial(100, 200).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut()[0] = BasicData::Partial(100, 200);
        expected_data.data_block_mut().cursor = 1;

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
            Err(DataError::new(
                "Not of type",
                DataErrorType::NotType(GarnishDataType::Number, GarnishDataType::Symbol)
            ))
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
        assert_eq!(
            result,
            Err(DataError::new(
                "Not of type",
                DataErrorType::NotType(GarnishDataType::Type, GarnishDataType::Symbol)
            ))
        );
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
        assert_eq!(
            result,
            Err(DataError::new(
                "Not of type",
                DataErrorType::NotType(GarnishDataType::Char, GarnishDataType::Number)
            ))
        );
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
        assert_eq!(
            result,
            Err(DataError::new(
                "Not of type",
                DataErrorType::NotType(GarnishDataType::Byte, GarnishDataType::Number)
            ))
        );
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
            Err(DataError::new(
                "Not of type",
                DataErrorType::NotType(GarnishDataType::Symbol, GarnishDataType::Number)
            ))
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
            Err(DataError::new(
                "Not of type",
                DataErrorType::NotType(GarnishDataType::Expression, GarnishDataType::Number)
            ))
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
            Err(DataError::new(
                "Not of type",
                DataErrorType::NotType(GarnishDataType::External, GarnishDataType::Number)
            ))
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
        assert_eq!(
            result,
            Err(DataError::new(
                "Not of type",
                DataErrorType::NotType(GarnishDataType::Pair, GarnishDataType::Number)
            ))
        );
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
            Err(DataError::new(
                "Not of type",
                DataErrorType::NotType(GarnishDataType::Partial, GarnishDataType::Number)
            ))
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
            Err(DataError::new(
                "Not of type",
                DataErrorType::NotType(GarnishDataType::Concatenation, GarnishDataType::Number)
            ))
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
        assert_eq!(
            result,
            Err(DataError::new(
                "Not of type",
                DataErrorType::NotType(GarnishDataType::Range, GarnishDataType::Number)
            ))
        );
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
        assert_eq!(
            result,
            Err(DataError::new(
                "Not of type",
                DataErrorType::NotType(GarnishDataType::Slice, GarnishDataType::Number)
            ))
        );
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
        expected_data.data_mut()[0] = BasicData::UninitializedList(3, 0);
        expected_data.data_block_mut().cursor = 7;

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
        expected_data.data_mut()[0] = BasicData::Number(100.into());
        expected_data.data_mut()[1] = BasicData::UninitializedList(3, 1);
        expected_data.data_mut()[2] = BasicData::ListItem(v1);
        expected_data.data_block_mut().cursor = 8;

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
        expected_data.data_mut()[0] = BasicData::Number(100.into());
        expected_data.data_mut()[1] = BasicData::Number(200.into());
        expected_data.data_mut()[2] = BasicData::Number(300.into());
        expected_data.data_mut()[3] = BasicData::List(3, 0);
        expected_data.data_mut()[4] = BasicData::ListItem(v1);
        expected_data.data_mut()[5] = BasicData::ListItem(v2);
        expected_data.data_mut()[6] = BasicData::ListItem(v3);
        expected_data.data_block_mut().cursor = 10;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn create_list_with_associations() {
        let mut data = test_data();
        let v1 = data
            .push_object_to_data_block(basic_object!((SymRaw 20) = (Number 100)))
            .unwrap();
        let v2 = data
            .push_object_to_data_block(basic_object!((SymRaw 30) = (Number 200)))
            .unwrap();
        let v3 = data
            .push_object_to_data_block(basic_object!((SymRaw 10) = (Number 300)))
            .unwrap();

        let mut list_index = data.start_list(3).unwrap();
        list_index = data.add_to_list(list_index, v1).unwrap();
        list_index = data.add_to_list(list_index, v2).unwrap();
        list_index = data.add_to_list(list_index, v3).unwrap();
        let list_index = data.end_list(list_index).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut().resize(20, BasicData::Empty);

        expected_data.data_mut()[0] = BasicData::Symbol(20);
        expected_data.data_mut()[1] = BasicData::Number(100.into());
        expected_data.data_mut()[2] = BasicData::Pair(0, 1);
        expected_data.data_mut()[3] = BasicData::Symbol(30);
        expected_data.data_mut()[4] = BasicData::Number(200.into());
        expected_data.data_mut()[5] = BasicData::Pair(3, 4);
        expected_data.data_mut()[6] = BasicData::Symbol(10);
        expected_data.data_mut()[7] = BasicData::Number(300.into());
        expected_data.data_mut()[8] = BasicData::Pair(6, 7);
        expected_data.data_mut()[9] = BasicData::List(3, 3);
        expected_data.data_mut()[10] = BasicData::ListItem(2);
        expected_data.data_mut()[11] = BasicData::ListItem(5);
        expected_data.data_mut()[12] = BasicData::ListItem(8);
        expected_data.data_mut()[13] = BasicData::AssociativeItem(10, 7);
        expected_data.data_mut()[14] = BasicData::AssociativeItem(20, 1);
        expected_data.data_mut()[15] = BasicData::AssociativeItem(30, 4);
        expected_data.data_block_mut().cursor = 16;
        expected_data.data_block_mut().size = 20;
        expected_data.custom_data_block_mut().start = 20;

        assert_eq!(list_index, 9);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn create_list_with_some_associations() {
        let mut data = test_data();
        let v1 = data.push_object_to_data_block(basic_object!(Number 50)).unwrap();
        let v2 = data
            .push_object_to_data_block(basic_object!((SymRaw 30) = (Number 100)))
            .unwrap();
        let v3 = data
            .push_object_to_data_block(basic_object!((SymRaw 20) = (Number 200)))
            .unwrap();
        let v4 = data.push_object_to_data_block(basic_object!(Number 300)).unwrap();

        let mut list_index = data.start_list(4).unwrap();
        list_index = data.add_to_list(list_index, v1).unwrap();
        list_index = data.add_to_list(list_index, v2).unwrap();
        list_index = data.add_to_list(list_index, v3).unwrap();
        list_index = data.add_to_list(list_index, v4).unwrap();
        let list_index = data.end_list(list_index).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut().resize(20, BasicData::Empty);

        expected_data.data_mut()[0] = BasicData::Number(50.into());
        expected_data.data_mut()[1] = BasicData::Symbol(30);
        expected_data.data_mut()[2] = BasicData::Number(100.into());
        expected_data.data_mut()[3] = BasicData::Pair(1, 2);
        expected_data.data_mut()[4] = BasicData::Symbol(20);
        expected_data.data_mut()[5] = BasicData::Number(200.into());
        expected_data.data_mut()[6] = BasicData::Pair(4, 5);
        expected_data.data_mut()[7] = BasicData::Number(300.into());
        expected_data.data_mut()[8] = BasicData::List(4, 2);
        expected_data.data_mut()[9] = BasicData::ListItem(0);
        expected_data.data_mut()[10] = BasicData::ListItem(3);
        expected_data.data_mut()[11] = BasicData::ListItem(6);
        expected_data.data_mut()[12] = BasicData::ListItem(7);
        expected_data.data_mut()[13] = BasicData::AssociativeItem(20, 5);
        expected_data.data_mut()[14] = BasicData::AssociativeItem(30, 2);
        expected_data.data_block_mut().cursor = 17;
        expected_data.data_block_mut().size = 20;
        expected_data.custom_data_block_mut().start = 20;

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
        assert_eq!(
            result,
            Err(DataError::new(
                "Not of type",
                DataErrorType::NotType(GarnishDataType::List, GarnishDataType::Number)
            ))
        );
    }

    #[test]
    fn get_list_item() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!((Number 100), (Number 200), (Number 300)))
        .unwrap();
        let result = data.get_list_item(3, 1.into()).unwrap();
        assert_eq!(result, Some(1));
    }

    #[test]
    fn get_list_item_with_float_index() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!((Number 100), (Number 200), (Number 300)))
        .unwrap();
        let result = data.get_list_item(3, 1.5.into()).unwrap();
        assert_eq!(result, Some(1));
    }

    #[test]
    fn get_list_item_invalid_index() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!((Number 100), (Number 200), (Number 300)))
        .unwrap();
        let result = data.get_list_item(3, 4.into());
        assert_eq!(
            result,
            Err(DataError::new("Invalid list item index", DataErrorType::InvalidListItemIndex(4, 3)))
        );
    }

    #[test]
    fn get_list_item_not_list() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        let result = data.get_list_item(0, 1.into());
        assert_eq!(
            result,
            Err(DataError::new(
                "Not of type",
                DataErrorType::NotType(GarnishDataType::List, GarnishDataType::Number)
            ))
        );
    }

    #[test]
    fn get_list_item_with_symbol_ok() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(
            ((SymRaw 100) = (Number 200)),
            ((SymRaw 200) = (Number 300)),
            ((SymRaw 300) = (Number 400))
        ))
        .unwrap();
        let result = data.get_list_item_with_symbol(9, 200);
        assert_eq!(result, Ok(Some(4)));
    }

    #[test]
    fn get_list_item_with_symbol_invalid_symbol() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(
            ((SymRaw 100) = (Number 200)),
            ((SymRaw 200) = (Number 300)),
            ((SymRaw 300) = (Number 400))
        ))
        .unwrap();
        let result = data.get_list_item_with_symbol(9, 500);
        assert_eq!(result, Ok(None));
    }

    #[test]
    fn get_list_item_with_symbol_not_list() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        let result = data.get_list_item_with_symbol(0, 100);
        assert_eq!(
            result,
            Err(DataError::new(
                "Not of type",
                DataErrorType::NotType(GarnishDataType::List, GarnishDataType::Number)
            ))
        );
    }

    #[test]
    fn get_list_item_with_symbol_invalid_index() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(
            ((SymRaw 100) = (Number 200)),
            ((SymRaw 200) = (Number 300)),
            ((SymRaw 300) = (Number 400))
        ))
        .unwrap();
        let result = data.get_list_item_with_symbol(100, 100);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(100))));
    }

    #[test]
    fn get_char_list_len_ok() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(CharList "abcde")).unwrap();
        let result = data.get_char_list_len(0);
        assert_eq!(result, Ok(5));
    }

    #[test]
    fn get_char_list_len_invalid_index() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(CharList "abcde")).unwrap();
        let result = data.get_char_list_len(100);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(100))));
    }

    #[test]
    fn get_char_list_len_not_char_list() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        let result = data.get_char_list_len(0);
        assert_eq!(
            result,
            Err(DataError::new(
                "Not of type",
                DataErrorType::NotType(GarnishDataType::CharList, GarnishDataType::Number)
            ))
        );
    }

    #[test]
    fn get_char_list_item_ok() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(CharList "abcde")).unwrap();
        let result = data.get_char_list_item(0, 1.into());
        assert_eq!(result, Ok(Some('b')));
    }

    #[test]
    fn get_char_list_item_ok_with_float() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(CharList "abcde")).unwrap();
        let result = data.get_char_list_item(0, 1.5.into());
        assert_eq!(result, Ok(Some('b')));
    }

    #[test]
    fn get_char_list_item_invalid_index() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(CharList "abcde")).unwrap();
        let result = data.get_char_list_item(100, 0.into());
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(100))));
    }

    #[test]
    fn get_char_list_item_invalid_item_index() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(CharList "abcde")).unwrap();
        let result = data.get_char_list_item(0, 100.into());
        assert_eq!(result, Ok(None));
    }

    #[test]
    fn get_char_list_item_not_char_list() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        let result = data.get_char_list_item(0, 1.into());
        assert_eq!(
            result,
            Err(DataError::new(
                "Not of type",
                DataErrorType::NotType(GarnishDataType::CharList, GarnishDataType::Number)
            ))
        );
    }

    #[test]
    fn get_char_list_iter_ok() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(CharList "abcde")).unwrap();
        let result: Vec<char> = data.get_char_list_iter(0, Extents::new(0.into(), 5.into())).unwrap().collect();
        assert_eq!(result, vec!['a', 'b', 'c', 'd', 'e']);
    }

    #[test]
    fn get_char_list_iter_empty_ok() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(CharList "")).unwrap();
        let result: Vec<char> = data.get_char_list_iter(0, Extents::new(0.into(), 4.into())).unwrap().collect();
        assert_eq!(result, vec![]);
    }

    #[test]
    fn get_char_list_iter_ok_with_extents() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(CharList "abcde")).unwrap();
        let result: Vec<char> = data.get_char_list_iter(0, Extents::new(1.into(), 3.into())).unwrap().collect();
        assert_eq!(result, vec!['b', 'c']);
    }

    #[test]
    fn get_char_list_iter_invalid_index() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(CharList "abcde")).unwrap();
        let result = data.get_char_list_iter(100, Extents::new(0.into(), 5.into())).err();
        assert_eq!(result, Some(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(100))));
    }

    #[test]
    fn get_char_list_iter_not_char_list() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        let result = data.get_char_list_iter(0, Extents::new(0.into(), 5.into())).err();
        assert_eq!(
            result,
            Some(DataError::new(
                "Not of type",
                DataErrorType::NotType(GarnishDataType::CharList, GarnishDataType::Number)
            ))
        );
    }

    #[test]
    fn get_char_list_iter_start_negative_clamps_to_char_list_start() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(CharList "abcde")).unwrap();
        let result: Vec<char> = data.get_char_list_iter(0, Extents::new((-5).into(), 5.into())).unwrap().collect();
        assert_eq!(result, vec!['a', 'b', 'c', 'd', 'e']);
    }

    #[test]
    fn get_char_list_iter_start_out_of_bounds_is_empty() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(CharList "abcde")).unwrap();
        let result: Vec<char> = data.get_char_list_iter(0, Extents::new(6.into(), 10.into())).unwrap().collect();
        assert_eq!(result, vec![]);
    }

    #[test]
    fn get_char_list_iter_end_out_of_bounds_clamps_to_char_list_length() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(CharList "abcde")).unwrap();
        let result: Vec<char> = data.get_char_list_iter(0, Extents::new(0.into(), 10.into())).unwrap().collect();
        assert_eq!(result, vec!['a', 'b', 'c', 'd', 'e']);
    }

    #[test]
    fn get_byte_list_len_ok() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(ByteList 1, 2, 3, 4, 5)).unwrap();
        let result = data.get_byte_list_len(0);
        assert_eq!(result, Ok(5));
    }

    #[test]
    fn get_byte_list_len_invalid_index() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(ByteList 1, 2, 3, 4, 5)).unwrap();
        let result = data.get_byte_list_len(100);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(100))));
    }

    #[test]
    fn get_byte_list_len_not_byte_list() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        let result = data.get_byte_list_len(0);
        assert_eq!(
            result,
            Err(DataError::new(
                "Not of type",
                DataErrorType::NotType(GarnishDataType::ByteList, GarnishDataType::Number)
            ))
        );
    }

    #[test]
    fn get_byte_list_item_ok() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(ByteList 1, 2, 3, 4, 5)).unwrap();
        let result = data.get_byte_list_item(0, 1.into());
        assert_eq!(result, Ok(Some(2)));
    }

    #[test]
    fn get_byte_list_item_invalid_index() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(ByteList 1, 2, 3, 4, 5)).unwrap();
        let result = data.get_byte_list_item(100, 1.into());
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(100))));
    }

    #[test]
    fn get_byte_list_item_invalid_item_index() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(ByteList 1, 2, 3, 4, 5)).unwrap();
        let result = data.get_byte_list_item(0, 100.into());
        assert_eq!(result, Ok(None));
    }

    #[test]
    fn get_byte_list_item_not_byte_list() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        let result = data.get_byte_list_item(0, 1.into());
        assert_eq!(
            result,
            Err(DataError::new(
                "Not of type",
                DataErrorType::NotType(GarnishDataType::ByteList, GarnishDataType::Number)
            ))
        );
    }

    #[test]
    fn get_byte_list_iter_ok() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(ByteList 1, 2, 3, 4, 5)).unwrap();
        let result: Vec<u8> = data.get_byte_list_iter(0, Extents::new(0.into(), 5.into())).unwrap().collect();
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn get_byte_list_iter_empty_ok() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(ByteList)).unwrap();
        let result: Vec<u8> = data.get_byte_list_iter(0, Extents::new(0.into(), 4.into())).unwrap().collect();
        assert_eq!(result, vec![]);
    }

    #[test]
    fn get_byte_list_iter_ok_with_extents() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(ByteList 1, 2, 3, 4, 5)).unwrap();
        let result: Vec<u8> = data.get_byte_list_iter(0, Extents::new(1.into(), 3.into())).unwrap().collect();
        assert_eq!(result, vec![2, 3]);
    }

    #[test]
    fn get_byte_list_iter_invalid_index() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(ByteList 1, 2, 3, 4, 5)).unwrap();
        let result = data.get_byte_list_iter(100, Extents::new(0.into(), 4.into())).err();
        assert_eq!(result, Some(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(100))));
    }

    #[test]
    fn get_byte_list_iter_not_byte_list() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        let result = data.get_byte_list_iter(0, Extents::new(0.into(), 4.into())).err();
        assert_eq!(
            result,
            Some(DataError::new(
                "Not of type",
                DataErrorType::NotType(GarnishDataType::ByteList, GarnishDataType::Number)
            ))
        );
    }

    #[test]
    fn get_byte_list_iter_start_negative_clamps_to_byte_list_start() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(ByteList 1, 2, 3, 4, 5)).unwrap();
        let result: Vec<u8> = data.get_byte_list_iter(0, Extents::new((-5).into(), 5.into())).unwrap().collect();
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn get_byte_list_iter_start_out_of_bounds_is_empty() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(ByteList 1, 2, 3, 4, 5)).unwrap();
        let result: Vec<u8> = data.get_byte_list_iter(0, Extents::new(6.into(), 10.into())).unwrap().collect();
        assert_eq!(result, vec![]);
    }

    #[test]
    fn get_byte_list_iter_end_out_of_bounds_clamps_to_byte_list_length() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(ByteList 1, 2, 3, 4, 5)).unwrap();
        let result: Vec<u8> = data.get_byte_list_iter(0, Extents::new(0.into(), 10.into())).unwrap().collect();
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn get_symbol_list_len_ok() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(SymList(SymRaw 1, SymRaw 2, SymRaw 3)))
        .unwrap();
        let result = data.get_symbol_list_len(0);
        assert_eq!(result, Ok(3));
    }

    #[test]
    fn get_symbol_list_len_invalid_index() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(SymList(SymRaw 1, SymRaw 2, SymRaw 3)))
        .unwrap();
        let result = data.get_symbol_list_len(100);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(100))));
    }

    #[test]
    fn get_symbol_list_len_not_symbol_list() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        let result = data.get_symbol_list_len(0);
        assert_eq!(
            result,
            Err(DataError::new(
                "Not of type",
                DataErrorType::NotType(GarnishDataType::SymbolList, GarnishDataType::Number)
            ))
        );
    }

    #[test]
    fn get_symbol_list_item_ok_symbol() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(SymList(SymRaw 1, SymRaw 2, SymRaw 3)))
        .unwrap();
        let result = data.get_symbol_list_item(0, 1.into());
        assert_eq!(result, Ok(Some(SymbolListPart::Symbol(2))));
    }

    #[test]
    fn get_symbol_list_item_ok_number() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(SymList(Number 1, Number 2, Number 3)))
        .unwrap();
        let result = data.get_symbol_list_item(0, 1.into());
        assert_eq!(result, Ok(Some(SymbolListPart::Number(2.into()))));
    }

    #[test]
    fn get_symbol_list_item_invalid_index() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(SymList(SymRaw 1, SymRaw 2, SymRaw 3)))
        .unwrap();
        let result = data.get_symbol_list_item(100, 1.into());
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(100))));
    }

    #[test]
    fn get_symbol_list_item_invalid_item_index() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(SymList(SymRaw 1, SymRaw 2, SymRaw 3)))
        .unwrap();
        let result = data.get_symbol_list_item(0, 100.into());
        assert_eq!(result, Ok(None));
    }

    #[test]
    fn get_symbol_list_item_not_symbol_list() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        let result = data.get_symbol_list_item(0, 1.into());
        assert_eq!(
            result,
            Err(DataError::new(
                "Not of type",
                DataErrorType::NotType(GarnishDataType::SymbolList, GarnishDataType::Number)
            ))
        );
    }

    #[test]
    fn get_symbol_list_iter_ok() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(SymList(SymRaw 1, SymRaw 2, SymRaw 3)))
        .unwrap();
        let result: Vec<SymbolListPart<u64, BasicNumber>> = data.get_symbol_list_iter(0, Extents::new(0.into(), 3.into())).unwrap().collect();
        assert_eq!(
            result,
            vec![SymbolListPart::Symbol(1), SymbolListPart::Symbol(2), SymbolListPart::Symbol(3)]
        );
    }

    #[test]
    fn get_symbol_list_iter_empty_ok() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(SymList())).unwrap();
        let result: Vec<SymbolListPart<u64, BasicNumber>> = data.get_symbol_list_iter(0, Extents::new(0.into(), 2.into())).unwrap().collect();
        assert_eq!(result, vec![]);
    }

    #[test]
    fn get_symbol_list_iter_ok_with_extents() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(SymList(SymRaw 1, SymRaw 2, SymRaw 3)))
        .unwrap();
        let result: Vec<SymbolListPart<u64, BasicNumber>> = data.get_symbol_list_iter(0, Extents::new(1.into(), 3.into())).unwrap().collect();
        assert_eq!(result, vec![SymbolListPart::Symbol(2), SymbolListPart::Symbol(3)]);
    }

    #[test]
    fn get_symbol_list_iter_invalid_index() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(SymList(SymRaw 1, SymRaw 2, SymRaw 3)))
        .unwrap();
        let result = data.get_symbol_list_iter(100, Extents::new(0.into(), 2.into())).err();
        assert_eq!(result, Some(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(100))));
    }

    #[test]
    fn get_symbol_list_iter_not_symbol_list() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        let result = data.get_symbol_list_iter(0, Extents::new(0.into(), 2.into())).err();
        assert_eq!(
            result,
            Some(DataError::new(
                "Not of type",
                DataErrorType::NotType(GarnishDataType::SymbolList, GarnishDataType::Number)
            ))
        );
    }

    #[test]
    fn get_symbol_list_iter_start_negative_clamps_to_symbol_list_start() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(SymList(SymRaw 1, SymRaw 2, SymRaw 3)))
        .unwrap();
        let result: Vec<SymbolListPart<u64, BasicNumber>> = data.get_symbol_list_iter(0, Extents::new((-5).into(), 3.into())).unwrap().collect();
        assert_eq!(
            result,
            vec![SymbolListPart::Symbol(1), SymbolListPart::Symbol(2), SymbolListPart::Symbol(3)]
        );
    }

    #[test]
    fn get_symbol_list_iter_end_out_of_bounds_clamps_to_symbol_list_length() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(SymList(SymRaw 1, SymRaw 2, SymRaw 3)))
        .unwrap();
        let result: Vec<SymbolListPart<u64, BasicNumber>> = data.get_symbol_list_iter(0, Extents::new(0.into(), 10.into())).unwrap().collect();
        assert_eq!(
            result,
            vec![SymbolListPart::Symbol(1), SymbolListPart::Symbol(2), SymbolListPart::Symbol(3)]
        );
    }

    #[test]
    fn get_symbol_list_iter_start_out_of_bounds_clamps_to_symbol_list_start() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(SymList(SymRaw 1, SymRaw 2, SymRaw 3)))
        .unwrap();
        let result: Vec<SymbolListPart<u64, BasicNumber>> = data.get_symbol_list_iter(0, Extents::new(10.into(), 3.into())).unwrap().collect();
        assert_eq!(result, vec![]);
    }

    #[test]
    fn get_list_item_iter_ok() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 400)).unwrap();
        data.push_object_to_data_block(basic_object!(Number 500)).unwrap();
        data.push_object_to_data_block(basic_object!(Number 600)).unwrap();
        data.push_object_to_data_block(basic_object!(Number 700)).unwrap();
        let list_index = data
            .push_object_to_data_block(basic_object!((Number 100), (Number 200), (Number 300)))
            .unwrap();
        let result: Vec<usize> = data.get_list_item_iter(list_index, Extents::new(0.into(), 3.into())).unwrap().collect();
        assert_eq!(result, vec![4, 5, 6]);
    }

    #[test]
    fn get_list_item_iter_not_list() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        let result = data.get_list_item_iter(0, Extents::new(0.into(), 2.into())).err();
        assert_eq!(
            result,
            Some(DataError::new(
                "Not of type",
                DataErrorType::NotType(GarnishDataType::List, GarnishDataType::Number)
            ))
        );
    }

    #[test]
    fn get_list_item_iter_invalid_index() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        let result = data.get_list_item_iter(100, Extents::new(0.into(), 2.into())).err();
        assert_eq!(result, Some(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(100))));
    }

    #[test]
    fn get_list_item_iter_not_list_item() {
        let mut data = test_data();
        let list_index = data.push_to_data_block(BasicData::List(1, 0)).unwrap();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        let result = data.get_list_item_iter(list_index, Extents::new(0.into(), 1.into())).err();
        assert_eq!(
            result,
            Some(DataError::new("Not a list item", DataErrorType::NotAListItem(GarnishDataType::Number)))
        );
    }

    #[test]
    fn get_list_item_iter_start_negative_clamps_to_list_start() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 400)).unwrap();
        data.push_object_to_data_block(basic_object!(Number 500)).unwrap();
        data.push_object_to_data_block(basic_object!(Number 600)).unwrap();
        data.push_object_to_data_block(basic_object!(Number 700)).unwrap();
        let list_index = data
            .push_object_to_data_block(basic_object!((Number 100), (Number 200), (Number 300)))
            .unwrap();
        let result: Vec<usize> = data
            .get_list_item_iter(list_index, Extents::new((-5).into(), 3.into()))
            .unwrap()
            .collect();
        assert_eq!(result, vec![4, 5, 6]);
    }

    #[test]
    fn get_list_item_iter_end_out_of_bounds_clamps_to_list_length() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 400)).unwrap();
        data.push_object_to_data_block(basic_object!(Number 500)).unwrap();
        data.push_object_to_data_block(basic_object!(Number 600)).unwrap();
        data.push_object_to_data_block(basic_object!(Number 700)).unwrap();
        let list_index = data
            .push_object_to_data_block(basic_object!((Number 100), (Number 200), (Number 300)))
            .unwrap();
        let result: Vec<usize> = data.get_list_item_iter(list_index, Extents::new(0.into(), 10.into())).unwrap().collect();
        assert_eq!(result, vec![4, 5, 6]);
    }

    #[test]
    fn get_list_item_iter_start_out_of_bounds_clamps_to_list_end() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 400)).unwrap();
        data.push_object_to_data_block(basic_object!(Number 500)).unwrap();
        data.push_object_to_data_block(basic_object!(Number 600)).unwrap();
        data.push_object_to_data_block(basic_object!(Number 700)).unwrap();
        let list_index = data
            .push_object_to_data_block(basic_object!((Number 100), (Number 200), (Number 300)))
            .unwrap();
        let result: Vec<usize> = data.get_list_item_iter(list_index, Extents::new(10.into(), 3.into())).unwrap().collect();
        assert_eq!(result, vec![]);
    }

    #[test]
    fn get_list_item_iter_end_negative_clamps_to_list_start() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 400)).unwrap();
        data.push_object_to_data_block(basic_object!(Number 500)).unwrap();
        data.push_object_to_data_block(basic_object!(Number 600)).unwrap();
        data.push_object_to_data_block(basic_object!(Number 700)).unwrap();
        let list_index = data
            .push_object_to_data_block(basic_object!((Number 100), (Number 200), (Number 300)))
            .unwrap();
        let result: Vec<usize> = data
            .get_list_item_iter(list_index, Extents::new(0.into(), (-5).into()))
            .unwrap()
            .collect();
        assert_eq!(result, vec![]);
    }

    #[test]
    fn get_concatenation_iter_ok() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!((Number 1) <> (Number 2)))
        .unwrap();
        let result: Vec<usize> = data.get_concatenation_iter(2, Extents::new(0.into(), 3.into())).unwrap().collect();
        assert_eq!(result, vec![0, 1]);
    }

    #[test]
    fn get_concatenation_iter_not_concatenation() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        let result = data.get_concatenation_iter(0, Extents::new(0.into(), 3.into())).err();
        assert_eq!(
            result,
            Some(DataError::new(
                "Not of type",
                DataErrorType::NotType(GarnishDataType::Concatenation, GarnishDataType::Number)
            ))
        );
    }

    #[test]
    fn get_concatenation_iter_invalid_index() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!((Number 100) <> (Number 200)))
        .unwrap();
        let result = data.get_concatenation_iter(10, Extents::new(0.into(), 3.into())).err();
        assert_eq!(result, Some(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(10))));
    }

    #[test]
    fn get_concatenation_iter_with_list() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(
            ((Number 100), (Number 200), (Number 300)) <> (Number 400)
        ))
        .unwrap();
        let result: Vec<usize> = data.get_concatenation_iter(11, Extents::new(0.into(), 4.into())).unwrap().collect();
        assert_eq!(result, vec![0, 1, 2, 10]);
    }

    #[test]
    fn get_concatenation_iter_with_list_start_out_of_bounds() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(
            ((Number 100), (Number 200), (Number 300)) <> (Number 400)
        ))
        .unwrap();
        let result: Vec<usize> = data.get_concatenation_iter(11, Extents::new(10.into(), 4.into())).unwrap().collect();
        assert_eq!(result, vec![]);
    }

    #[test]
    fn get_concatenation_iter_with_list_end_out_of_bounds() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(
            ((Number 100), (Number 200), (Number 300)) <> (Number 400)
        ))
        .unwrap();
        let result: Vec<usize> = data.get_concatenation_iter(11, Extents::new(0.into(), 10.into())).unwrap().collect();
        assert_eq!(result, vec![0, 1, 2, 10]);
    }

    #[test]
    fn get_concatenation_iter_with_list_start_negative_clamps_to_list_start() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(
            ((Number 100), (Number 200), (Number 300)) <> (Number 400)
        ))
        .unwrap();
        let result: Vec<usize> = data.get_concatenation_iter(11, Extents::new((-5).into(), 4.into())).unwrap().collect();
        assert_eq!(result, vec![0, 1, 2, 10]);
    }

    #[test]
    fn get_concatenation_iter_with_list_end_negative_clamps_to_list_start() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(
            ((Number 100), (Number 200), (Number 300)) <> (Number 400)
        ))
        .unwrap();
        let result: Vec<usize> = data.get_concatenation_iter(11, Extents::new(0.into(), (-5).into())).unwrap().collect();
        assert_eq!(result, vec![]);
    }

    #[test]
    fn push_value_stack() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        data.push_value_stack(0).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut()[0] = BasicData::Number(100.into());
        expected_data.data_mut()[1] = BasicData::ValueRoot(0);
        expected_data.data_block_mut().cursor = 2;
        expected_data.set_current_value(Some(1));
        assert_eq!(data, expected_data);
    }

    #[test]
    fn push_value_stack_multiple() {
        let mut data = test_data();
        let index = data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        data.push_value_stack(index).unwrap();
        let index = data.push_object_to_data_block(basic_object!(Number 200)).unwrap();
        data.push_value_stack(index).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut()[0] = BasicData::Number(100.into());
        expected_data.data_mut()[1] = BasicData::ValueRoot(0);
        expected_data.data_mut()[2] = BasicData::Number(200.into());
        expected_data.data_mut()[3] = BasicData::Value(1, 2);
        expected_data.data_block_mut().cursor = 4;
        expected_data.set_current_value(Some(3));
        assert_eq!(data, expected_data);
    }

    #[test]
    fn pop_value_stack() {
        let mut data = test_data();
        let index = data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let value_index = data.push_to_data_block(BasicData::ValueRoot(index)).unwrap();
        data.set_current_value(Some(value_index));

        let index = data.pop_value_stack().unwrap();

        let mut expected_data = test_data();

        assert_eq!(index, 0);
        expected_data.data_mut()[0] = BasicData::Number(100.into());
        expected_data.data_mut()[1] = BasicData::ValueRoot(0);
        expected_data.data_block_mut().cursor = 2;
        expected_data.set_current_value(None);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn pop_value_stack_multiple() {
        let mut data = test_data();
        let index = data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let value_index = data.push_to_data_block(BasicData::ValueRoot(index)).unwrap();
        let index = data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let value_index = data.push_to_data_block(BasicData::Value(value_index, index)).unwrap();
        data.set_current_value(Some(value_index));

        let index = data.pop_value_stack().unwrap();

        let mut expected_data = test_data();

        assert_eq!(index, 2);
        expected_data.data_mut()[0] = BasicData::Number(100.into());
        expected_data.data_mut()[1] = BasicData::ValueRoot(0);
        expected_data.data_mut()[2] = BasicData::Number(100.into());
        expected_data.data_mut()[3] = BasicData::Value(1, 2);
        expected_data.data_block_mut().cursor = 4;
        expected_data.set_current_value(Some(1));
        assert_eq!(data, expected_data);
    }

    #[test]
    fn pop_value_stack_no_value() {
        let mut data = test_data();
        data.push_to_data_block(BasicData::Number(100.into())).unwrap();

        let index = data.pop_value_stack();

        let mut expected_data = test_data();

        assert_eq!(index, None);
        expected_data.data_mut()[0] = BasicData::Number(100.into());
        expected_data.data_block_mut().cursor = 1;
        expected_data.set_current_value(None);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn get_current_value() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        data.push_value_stack(0).unwrap();
        let value = data.get_current_value().unwrap();
        assert_eq!(value, 0);
    }

    #[test]
    fn get_current_value_none() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        let value = data.get_current_value();
        assert_eq!(value, None);
    }

    #[test]
    fn get_current_value_mut() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        data.push_value_stack(0).unwrap();
        let value = data.get_current_value_mut().unwrap();
        *value = 200;
        let value = data.get_current_value().unwrap();
        assert_eq!(value, 200);
    }

    #[test]
    fn get_current_value_mut_none() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        let value = data.get_current_value_mut();
        assert_eq!(value, None);
    }

    #[test]
    fn push_register() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        data.push_register(0).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut()[0] = BasicData::Number(100.into());
        expected_data.data_mut()[1] = BasicData::RegisterRoot(0);
        expected_data.data_block_mut().cursor = 2;
        expected_data.set_current_register(Some(1));
        assert_eq!(data, expected_data);
    }

    #[test]
    fn push_register_multiple() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        data.push_register(0).unwrap();
        data.push_object_to_data_block(basic_object!(Number 200)).unwrap();
        data.push_register(2).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut()[0] = BasicData::Number(100.into());
        expected_data.data_mut()[1] = BasicData::RegisterRoot(0);
        expected_data.data_mut()[2] = BasicData::Number(200.into());
        expected_data.data_mut()[3] = BasicData::Register(1, 2);
        expected_data.data_block_mut().cursor = 4;
        expected_data.set_current_register(Some(3));
        assert_eq!(data, expected_data);
    }

    #[test]
    fn get_register() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        data.push_register(100).unwrap();
        data.push_object_to_data_block(basic_object!(Number 200)).unwrap();
        data.push_register(200).unwrap();
        let register = data.get_register(1).unwrap();
        assert_eq!(register, 200);
    }

    #[test]
    fn get_register_last() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        data.push_register(100).unwrap();
        data.push_object_to_data_block(basic_object!(Number 200)).unwrap();
        data.push_register(200).unwrap();
        let register = data.get_register(0).unwrap();
        assert_eq!(register, 100);
    }

    #[test]
    fn get_register_none() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        data.push_register(100).unwrap();
        let register = data.get_register(1);
        assert_eq!(register, None);
    }

    #[test]
    fn pop_register() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        data.push_register(100).unwrap();
        let register = data.pop_register().unwrap().unwrap();
        assert_eq!(register, 100);
    }

    #[test]
    fn pop_register_multiple() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        data.push_register(100).unwrap();
        data.push_object_to_data_block(basic_object!(Number 200)).unwrap();
        data.push_register(200).unwrap();
        let register = data.pop_register().unwrap().unwrap();
        assert_eq!(register, 200);
    }

    #[test]
    fn pop_register_none() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        let register = data.pop_register().unwrap();
        assert_eq!(register, None);
    }

    #[test]
    fn get_register_len() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        data.push_register(100).unwrap();
        data.push_object_to_data_block(basic_object!(Number 200)).unwrap();
        data.push_register(200).unwrap();
        let len = data.get_register_len();
        assert_eq!(len, 2);
    }

    #[test]
    fn push_instruction() {
        let mut data = instruction_test_data();
        data.push_instruction(Instruction::Add, None).unwrap();
        let mut expected_data = instruction_test_data();
        expected_data.data_mut()[0] = BasicData::Instruction(Instruction::Add);
        expected_data.instruction_block_mut().cursor = 1;
        assert_eq!(data, expected_data);
    }

    #[test]
    fn get_instruction_len() {
        let data = test_data();
        let len = data.get_instruction_len();
        assert_eq!(len, 0);
    }

    #[test]
    fn get_instruction_len_multiple() {
        let mut data = test_data();
        data.push_instruction(Instruction::Add, None).unwrap();
        data.push_instruction(Instruction::Subtract, None).unwrap();
        let len = data.get_instruction_len();
        assert_eq!(len, 2);
    }

    #[test]
    fn get_instruction_none() {
        let data = instruction_test_data();
        let instruction = data.get_instruction(0);
        assert_eq!(instruction, None);
    }

    #[test]
    fn get_instruction_multiple() {
        let mut data = instruction_test_data();
        data.push_instruction(Instruction::Add, None).unwrap();
        data.push_instruction(Instruction::Subtract, None).unwrap();
        let instruction = data.get_instruction(1).unwrap();
        assert_eq!(instruction, (Instruction::Subtract, None));
    }

    #[test]
    fn get_instruction_invalid_index() {
        let mut data = instruction_test_data();
        data.push_instruction(Instruction::Add, None).unwrap();
        data.push_instruction(Instruction::Subtract, None).unwrap();
        let instruction = data.get_instruction(10);
        assert_eq!(instruction, None);
    }

    #[test]
    fn get_instrucion_not_instruction() {
        let mut data = test_data();
        data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        let instruction = data.get_instruction(0);
        assert_eq!(instruction, None);
    }

    #[test]
    fn get_instruction_pointer() {
        let data = instruction_test_data();
        let cursor = data.get_instruction_cursor();
        assert_eq!(cursor, 0);
    }

    #[test]
    fn get_instruction_iter() {
        let mut data = instruction_test_data();
        data.push_instruction(Instruction::Add, None).unwrap();
        data.push_instruction(Instruction::Subtract, None).unwrap();
        data.push_instruction(Instruction::Multiply, None).unwrap();
        data.push_instruction(Instruction::Divide, None).unwrap();

        let mut iter = data.get_instruction_iter();
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn set_instruction_pointer() {
        let mut data = instruction_test_data();
        data.set_instruction_cursor(100).unwrap();
        assert_eq!(data.instruction_pointer(), 100);
    }

    #[test]
    fn get_instruction_pointer_after_set() {
        let mut data = instruction_test_data();
        data.push_instruction(Instruction::Add, None).unwrap();
        data.push_instruction(Instruction::Subtract, None).unwrap();
        data.set_instruction_cursor(2).unwrap();
        let cursor = data.get_instruction_cursor();
        assert_eq!(cursor, 2);
    }

    #[test]
    fn get_jump_table_len() {
        let data = jump_table_test_data();
        let len = data.get_jump_table_len();
        assert_eq!(len, 0);
    }

    #[test]
    fn get_jump_table_len_multiple() {
        let mut data = jump_table_test_data();
        data.push_to_jump_table_block(100).unwrap();
        data.push_to_jump_table_block(200).unwrap();
        let len = data.get_jump_table_len();
        assert_eq!(len, 2);
    }

    #[test]
    fn get_jump_table_none() {
        let data = jump_table_test_data();
        let jump_table = data.get_from_jump_table(0);
        assert_eq!(jump_table, None);
    }

    #[test]
    fn get_from_jump_table_mut() {
        let mut data = jump_table_test_data();
        data.push_to_jump_table(100).unwrap();
        data.push_to_jump_table(200).unwrap();
        let jump_table = data.get_from_jump_table_mut(1).unwrap();
        *jump_table = 300;
        assert_eq!(data.get_from_jump_table(1).unwrap(), 300);
    }

    #[test]
    fn push_to_jump_table() {
        let mut data = jump_table_test_data();
        data.push_to_jump_table(100).unwrap();
        let mut expected_data = jump_table_test_data();
        expected_data.data_mut()[0] = BasicData::JumpPoint(100);
        expected_data.jump_table_block_mut().cursor = 1;
        assert_eq!(data, expected_data);
    }

    #[test]
    fn push_jump_path() {
        let mut data = test_data();
        data.set_current_register(Some(500));
        data.push_frame(100).unwrap();
        let mut expected_data = test_data();
        expected_data.data_mut()[0] = BasicData::JumpPoint(100);
        expected_data.data_mut()[1] = BasicData::FrameRegister(500);
        expected_data.set_current_register(Some(500));
        expected_data.set_current_frame(Some(1));
        expected_data.data_block_mut().cursor = 2;
        assert_eq!(data, expected_data);
    }

    #[test]
    fn pop_frame() {
        let mut data = test_data();
        data.set_current_register(Some(500));
        data.push_frame(100).unwrap();
        let jump_path = data.pop_frame().unwrap();
        assert_eq!(jump_path, Some(100));
        let mut expected_data = test_data();
        expected_data.data_mut()[0] = BasicData::JumpPoint(100);
        expected_data.data_mut()[1] = BasicData::FrameRegister(500);
        expected_data.set_current_register(Some(500));
        expected_data.data_block_mut().cursor = 2;
        expected_data.set_current_frame(None);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn pop_frame_multiple() {
        let mut data = test_data();
        data.set_current_register(Some(500));
        data.push_frame(100).unwrap();
        data.push_frame(200).unwrap();
        let jump_path = data.pop_frame().unwrap().unwrap();
        assert_eq!(jump_path, 200);
        let mut expected_data = test_data();
        expected_data.data_mut()[0] = BasicData::JumpPoint(100);
        expected_data.data_mut()[1] = BasicData::FrameRegister(500);
        expected_data.data_mut()[2] = BasicData::JumpPoint(200);
        expected_data.data_mut()[3] = BasicData::Frame(1, 500);
        expected_data.set_current_register(Some(500));
        expected_data.data_block_mut().cursor = 4;
        expected_data.set_current_frame(Some(1));
        assert_eq!(data, expected_data);
    }

    #[test]
    fn pop_frame_resets_register() {
        let mut data = test_data();
        data.push_frame(100).unwrap();
        data.push_register(5).unwrap();
        data.push_frame(200).unwrap();
        data.push_register(6).unwrap();
        data.push_register(7).unwrap();
        data.push_frame(300).unwrap();
        data.push_register(8).unwrap();
        data.push_register(9).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut().resize(20, BasicData::Empty);
        expected_data.data_mut()[0] = BasicData::JumpPoint(100);
        expected_data.data_mut()[1] = BasicData::FrameRoot;
        expected_data.data_mut()[2] = BasicData::RegisterRoot(5);
        expected_data.data_mut()[3] = BasicData::JumpPoint(200);
        expected_data.data_mut()[4] = BasicData::Frame(1, 2);
        expected_data.data_mut()[5] = BasicData::Register(2, 6);
        expected_data.data_mut()[6] = BasicData::Register(5, 7);
        expected_data.data_mut()[7] = BasicData::JumpPoint(300);
        expected_data.data_mut()[8] = BasicData::Frame(4, 6);
        expected_data.data_mut()[9] = BasicData::Register(6, 8);
        expected_data.data_mut()[10] = BasicData::Register(9, 9);
        expected_data.set_current_register(Some(10));
        expected_data.data_block_mut().cursor = 11;
        expected_data.data_block_mut().size = 20;
        expected_data.custom_data_block_mut().start = 20;
        expected_data.set_current_frame(Some(8));
        assert_eq!(data, expected_data);
    
        let jump_path = data.pop_frame().unwrap();
        assert_eq!(jump_path, Some(300));
        assert_eq!(data.current_frame(), Some(4));
        assert_eq!(data.current_register(), Some(6));

        let jump_path = data.pop_frame().unwrap();
        assert_eq!(jump_path, Some(200));
        assert_eq!(data.current_frame(), Some(1));
        assert_eq!(data.current_register(), Some(2));

        let jump_path = data.pop_frame().unwrap();
        assert_eq!(jump_path, Some(100));
        assert_eq!(data.current_frame(), None);
        assert_eq!(data.current_register(), None);
    }

    #[test]
    fn pop_frame_none() {
        let mut data = test_data();
        let jump_path = data.pop_frame().unwrap();
        assert_eq!(jump_path, None);
    }

    #[test]
    fn parse_add_symbol() {
        let mut data = test_data();
        let symbol = data.parse_add_symbol("my_symbol").unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut().resize(30, BasicData::Empty);
        expected_data.data_mut()[0] = BasicData::AssociativeItem(8904929874702161741, 1);
        expected_data.data_mut()[10] = BasicData::Symbol(8904929874702161741);
        expected_data.data_mut()[11] = BasicData::CharList(9);
        expected_data.data_mut()[12] = BasicData::Char('m');
        expected_data.data_mut()[13] = BasicData::Char('y');
        expected_data.data_mut()[14] = BasicData::Char('_');
        expected_data.data_mut()[15] = BasicData::Char('s');
        expected_data.data_mut()[16] = BasicData::Char('y');
        expected_data.data_mut()[17] = BasicData::Char('m');
        expected_data.data_mut()[18] = BasicData::Char('b');
        expected_data.data_mut()[19] = BasicData::Char('o');
        expected_data.data_mut()[20] = BasicData::Char('l');
        expected_data.symbol_table_block_mut().cursor = 1;
        expected_data.symbol_table_block_mut().size = 10;
        expected_data.expression_symbol_block_mut().start = 10;
        expected_data.data_block_mut().start = 10;
        expected_data.data_block_mut().size = 20;
        expected_data.data_block_mut().cursor = 11;
        expected_data.custom_data_block_mut().start = 30;

        assert_eq!(symbol, 0);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_byte_list_from() {
        let mut data = test_data();
        let index = data.push_object_to_data_block(basic_object!((Number 100), (CharList "Some Text"))).unwrap();
        data.add_byte_list_from(index).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut().resize(60, BasicData::Empty);
        expected_data.data_mut()[0] = BasicData::Number(100.into());
        expected_data.data_mut()[1] = BasicData::CharList(9);
        expected_data.data_mut()[2] = BasicData::Char('S');
        expected_data.data_mut()[3] = BasicData::Char('o');
        expected_data.data_mut()[4] = BasicData::Char('m');
        expected_data.data_mut()[5] = BasicData::Char('e');
        expected_data.data_mut()[6] = BasicData::Char(' ');
        expected_data.data_mut()[7] = BasicData::Char('T');
        expected_data.data_mut()[8] = BasicData::Char('e');
        expected_data.data_mut()[9] = BasicData::Char('x');
        expected_data.data_mut()[10] = BasicData::Char('t');
        expected_data.data_mut()[11] = BasicData::List(2, 0);
        expected_data.data_mut()[12] = BasicData::ListItem(0);
        expected_data.data_mut()[13] = BasicData::ListItem(1);
        expected_data.data_mut()[14] = BasicData::Empty;
        expected_data.data_mut()[15] = BasicData::Empty;
        expected_data.data_mut()[16] = BasicData::ByteList(40);
        expected_data.data_mut()[17] = BasicData::Byte(100);
        expected_data.data_mut()[18] = BasicData::Byte(0);
        expected_data.data_mut()[19] = BasicData::Byte(0);
        expected_data.data_mut()[20] = BasicData::Byte(0);
        expected_data.data_mut()[21] = BasicData::Byte(83);
        expected_data.data_mut()[22] = BasicData::Byte(0);
        expected_data.data_mut()[23] = BasicData::Byte(0);
        expected_data.data_mut()[24] = BasicData::Byte(0);
        expected_data.data_mut()[25] = BasicData::Byte(111);
        expected_data.data_mut()[26] = BasicData::Byte(0);
        expected_data.data_mut()[27] = BasicData::Byte(0);
        expected_data.data_mut()[28] = BasicData::Byte(0);
        expected_data.data_mut()[29] = BasicData::Byte(109);
        expected_data.data_mut()[30] = BasicData::Byte(0);
        expected_data.data_mut()[31] = BasicData::Byte(0);
        expected_data.data_mut()[32] = BasicData::Byte(0);
        expected_data.data_mut()[33] = BasicData::Byte(101);
        expected_data.data_mut()[34] = BasicData::Byte(0);
        expected_data.data_mut()[35] = BasicData::Byte(0);
        expected_data.data_mut()[36] = BasicData::Byte(0);
        expected_data.data_mut()[37] = BasicData::Byte(32);
        expected_data.data_mut()[38] = BasicData::Byte(0);
        expected_data.data_mut()[39] = BasicData::Byte(0);
        expected_data.data_mut()[40] = BasicData::Byte(0);
        expected_data.data_mut()[41] = BasicData::Byte(84);
        expected_data.data_mut()[42] = BasicData::Byte(0);
        expected_data.data_mut()[43] = BasicData::Byte(0);
        expected_data.data_mut()[44] = BasicData::Byte(0);
        expected_data.data_mut()[45] = BasicData::Byte(101);
        expected_data.data_mut()[46] = BasicData::Byte(0);
        expected_data.data_mut()[47] = BasicData::Byte(0);
        expected_data.data_mut()[48] = BasicData::Byte(0);
        expected_data.data_mut()[49] = BasicData::Byte(120);
        expected_data.data_mut()[50] = BasicData::Byte(0);
        expected_data.data_mut()[51] = BasicData::Byte(0);
        expected_data.data_mut()[52] = BasicData::Byte(0);
        expected_data.data_mut()[53] = BasicData::Byte(116);
        expected_data.data_mut()[54] = BasicData::Byte(0);
        expected_data.data_mut()[55] = BasicData::Byte(0);
        expected_data.data_mut()[56] = BasicData::Byte(0);

        expected_data.data_block_mut().cursor = 57;
        expected_data.data_block_mut().size = 60;
        expected_data.data_block_mut().start = 0;
        expected_data.custom_data_block_mut().start = 60;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_number_from() {
        let mut data = test_data();
        let index = data.push_object_to_data_block(basic_object!(CharList "123456789")).unwrap();
        data.add_number_from(index).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut().resize(20, BasicData::Empty);
        expected_data.data_mut()[0] = BasicData::CharList(9);
        expected_data.data_mut()[1] = BasicData::Char('1');
        expected_data.data_mut()[2] = BasicData::Char('2');
        expected_data.data_mut()[3] = BasicData::Char('3');
        expected_data.data_mut()[4] = BasicData::Char('4');
        expected_data.data_mut()[5] = BasicData::Char('5');
        expected_data.data_mut()[6] = BasicData::Char('6');
        expected_data.data_mut()[7] = BasicData::Char('7');
        expected_data.data_mut()[8] = BasicData::Char('8');
        expected_data.data_mut()[9] = BasicData::Char('9');
        expected_data.data_mut()[10] = BasicData::Number(123456789.into());
        expected_data.data_block_mut().cursor = 11;
        expected_data.data_block_mut().size = 20;
        expected_data.custom_data_block_mut().start = 20;
        assert_eq!(data, expected_data);
    }

    #[test]
    fn add_number_from_invalid_number() {
        let mut data = test_data();
        let index = data.push_object_to_data_block(basic_object!(CharList "abcde")).unwrap();
        data.add_number_from(index).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut().resize(10, BasicData::Empty);
        expected_data.data_mut()[0] = BasicData::CharList(5);
        expected_data.data_mut()[1] = BasicData::Char('a');
        expected_data.data_mut()[2] = BasicData::Char('b');
        expected_data.data_mut()[3] = BasicData::Char('c');
        expected_data.data_mut()[4] = BasicData::Char('d');
        expected_data.data_mut()[5] = BasicData::Char('e');
        expected_data.data_mut()[6] = BasicData::Unit;
        expected_data.data_block_mut().cursor = 7;
        assert_eq!(data, expected_data);
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
    struct Foo {
        value: String,
    }

    impl BasicDataCustom for Foo {}

    #[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd)]
    struct FooCompanion;

    impl BasicDataCompanion<Foo> for FooCompanion {
        fn resolve(data: &mut BasicGarnishData<Foo, Self>, _symbol: u64) -> Result<bool, DataError> {
            data.push_object_to_data_block(basic_object!(Custom Foo {
                value: "resolved".to_string(),
            }))?;
            Ok(true)
        }
    
        fn apply(data: &mut BasicGarnishData<Foo, Self>, _external_value: usize, _input_addr: usize) -> Result<bool, DataError> {
            data.push_object_to_data_block(basic_object!(Custom Foo {
                value: "applied".to_string(),
            }))?;
            Ok(true)
        }
    
        fn defer_op(
            data: &mut BasicGarnishData<Foo, Self>,
            _operation: Instruction,
            _left: (GarnishDataType, usize),
            _right: (GarnishDataType, usize),
        ) -> Result<bool, DataError> {
            data.push_object_to_data_block(basic_object!(Custom Foo {
                value: "deferred".to_string(),
            }))?;
            Ok(true)
        }
    }

    #[test]
    fn custom_data_resolved() {
        let mut data = BasicGarnishData::<Foo, FooCompanion>::new_with_settings(
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
        ).unwrap();
        data.resolve(100).unwrap();
        assert_eq!(data.data()[0], BasicData::Custom(Foo { value: "resolved".to_string() }));
    }

    #[test]
    fn custom_data_applied() {
        let mut data = BasicGarnishData::<Foo, FooCompanion>::new_with_settings(
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
        ).unwrap();
        data.apply(100, 200).unwrap();
        assert_eq!(data.data()[0], BasicData::Custom(Foo { value: "applied".to_string() }));
    }
    
    #[test]
    fn custom_data_deferred() {
        let mut data = BasicGarnishData::<Foo, FooCompanion>::new_with_settings(
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(10, usize::MAX, ReallocationStrategy::FixedSize(10)),
            StorageSettings::new(0, usize::MAX, ReallocationStrategy::FixedSize(10)),
        ).unwrap();
        data.defer_op(Instruction::Add, (GarnishDataType::Number, 100), (GarnishDataType::Number, 200)).unwrap();
        assert_eq!(data.data()[0], BasicData::Custom(Foo { value: "deferred".to_string() }));
    }
}
