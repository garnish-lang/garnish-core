use std::convert::TryInto;
use std::fmt::Debug;
use std::hash::Hash;

use garnish_lang_traits::GarnishData;

use crate::data::{NumberIterator, SimpleNumber, SizeIterator, parse_byte_list, parse_char_list, parse_simple_number};
use crate::{DataError, GarnishDataType, Instruction, DataIndexIterator, SimpleData, SimpleGarnishData, SimpleInstruction, SimpleStackFrame, symbol_value, UNIT_INDEX};

impl<T> GarnishData for SimpleGarnishData<T>
where
    T: Clone + PartialEq + Eq + PartialOrd + Debug + Hash,
{
    type Error = DataError;
    type Symbol = u64;
    type Byte = u8;
    type Char = char;
    type Number = SimpleNumber;
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

    fn get_data_len(&self) -> usize {
        self.data.len()
    }

    fn get_data_iter(&self) -> SizeIterator {
        SizeIterator::new(0, self.data.len())
    }

    fn get_value_stack_len(&self) -> usize {
        self.values.len()
    }

    fn push_value_stack(&mut self, addr: usize) -> Result<(), Self::Error> {
        self.values.push(addr);
        Ok(())
    }

    fn pop_value_stack(&mut self) -> Option<usize> {
        self.values.pop()
    }

    fn get_value(&self, index: usize) -> Option<usize> {
        self.values.get(index).cloned()
    }

    fn get_value_mut(&mut self, index: usize) -> Option<&mut usize> {
        self.values.get_mut(index)
    }

    fn get_current_value(&self) -> Option<usize> {
        self.values.last().cloned()
    }

    fn get_current_value_mut(&mut self) -> Option<&mut usize> {
        self.values.last_mut()
    }

    fn get_value_iter(&self) -> Self::ValueIndexIterator {
        SizeIterator::new(0, self.values.len())
    }

    fn get_data_type(&self, index: usize) -> Result<GarnishDataType, Self::Error> {
        let d = self.get(index)?;

        Ok(d.get_data_type())
    }

    fn get_number(&self, index: usize) -> Result<SimpleNumber, Self::Error> {
        self.get(index)?.as_number()
    }

    fn get_type(&self, addr: Self::Size) -> Result<GarnishDataType, Self::Error> {
        self.get(addr)?.as_type()
    }

    fn get_char(&self, index: Self::Size) -> Result<Self::Char, Self::Error> {
        self.get(index)?.as_char()
    }

    fn get_byte(&self, addr: Self::Size) -> Result<Self::Byte, Self::Error> {
        self.get(addr)?.as_byte()
    }

    fn get_symbol(&self, index: usize) -> Result<u64, Self::Error> {
        self.get(index)?.as_symbol()
    }

    fn get_expression(&self, index: usize) -> Result<usize, Self::Error> {
        self.get(index)?.as_expression()
    }

    fn get_external(&self, index: usize) -> Result<usize, Self::Error> {
        self.get(index)?.as_external()
    }

    fn get_pair(&self, index: usize) -> Result<(usize, usize), Self::Error> {
        self.get(index)?.as_pair()
    }

    fn get_concatenation(&self, index: usize) -> Result<(usize, usize), Self::Error> {
        self.get(index)?.as_concatenation()
    }

    fn get_range(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
        self.get(addr)?.as_range()
    }

    fn get_slice(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
        self.get(addr)?.as_slice()
    }

    fn get_list_len(&self, index: usize) -> Result<usize, Self::Error> {
        Ok(self.get(index)?.as_list()?.0.len())
    }

    fn get_list_item(&self, list_index: usize, item_index: SimpleNumber) -> Result<usize, Self::Error> {
        match item_index {
            SimpleNumber::Integer(item_index) => match self.get(list_index)?.as_list()?.0.get(item_index as usize) {
                None => Err(format!("No list item at index {:?} for list at addr {:?}", item_index, list_index))?,
                Some(v) => Ok(*v),
            },
            SimpleNumber::Float(_) => Err(DataError::from("Cannot index list with decimal value.".to_string())), // should return None
        }
    }

    fn get_list_associations_len(&self, index: usize) -> Result<usize, Self::Error> {
        Ok(self.get(index)?.as_list()?.1.len())
    }

    fn get_list_association(&self, list_index: usize, item_index: SimpleNumber) -> Result<usize, Self::Error> {
        match item_index {
            SimpleNumber::Integer(item_index) => match self.get(list_index)?.as_list()?.1.get(item_index as usize) {
                None => Err(format!("No list item at index {:?} for list at addr {:?}", item_index, list_index))?,
                Some(v) => Ok(*v),
            },
            SimpleNumber::Float(_) => Err(DataError::from("Cannot index list with decimal value.".to_string())), // should return None
        }
    }

    fn get_list_item_with_symbol(&self, list_addr: usize, sym: u64) -> Result<Option<usize>, Self::Error> {
        let associations_len = self.get_list_associations_len(list_addr)?;

        if associations_len == 0 {
            return Ok(None);
        }

        let mut i = sym as usize % associations_len;
        let mut count = 0;

        loop {
            // check to make sure item has same symbol
            let association_ref = self.get_list_association(list_addr, i.into())?;

            // should have symbol on left
            match self.get_data_type(association_ref)? {
                GarnishDataType::Pair => {
                    let (left, right) = self.get_pair(association_ref)?;

                    let left_ref = left;

                    match self.get_data_type(left_ref)? {
                        GarnishDataType::Symbol => {
                            let v = self.get_symbol(left_ref)?;

                            if v == sym {
                                // found match
                                // insert pair right as value
                                return Ok(Some(right));
                            }
                        }
                        t => Err(format!("Association created with non-symbol type {:?} on pair left.", t))?,
                    }
                }
                t => Err(format!("Association created with non-pair type {:?}.", t))?,
            }

            i += 1;
            if i >= associations_len {
                i = 0;
            }

            count += 1;
            if count > associations_len {
                return Ok(None);
            }
        }
    }

    fn get_list_items_iter(&self, list_addr: Self::Size) -> Self::ListIndexIterator {
        self.get_list_len(list_addr)
            .and_then(|len| Ok(NumberIterator::new(SimpleNumber::Integer(0), Self::size_to_number(len))))
            .unwrap_or(NumberIterator::new(SimpleNumber::Integer(0), SimpleNumber::Integer(0)))
    }

    fn get_list_associations_iter(&self, list_addr: Self::Size) -> Self::ListIndexIterator {
        self.get_list_associations_len(list_addr)
            .and_then(|len| Ok(NumberIterator::new(SimpleNumber::Integer(0), Self::size_to_number(len))))
            .unwrap_or(NumberIterator::new(SimpleNumber::Integer(0), SimpleNumber::Integer(0)))
    }

    fn get_char_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        Ok(self.get(addr)?.as_char_list()?.len())
    }

    fn get_char_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Self::Char, Self::Error> {
        match item_index {
            SimpleNumber::Integer(item_index) => match self.get(addr)?.as_char_list()?.chars().nth(item_index as usize) {
                None => Err(format!("No value at index {:?} for char list at {:?}", item_index, addr))?,
                Some(c) => Ok(c),
            },
            SimpleNumber::Float(_) => Err(DataError::from("Cannot index char list with decimal value.".to_string())), // should return None
        }
    }

    fn get_char_list_iter(&self, list_addr: Self::Size) -> Self::ListIndexIterator {
        self.get_char_list_len(list_addr)
            .and_then(|len| Ok(NumberIterator::new(SimpleNumber::Integer(0), Self::size_to_number(len))))
            .unwrap_or(NumberIterator::new(SimpleNumber::Integer(0), SimpleNumber::Integer(0)))
    }

    fn get_byte_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        Ok(self.get(addr)?.as_byte_list()?.len())
    }

    fn get_byte_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Self::Byte, Self::Error> {
        match item_index {
            SimpleNumber::Integer(item_index) => match self.get(addr)?.as_byte_list()?.get(item_index as usize) {
                None => Err(format!("No value at index {:?} for byte list at {:?}", item_index, addr))?,
                Some(c) => Ok(*c),
            },
            SimpleNumber::Float(_) => Err(DataError::from("Cannot index byte list with decimal value.".to_string())), // should return None
        }
    }

    fn get_byte_list_iter(&self, list_addr: Self::Size) -> Self::ListIndexIterator {
        self.get_byte_list_len(list_addr)
            .and_then(|len| Ok(NumberIterator::new(SimpleNumber::Integer(0), Self::size_to_number(len))))
            .unwrap_or(NumberIterator::new(SimpleNumber::Integer(0), SimpleNumber::Integer(0)))
    }

    fn get_symbol_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        Ok(self.get(addr)?.as_symbol_list()?.len())
    }

    fn get_symbol_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Self::Symbol, Self::Error> {
        match item_index {
            SimpleNumber::Integer(item_index) => match self.get(addr)?.as_symbol_list()?.get(item_index as usize) {
                None => Err(format!("No value at index {:?} for symbol list at {:?}", item_index, addr))?,
                Some(c) => Ok(*c),
            },
            SimpleNumber::Float(_) => Err(DataError::from("Cannot index symbol list with decimal value.".to_string())), // should return None
        }
    }

    fn get_symbol_list_iter(&self, list_addr: Self::Size) -> Self::ListIndexIterator {
        self.get_symbol_list_len(list_addr)
            .and_then(|len| Ok(NumberIterator::new(SimpleNumber::Integer(0), Self::size_to_number(len))))
            .unwrap_or(NumberIterator::new(SimpleNumber::Integer(0), SimpleNumber::Integer(0)))
    }

    fn get_list_item_iter(&self, list_addr: Self::Size) -> Self::ListItemIterator {
        match self.get_data().get(list_addr) {
            Some(SimpleData::List(items, _)) => DataIndexIterator::new(items.clone()),
            _ => DataIndexIterator::new(vec![]),
        }
    }

    fn get_concatenation_iter(&self, addr: Self::Size) -> Self::ConcatenationItemIterator {
        match self.get_data().get(addr) {
            Some(SimpleData::Concatenation(left, right)) => {
                let mut items = vec![];
                let mut con_stack = vec![right.clone(), left.clone()];

                while let Some(item) = con_stack.pop() {
                    match self.get_data().get(item) {
                        None => items.push(UNIT_INDEX),
                        Some(SimpleData::Concatenation(left, right)) => {
                            con_stack.push(right.clone());
                            con_stack.push(left.clone());
                        }
                        Some(SimpleData::List(list_items, _)) => {
                            for item in list_items {
                                items.push(item.clone());
                            }
                        }
                        Some(_) => items.push(item),
                    }
                }

                DataIndexIterator::new(items)
            }
            _ => DataIndexIterator::new(vec![]),
        }
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

    fn add_unit(&mut self) -> Result<usize, Self::Error> {
        Ok(0)
    }

    fn add_true(&mut self) -> Result<usize, Self::Error> {
        Ok(2)
    }

    fn add_false(&mut self) -> Result<usize, Self::Error> {
        Ok(1)
    }

    fn add_number(&mut self, value: SimpleNumber) -> Result<usize, Self::Error> {
        self.cache_add(SimpleData::Number(value))
    }

    fn add_type(&mut self, value: GarnishDataType) -> Result<Self::Size, Self::Error> {
        self.cache_add(SimpleData::Type(value))
    }

    fn add_char(&mut self, value: Self::Char) -> Result<Self::Size, Self::Error> {
        self.cache_add(SimpleData::Char(value))
    }

    fn add_byte(&mut self, value: Self::Byte) -> Result<Self::Size, Self::Error> {
        self.cache_add(SimpleData::Byte(value))
    }

    fn add_symbol(&mut self, value: u64) -> Result<usize, Self::Error> {
        self.cache_add(SimpleData::Symbol(value))
    }

    fn add_expression(&mut self, value: usize) -> Result<usize, Self::Error> {
        self.cache_add(SimpleData::Expression(value))
    }

    fn add_external(&mut self, value: usize) -> Result<usize, Self::Error> {
        self.cache_add(SimpleData::External(value))
    }

    fn add_pair(&mut self, value: (usize, usize)) -> Result<usize, Self::Error> {
        self.data.push(SimpleData::Pair(value.0, value.1));
        Ok(self.data.len() - 1)
    }

    fn add_concatenation(&mut self, left: Self::Size, right: Self::Size) -> Result<Self::Size, Self::Error> {
        self.data.push(SimpleData::Concatenation(left, right));
        Ok(self.data.len() - 1)
    }

    fn add_range(&mut self, start: Self::Size, end: Self::Size) -> Result<Self::Size, Self::Error> {
        self.data.push(SimpleData::Range(start, end));
        Ok(self.data.len() - 1)
    }

    fn add_slice(&mut self, list: Self::Size, range: Self::Size) -> Result<Self::Size, Self::Error> {
        self.data.push(SimpleData::Slice(list, range));
        Ok(self.data.len() - 1)
    }

    fn merge_to_symbol_list(&mut self, first: Self::Size, second: Self::Size) -> Result<Self::Size, Self::Error> {
        match (self.get_data().get(first), self.get_data().get(second)) {
            (Some(SimpleData::Symbol(sym1)), Some(SimpleData::Symbol(sym2))) => {
                self.data.push(SimpleData::SymbolList(vec![*sym1, *sym2]));
            }
            (Some(SimpleData::SymbolList(list1)), Some(SimpleData::SymbolList(list2))) => {
                self.data.push(SimpleData::SymbolList(list1.iter().chain(list2).map(|i| *i).collect()));
            }
            (Some(SimpleData::SymbolList(list)), Some(SimpleData::Symbol(sym))) => {
                let mut new_list = list.clone();
                new_list.push(*sym);
                self.data.push(SimpleData::SymbolList(new_list));
            }
            (Some(SimpleData::Symbol(sym)), Some(SimpleData::SymbolList(list))) => {
                let mut new_list = list.clone();
                new_list.insert(0, *sym);
                self.data.push(SimpleData::SymbolList(new_list));
            }
            (Some(t1), Some(t2)) => Err(format!("Cannot create symbol list from types: {:?} {:?}", t1, t2))?,
            (None, None) => Err(format!(
                "Failed to create symbol list. No data at either operand indices, {}, {}",
                first, second
            ))?,
            (None, _) => Err(format!("Failed to create symbol list. No data at left operand index, {}", first))?,
            (_, None) => Err(format!("Failed to create symbol list. No data at right operand index, {}", second))?,
        }

        Ok(self.data.len() - 1)
    }

    fn start_list(&mut self, _: usize) -> Result<(), Self::Error> {
        self.current_list = Some((vec![], vec![]));
        Ok(())
    }

    fn add_to_list(&mut self, addr: usize, is_associative: bool) -> Result<(), Self::Error> {
        match &mut self.current_list {
            None => Err("Not currently creating a list.".to_string())?,
            Some((items, associations)) => {
                items.push(addr);

                if is_associative {
                    associations.push(addr);
                }

                Ok(())
            }
        }
    }

    fn end_list(&mut self) -> Result<usize, Self::Error> {
        match &mut self.current_list {
            None => Err("Not currently creating a list.".to_string())?,
            Some((items, associations)) => {
                // reorder associative values by modulo value
                let mut ordered = vec![0usize; associations.len()];
                for index in 0..associations.len() {
                    let item = associations[index];
                    let mut i = item % associations.len();
                    let mut count = 0;
                    while ordered[i] != 0 {
                        i += 1;
                        if i >= associations.len() {
                            i = 0;
                        }

                        count += 1;
                        if count > associations.len() {
                            Err("Could not place associative value".to_string())?;
                        }
                    }

                    ordered[i] = item;
                }

                self.data.push(SimpleData::List(items.to_vec(), ordered));
                Ok(self.data.len() - 1)
            }
        }
    }

    fn start_char_list(&mut self) -> Result<(), Self::Error> {
        self.current_char_list = Some(String::new());
        Ok(())
    }

    fn add_to_char_list(&mut self, c: Self::Char) -> Result<(), Self::Error> {
        match &mut self.current_char_list {
            None => Err("Attempting to add to unstarted char list.".to_string())?,
            Some(s) => s.push(c),
        }

        Ok(())
    }

    fn end_char_list(&mut self) -> Result<Self::Size, Self::Error> {
        let data = match &self.current_char_list {
            None => Err("Attempting to end unstarted char list.".to_string())?,
            Some(s) => SimpleData::CharList(s.clone()),
        };

        let addr = self.cache_add(data)?;

        self.current_char_list = None;

        Ok(addr)
    }

    fn start_byte_list(&mut self) -> Result<(), Self::Error> {
        self.current_byte_list = Some(Vec::new());
        Ok(())
    }

    fn add_to_byte_list(&mut self, c: Self::Byte) -> Result<(), Self::Error> {
        match &mut self.current_byte_list {
            None => Err("Attempting to add to unstarted byte list.".to_string())?,
            Some(l) => l.push(c),
        }

        Ok(())
    }

    fn end_byte_list(&mut self) -> Result<Self::Size, Self::Error> {
        let data = match &self.current_byte_list {
            None => Err("Attempting to end unstarted byte list.".to_string())?,
            Some(l) => SimpleData::ByteList(l.clone()),
        };

        let addr = self.cache_add(data)?;

        self.current_byte_list = None;

        Ok(addr)
    }

    fn get_register_len(&self) -> Self::Size {
        self.register.len()
    }

    fn push_register(&mut self, addr: usize) -> Result<(), Self::Error> {
        self.register.push(addr);
        Ok(())
    }

    fn get_register(&self, addr: Self::Size) -> Option<Self::Size> {
        self.register.get(addr).cloned()
    }

    fn pop_register(&mut self) -> Result<Option<Self::Size>, Self::Error> {
        match self.register.pop() {
            None => Ok(None),
            Some(value) => match self.data.get(value) {
                None => Err(format!("Register address ({}) has no data", value))?,
                Some(data) => match data {
                    SimpleData::StackFrame(_) => Err("Popped StackFrame from registers. Should only be done when popping jump path.".to_string())?,
                    _ => Ok(Some(value)),
                },
            },
        }
    }

    fn get_register_iter(&self) -> Self::RegisterIndexIterator {
        SizeIterator::new(0, self.register.len())
    }

    fn get_instruction_len(&self) -> usize {
        self.instructions.len()
    }

    fn push_instruction(&mut self, instruction: Instruction, data: Option<usize>) -> Result<usize, Self::Error> {
        self.instructions.push(SimpleInstruction::new(instruction, data));
        Ok(self.instructions.len() - 1)
    }

    fn get_instruction(&self, index: usize) -> Option<(Instruction, Option<usize>)> {
        self.instructions.get(index).and_then(|i| Some((i.instruction, i.data)))
    }

    fn get_instruction_iter(&self) -> Self::InstructionIterator {
        SizeIterator::new(0, self.instructions.len())
    }

    fn get_instruction_cursor(&self) -> usize {
        self.instruction_cursor
    }

    fn set_instruction_cursor(&mut self, index: usize) -> Result<(), Self::Error> {
        self.instruction_cursor = index;
        Ok(())
    }

    fn get_jump_table_len(&self) -> usize {
        self.expression_table.len()
    }

    fn push_jump_point(&mut self, index: usize) -> Result<(), Self::Error> {
        self.expression_table.push(index);
        Ok(())
    }

    fn get_jump_point(&self, index: usize) -> Option<usize> {
        self.expression_table.get(index).cloned()
    }

    fn get_jump_point_mut(&mut self, index: usize) -> Option<&mut usize> {
        self.expression_table.get_mut(index)
    }

    fn get_jump_table_iter(&self) -> Self::JumpTableIndexIterator {
        SizeIterator::new(0, self.expression_table.len())
    }

    fn push_jump_path(&mut self, index: usize) -> Result<(), Self::Error> {
        let r = self.add_stack_frame(SimpleStackFrame::new(index))?;
        self.push_register(r)
    }

    fn pop_jump_path(&mut self) -> Option<usize> {
        while let Some(item) = self.register.pop() {
            match self.data.get(item) {
                None => return None, // should probably be error
                Some(data) => match data {
                    SimpleData::StackFrame(frame) => return Some(frame.return_addr()),
                    _ => (),
                },
            }
        }

        None
    }

    fn get_jump_path_iter(&self) -> Self::JumpPathIndexIterator {
        unimplemented!() // not sure whether this function is needed currently unused by core
    }

    //
    // Casting
    //

    fn size_to_number(from: Self::Size) -> Self::Number {
        from.into()
    }

    fn number_to_size(from: Self::Number) -> Option<Self::Size> {
        Some(from.into())
    }

    fn number_to_char(from: Self::Number) -> Option<Self::Char> {
        match from {
            SimpleNumber::Integer(v) => match (v as u8).try_into() {
                Ok(c) => Some(c),
                Err(_) => None,
            },
            SimpleNumber::Float(_) => None,
        }
    }

    fn number_to_byte(from: Self::Number) -> Option<Self::Byte> {
        match from {
            SimpleNumber::Integer(v) => match v.try_into() {
                Ok(b) => Some(b),
                Err(_) => None,
            },
            SimpleNumber::Float(_) => None,
        }
    }

    fn char_to_number(from: Self::Char) -> Option<Self::Number> {
        Some((from as i32).into())
    }

    fn char_to_byte(from: Self::Char) -> Option<Self::Byte> {
        Some(from as u8)
    }

    fn byte_to_number(from: Self::Byte) -> Option<Self::Number> {
        Some((from as i32).into())
    }

    fn byte_to_char(from: Self::Byte) -> Option<Self::Char> {
        Some(from.into())
    }

    //
    // Add Conversions
    //

    fn add_char_list_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
        self.start_char_list()?;
        self.add_to_current_char_list(from, 0)?;
        self.end_char_list()
    }

    fn add_byte_list_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
        match self.get_data_type(from)? {
            GarnishDataType::Unit => {
                self.start_byte_list()?;
                self.end_byte_list()
            }
            t => Err(DataError::from(format!("No cast to ByteList available for {:?}", t))),
        }
    }

    fn add_symbol_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
        let addr = self.add_char_list_from(from)?;
        match self.data.get(addr) {
            None => Err(DataError::from("No data after creating char list".to_string())),
            Some(data) => match data {
                SimpleData::CharList(s) => {
                    let v = symbol_value(s);
                    self.cache_add(SimpleData::Symbol(v))
                }
                t => Err(DataError::from(format!("Found {:?} instead of CharList after creating a CharList.", t))),
            },
        }
    }

    fn add_byte_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
        match self.get_data_type(from)? {
            GarnishDataType::CharList => {
                let len = self.get_char_list_len(from)?;
                let mut s = String::new();
                for i in 0..len {
                    let c = self.get_char_list_item(from, i.into())?;
                    s.push(c);
                }

                match s.parse::<u8>() {
                    Ok(v) => self.add_byte(v),
                    Err(_) => self.add_unit(),
                }
            }
            _ => self.add_unit(),
        }
    }

    fn add_number_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
        match self.get_data_type(from)? {
            GarnishDataType::CharList => {
                let len = self.get_char_list_len(from)?;
                let mut s = String::new();
                for i in 0..len {
                    let c = self.get_char_list_item(from, i.into())?;
                    s.push(c);
                }

                match s.parse::<i32>() {
                    Ok(v) => self.add_number(v.into()),
                    Err(_) => self.add_unit(),
                }
            }
            _ => self.add_unit(),
        }
    }

    //
    // Parsing
    //

    fn parse_number(from: &str) -> Result<Self::Number, Self::Error> {
        parse_simple_number(from)
    }

    fn parse_symbol(from: &str) -> Result<Self::Symbol, Self::Error> {
        Ok(symbol_value(from.trim_matches(':')))
    }

    fn parse_char(from: &str) -> Result<Self::Char, Self::Error> {
        let l = parse_char_list(from)?;
        if l.len() == 1 {
            Ok(l.chars().nth(0).unwrap())
        } else {
            Err(DataError::from(format!("Could not parse char from {:?}", from)))
        }
    }

    fn parse_byte(from: &str) -> Result<Self::Byte, Self::Error> {
        let l = parse_byte_list(from)?;
        if l.len() == 1 {
            Ok(l[0])
        } else {
            Err(DataError::from(format!("Could not parse byte from {:?}", from)))
        }
    }

    fn parse_char_list(from: &str) -> Result<Vec<Self::Char>, Self::Error> {
        Ok(parse_char_list(from)?.chars().collect())
    }

    fn parse_byte_list(from: &str) -> Result<Vec<Self::Byte>, Self::Error> {
        parse_byte_list(from)
    }

    // overrides

    fn parse_add_symbol(&mut self, from: &str) -> Result<Self::Size, Self::Error> {
        let sym = Self::parse_symbol(from)?;
        self.data.insert_symbol(sym, from.to_string());
        self.add_symbol(sym)
    }

    // iterator factories

    fn make_size_iterator_range(min: Self::Size, max: Self::Size) -> Self::SizeIterator {
        SizeIterator::new(min, max)
    }

    fn make_number_iterator_range(min: Self::Number, max: Self::Number) -> Self::NumberIterator {
        NumberIterator::new(min, max)
    }
}

