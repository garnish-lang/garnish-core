use std::{collections::HashMap, hash::Hasher};
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

pub use error::DataError;
use garnish_traits::{ExpressionDataType, GarnishLangRuntimeData, Instruction};
use garnish_traits::helpers::iterate_concatenation_mut;

use crate::data::{SimpleData, SimpleDataList};
pub use crate::instruction::InstructionData;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub mod data;
mod error;
pub mod instruction;
mod runtime;

pub fn symbol_value(value: &str) -> u64 {
    let mut h = DefaultHasher::new();
    value.hash(&mut h);
    let hv = h.finish();

    hv
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialOrd, PartialEq, Eq, Debug, Hash)]
pub struct NoCustom {}

impl Display for NoCustom {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("NoCustom")
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct SimpleRuntimeData<T = NoCustom>
where
    T: Clone + Copy + PartialEq + Eq + PartialOrd + Debug + Hash,
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
    cache: HashMap<u64, usize>,
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
            cache: HashMap::new(),
            max_char_list_depth: 1000,
        }
    }
}

impl<T> SimpleRuntimeData<T>
where
    T: Clone + Copy + PartialEq + Eq + PartialOrd + Debug + Hash,
{
    pub fn new_custom() -> Self {
        SimpleRuntimeData {
            register: vec![],
            data: SimpleDataList::<T>::default(),
            end_of_constant_data: 0,
            values: vec![],
            instruction_cursor: 0,
            instructions: vec![],
            expression_table: vec![],
            jump_path: vec![],
            current_list: None,
            current_char_list: None,
            current_byte_list: None,
            cache: HashMap::new(),
            max_char_list_depth: 1000,
        }
    }

    pub(crate) fn get(&self, index: usize) -> Result<&SimpleData<T>, DataError> {
        match self.data.get(index) {
            None => Err(format!("No data at addr {:?}", index))?,
            Some(d) => Ok(d),
        }
    }

    pub fn add_custom(&mut self, data: T) -> Result<usize, DataError> {
        self.data.push(SimpleData::Custom(data));
        Ok(self.data.len() - 1)
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

    pub fn get_data_mut(&mut self) -> &mut SimpleDataList<T> {
        &mut self.data
    }

    pub fn get_raw_data(&self, index: usize) -> Option<SimpleData<T>> {
        self.data.get(index).cloned()
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

    pub fn display_current_value(&self) -> String where T:Display {
        self.values.last().and_then(|l| Some(self.data.display_for_item(*l))).unwrap_or("<NoData>".to_string())
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
            ExpressionDataType::Invalid => todo!(),
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
                let s = match self.data.get_symbol(sym) {
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
            ExpressionDataType::Concatenation => {
                let (left, right) = self.get_concatenation(from)?;
                self.add_to_current_char_list(left, depth + 1)?;
                self.add_to_current_char_list(right, depth + 1)?;
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
            ExpressionDataType::Slice => {
                let (value, range) = self.get_slice(from)?;
                let (start, end) = self.get_range(range)?;
                let (start, end) = (
                    self.get_number(start)?.to_integer().as_integer()?,
                    self.get_number(end)?.to_integer().as_integer()?
                );

                match self.get_data_type(value)? {
                    ExpressionDataType::CharList => {
                        for i in start..=end {
                            let c = self.get_char_list_item(value, i.into())?;
                            self.add_to_char_list(c)?;
                        }
                    }
                    ExpressionDataType::List => {
                        if depth > 0 {
                            self.add_to_char_list('(')?;
                        }

                        for i in start..=end {
                            let item = self.get_list_item(value, i.into())?;
                            self.add_to_current_char_list(item, depth + 1)?;

                            if i != end {
                                self.add_to_char_list(',')?;
                                self.add_to_char_list(' ')?;
                            }
                        }

                        if depth > 0 {
                            self.add_to_char_list(')')?;
                        }
                    }
                    ExpressionDataType::Concatenation => {
                        iterate_concatenation_mut(self, value, |this, index, addr| {
                            let i = index.to_integer().as_integer()?;
                            if i >= start && i <= end {
                                this.add_to_current_char_list(addr, depth + 1)?;
                            }

                            Ok(None)
                        }).or_else(|err| Err(DataError::from(format!("{:?}", err))))?;
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
    use crate::{GarnishLangRuntimeData, SimpleRuntimeData, symbol_value};

    #[test]
    fn unit() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_unit().unwrap();
        let addr = runtime.add_symbol_from(d1).unwrap();

        let val = symbol_value("()");
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
    use crate::{ExpressionDataType, GarnishLangRuntimeData, NoCustom, SimpleRuntimeData};

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
            runtime
                .add_symbol(SimpleRuntimeData::<NoCustom>::parse_symbol("my_symbol").unwrap())
                .unwrap()
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
