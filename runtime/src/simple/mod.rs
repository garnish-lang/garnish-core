use std::collections::hash_map::DefaultHasher;
use std::convert::TryInto;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::{collections::HashMap, hash::Hasher};

use crate::{
    EmptyContext, ExpressionDataType, GarnishLangRuntimeContext, GarnishLangRuntimeData, GarnishLangRuntimeState, GarnishRuntime, Instruction,
    InstructionData, RuntimeError, SimpleData, SimpleDataList, SimpleNumber,
};

pub mod data;

pub fn symbol_value(value: &str) -> u64 {
    let mut h = DefaultHasher::new();
    value.hash(&mut h);
    let hv = h.finish();

    hv
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Eq, Debug, Hash)]
pub struct NoCustom {

}

#[derive(Debug)]
pub struct SimpleRuntimeData<T=NoCustom>
where
    T: Clone + PartialEq + Eq + PartialOrd + Debug + Hash,
{
    register: Vec<usize>,
    data: SimpleDataList<T>,
    end_of_constant_data: usize,
    values: Vec<usize>,
    instructions: Vec<InstructionData>,
    instruction_cursor: usize,
    expression_table: Vec<usize>,
    jump_path: Vec<usize>,
    current_list: Option<(Vec<usize>, Vec<usize>)>,
    current_char_list: Option<String>,
    current_byte_list: Option<Vec<u8>>,
    symbols: HashMap<u64, String>,
    cache: HashMap<u64, usize>,
    lease_stack: Vec<usize>,
    max_char_list_depth: usize,
}

// generic default not being inferred
// utility type for tests, and default implementations
pub type SimpleDataRuntimeNC = SimpleRuntimeData<NoCustom>;

impl SimpleRuntimeData<NoCustom> {
    pub fn new() -> Self {
        SimpleRuntimeData {
            register: vec![],
            data: SimpleDataList::default(),
            end_of_constant_data: 0,
            values: vec![],
            instruction_cursor: 0,
            instructions: vec![],
            expression_table: vec![],
            jump_path: vec![],
            current_list: None,
            current_char_list: None,
            current_byte_list: None,
            symbols: HashMap::new(),
            cache: HashMap::new(),
            lease_stack: vec![],
            max_char_list_depth: 1000,
        }
    }
}