#[cfg(test)]
mod tests {
    use garnish_lang_traits::GarnishData;

    use crate::{GarnishDataType, Instruction, SimpleGarnishData, SimpleNumber};

    #[test]
    fn type_of() {
        let mut runtime = SimpleGarnishData::new();
        runtime.add_number(10.into()).unwrap();

        assert_eq!(runtime.get_data_type(3).unwrap(), GarnishDataType::Number);
    }

    #[test]
    fn add_instruction() {
        let mut runtime = SimpleGarnishData::new();

        runtime.push_instruction(Instruction::Put, Some(0)).unwrap();

        assert_eq!(runtime.get_instructions().len(), 1);
    }

    #[test]
    fn get_instruction() {
        let mut runtime = SimpleGarnishData::new();

        runtime.push_instruction(Instruction::Put, None).unwrap();

        assert_eq!(runtime.get_instruction(0).unwrap().0, Instruction::Put);
    }

    #[test]
    fn get_current_instruction() {
        let mut runtime = SimpleGarnishData::new();

        runtime.push_instruction(Instruction::Put, None).unwrap();

        runtime.set_instruction_cursor(0).unwrap();

        assert_eq!(runtime.get_current_instruction().unwrap().0, Instruction::Put);
    }

    #[test]
    fn set_instruction_cursor() {
        let mut runtime = SimpleGarnishData::new();

        runtime.push_instruction(Instruction::Put, None).unwrap();
        runtime.push_instruction(Instruction::Put, None).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();

        runtime.set_instruction_cursor(2).unwrap();

        assert_eq!(runtime.get_current_instruction().unwrap().0, Instruction::Add);
    }

