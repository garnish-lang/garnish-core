use crate::SimpleNumber;
use crate::data::{DisplayForCustomItem, SimpleDataList, SimpleStackFrame, UNIT_INDEX};
use crate::error::DataError;
use crate::instruction::SimpleInstruction;
use garnish_lang_traits::helpers::iterate_concatenation_mut;
use garnish_lang_traits::{Extents, GarnishData, GarnishDataType, Instruction, SymbolListPart, TypeConstants};
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::{collections::HashMap, hash::Hasher};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub trait SimpleDataType: Clone + PartialEq + Eq + PartialOrd + Debug + Hash {}

/// Default custom type for [`SimpleGarnishData`] when no custom types are needed.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialOrd, PartialEq, Eq, Debug, Hash)]
pub struct NoCustom {}

impl SimpleDataType for NoCustom {}

impl Display for NoCustom {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("NoCustom")
    }
}

impl DisplayForCustomItem for NoCustom {
    fn display_with_list(&self, _list: &SimpleDataList, _level: usize) -> String {
        format!("{}", self)
    }
}

pub fn default_resolver<T, A>(_data: &mut SimpleGarnishData<T, A>, _symbol: u64) -> Result<bool, DataError>
where
    T: SimpleDataType,
{
    Ok(false)
}

pub fn default_op_handler<T, A>(
    _data: &mut SimpleGarnishData<T, A>,
    _instruction: Instruction,
    _left: (GarnishDataType, usize),
    _right: (GarnishDataType, usize),
) -> Result<bool, DataError>
where
    T: SimpleDataType,
{
    Ok(false)
}

pub type SimpleResolver<T, A> = fn(&mut SimpleGarnishData<T, A>, u64) -> Result<bool, DataError>;
pub type SimpleOpHandler<T, A> =
    fn(&mut SimpleGarnishData<T, A>, Instruction, (GarnishDataType, usize), (GarnishDataType, usize)) -> Result<bool, DataError>;

/// Implementation of [`GarnishData`]. Uses standard Rust collections for storing data.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct SimpleGarnishData<T = NoCustom, A = ()>
where
    T: SimpleDataType,
{
    pub(crate) register: Vec<usize>,
    pub(crate) data: SimpleDataList<T>,
    pub(crate) end_of_constant_data: usize,
    pub(crate) values: Vec<usize>,
    pub(crate) instructions: Vec<SimpleInstruction>,
    pub(crate) instruction_cursor: usize,
    pub(crate) expression_table: Vec<usize>,
    pub(crate) current_list: Option<(Vec<usize>, Vec<usize>)>,
    pub(crate) current_char_list: Option<String>,
    pub(crate) current_byte_list: Option<Vec<u8>>,
    pub(crate) cache: HashMap<u64, usize>,
    pub(crate) max_char_list_depth: usize,
    pub(crate) resolver: SimpleResolver<T, A>,
    pub(crate) op_handler: SimpleOpHandler<T, A>,
    pub(crate) auxiliary_data: A,
}

/// Alias for [`SimpleGarnishData`] with [`NoCustom`] type parameter.
pub type SimpleDataRuntimeNC = SimpleGarnishData<NoCustom>;

impl SimpleGarnishData<NoCustom> {
    pub fn new() -> Self {
        SimpleGarnishData {
            register: vec![],
            data: SimpleDataList::default(),
            end_of_constant_data: 0,
            values: vec![],
            instruction_cursor: 0,
            instructions: vec![],
            expression_table: vec![],
            current_list: None,
            current_char_list: None,
            current_byte_list: None,
            cache: HashMap::new(),
            max_char_list_depth: 1000,
            resolver: default_resolver,
            op_handler: default_op_handler,
            auxiliary_data: (),
        }
    }
}