impl<T> SimpleRuntimeData<T>
where
    T: Clone + PartialEq + Eq + PartialOrd + Debug + Hash,
{

    pub(crate) fn get(&self, index: usize) -> Result<&SimpleData<T>, DataError> {
        match self.data.get(index) {
            None => Err(format!("No data at addr {:?}", index))?,
            Some(d) => Ok(d),
        }
    }

    pub fn get_symbols(&self) -> &HashMap<u64, String> {
        &self.symbols
    }

    pub fn get_registers(&self) -> &Vec<usize> {
        &self.register
    }

    pub fn get_jump_path_vec(&self) -> &Vec<usize> {
        &self.jump_path
    }

    pub fn get_jump_points(&self) -> &Vec<usize> {
        &self.expression_table
    }

    pub fn get_instructions(&self) -> &Vec<InstructionData> {
        &self.instructions
    }

    pub fn get_data(&self) -> &SimpleDataList<T> {
        &self.data
    }

    pub fn execute_all_instructions(&mut self) -> Result<(), RuntimeError<DataError>> {
        loop {
            match self.execute_current_instruction::<EmptyContext>(None) {
                Err(e) => return Err(e),
                Ok(data) => match data.get_state() {
                    GarnishLangRuntimeState::Running => (),
                    GarnishLangRuntimeState::End => return Ok(()),
                },
            }
        }
    }

    pub fn execute_all_instructions_with_context<Context: GarnishLangRuntimeContext<Self>>(
        &mut self,
        context: &mut Context,
    ) -> Result<(), RuntimeError<DataError>> {
        loop {
            match self.execute_current_instruction(Some(context)) {
                Err(e) => return Err(e),
                Ok(data) => match data.get_state() {
                    GarnishLangRuntimeState::Running => (),
                    GarnishLangRuntimeState::End => return Ok(()),
                },
            }
        }
    }

    pub fn set_end_of_constant(&mut self, addr: usize) -> Result<(), DataError> {
        self.end_of_constant_data = addr;
        Ok(())
    }

    pub fn get_end_of_constant_data(&self) -> usize {
        self.end_of_constant_data
    }

    pub fn get_jump_path(&self, index: usize) -> Option<usize> {
        self.jump_path.get(index).cloned()
    }

    pub fn get_current_instruction(&self) -> Option<(Instruction, Option<usize>)> {
        self.get_instruction(self.get_instruction_cursor())
    }

    pub fn advance_instruction_cursor(&mut self) -> Result<(), String> {
        self.instruction_cursor += 1;
        Ok(())
    }

    fn cache_add(&mut self, value: SimpleData<T>) -> Result<usize, DataError> {
        let mut h = DefaultHasher::new();
        value.hash(&mut h);
        value.get_data_type().hash(&mut h);
        let hv = h.finish();

        match self.cache.get(&hv) {
            Some(addr) => Ok(*addr),
            None => {
                let addr = self.data.len();
                self.data.push(value);
                self.cache.insert(hv, addr);
                Ok(addr)
            }
        }
    }

    fn add_to_current_char_list(&mut self, from: usize, depth: usize) -> Result<(), DataError> {
        if depth >= self.max_char_list_depth {
            return Ok(());
        }

        match self.get_data_type(from)? {
            ExpressionDataType::Custom => todo!(),
            ExpressionDataType::Unit => {
                self.add_to_char_list('(')?;
                self.add_to_char_list(')')?;
            }
            ExpressionDataType::True => {
                self.add_to_char_list('$')?;
                self.add_to_char_list('?')?;
            }
            ExpressionDataType::False => {
                self.add_to_char_list('$')?;
                self.add_to_char_list('!')?;
            }
            ExpressionDataType::Type => {
                let s = format!("{:?}", self.get_type(from)?);
                for c in s.chars() {
                    self.add_to_char_list(c)?;
                }
            }
            ExpressionDataType::Number => {
                let x = self.get_number(from)?;
                let s = x.to_string();
                for c in s.chars() {
                    self.add_to_char_list(c)?;
                }
            }
            ExpressionDataType::Char => {
                let c = self.get_char(from)?;
                self.add_to_char_list(c)?;
            }
            ExpressionDataType::CharList => {
                let len = self.get_char_list_len(from)?;
                for i in 0..len {
                    let c = self.get_char_list_item(from, i.into())?;
                    self.add_to_char_list(c)?;
                }
            }
            ExpressionDataType::Byte => {
                let b = self.get_byte(from)?;
                let s = b.to_string();
                self.start_char_list()?;
                for c in s.chars() {
                    self.add_to_char_list(c)?;
                }
            }
            ExpressionDataType::ByteList => {
                let len = self.get_byte_list_len(from)?;
                let mut strs = vec![];
                for i in 0..len {
                    let b = self.get_byte_list_item(from, i.into())?;
                    strs.push(format!("'{}'", b));
                }
                let s = strs.join(" ");
                self.start_char_list()?;
                for c in s.chars() {
                    self.add_to_char_list(c)?;
                }
            }
            ExpressionDataType::Symbol => {
                let sym = self.get_symbol(from)?;
                let s = match self.symbols.get(&sym) {
                    None => sym.to_string(),
                    Some(s) => s.clone(),
                };

                for c in s.chars() {
                    self.add_to_char_list(c)?;
                }
            }
            ExpressionDataType::Expression => {
                let e = self.get_expression(from)?;
                let s = format!("Expression({})", e);

                for c in s.chars() {
                    self.add_to_char_list(c)?;
                }
            }
            ExpressionDataType::External => {
                let e = self.get_external(from)?;
                let s = format!("External({})", e);

                for c in s.chars() {
                    self.add_to_char_list(c)?;
                }
            }
            ExpressionDataType::Range => {
                let (start, end) = self
                    .get_range(from)
                    .and_then(|(start, end)| Ok((self.get_number(start)?, self.get_number(end)?)))?;

                let s = format!("{}..{}", start, end);

                for c in s.chars() {
                    self.add_to_char_list(c)?;
                }
            }
            ExpressionDataType::Pair => {
                let (left, right) = self.get_pair(from)?;
                if depth > 0 {
                    self.add_to_char_list('(')?;
                }
                self.add_to_current_char_list(left, depth + 1)?;
                self.add_to_char_list(' ')?;
                self.add_to_char_list('=')?;
                self.add_to_char_list(' ')?;
                self.add_to_current_char_list(right, depth + 1)?;
                if depth > 0 {
                    self.add_to_char_list(')')?;
                }
            }
            ExpressionDataType::List => {
                let len = self.get_list_len(from)?;

                if depth > 0 {
                    self.add_to_char_list('(')?;
                }

                for i in 0..len {
                    let item = self.get_list_item(from, i.into())?;
                    self.add_to_current_char_list(item, depth + 1)?;

                    if i < len - 1 {
                        self.add_to_char_list(',')?;
                        self.add_to_char_list(' ')?;
                    }
                }

                if depth > 0 {
                    self.add_to_char_list(')')?;
                }
            }
            ExpressionDataType::Link => {
                let (value, linked, is_append) = self.get_link(from)?;

                match self.get_data_type(linked)? {
                    ExpressionDataType::Unit => {
                        self.add_to_current_char_list(value, depth + 1)?;
                    }
                    ExpressionDataType::Link => {
                        if is_append {
                            self.add_to_current_char_list(linked, depth + 1)?;
                            self.add_to_char_list(' ')?;
                            self.add_to_char_list('-')?;
                            self.add_to_char_list('>')?;
                            self.add_to_char_list(' ')?;
                            self.add_to_current_char_list(value, depth + 1)?;
                        } else {
                            self.add_to_current_char_list(value, depth + 1)?;
                            self.add_to_char_list(' ')?;
                            self.add_to_char_list('<')?;
                            self.add_to_char_list('-')?;
                            self.add_to_char_list(' ')?;
                            self.add_to_current_char_list(linked, depth + 1)?;
                        }
                    }
                    t => Err(DataError::from(format!("Invalid linked type {:?}", t)))?,
                }
            }
            ExpressionDataType::Slice => {
                let (value, range) = self.get_slice(from)?;
                self.add_to_current_char_list(value, depth + 1)?;
                self.add_to_char_list(' ')?;
                self.add_to_char_list('~')?;
                self.add_to_char_list(' ')?;
                self.add_to_current_char_list(range, depth + 1)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct DataError {
    message: String,
}

impl Display for DataError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message.as_str())
    }
}

impl std::error::Error for DataError {}

impl From<String> for DataError {
    fn from(s: String) -> Self {
        DataError { message: s }
    }
}

impl<T> GarnishLangRuntimeData for SimpleRuntimeData<T>
where
    T: Clone + PartialEq + Eq + PartialOrd + Debug + Hash,
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
        match from.parse::<i32>() {
            Ok(v) => Ok(v.into()),
            Err(_) => Err(DataError::from(format!("Could not parse number from {:?}", from))),
        }
    }

    fn parse_symbol(from: &str) -> Result<Self::Symbol, Self::Error> {
        Ok(symbol_value(from))
    }

    fn parse_char(from: &str) -> Result<Self::Char, Self::Error> {
        let l = SimpleRuntimeData::<T>::parse_char_list(from)?;
        if l.len() == 1 {
            Ok(l[0])
        } else {
            Err(DataError::from(format!("Could not parse char from {:?}", from)))
        }
    }

    fn parse_byte(from: &str) -> Result<Self::Byte, Self::Error> {
        let l = SimpleRuntimeData::<T>::parse_byte_list(from)?;
        if l.len() == 1 {
            Ok(l[0])
        } else {
            Err(DataError::from(format!("Could not parse byte from {:?}", from)))
        }
    }

    fn parse_char_list(from: &str) -> Result<Vec<Self::Char>, Self::Error> {
        Ok(from.trim_matches('"').chars().collect())
    }

    fn parse_byte_list(from: &str) -> Result<Vec<Self::Byte>, Self::Error> {
        Ok(from.trim_matches('\'').bytes().collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::{ExpressionDataType, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

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

#[cfg(test)]
mod data_storage {
    use crate::{GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn unit() {
        let mut runtime = SimpleRuntimeData::new();

        assert_eq!(runtime.add_unit().unwrap(), 0);
        assert_eq!(runtime.add_unit().unwrap(), 0);
        assert_eq!(runtime.add_unit().unwrap(), 0);

        assert_eq!(runtime.get_data_len(), 3);
        assert!(runtime.data.get(0).unwrap().is_unit());
    }

    #[test]
    fn false_data() {
        let mut runtime = SimpleRuntimeData::new();

        assert_eq!(runtime.add_false().unwrap(), 1);
        assert_eq!(runtime.add_false().unwrap(), 1);
        assert_eq!(runtime.add_false().unwrap(), 1);

        assert_eq!(runtime.get_data_len(), 3);
        assert!(runtime.data.get(1).unwrap().is_false());
    }

    #[test]
    fn true_data() {
        let mut runtime = SimpleRuntimeData::new();

        assert_eq!(runtime.add_true().unwrap(), 2);
        assert_eq!(runtime.add_true().unwrap(), 2);
        assert_eq!(runtime.add_true().unwrap(), 2);

        assert_eq!(runtime.get_data_len(), 3);
        assert!(runtime.data.get(2).unwrap().is_true());
    }

    #[test]
    fn integers() {
        let mut runtime = SimpleRuntimeData::new();

        let start = runtime.get_data_len();
        let i1 = runtime.add_number(10.into()).unwrap();
        let i2 = runtime.add_number(20.into()).unwrap();
        let i3 = runtime.add_number(10.into()).unwrap();

        assert_eq!(i1, start);
        assert_eq!(i2, start + 1);
        assert_eq!(i3, i1);

        assert_eq!(runtime.get_data_len(), 5);
        assert_eq!(runtime.data.get(3).unwrap().as_number().unwrap(), 10.into());
        assert_eq!(runtime.data.get(4).unwrap().as_number().unwrap(), 20.into());
    }

    #[test]
    fn symbols() {
        let mut runtime = SimpleRuntimeData::new();

        let start = runtime.get_data_len();
        let i1 = runtime.add_symbol(1).unwrap();
        let i2 = runtime.add_symbol(2).unwrap();
        let i3 = runtime.add_symbol(1).unwrap();

        assert_eq!(i1, start);
        assert_eq!(i2, start + 1);
        assert_eq!(i3, i1);

        assert_eq!(runtime.get_data_len(), 5);
        assert_eq!(runtime.data.get(3).unwrap().as_symbol().unwrap(), 1);
        assert_eq!(runtime.data.get(4).unwrap().as_symbol().unwrap(), 2);
    }

    #[test]
    fn expression() {
        let mut runtime = SimpleRuntimeData::new();

        let start = runtime.get_data_len();
        let i1 = runtime.add_expression(10).unwrap();
        let i2 = runtime.add_expression(20).unwrap();
        let i3 = runtime.add_expression(10).unwrap();

        assert_eq!(i1, start);
        assert_eq!(i2, start + 1);
        assert_eq!(i3, i1);

        assert_eq!(runtime.get_data_len(), 5);
        assert_eq!(runtime.data.get(3).unwrap().as_expression().unwrap(), 10);
        assert_eq!(runtime.data.get(4).unwrap().as_expression().unwrap(), 20);
    }

    #[test]
    fn external() {
        let mut runtime = SimpleRuntimeData::new();

        let start = runtime.get_data_len();
        let i1 = runtime.add_external(10).unwrap();
        let i2 = runtime.add_external(20).unwrap();
        let i3 = runtime.add_external(10).unwrap();

        assert_eq!(i1, start);
        assert_eq!(i2, start + 1);
        assert_eq!(i3, i1);

        assert_eq!(runtime.get_data_len(), 5);
        assert_eq!(runtime.data.get(3).unwrap().as_external().unwrap(), 10);
        assert_eq!(runtime.data.get(4).unwrap().as_external().unwrap(), 20);
    }

    #[test]
    fn similar_values_cache_differently() {
        let mut runtime = SimpleRuntimeData::new();

        let start = runtime.get_data_len();
        let i1 = runtime.add_external(10).unwrap();
        let i2 = runtime.add_expression(10).unwrap();

        assert_eq!(i1, start);
        assert_eq!(i2, start + 1);
        assert_ne!(i1, i2);

        assert_eq!(runtime.get_data_len(), 5);
        assert_eq!(runtime.data.get(3).unwrap().as_external().unwrap(), 10);
        assert_eq!(runtime.data.get(4).unwrap().as_expression().unwrap(), 10);
    }
}

#[cfg(test)]
mod to_symbol {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hash;
    use std::hash::Hasher;

    use crate::{GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn unit() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_unit().unwrap();
        let addr = runtime.add_symbol_from(d1).unwrap();

        let mut h = DefaultHasher::new();
        for c in "()".chars() {
            c.hash(&mut h);
        }

        let val = h.finish();
        assert_eq!(runtime.get_symbol(addr).unwrap(), val);
    }
}

#[cfg(test)]
mod to_byte_list {
    use crate::{GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn unit() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_unit().unwrap();
        let addr = runtime.add_byte_list_from(d1).unwrap();
        let len = runtime.get_byte_list_len(addr).unwrap();

        assert_eq!(len, 0);
    }
}

#[cfg(test)]
mod to_char_list {
    use crate::{ExpressionDataType, GarnishLangRuntimeData, SimpleRuntimeData};
    use crate::simple::NoCustom;

    fn assert_to_char_list<Func>(expected: &str, setup: Func)
    where
        Func: FnOnce(&mut SimpleRuntimeData) -> usize,
    {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = setup(&mut runtime);

        let addr = runtime.add_char_list_from(d1).unwrap();
        let len = runtime.get_char_list_len(addr).unwrap();

        let mut chars = String::new();

        for i in 0..len {
            let c = runtime.get_char_list_item(addr, i.into()).unwrap();
            chars.push(c);
        }

        assert_eq!(chars, expected, "{:?} != {:?}", chars, expected);
    }

    #[test]
    fn unit() {
        assert_to_char_list("()", |runtime| runtime.add_unit().unwrap())
    }

    #[test]
    fn true_boolean() {
        assert_to_char_list("$?", |runtime| runtime.add_true().unwrap())
    }

    #[test]
    fn false_boolean() {
        assert_to_char_list("$!", |runtime| runtime.add_false().unwrap())
    }

    #[test]
    fn integer() {
        assert_to_char_list("10", |runtime| runtime.add_number(10.into()).unwrap())
    }

    #[test]
    fn char() {
        assert_to_char_list("c", |runtime| runtime.add_char('c').unwrap())
    }

    #[test]
    fn type_data() {
        assert_to_char_list("Unit", |runtime| runtime.add_type(ExpressionDataType::Unit).unwrap())
    }

    #[test]
    fn char_list() {
        assert_to_char_list("characters", |runtime| {
            runtime.start_char_list().unwrap();
            runtime.add_to_char_list('c').unwrap();
            runtime.add_to_char_list('h').unwrap();
            runtime.add_to_char_list('a').unwrap();
            runtime.add_to_char_list('r').unwrap();
            runtime.add_to_char_list('a').unwrap();
            runtime.add_to_char_list('c').unwrap();
            runtime.add_to_char_list('t').unwrap();
            runtime.add_to_char_list('e').unwrap();
            runtime.add_to_char_list('r').unwrap();
            runtime.add_to_char_list('s').unwrap();
            runtime.end_char_list().unwrap()
        })
    }

    #[test]
    fn byte() {
        assert_to_char_list("100", |runtime| runtime.add_byte(100).unwrap())
    }

    #[test]
    fn byte_list() {
        assert_to_char_list("'10' '20' '30'", |runtime| {
            runtime.start_byte_list().unwrap();
            runtime.add_to_byte_list(10).unwrap();
            runtime.add_to_byte_list(20).unwrap();
            runtime.add_to_byte_list(30).unwrap();
            runtime.end_byte_list().unwrap()
        })
    }

    #[test]
    fn symbol() {
        let s = SimpleRuntimeData::<NoCustom>::parse_symbol("my_symbol").unwrap().to_string();
        assert_to_char_list(s.as_str(), |runtime| {
            runtime.add_symbol(SimpleRuntimeData::<NoCustom>::parse_symbol("my_symbol").unwrap()).unwrap()
        })
    }

    #[test]
    fn expression() {
        assert_to_char_list("Expression(10)", |runtime| runtime.add_expression(10).unwrap())
    }

    #[test]
    fn external() {
        assert_to_char_list("External(10)", |runtime| runtime.add_external(10).unwrap())
    }

    #[test]
    fn range() {
        assert_to_char_list("5..10", |runtime| {
            let d1 = runtime.add_number(5.into()).unwrap();
            let d2 = runtime.add_number(10.into()).unwrap();
            runtime.add_range(d1, d2).unwrap()
        })
    }

    #[test]
    fn pair() {
        assert_to_char_list("10 = 10", |runtime| {
            let d1 = runtime.add_number(10.into()).unwrap();
            let d2 = runtime.add_number(10.into()).unwrap();
            runtime.add_pair((d1, d2)).unwrap()
        })
    }

    #[test]
    fn pair_nested() {
        assert_to_char_list("10 = (20 = 30)", |runtime| {
            let d1 = runtime.add_number(10.into()).unwrap();
            let d2 = runtime.add_number(20.into()).unwrap();
            let d3 = runtime.add_number(30.into()).unwrap();
            let d4 = runtime.add_pair((d2, d3)).unwrap();
            runtime.add_pair((d1, d4)).unwrap()
        })
    }

    #[test]
    fn pair_nested_two() {
        assert_to_char_list("10 = (20 = (30 = 40))", |runtime| {
            let d1 = runtime.add_number(10.into()).unwrap();
            let d2 = runtime.add_number(20.into()).unwrap();
            let d3 = runtime.add_number(30.into()).unwrap();
            let d4 = runtime.add_number(40.into()).unwrap();
            let d5 = runtime.add_pair((d3, d4)).unwrap();
            let d6 = runtime.add_pair((d2, d5)).unwrap();
            runtime.add_pair((d1, d6)).unwrap()
        })
    }

    #[test]
    fn list() {
        assert_to_char_list("10, 20, 30", |runtime| {
            let d1 = runtime.add_number(10.into()).unwrap();
            let d2 = runtime.add_number(20.into()).unwrap();
            let d3 = runtime.add_number(30.into()).unwrap();
            runtime.start_list(3).unwrap();
            runtime.add_to_list(d1, false).unwrap();
            runtime.add_to_list(d2, false).unwrap();
            runtime.add_to_list(d3, false).unwrap();
            runtime.end_list().unwrap()
        })
    }

    #[test]
    fn list_nested() {
        assert_to_char_list("10, (20, 30), 40", |runtime| {
            let d1 = runtime.add_number(10.into()).unwrap();
            let d2 = runtime.add_number(20.into()).unwrap();
            let d3 = runtime.add_number(30.into()).unwrap();
            let d4 = runtime.add_number(40.into()).unwrap();

            runtime.start_list(2).unwrap();
            runtime.add_to_list(d2, false).unwrap();
            runtime.add_to_list(d3, false).unwrap();
            let list = runtime.end_list().unwrap();

            runtime.start_list(3).unwrap();
            runtime.add_to_list(d1, false).unwrap();
            runtime.add_to_list(list, false).unwrap();
            runtime.add_to_list(d4, false).unwrap();
            runtime.end_list().unwrap()
        })
    }

    #[test]
    fn link_append() {
        assert_to_char_list("10 -> 20", |runtime| {
            let unit = runtime.add_unit().unwrap();
            let d1 = runtime.add_number(10.into()).unwrap();
            let d2 = runtime.add_number(20.into()).unwrap();
            let link1 = runtime.add_link(d1, unit, true).unwrap();
            runtime.add_link(d2, link1, true).unwrap()
        })
    }

    #[test]
    fn link_prepend() {
        assert_to_char_list("10 <- 20", |runtime| {
            let unit = runtime.add_unit().unwrap();
            let d1 = runtime.add_number(10.into()).unwrap();
            let d2 = runtime.add_number(20.into()).unwrap();
            let link1 = runtime.add_link(d2, unit, false).unwrap();
            runtime.add_link(d1, link1, false).unwrap()
        })
    }

    #[test]
    fn link_append_multiple() {
        assert_to_char_list("10 -> 20 -> 30", |runtime| {
            let unit = runtime.add_unit().unwrap();
            let d1 = runtime.add_number(10.into()).unwrap();
            let d2 = runtime.add_number(20.into()).unwrap();
            let d3 = runtime.add_number(30.into()).unwrap();

            let link1 = runtime.add_link(d1, unit, true).unwrap();
            let link2 = runtime.add_link(d2, link1, true).unwrap();
            runtime.add_link(d3, link2, true).unwrap()
        })
    }

    #[test]
    fn link_prepend_multiple() {
        assert_to_char_list("10 <- 20 <- 30", |runtime| {
            let unit = runtime.add_unit().unwrap();
            let d1 = runtime.add_number(10.into()).unwrap();
            let d2 = runtime.add_number(20.into()).unwrap();
            let d3 = runtime.add_number(30.into()).unwrap();

            let link1 = runtime.add_link(d3, unit, false).unwrap();
            let link2 = runtime.add_link(d2, link1, false).unwrap();
            runtime.add_link(d1, link2, false).unwrap()
        })
    }

    #[test]
    fn slice() {
        assert_to_char_list("(10, 20, 30) ~ 5..10", |runtime| {
            let d1 = runtime.add_number(5.into()).unwrap();
            let d2 = runtime.add_number(10.into()).unwrap();
            let d3 = runtime.add_number(20.into()).unwrap();
            let d4 = runtime.add_number(30.into()).unwrap();

            runtime.start_list(3).unwrap();
            runtime.add_to_list(d2, false).unwrap();
            runtime.add_to_list(d3, false).unwrap();
            runtime.add_to_list(d4, false).unwrap();
            let list = runtime.end_list().unwrap();

            let range = runtime.add_range(d1, d2).unwrap();

            runtime.add_slice(list, range).unwrap()
        })
    }
}