    #[test]
    fn pop_jump_path_clears_registers_to_current_frame() {
        let mut data = SimpleGarnishData::new();

        let data_addr = data.add_number(SimpleNumber::Integer(10)).unwrap(); // 3

        data.push_register(data_addr).unwrap();
        data.push_register(data_addr).unwrap();
        data.push_jump_path(10).unwrap();
        data.push_register(data_addr).unwrap();
        data.push_register(data_addr).unwrap();
        data.push_register(data_addr).unwrap();
        data.push_jump_path(20).unwrap();
        data.push_register(data_addr).unwrap();
        data.push_register(data_addr).unwrap();

        assert_eq!(data.register.len(), 9);

        let r = data.pop_jump_path();
        assert_eq!(r, Some(20));
        assert_eq!(data.register.len(), 6);

        let r = data.pop_jump_path();
        assert_eq!(r, Some(10));
        assert_eq!(data.register.len(), 2);

        let r = data.pop_jump_path();
        assert_eq!(r, None);

        assert_eq!(data.register.len(), 0);
    }

    #[test]
    fn pop_register_of_stack_frame_gives_error() {
        let mut data = SimpleGarnishData::new();

        let data_addr = data.add_number(SimpleNumber::Integer(10)).unwrap(); // 3

        data.push_register(data_addr).unwrap();
        data.push_register(data_addr).unwrap();
        data.push_jump_path(10).unwrap();

        let result = data.pop_register();

        assert!(result.is_err());
    }
}