impl<T, A> SimpleGarnishData<T, A>
where
    T: SimpleDataType,
    A: Default,
{
    pub fn new_custom() -> Self {
        let data = SimpleDataList::<T>::default();
        SimpleGarnishData {
            register: vec![],
            end_of_constant_data: data.len() - 1,
            data,
            values: vec![],
            instruction_cursor: 0,
            instructions: vec![],
            expression_table: vec![],
            current_list: None,
            current_char_list: None,
            current_byte_list: None,
            cache: HashMap::new(),
            max_char_list_depth: 1000,
            resolver: default_resolver,
            op_handler: default_op_handler,
            auxiliary_data: A::default(),
        }
    }

    pub fn auxiliary_data(&self) -> &A {
        &self.auxiliary_data
    }

    pub fn auxiliary_data_mut(&mut self) -> &mut A {
        &mut self.auxiliary_data
    }

    pub fn set_resolver(&mut self, resolver: SimpleResolver<T, A>) {
        self.resolver = resolver;
    }

    pub fn call_resolver(&mut self, symbol: u64) -> Result<bool, DataError> {
        (self.resolver)(self, symbol)
    }

    pub fn set_op_handler(&mut self, op_handler: SimpleOpHandler<T, A>) {
        self.op_handler = op_handler;
    }

    pub fn call_op_handler(
        &mut self,
        instruction: Instruction,
        left: (GarnishDataType, usize),
        right: (GarnishDataType, usize),
    ) -> Result<bool, DataError> {
        (self.op_handler)(self, instruction, left, right)
    }

    pub(crate) fn get(&self, index: usize) -> Result<&crate::data::SimpleData<T>, DataError> {
        match self.data.get(index) {
            None => Err(format!("No data at addr {:?}", index))?,
            Some(d) => Ok(d),
        }
    }

    pub fn add(&mut self, data: crate::data::SimpleData<T>) -> Result<usize, DataError> {
        self.cache_add(data)
    }

    pub fn add_custom(&mut self, data: T) -> Result<usize, DataError> {
        self.data.push(crate::data::SimpleData::Custom(data));
        Ok(self.data.len() - 1)
    }

    pub fn add_stack_frame(&mut self, frame: SimpleStackFrame) -> Result<usize, DataError> {
        self.data.push(crate::data::SimpleData::StackFrame(frame));
        Ok(self.data.len() - 1)
    }

    pub fn add_string(&mut self, s: impl Into<String>) -> Result<usize, DataError> {
        self.data.push(crate::data::SimpleData::CharList(s.into()));
        Ok(self.data.len() - 1)
    }

    pub fn add_u8_vec(&mut self, vec: impl Into<Vec<u8>>) -> Result<usize, DataError> {
        self.data.push(crate::data::SimpleData::ByteList(vec.into()));
        Ok(self.data.len() - 1)
    }

    pub fn add_symbol_list(&mut self, list: impl Into<Vec<u64>>) -> Result<usize, DataError> {
        self.data.push(crate::data::SimpleData::SymbolList(list.into()));
        Ok(self.data.len() - 1)
    }

    pub fn add_pair_from(&mut self, left: crate::data::SimpleData<T>, right: crate::data::SimpleData<T>) -> Result<usize, DataError> {
        let left = self.cache_add(left)?;
        let right = self.cache_add(right)?;
        self.add_pair((left, right))
    }

    pub fn add_plain_list_from(&mut self, list: Vec<crate::data::SimpleData<T>>) -> Result<usize, DataError> {
        let mut items = vec![];

        for item in list {
            let i = self.cache_add(item.into())?;
            items.push(i);
        }

        self.cache_add(crate::data::SimpleData::List(items, vec![]))
    }

    pub fn add_associative_list_from(&mut self, list: Vec<(impl Into<String>, crate::data::SimpleData<T>)>) -> Result<usize, DataError> {
        self.start_list(list.len())?;
        for (key, value) in list {
            let symbol = self.parse_add_symbol(&key.into())?;
            let i = self.cache_add(value.into())?;
            let pair = self.add_pair((symbol, i))?;
            self.add_to_list(pair, true)?;
        }
        self.end_list()
    }

    pub fn add_concatenation_from(
        &mut self,
        first: crate::data::SimpleData<T>,
        second: crate::data::SimpleData<T>,
        additional: Vec<crate::data::SimpleData<T>>,
    ) -> Result<usize, DataError> {
        let first = self.cache_add(first.into())?;
        let second = self.cache_add(second.into())?;
        let mut current = self.add_concatenation(first, second)?;

        for item in additional {
            let next = self.cache_add(item)?;
            current = self.add_concatenation(current, next)?;
        }

        Ok(current)
    }

    pub fn get_custom(&self, addr: usize) -> Result<T, DataError> {
        self.get(addr)?.as_custom()
    }

    pub fn get_symbols(&self) -> &HashMap<u64, String> {
        self.data.symbol_to_name()
    }

    pub fn get_registers(&self) -> &Vec<usize> {
        &self.register
    }

    pub fn get_jump_points(&self) -> &Vec<usize> {
        &self.expression_table
    }

    pub fn get_instructions(&self) -> &Vec<SimpleInstruction> {
        &self.instructions
    }

    pub fn get_data(&self) -> &SimpleDataList<T> {
        &self.data
    }

    pub fn get_data_mut(&mut self) -> &mut SimpleDataList<T> {
        &mut self.data
    }

    pub fn get_raw_data(&self, index: usize) -> Option<crate::data::SimpleData<T>> {
        self.data.get(index).cloned()
    }

    pub fn set_end_of_constant(&mut self, addr: usize) -> Result<(), DataError> {
        if addr >= self.data.len() {
            Err(DataError::from(
                "Cannot set end of constant data to be over current data amount".to_string(),
            ))
        } else {
            self.end_of_constant_data = addr;
            Ok(())
        }
    }

    pub fn get_end_of_constant_data(&self) -> usize {
        self.end_of_constant_data
    }

    pub fn get_current_instruction(&self) -> Option<(Instruction, Option<usize>)> {
        self.get_instruction(self.get_instruction_cursor())
    }

    pub fn advance_instruction_cursor(&mut self) -> Result<(), String> {
        self.instruction_cursor += 1;
        Ok(())
    }

    pub fn display_current_value(&self) -> String
    where
        T: Display + DisplayForCustomItem,
    {
        self.values
            .last()
            .and_then(|l| Some(self.data.display_for_item(*l)))
            .unwrap_or("<NoData>".to_string())
    }

    pub fn collect_concatenation_indices(&self, left: usize, right: usize) -> Result<Vec<usize>, DataError> {
        let mut items = vec![];
        let mut con_stack = vec![right, left];

        while let Some(item) = con_stack.pop() {
            match self.get_data().get(item) {
                None => items.push(UNIT_INDEX),
                Some(crate::data::SimpleData::Concatenation(left, right)) => {
                    con_stack.push(right.clone());
                    con_stack.push(left.clone());
                }
                Some(crate::data::SimpleData::List(list_items, _)) => {
                    for item in list_items {
                        items.push(item.clone());
                    }
                }
                Some(crate::data::SimpleData::Slice(list, range)) => match (self.get_data().get(*list), self.get_data().get(*range)) {
                    (Some(crate::data::SimpleData::List(_, _)), Some(crate::data::SimpleData::Range(start, end))) => {
                        let start = self.get_number(*start)?.to_integer();
                        let end = self.get_number(*end)?.to_integer();
                        let iter = self.get_list_item_iter(*list, Extents::new(start, end))?;

                        for item in iter {
                            items.push(item);
                        }
                    }
                    (Some(crate::data::SimpleData::Concatenation(left, right)), Some(crate::data::SimpleData::Range(start, end))) => {
                        match (self.get_data().get(*start), self.get_data().get(*end)) {
                            (Some(crate::data::SimpleData::Number(crate::data::SimpleNumber::Integer(start))), Some(crate::data::SimpleData::Number(crate::data::SimpleNumber::Integer(end)))) => {
                                let mut nested_con_stack = vec![*right, *left];
                                let mut top_level_con_items = vec![];

                                while let Some(item) = nested_con_stack.pop() {
                                    match self.get_data().get(item) {
                                        None => items.push(UNIT_INDEX),
                                        Some(crate::data::SimpleData::Concatenation(left, right)) => {
                                            nested_con_stack.push(right.clone());
                                            nested_con_stack.push(left.clone());
                                        }
                                        _ => top_level_con_items.push(item.clone()),
                                    }
                                }

                                top_level_con_items
                                    .iter()
                                    .skip(*start as usize)
                                    .take((end - start) as usize + 1)
                                    .map(usize::clone)
                                    .for_each(|i| items.push(i));
                            }
                            _ => items.push(UNIT_INDEX),
                        }
                    }
                    _ => items.push(UNIT_INDEX),
                },
                Some(_) => items.push(item),
            }
        }

        Ok(items)
    }

    pub(crate) fn cache_add(&mut self, value: crate::data::SimpleData<T>) -> Result<usize, DataError> {
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

    pub(crate) fn add_to_current_char_list(&mut self, from: usize, depth: usize) -> Result<(), DataError> {
        if depth >= self.max_char_list_depth {
            return Ok(());
        }

        match self.get_data_type(from)? {
            GarnishDataType::Invalid => todo!(),
            GarnishDataType::Custom => todo!(),
            GarnishDataType::Unit => {
                self.add_to_char_list('(')?;
                self.add_to_char_list(')')?;
            }
            GarnishDataType::True => {
                self.add_to_char_list('$')?;
                self.add_to_char_list('?')?;
            }
            GarnishDataType::False => {
                self.add_to_char_list('$')?;
                self.add_to_char_list('!')?;
            }
            GarnishDataType::Type => {
                let s = format!("{:?}", self.get_type(from)?);
                for c in s.chars() {
                    self.add_to_char_list(c)?;
                }
            }
            GarnishDataType::Number => {
                let x = self.get_number(from)?;
                let s = x.to_string();
                for c in s.chars() {
                    self.add_to_char_list(c)?;
                }
            }
            GarnishDataType::Char => {
                let c = self.get_char(from)?;
                self.add_to_char_list(c)?;
            }
            GarnishDataType::CharList => {
                let iter = self.get_char_list_iter(from, Extents::new(SimpleNumber::zero(), SimpleNumber::max_value()))?;
                for c in iter {
                    self.add_to_char_list(c)?;
                }
            }
            GarnishDataType::Byte => {
                let b = self.get_byte(from)?;
                let s = b.to_string();
                self.start_char_list()?;
                for c in s.chars() {
                    self.add_to_char_list(c)?;
                }
            }
            GarnishDataType::ByteList => {
                let iter = self.get_byte_list_iter(from, Extents::new(SimpleNumber::zero(), SimpleNumber::max_value()))?;
                let mut strs = vec![];
                for b in iter {
                    strs.push(format!("'{}'", b));
                }
                let s = strs.join(" ");
                self.start_char_list()?;
                for c in s.chars() {
                    self.add_to_char_list(c)?;
                }
            }
            GarnishDataType::Symbol => {
                let sym = self.get_symbol(from)?;
                let s = match self.data.get_symbol(sym) {
                    None => sym.to_string(),
                    Some(s) => s.clone(),
                };

                for c in s.chars() {
                    self.add_to_char_list(c)?;
                }
            }
            GarnishDataType::SymbolList => {
                let iter = self.get_symbol_list_iter(from, Extents::new(SimpleNumber::zero(), SimpleNumber::max_value()))?;
                let mut strs = vec![];
                for part in iter {
                    let s = match part {
                        SymbolListPart::Symbol(sym) => match self.data.get_symbol(sym) {
                            None => sym.to_string(),
                            Some(s) => s.clone(),
                        },
                        SymbolListPart::Number(num) => num.to_integer().as_integer()?.to_string(),
                    };
                    strs.push(format!("{}", s));
                }
                let s = strs.join(", ");
                self.start_char_list()?;
                for c in s.chars() {
                    self.add_to_char_list(c)?;
                }
            }
            GarnishDataType::Expression => {
                let e = self.get_expression(from)?;
                let s = format!("Expression({})", e);

                for c in s.chars() {
                    self.add_to_char_list(c)?;
                }
            }
            GarnishDataType::External => {
                let e = self.get_external(from)?;
                let s = format!("External({})", e);

                for c in s.chars() {
                    self.add_to_char_list(c)?;
                }
            }
            GarnishDataType::Range => {
                let (start, end) = self
                    .get_range(from)
                    .and_then(|(start, end)| Ok((self.get_number(start)?, self.get_number(end)?)))?;

                let s = format!("{}..{}", start, end);

                for c in s.chars() {
                    self.add_to_char_list(c)?;
                }
            }
            GarnishDataType::Pair => {
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
            GarnishDataType::Partial => {
                let (left, right) = self.get_pair(from)?;
                if depth > 0 {
                    self.add_to_char_list('(')?;
                }
                self.add_to_current_char_list(left, depth + 1)?;
                self.add_to_char_list(' ')?;
                self.add_to_char_list('~')?;
                self.add_to_char_list(' ')?;
                self.add_to_current_char_list(right, depth + 1)?;
                if depth > 0 {
                    self.add_to_char_list(')')?;
                }
            }
            GarnishDataType::Concatenation => {
                let (left, right) = self.get_concatenation(from)?;
                self.add_to_current_char_list(left, depth + 1)?;
                self.add_to_current_char_list(right, depth + 1)?;
            }
            GarnishDataType::List => {
                let len = self.get_list_len(from)?;

                if depth > 0 {
                    self.add_to_char_list('(')?;
                }

                for i in 0..len {
                    let item = self.get_list_item(from, i.into())?;
                    match item {
                        Some(item) => self.add_to_current_char_list(item, depth + 1)?,
                        None => continue,
                    };

                    if i < len - 1 {
                        self.add_to_char_list(',')?;
                        self.add_to_char_list(' ')?;
                    }
                }

                if depth > 0 {
                    self.add_to_char_list(')')?;
                }
            }
            GarnishDataType::Slice => {
                let (value, range) = self.get_slice(from)?;
                let (start, end) = self.get_range(range)?;
                let (start, end) = (
                    self.get_number(start)?.to_integer().as_integer()?,
                    self.get_number(end)?.to_integer().as_integer()?,
                );

                match self.get_data_type(value)? {
                    GarnishDataType::CharList => {
                        for i in start..=end {
                            let c = self.get_char_list_item(value, i.into())?;
                            match c {
                                Some(c) => self.add_to_char_list(c)?,
                                None => continue,
                            };
                        }
                    }
                    GarnishDataType::List => {
                        if depth > 0 {
                            self.add_to_char_list('(')?;
                        }

                        for i in start..=end {
                            let item = self.get_list_item(value, i.into())?;
                            match item {
                                Some(item) => self.add_to_current_char_list(item, depth + 1)?,
                                None => continue,
                            };

                            if i != end {
                                self.add_to_char_list(',')?;
                                self.add_to_char_list(' ')?;
                            }
                        }

                        if depth > 0 {
                            self.add_to_char_list(')')?;
                        }
                    }
                    GarnishDataType::Concatenation => {
                        iterate_concatenation_mut(self, value, |this, index, addr| {
                            let i = index.to_integer().as_integer()?;
                            if i >= start && i <= end {
                                this.add_to_current_char_list(addr, depth + 1)?;
                            }

                            Ok(None)
                        })
                        .or_else(|err| Err(DataError::from(format!("{:?}", err))))?;
                    }
                    _ => {
                        self.add_to_current_char_list(value, depth + 1)?;
                        self.add_to_char_list(' ')?;
                        self.add_to_char_list('~')?;
                        self.add_to_char_list(' ')?;
                        self.add_to_current_char_list(range, depth + 1)?;
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod utilities {
    use super::SimpleGarnishData;

    #[test]
    fn set_end_of_constant_data_error_over_max_data() {
        let mut data = SimpleGarnishData::new();
        let result = data.set_end_of_constant(10);

        assert!(result.is_err());
    }

    #[test]
    fn set_end_of_constant_data_is_inclusive() {
        let mut data = SimpleGarnishData::new();
        data.set_end_of_constant(2).unwrap();

        assert_eq!(data.end_of_constant_data, 2);
    }
}

#[cfg(test)]
mod data_storage {
    use super::SimpleGarnishData;
    use garnish_lang_traits::GarnishData;

    #[test]
    fn unit() {
        let mut runtime = SimpleGarnishData::new();

        assert_eq!(runtime.add_unit().unwrap(), 0);
        assert_eq!(runtime.add_unit().unwrap(), 0);
        assert_eq!(runtime.add_unit().unwrap(), 0);

        assert_eq!(runtime.get_data_len(), 3);
        assert!(runtime.data.get(0).unwrap().is_unit());
    }

    #[test]
    fn false_data() {
        let mut runtime = SimpleGarnishData::new();

        assert_eq!(runtime.add_false().unwrap(), 1);
        assert_eq!(runtime.add_false().unwrap(), 1);
        assert_eq!(runtime.add_false().unwrap(), 1);

        assert_eq!(runtime.get_data_len(), 3);
        assert!(runtime.data.get(1).unwrap().is_false());
    }

    #[test]
    fn true_data() {
        let mut runtime = SimpleGarnishData::new();

        assert_eq!(runtime.add_true().unwrap(), 2);
        assert_eq!(runtime.add_true().unwrap(), 2);
        assert_eq!(runtime.add_true().unwrap(), 2);

        assert_eq!(runtime.get_data_len(), 3);
        assert!(runtime.data.get(2).unwrap().is_true());
    }

    #[test]
    fn integers() {
        let mut runtime = SimpleGarnishData::new();

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
        let mut runtime = SimpleGarnishData::new();

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
        let mut runtime = SimpleGarnishData::new();

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
        let mut runtime = SimpleGarnishData::new();

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
        let mut runtime = SimpleGarnishData::new();

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
    use super::SimpleGarnishData;
    use crate::symbol_value;
    use garnish_lang_traits::GarnishData;

    #[test]
    fn unit() {
        let mut runtime = SimpleGarnishData::new();

        let d1 = runtime.add_unit().unwrap();
        let addr = runtime.add_symbol_from(d1).unwrap();

        let val = symbol_value("()");
        assert_eq!(runtime.get_symbol(addr).unwrap(), val);
    }
}

#[cfg(test)]
mod to_byte_list {
    use super::SimpleGarnishData;
    use garnish_lang_traits::GarnishData;

    #[test]
    fn unit() {
        let mut runtime = SimpleGarnishData::new();

        let d1 = runtime.add_unit().unwrap();
        let addr = runtime.add_byte_list_from(d1).unwrap();
        let len = runtime.get_byte_list_len(addr).unwrap();

        assert_eq!(len, 0);
    }
}

#[cfg(test)]
mod to_char_list {
    use super::{NoCustom, SimpleGarnishData};
    use garnish_lang_traits::{GarnishData, GarnishDataType};

    fn assert_to_char_list<Func>(expected: &str, setup: Func)
    where
        Func: FnOnce(&mut SimpleGarnishData) -> usize,
    {
        let mut runtime = SimpleGarnishData::new();

        let d1 = setup(&mut runtime);

        let addr = runtime.add_char_list_from(d1).unwrap();
        let len = runtime.get_char_list_len(addr).unwrap();

        let mut chars = String::new();

        for i in 0..len {
            let c = runtime.get_char_list_item(addr, i.into()).unwrap();
            match c {
                Some(c) => chars.push(c),
                None => assert!(false, "Expected Some(c) but got None"),
            };
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
        assert_to_char_list("Unit", |runtime| runtime.add_type(GarnishDataType::Unit).unwrap())
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
        let s = SimpleGarnishData::<NoCustom>::parse_symbol("my_symbol").unwrap().to_string();
        assert_to_char_list(s.as_str(), |runtime| {
            runtime
                .add_symbol(SimpleGarnishData::<NoCustom>::parse_symbol("my_symbol").unwrap())
                .unwrap()
        })
    }

    #[test]
    fn symbol_list() {
        assert_to_char_list("symbol_one, symbol_two", |runtime| {
            let sym1 = runtime.parse_add_symbol("symbol_one").unwrap();
            let sym2 = runtime.parse_add_symbol("symbol_two").unwrap();
            runtime.merge_to_symbol_list(sym1, sym2).unwrap()
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
    fn slice_of_char_list() {
        assert_to_char_list("cde", |runtime| {
            let s = runtime.parse_add_char_list("abcdef").unwrap();

            let start = runtime.add_number(2.into()).unwrap();
            let end = runtime.add_number(4.into()).unwrap();
            let range = runtime.add_range(start, end).unwrap();

            runtime.add_slice(s, range).unwrap()
        })
    }

    #[test]
    fn slice_of_concatenation() {
        assert_to_char_list("304050", |runtime| {
            let d1 = runtime.add_number(10.into()).unwrap();
            let d2 = runtime.add_number(20.into()).unwrap();
            let d3 = runtime.add_number(30.into()).unwrap();
            let d4 = runtime.add_number(40.into()).unwrap();
            let d5 = runtime.add_number(50.into()).unwrap();
            let d6 = runtime.add_number(60.into()).unwrap();

            let cat1 = runtime.add_concatenation(d1, d2).unwrap();
            let cat2 = runtime.add_concatenation(cat1, d3).unwrap();
            let cat3 = runtime.add_concatenation(cat2, d4).unwrap();
            let cat4 = runtime.add_concatenation(cat3, d5).unwrap();
            let cat5 = runtime.add_concatenation(cat4, d6).unwrap();

            let start = runtime.add_number(2.into()).unwrap();
            let end = runtime.add_number(4.into()).unwrap();
            let range = runtime.add_range(start, end).unwrap();

            runtime.add_slice(cat5, range).unwrap()
        })
    }

    #[test]
    fn slice_of_list() {
        assert_to_char_list("30, 40, 50", |runtime| {
            let d1 = runtime.add_number(10.into()).unwrap();
            let d2 = runtime.add_number(20.into()).unwrap();
            let d3 = runtime.add_number(30.into()).unwrap();
            let d4 = runtime.add_number(40.into()).unwrap();
            let d5 = runtime.add_number(50.into()).unwrap();
            let d6 = runtime.add_number(60.into()).unwrap();

            runtime.start_list(3).unwrap();
            runtime.add_to_list(d1, false).unwrap();
            runtime.add_to_list(d2, false).unwrap();
            runtime.add_to_list(d3, false).unwrap();
            runtime.add_to_list(d4, false).unwrap();
            runtime.add_to_list(d5, false).unwrap();
            runtime.add_to_list(d6, false).unwrap();
            let list = runtime.end_list().unwrap();

            let start = runtime.add_number(2.into()).unwrap();
            let end = runtime.add_number(4.into()).unwrap();
            let range = runtime.add_range(start, end).unwrap();

            runtime.add_slice(list, range).unwrap()
        })
    }

    #[test]
    fn concatenation() {
        assert_to_char_list("Hello World!", |runtime| {
            let d1 = runtime.parse_add_char_list("Hello ").unwrap();
            let d2 = runtime.parse_add_char_list("World!").unwrap();
            runtime.add_concatenation(d1, d2).unwrap()
        })
    }
}
