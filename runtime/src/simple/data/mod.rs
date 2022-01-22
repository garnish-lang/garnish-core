mod number;
mod parsing;

pub use number::*;
pub use parsing::*;

use crate::simple::NoCustom;
use crate::{DataError, ExpressionDataType};
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Debug, Eq, PartialEq)]
pub struct SimpleDataList<T = NoCustom>
where
    T: Clone + PartialEq + Eq + PartialOrd + Debug + Hash,
{
    list: Vec<SimpleData<T>>,
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
        SimpleDataList { list: vec![] }
    }

    pub fn append(mut self, item: SimpleData<T>) -> Self {
        self.list.push(item);
        self
    }

    pub fn push(&mut self, item: SimpleData<T>) {
        self.list.push(item);
    }

    pub(crate) fn get(&self, index: usize) -> Option<&SimpleData<T>> {
        self.list.get(index)
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }
}

pub type DataCastResult<T> = Result<T, DataError>;

// generic default not being inferred
// utility type for tests, mainly
pub type SimpleDataNC = SimpleData<NoCustom>;

#[derive(Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
pub enum SimpleData<T = NoCustom>
where
    T: Clone + PartialEq + Eq + PartialOrd + Debug + Hash,
{
    Unit,
    True,
    False,
    Type(ExpressionDataType),
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
    Link(usize, usize, bool),
    List(Vec<usize>, Vec<usize>),
    Custom(T),
}

impl<T> SimpleData<T>
where
    T: Clone + PartialEq + Eq + PartialOrd + Debug + Hash,
{
    pub fn get_data_type(&self) -> ExpressionDataType {
        match self {
            SimpleData::Unit => ExpressionDataType::Unit,
            SimpleData::True => ExpressionDataType::True,
            SimpleData::False => ExpressionDataType::False,
            SimpleData::Type(_) => ExpressionDataType::Type,
            SimpleData::Number(_) => ExpressionDataType::Number,
            SimpleData::Char(_) => ExpressionDataType::Char,
            SimpleData::Byte(_) => ExpressionDataType::Byte,
            SimpleData::Symbol(_) => ExpressionDataType::Symbol,
            SimpleData::Expression(_) => ExpressionDataType::Expression,
            SimpleData::External(_) => ExpressionDataType::External,
            SimpleData::CharList(_) => ExpressionDataType::CharList,
            SimpleData::ByteList(_) => ExpressionDataType::ByteList,
            SimpleData::Pair(_, _) => ExpressionDataType::Pair,
            SimpleData::Range(_, _) => ExpressionDataType::Range,
            SimpleData::Slice(_, _) => ExpressionDataType::Slice,
            SimpleData::Link(_, _, _) => ExpressionDataType::Link,
            SimpleData::List(_, _) => ExpressionDataType::List,
            SimpleData::Custom(_) => ExpressionDataType::Custom,
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

    pub fn as_type(&self) -> DataCastResult<ExpressionDataType> {
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

    pub fn as_range(&self) -> DataCastResult<(usize, usize)> {
        match self {
            SimpleData::Range(s, e) => Ok((*s, *e)),
            _ => Err(DataError::from(format!("{:?} is not a Range", self))),
        }
    }

    pub fn as_link(&self) -> DataCastResult<(usize, usize, bool)> {
        match self {
            SimpleData::Link(v, l, a) => Ok((*v, *l, *a)),
            _ => Err(DataError::from(format!("{:?} is not a Link", self))),
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
    use crate::{ExpressionDataType, SimpleDataNC, SimpleNumber};

    #[test]
    fn get_data_type() {
        assert_eq!(SimpleDataNC::Unit.get_data_type(), ExpressionDataType::Unit);
        assert_eq!(SimpleDataNC::True.get_data_type(), ExpressionDataType::True);
        assert_eq!(SimpleDataNC::False.get_data_type(), ExpressionDataType::False);
        assert_eq!(SimpleDataNC::Number(SimpleNumber::Integer(0)).get_data_type(), ExpressionDataType::Number);
        assert_eq!(SimpleDataNC::Char('a').get_data_type(), ExpressionDataType::Char);
        assert_eq!(SimpleDataNC::Byte(0).get_data_type(), ExpressionDataType::Byte);
        assert_eq!(SimpleDataNC::Symbol(0).get_data_type(), ExpressionDataType::Symbol);
        assert_eq!(SimpleDataNC::Expression(0).get_data_type(), ExpressionDataType::Expression);
        assert_eq!(SimpleDataNC::External(0).get_data_type(), ExpressionDataType::External);
        assert_eq!(SimpleDataNC::CharList(String::new()).get_data_type(), ExpressionDataType::CharList);
        assert_eq!(SimpleDataNC::ByteList(vec![]).get_data_type(), ExpressionDataType::ByteList);
        assert_eq!(SimpleDataNC::Pair(0, 0).get_data_type(), ExpressionDataType::Pair);
        assert_eq!(SimpleDataNC::Range(0, 0).get_data_type(), ExpressionDataType::Range);
        assert_eq!(SimpleDataNC::Slice(0, 0).get_data_type(), ExpressionDataType::Slice);
        assert_eq!(SimpleDataNC::Link(0, 0, true).get_data_type(), ExpressionDataType::Link);
        assert_eq!(SimpleDataNC::List(vec![], vec![]).get_data_type(), ExpressionDataType::List);
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
    fn is_type() {
        assert_eq!(SimpleDataNC::Type(ExpressionDataType::Unit).as_type().unwrap(), ExpressionDataType::Unit);
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
    fn as_range() {
        assert_eq!(SimpleDataNC::Range(10, 20).as_range().unwrap(), (10, 20));
    }

    #[test]
    fn as_range_not_range() {
        assert!(SimpleDataNC::Unit.as_range().is_err());
    }

    #[test]
    fn as_link() {
        assert_eq!(SimpleDataNC::Link(10, 20, true).as_link().unwrap(), (10, 20, true));
    }

    #[test]
    fn as_link_not_link() {
        assert!(SimpleDataNC::Unit.as_link().is_err());
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