#[cfg(test)]
mod add_data {
    mod parsing {
        use crate::SimpleDataRuntimeNC;
        use garnish_lang_traits::GarnishData;

        #[test]
        fn symbols_are_stripped_of_colon() {
            let s1 = SimpleDataRuntimeNC::parse_symbol(":my_symbol").unwrap();
            let s2 = SimpleDataRuntimeNC::parse_symbol("my_symbol").unwrap();

            assert_eq!(s1, s2);
        }
    }
    mod symbol_list {
        use crate::{SimpleDataRuntimeNC, SimpleGarnishData};
        use garnish_lang_traits::{GarnishData, GarnishDataType};

        fn s1() -> u64 {
            SimpleDataRuntimeNC::parse_symbol("symbol_one").unwrap()
        }

        fn s2() -> u64 {
            SimpleDataRuntimeNC::parse_symbol("symbol_two").unwrap()
        }

        fn s3() -> u64 {
            SimpleDataRuntimeNC::parse_symbol("symbol_three").unwrap()
        }

        fn s4() -> u64 {
            SimpleDataRuntimeNC::parse_symbol("symbol_four").unwrap()
        }

        #[test]
        fn add_symbol_list_with_symbol_symbol() {
            let mut runtime = SimpleGarnishData::new();
            let sym1 = runtime.add_symbol(s1()).unwrap();
            let sym2 = runtime.add_symbol(s2()).unwrap();

            let sym_list = runtime.merge_to_symbol_list(sym1, sym2).unwrap();

            assert_eq!(runtime.get_data_type(sym_list).unwrap(), GarnishDataType::SymbolList);
            let data = runtime.get_data().get(sym_list).unwrap().as_symbol_list().unwrap();

            assert_eq!(data, vec![s1(), s2()]);
        }

