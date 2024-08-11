use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use garnish_lang_traits::GarnishDataType;
pub use number::*;
pub use parsing::*;
pub use iterators::*;
pub use stack_frame::*;

use crate::{DataError, NoCustom, symbol_value};

mod display;
mod number;
mod parsing;
mod iterators;
mod stack_frame;

pub use display::*;

pub type CustomDataDisplayHandler<T> = fn(&SimpleDataList<T>, &T) -> String;

/// List of [`SimpleData`] with maps to convert symbolic values to original string.
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SimpleDataList<T = NoCustom>
where
    T: Clone + PartialEq + Eq + PartialOrd + Debug + Hash,
{
    list: Vec<SimpleData<T>>,
    symbol_to_name: HashMap<u64, String>,
    expression_to_symbol: HashMap<usize, u64>,
    external_to_symbol: HashMap<usize, u64>
}

impl<T> Default for SimpleDataList<T>
where
    T: Clone + PartialEq + Eq + PartialOrd + Debug + Hash,
{
    fn default() -> Self {
        SimpleDataList::new()
            .append(SimpleData::Unit)
            .append(SimpleData::False)
            .append(SimpleData::True)
    }
}

impl<T> SimpleDataList<T>
where
    T: Clone + PartialEq + Eq + PartialOrd + Debug + Hash,
{
    pub fn new() -> Self {
        SimpleDataList {
            list: vec![],
            symbol_to_name: HashMap::new(),
            expression_to_symbol: HashMap::new(),
            external_to_symbol: HashMap::new(),
        }
    }

    pub fn append(mut self, item: SimpleData<T>) -> Self {
        self.list.push(item);
        self
    }

    pub fn append_symbol<S: Into<String>>(mut self, s: S) -> Self {
        let s: String = s.into();
        let sym = symbol_value(&s);
        self.list.push(SimpleData::Symbol(sym));
        self.symbol_to_name.insert(sym, s);
        self
    }

    pub fn push(&mut self, item: SimpleData<T>) {
        self.list.push(item);
    }

    pub fn get(&self, index: usize) -> Option<&SimpleData<T>> {
        self.list.get(index)
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn symbol_to_name(&self) -> &HashMap<u64, String> {
        &self.symbol_to_name
    }

    pub fn insert_symbol<S: Into<String>>(&mut self, sym: u64, name: S) {
        self.symbol_to_name.insert(sym, name.into());
    }

    pub fn get_symbol(&self, sym: u64) -> Option<&String> {
        self.symbol_to_name.get(&sym)
    }

    pub fn insert_expression(&mut self, expression: usize, sym: u64) {
        self.expression_to_symbol.insert(expression, sym);
    }

    pub fn insert_external(&mut self, external: usize, sym: u64) {
        self.external_to_symbol.insert(external, sym);
    }
}

/// Alias for simple error type
pub type DataCastResult<T> = Result<T, DataError>;

/// Alias for [`SimpleData`] with [`NoCustom`] type parameter.
pub type SimpleDataNC = SimpleData<NoCustom>;

/// Data object to give [`GarnishDataType`] typed values. Can be passed a type parameter to extend supported data.
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
pub enum SimpleData<T = NoCustom>
where
    T: Clone + PartialEq + Eq + PartialOrd + Debug + Hash,
{
    Unit,
    True,
    False,
    Type(GarnishDataType),
    Number(SimpleNumber),
    Char(char),
    Byte(u8),
    Symbol(u64),
    Expression(usize),
    External(usize),
    CharList(String),
    ByteList(Vec<u8>),
    Pair(usize, usize),
    Range(usize, usize),
    Slice(usize, usize),
    List(Vec<usize>, Vec<usize>),
    Concatenation(usize, usize),
    StackFrame(SimpleStackFrame),
    Custom(T),
}

impl<T> SimpleData<T>
where
    T: Clone + PartialEq + Eq + PartialOrd + Debug + Hash,
{
    pub fn get_data_type(&self) -> GarnishDataType {
        match self {
            SimpleData::Unit => GarnishDataType::Unit,
            SimpleData::True => GarnishDataType::True,
            SimpleData::False => GarnishDataType::False,
            SimpleData::Type(_) => GarnishDataType::Type,
            SimpleData::Number(_) => GarnishDataType::Number,
            SimpleData::Char(_) => GarnishDataType::Char,
            SimpleData::Byte(_) => GarnishDataType::Byte,
            SimpleData::Symbol(_) => GarnishDataType::Symbol,
            SimpleData::Expression(_) => GarnishDataType::Expression,
            SimpleData::External(_) => GarnishDataType::External,
            SimpleData::CharList(_) => GarnishDataType::CharList,
            SimpleData::ByteList(_) => GarnishDataType::ByteList,
            SimpleData::Pair(_, _) => GarnishDataType::Pair,
            SimpleData::Concatenation(_, _) => GarnishDataType::Concatenation,
            SimpleData::Range(_, _) => GarnishDataType::Range,
            SimpleData::Slice(_, _) => GarnishDataType::Slice,
            SimpleData::List(_, _) => GarnishDataType::List,
            SimpleData::StackFrame(_)
            | SimpleData::Custom(_) => GarnishDataType::Custom,
        }
    }

    pub fn is_unit(&self) -> bool {
        match self {
            SimpleData::Unit => true,
            _ => false,
        }
    }

    pub fn is_true(&self) -> bool {
        match self {
            SimpleData::True => true,
            _ => false,
        }
    }

    pub fn is_false(&self) -> bool {
        match self {
            SimpleData::False => true,
            _ => false,
        }
    }

    pub fn as_stack_frame(&self) -> DataCastResult<SimpleStackFrame> {
        match self {
            SimpleData::StackFrame(v) => Ok(*v),
            _ => Err(DataError::from(format!("{:?} is not a StackFrame", self))),
        }
    }

    pub fn as_custom(&self) -> DataCastResult<T> {
        match self {
            SimpleData::Custom(v) => Ok(v.clone()),
            _ => Err(DataError::from(format!("{:?} is not a Custom", self))),
        }
    }

    pub fn as_type(&self) -> DataCastResult<GarnishDataType> {
        match self {
            SimpleData::Type(v) => Ok(*v),
            _ => Err(DataError::from(format!("{:?} is not a Number", self))),
        }
    }

    pub fn as_number(&self) -> DataCastResult<SimpleNumber> {
        match self {
            SimpleData::Number(v) => Ok(*v),
            _ => Err(DataError::from(format!("{:?} is not a Number", self))),
        }
    }

    pub fn as_char(&self) -> DataCastResult<char> {
        match self {
            SimpleData::Char(v) => Ok(*v),
            _ => Err(DataError::from(format!("{:?} is not a Char", self))),
        }
    }

    pub fn as_byte(&self) -> DataCastResult<u8> {
        match self {
            SimpleData::Byte(v) => Ok(*v),
            _ => Err(DataError::from(format!("{:?} is not a Byte", self))),
        }
    }

    pub fn as_symbol(&self) -> DataCastResult<u64> {
        match self {
            SimpleData::Symbol(v) => Ok(*v),
            _ => Err(DataError::from(format!("{:?} is not a Symbol", self))),
        }
    }

    pub fn as_expression(&self) -> DataCastResult<usize> {
        match self {
            SimpleData::Expression(v) => Ok(*v),
            _ => Err(DataError::from(format!("{:?} is not an Expression", self))),
        }
    }

    pub fn as_external(&self) -> DataCastResult<usize> {
        match self {
            SimpleData::External(v) => Ok(*v),
            _ => Err(DataError::from(format!("{:?} is not an External", self))),
        }
    }

    pub fn as_char_list(&self) -> DataCastResult<String> {
        match self {
            SimpleData::CharList(v) => Ok(v.clone()),
            _ => Err(DataError::from(format!("{:?} is not a CharList", self))),
        }
    }

    pub fn as_byte_list(&self) -> DataCastResult<Vec<u8>> {
        match self {
            SimpleData::ByteList(v) => Ok(v.clone()),
            _ => Err(DataError::from(format!("{:?} is not a ByteList", self))),
        }
    }

    pub fn as_pair(&self) -> DataCastResult<(usize, usize)> {
        match self {
            SimpleData::Pair(l, r) => Ok((*l, *r)),
            _ => Err(DataError::from(format!("{:?} is not a Pair", self))),
        }
    }

    pub fn as_concatenation(&self) -> DataCastResult<(usize, usize)> {
        match self {
            SimpleData::Concatenation(l, r) => Ok((*l, *r)),
            _ => Err(DataError::from(format!("{:?} is not a Concatenation", self))),
        }
    }

    pub fn as_range(&self) -> DataCastResult<(usize, usize)> {
        match self {
            SimpleData::Range(s, e) => Ok((*s, *e)),
            _ => Err(DataError::from(format!("{:?} is not a Range", self))),
        }
    }

    pub fn as_slice(&self) -> DataCastResult<(usize, usize)> {
        match self {
            SimpleData::Slice(v, r) => Ok((*v, *r)),
            _ => Err(DataError::from(format!("{:?} is not a Slice", self))),
        }
    }

    pub fn as_list(&self) -> DataCastResult<(Vec<usize>, Vec<usize>)> {
        match self {
            SimpleData::List(v, a) => Ok((v.clone(), a.clone())),
            _ => Err(DataError::from(format!("{:?} is not a List", self))),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::data::{SimpleDataNC, SimpleNumber};
    use crate::{GarnishDataType, NoCustom};
    use crate::data::stack_frame::SimpleStackFrame;

    #[test]
    fn get_data_type() {
        assert_eq!(SimpleDataNC::Unit.get_data_type(), GarnishDataType::Unit);
        assert_eq!(SimpleDataNC::True.get_data_type(), GarnishDataType::True);
        assert_eq!(SimpleDataNC::False.get_data_type(), GarnishDataType::False);
        assert_eq!(SimpleDataNC::Number(SimpleNumber::Integer(0)).get_data_type(), GarnishDataType::Number);
        assert_eq!(SimpleDataNC::Char('a').get_data_type(), GarnishDataType::Char);
        assert_eq!(SimpleDataNC::Byte(0).get_data_type(), GarnishDataType::Byte);
        assert_eq!(SimpleDataNC::Symbol(0).get_data_type(), GarnishDataType::Symbol);
        assert_eq!(SimpleDataNC::Expression(0).get_data_type(), GarnishDataType::Expression);
        assert_eq!(SimpleDataNC::External(0).get_data_type(), GarnishDataType::External);
        assert_eq!(SimpleDataNC::CharList(String::new()).get_data_type(), GarnishDataType::CharList);
        assert_eq!(SimpleDataNC::ByteList(vec![]).get_data_type(), GarnishDataType::ByteList);
        assert_eq!(SimpleDataNC::Pair(0, 0).get_data_type(), GarnishDataType::Pair);
        assert_eq!(SimpleDataNC::Concatenation(0, 0).get_data_type(), GarnishDataType::Concatenation);
        assert_eq!(SimpleDataNC::Range(0, 0).get_data_type(), GarnishDataType::Range);
        assert_eq!(SimpleDataNC::Slice(0, 0).get_data_type(), GarnishDataType::Slice);
        assert_eq!(SimpleDataNC::List(vec![], vec![]).get_data_type(), GarnishDataType::List);
        assert_eq!(SimpleDataNC::StackFrame(SimpleStackFrame::new(0)).get_data_type(), GarnishDataType::Custom);
        assert_eq!(SimpleDataNC::Custom(NoCustom {}).get_data_type(), GarnishDataType::Custom);
    }

    #[test]
    fn is_unit() {
        assert!(SimpleDataNC::Unit.is_unit());
    }

    #[test]
    fn is_unit_not_unit() {
        assert!(!SimpleDataNC::True.is_unit());
    }

    #[test]
    fn is_true() {
        assert!(SimpleDataNC::True.is_true());
    }

    #[test]
    fn is_true_not_true() {
        assert!(!SimpleDataNC::Unit.is_true());
    }

    #[test]
    fn is_false() {
        assert!(SimpleDataNC::False.is_false());
    }

    #[test]
    fn is_false_not_false() {
        assert!(!SimpleDataNC::Unit.is_false());
    }

    #[test]
    fn is_custom() {
        assert_eq!(SimpleDataNC::StackFrame(SimpleStackFrame::new(0))
                       .as_stack_frame().unwrap(), SimpleStackFrame::new(0));
    }

    #[test]
    fn is_stack_frame() {
        assert_eq!(SimpleDataNC::Custom(NoCustom {}).as_custom().unwrap(), NoCustom {});
    }

    #[test]
    fn is_custom_not_custom() {
        assert!(SimpleDataNC::Unit.as_custom().is_err());
    }

    #[test]
    fn is_type() {
        assert_eq!(SimpleDataNC::Type(GarnishDataType::Unit).as_type().unwrap(), GarnishDataType::Unit);
    }

    #[test]
    fn as_type_not_type() {
        assert!(SimpleDataNC::Unit.as_type().is_err());
    }

    #[test]
    fn as_number() {
        assert_eq!(
            SimpleDataNC::Number(SimpleNumber::Integer(10)).as_number().unwrap(),
            SimpleNumber::Integer(10)
        );
    }

    #[test]
    fn as_number_not_number() {
        assert!(SimpleDataNC::Unit.as_number().is_err());
    }

    #[test]
    fn as_char() {
        assert_eq!(SimpleDataNC::Char('a').as_char().unwrap(), 'a');
    }

    #[test]
    fn as_char_not_char() {
        assert!(SimpleDataNC::Unit.as_char().is_err());
    }

    #[test]
    fn as_byte() {
        assert_eq!(SimpleDataNC::Byte(10).as_byte().unwrap(), 10.into());
    }

    #[test]
    fn as_byte_not_byte() {
        assert!(SimpleDataNC::Unit.as_byte().is_err());
    }

    #[test]
    fn as_symbol() {
        assert_eq!(SimpleDataNC::Symbol(10).as_symbol().unwrap(), 10);
    }

    #[test]
    fn as_symbol_not_symbol() {
        assert!(SimpleDataNC::Unit.as_symbol().is_err());
    }

    #[test]
    fn as_expression() {
        assert_eq!(SimpleDataNC::Expression(10).as_expression().unwrap(), 10);
    }

    #[test]
    fn as_expression_not_expression() {
        assert!(SimpleDataNC::Unit.as_expression().is_err());
    }

    #[test]
    fn as_external() {
        assert_eq!(SimpleDataNC::External(10).as_external().unwrap(), 10);
    }

    #[test]
    fn as_external_not_external() {
        assert!(SimpleDataNC::Unit.as_external().is_err());
    }

    #[test]
    fn as_char_list() {
        assert_eq!(SimpleDataNC::CharList(String::new()).as_char_list().unwrap(), "");
    }

    #[test]
    fn as_char_list_not_char_list() {
        assert!(SimpleDataNC::Unit.as_char_list().is_err());
    }

    #[test]
    fn as_byte_list() {
        assert_eq!(SimpleDataNC::ByteList(vec![10]).as_byte_list().unwrap(), vec![10]);
    }

    #[test]
    fn as_byte_list_not_byte_list() {
        assert!(SimpleDataNC::Unit.as_byte_list().is_err());
    }

    #[test]
    fn as_pair() {
        assert_eq!(SimpleDataNC::Pair(10, 20).as_pair().unwrap(), (10, 20));
    }

    #[test]
    fn as_pair_not_pair() {
        assert!(SimpleDataNC::Unit.as_pair().is_err());
    }

    #[test]
    fn as_concatenation() {
        assert_eq!(SimpleDataNC::Concatenation(10, 20).as_concatenation().unwrap(), (10, 20));
    }

    #[test]
    fn as_concatenation_not_concatenation() {
        assert!(SimpleDataNC::Unit.as_concatenation().is_err());
    }

    #[test]
    fn as_range() {
        assert_eq!(SimpleDataNC::Range(10, 20).as_range().unwrap(), (10, 20));
    }

    #[test]
    fn as_range_not_range() {
        assert!(SimpleDataNC::Unit.as_range().is_err());
    }

    #[test]
    fn as_slice() {
        assert_eq!(SimpleDataNC::Slice(10, 20).as_slice().unwrap(), (10, 20));
    }

    #[test]
    fn as_slice_not_slice() {
        assert!(SimpleDataNC::Unit.as_slice().is_err());
    }

    #[test]
    fn as_list() {
        assert_eq!(
            SimpleDataNC::List(vec![10, 20], vec![20, 10]).as_list().unwrap(),
            (vec![10, 20], vec![20, 10])
        );
    }

    #[test]
    fn as_list_not_list() {
        assert!(SimpleDataNC::Unit.as_list().is_err());
    }
}
