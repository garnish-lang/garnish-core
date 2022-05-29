use std::collections::hash_map::DefaultHasher;
use std::convert::TryInto;
use std::fmt::Debug;
use std::hash::Hash;
use std::hash::Hasher;

use crate::simple::SimpleRuntimeData;
use crate::{
    parse_byte_list, parse_char_list, parse_simple_number, symbol_value, DataError, ExpressionDataType, GarnishLangRuntimeData, Instruction,
    InstructionData, SimpleData, SimpleNumber,
};

impl<T> GarnishLangRuntimeData for SimpleRuntimeData<T>
where
    T: Clone + Copy + PartialEq + Eq + PartialOrd + Debug + Hash,
{
    type Error = DataError;
    type Symbol = u64;
    type Char = char;
    type Byte = u8;
    type Number = SimpleNumber;
    type Size = usize;

    fn get_data_type(&self, index: usize) -> Result<ExpressionDataType, Self::Error> {
        let d = self.get(index)?;

        Ok(d.get_data_type())
    }

    fn get_number(&self, index: usize) -> Result<SimpleNumber, Self::Error> {
        self.get(index)?.as_number()
    }

    fn get_type(&self, addr: Self::Size) -> Result<ExpressionDataType, Self::Error> {
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

    fn get_link(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size, bool), Self::Error> {
        self.get(addr)?.as_link()
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
            SimpleNumber::Float(_) => Err(DataError::from(format!("Cannot index list with decimal value."))), // should return None
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
            SimpleNumber::Float(_) => Err(DataError::from(format!("Cannot index list with decimal value."))), // should return None
        }
    }

    fn get_char_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        Ok(self.get(addr)?.as_char_list()?.len())
    }

    fn get_char_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Self::Char, Self::Error> {
        match item_index {
            SimpleNumber::Integer(item_index) => match self.get(addr)?.as_char_list()?.chars().nth(item_index as usize) {
                None => Err(format!("No character at index {:?} for char list at {:?}", item_index, addr))?,
                Some(c) => Ok(c),
            },
            SimpleNumber::Float(_) => Err(DataError::from(format!("Cannot index char list with decimal value."))), // should return None
        }
    }

    fn get_byte_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        Ok(self.get(addr)?.as_byte_list()?.len())
    }

    fn get_byte_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Self::Byte, Self::Error> {
        match item_index {
            SimpleNumber::Integer(item_index) => match self.get(addr)?.as_byte_list()?.get(item_index as usize) {
                None => Err(format!("No character at index {:?} for char list at {:?}", item_index, addr))?,
                Some(c) => Ok(*c),
            },
            SimpleNumber::Float(_) => Err(DataError::from(format!("Cannot index byte list with decimal value."))), // should return None
        }
    }

    fn add_unit(&mut self) -> Result<usize, Self::Error> {
        Ok(0)
    }

    fn add_false(&mut self) -> Result<usize, Self::Error> {
        Ok(1)
    }

    fn add_true(&mut self) -> Result<usize, Self::Error> {
        Ok(2)
    }

    fn add_type(&mut self, value: ExpressionDataType) -> Result<Self::Size, Self::Error> {
        self.cache_add(SimpleData::Type(value))
    }

    fn add_number(&mut self, value: SimpleNumber) -> Result<usize, Self::Error> {
        self.cache_add(SimpleData::Number(value))
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
        self.data.push(SimpleData::Concatentaion(left, right));
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

    fn add_link(&mut self, value: Self::Size, linked: Self::Size, is_append: bool) -> Result<Self::Size, Self::Error> {
        self.data.push(SimpleData::Link(value, linked, is_append));
        Ok(self.data.len() - 1)
    }

    fn start_list(&mut self, _: usize) -> Result<(), Self::Error> {
        self.current_list = Some((vec![], vec![]));
        Ok(())
    }

    fn add_to_list(&mut self, addr: usize, is_associative: bool) -> Result<(), Self::Error> {
        match &mut self.current_list {
            None => Err(format!("Not currently creating a list."))?,
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
            None => Err(format!("Not currently creating a list."))?,
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
                            Err(format!("Could not place associative value"))?;
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
            None => Err(format!("Attempting to add to unstarted char list."))?,
            Some(s) => s.push(c),
        }

        Ok(())
    }

    fn end_char_list(&mut self) -> Result<Self::Size, Self::Error> {
        let data = match &self.current_char_list {
            None => Err(format!("Attempting to end unstarted char list."))?,
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
            None => Err(format!("Attempting to add to unstarted byte list."))?,
            Some(l) => l.push(c),
        }

        Ok(())
    }

    fn end_byte_list(&mut self) -> Result<Self::Size, Self::Error> {
        let data = match &self.current_byte_list {
            None => Err(format!("Attempting to end unstarted byte list."))?,
            Some(l) => SimpleData::ByteList(l.clone()),
        };

        let addr = self.cache_add(data)?;

        self.current_byte_list = None;

        Ok(addr)
    }

    fn get_list_item_with_symbol(&self, list_addr: usize, sym: u64) -> Result<Option<usize>, Self::Error> {
        let assocations_len = self.get_list_associations_len(list_addr)?;

        if assocations_len == 0 {
            return Ok(None);
        }

        let mut i = sym as usize % assocations_len;
        let mut count = 0;

        loop {
            // check to make sure item has same symbol
            let association_ref = self.get_list_association(list_addr, i.into())?;

            // should have symbol on left
            match self.get_data_type(association_ref)? {
                ExpressionDataType::Pair => {
                    let (left, right) = self.get_pair(association_ref)?;

                    let left_ref = left;

                    match self.get_data_type(left_ref)? {
                        ExpressionDataType::Symbol => {
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
            if i >= assocations_len {
                i = 0;
            }

            count += 1;
            if count > assocations_len {
                return Ok(None);
            }
        }
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

    fn pop_register(&mut self) -> Option<usize> {
        self.register.pop()
    }

    fn get_data_len(&self) -> usize {
        self.data.len()
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

    fn get_value_stack_len(&self) -> usize {
        self.values.len()
    }

    fn get_current_value(&self) -> Option<usize> {
        self.values.last().cloned()
    }

    fn get_current_value_mut(&mut self) -> Option<&mut usize> {
        self.values.last_mut()
    }

    fn push_instruction(&mut self, instruction: Instruction, data: Option<usize>) -> Result<usize, Self::Error> {
        self.instructions.push(InstructionData::new(instruction, data));
        Ok(self.instructions.len() - 1)
    }

    fn get_instruction(&self, index: usize) -> Option<(Instruction, Option<usize>)> {
        self.instructions.get(index).and_then(|i| Some((i.instruction, i.data)))
    }

    fn set_instruction_cursor(&mut self, index: usize) -> Result<(), Self::Error> {
        self.instruction_cursor = index;
        Ok(())
    }

    fn get_instruction_cursor(&self) -> usize {
        self.instruction_cursor
    }

    fn get_instruction_len(&self) -> usize {
        self.instructions.len()
    }

    fn push_jump_point(&mut self, index: usize) -> Result<(), Self::Error> {
        // if index >= self.instructions.len() {
        //     Err(format!(
        //         "Specified jump point {:?} is out of bounds of instructions with length {:?}",
        //         index,
        //         self.instructions.len()
        //     ))?;
        // }

        self.expression_table.push(index);
        Ok(())
    }

    fn get_jump_point(&self, index: usize) -> Option<usize> {
        self.expression_table.get(index).cloned()
    }

    fn get_jump_table_len(&self) -> usize {
        self.expression_table.len()
    }

    fn push_jump_path(&mut self, index: usize) -> Result<(), Self::Error> {
        self.jump_path.push(index);
        Ok(())
    }

    fn pop_jump_path(&mut self) -> Option<usize> {
        self.jump_path.pop()
    }

    fn get_jump_point_mut(&mut self, index: usize) -> Option<&mut usize> {
        self.expression_table.get_mut(index)
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
            ExpressionDataType::Unit => {
                self.start_byte_list()?;
                self.end_byte_list()
            }
            t => Err(DataError::from(format!("No cast to ByteList available for {:?}", t))),
        }
    }

    fn add_symbol_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
        let addr = self.add_char_list_from(from)?;
        let len = self.get_char_list_len(addr)?;
        let mut h = DefaultHasher::new();

        for i in 0..len {
            let c = self.get_char_list_item(addr, i.into())?;
            c.hash(&mut h);
        }
        let hv = h.finish();

        self.cache_add(SimpleData::Symbol(hv))
    }

    fn add_byte_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
        match self.get_data_type(from)? {
            ExpressionDataType::CharList => {
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
            ExpressionDataType::CharList => {
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
        Ok(symbol_value(from))
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
        self.symbols.insert(sym, from.to_string());
        self.add_symbol(sym)
    }
}

#[cfg(test)]
mod tests {
    use crate::simple::SimpleRuntimeData;
    use crate::{ExpressionDataType, GarnishLangRuntimeData, Instruction};

    #[test]
    fn type_of() {
        let mut runtime = SimpleRuntimeData::new();
        runtime.add_number(10.into()).unwrap();

        assert_eq!(runtime.get_data_type(3).unwrap(), ExpressionDataType::Number);
    }

    // #[test]
    // fn add_jump_point_out_of_bounds() {
    //     let mut runtime = SimpleRuntimeData::new();
    //
    //     runtime.push_instruction(Instruction::EndExpression, None).unwrap();
    //     let result = runtime.push_jump_point(5);
    //
    //     assert!(result.is_err());
    // }

    #[test]
    fn add_instruction() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.push_instruction(Instruction::Put, Some(0)).unwrap();

        assert_eq!(runtime.get_instructions().len(), 1);
    }

    #[test]
    fn get_instruction() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.push_instruction(Instruction::Put, None).unwrap();

        assert_eq!(runtime.get_instruction(0).unwrap().0, Instruction::Put);
    }

    #[test]
    fn get_current_instruction() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.push_instruction(Instruction::Put, None).unwrap();

        runtime.set_instruction_cursor(0).unwrap();

        assert_eq!(runtime.get_current_instruction().unwrap().0, Instruction::Put);
    }

    #[test]
    fn set_instruction_cursor() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.push_instruction(Instruction::Put, None).unwrap();
        runtime.push_instruction(Instruction::Put, None).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();

        runtime.set_instruction_cursor(2).unwrap();

        assert_eq!(runtime.get_current_instruction().unwrap().0, Instruction::Add);
    }
}