        #[test]
        fn add_symbol_list_with_symbol_list_symbol() {
            let mut runtime = SimpleGarnishData::new();
            let sym1 = runtime.add_symbol(s1()).unwrap();
            let sym2 = runtime.add_symbol(s2()).unwrap();
            let sym3 = runtime.add_symbol(s3()).unwrap();

            let sym_list1 = runtime.merge_to_symbol_list(sym1, sym2).unwrap();

            let sym_list2 = runtime.merge_to_symbol_list(sym_list1, sym3).unwrap();

            assert_eq!(runtime.get_data_type(sym_list2).unwrap(), GarnishDataType::SymbolList);
            let data = runtime.get_data().get(sym_list2).unwrap().as_symbol_list().unwrap();

            assert_eq!(data, vec![s1(), s2(), s3()]);
        }

        #[test]
        fn add_symbol_list_with_symbol_symbol_list() {
            let mut runtime = SimpleGarnishData::new();
            let sym1 = runtime.add_symbol(s1()).unwrap();
            let sym2 = runtime.add_symbol(s2()).unwrap();
            let sym3 = runtime.add_symbol(s3()).unwrap();

            let sym_list1 = runtime.merge_to_symbol_list(sym1, sym2).unwrap();

            let sym_list2 = runtime.merge_to_symbol_list(sym3, sym_list1).unwrap();

            assert_eq!(runtime.get_data_type(sym_list2).unwrap(), GarnishDataType::SymbolList);
            let data = runtime.get_data().get(sym_list2).unwrap().as_symbol_list().unwrap();

            assert_eq!(data, vec![s3(), s1(), s2()]);
        }

