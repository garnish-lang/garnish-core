use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::Hash;
use std::{collections::HashMap, hash::Hasher};

use crate::{
    EmptyContext, ExpressionDataType, GarnishLangRuntimeContext, GarnishLangRuntimeData, GarnishLangRuntimeState, GarnishRuntime, Instruction,
    InstructionData, RuntimeError, SimpleData, SimpleDataList,
};

pub mod data;
mod error;
mod runtime;

pub use error::DataError;

pub fn symbol_value(value: &str) -> u64 {
    let mut h = DefaultHasher::new();
    value.hash(&mut h);
    let hv = h.finish();

    hv
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Eq, Debug, Hash)]
pub struct NoCustom {}

#[derive(Debug)]
pub struct SimpleRuntimeData<T = NoCustom>
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
    use crate::simple::NoCustom;
    use crate::{ExpressionDataType, GarnishLangRuntimeData, SimpleRuntimeData};

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