        #[test]
        fn add_symbol_list_with_symbol_list_symbol_list() {
            let mut runtime = SimpleGarnishData::new();
            let sym1 = runtime.add_symbol(s1()).unwrap();
            let sym2 = runtime.add_symbol(s2()).unwrap();
            let sym3 = runtime.add_symbol(s3()).unwrap();
            let sym4 = runtime.add_symbol(s4()).unwrap();

            let sym_list1 = runtime.merge_to_symbol_list(sym1, sym2).unwrap();

            let sym_list2 = runtime.merge_to_symbol_list(sym3, sym4).unwrap();

            let sym_list3 = runtime.merge_to_symbol_list(sym_list1, sym_list2).unwrap();

            assert_eq!(runtime.get_data_type(sym_list3).unwrap(), GarnishDataType::SymbolList);
            let data = runtime.get_data().get(sym_list3).unwrap().as_symbol_list().unwrap();

            assert_eq!(data, vec![s1(), s2(), s3(), s4()]);
        }

        #[test]
        fn add_symbol_list_with_number_symbol() {
            let mut runtime = SimpleGarnishData::new();
            let sym1 = runtime.add_number(10.into()).unwrap();
            let sym2 = runtime.add_symbol(s2()).unwrap();

            let sym_list = runtime.merge_to_symbol_list(sym1, sym2);

            assert!(sym_list.is_err());
        }

        #[test]
        fn add_symbol_list_with_symbol_number() {
            let mut runtime = SimpleGarnishData::new();
            let sym1 = runtime.add_symbol(s1()).unwrap();
            let sym2 = runtime.add_number(10.into()).unwrap();

            let sym_list = runtime.merge_to_symbol_list(sym1, sym2);

            assert!(sym_list.is_err());
        }

        #[test]
        fn add_symbol_list_with_number_number() {
            let mut runtime = SimpleGarnishData::new();
            let sym1 = runtime.add_number(10.into()).unwrap();
            let sym2 = runtime.add_number(10.into()).unwrap();

            let sym_list = runtime.merge_to_symbol_list(sym1, sym2);

            assert!(sym_list.is_err());
        }

        #[test]
        fn add_symbol_list_with_invalid_first_index() {
            let mut runtime = SimpleGarnishData::new();
            let sym2 = runtime.add_number(10.into()).unwrap();

            let sym_list = runtime.merge_to_symbol_list(100, sym2);

            assert!(sym_list.is_err());
        }

        #[test]
        fn add_symbol_list_with_invalid_second_index() {
            let mut runtime = SimpleGarnishData::new();
            let sym1 = runtime.add_number(10.into()).unwrap();

            let sym_list = runtime.merge_to_symbol_list(sym1, 200);

            assert!(sym_list.is_err());
        }

        #[test]
        fn add_symbol_list_with_invalid_indices() {
            let mut runtime = SimpleGarnishData::new();

            let sym_list = runtime.merge_to_symbol_list(100, 200);

            assert!(sym_list.is_err());
        }
    }
}

#[cfg(test)]
mod iterators {
    use crate::{SimpleData, SimpleGarnishData};
    use garnish_lang_traits::GarnishData;

    #[test]
    fn list_item_iterator() {
        let mut data = SimpleGarnishData::new();
        let list_index = data.get_data().len();
        data.get_data_mut().push(SimpleData::List(vec![100, 200, 300, 400, 500], vec![]));

        let mut iter = data.get_list_item_iter(list_index);

        assert_eq!(iter.next(), 100.into());
        assert_eq!(iter.next(), 200.into());
        assert_eq!(iter.next(), 300.into());
        assert_eq!(iter.next(), 400.into());
        assert_eq!(iter.next(), 500.into());
    }

    #[test]
    fn concatenation_item_iterator() {
        let mut data = SimpleGarnishData::new();
        let num1 = data.add_number(10.into()).unwrap();
        let num2 = data.add_number(10.into()).unwrap();
        let num3 = data.add_number(10.into()).unwrap();
        let list_index = data.get_data().len();
        data.get_data_mut().push(SimpleData::List(vec![num1, num2, num3], vec![]));
        let con1 = data.add_concatenation(list_index, 2).unwrap();
        let con2 = data.add_concatenation(con1, 1).unwrap();
        let con3 = data.add_concatenation(con2, 2).unwrap();

        let mut iter = data.get_concatenation_iter(con3);

        assert_eq!(iter.next(), num1.into());
        assert_eq!(iter.next(), num2.into());
        assert_eq!(iter.next(), num3.into());
        assert_eq!(iter.next(), 2.into());
        assert_eq!(iter.next(), 1.into());
        assert_eq!(iter.next(), 2.into());
    }
}
