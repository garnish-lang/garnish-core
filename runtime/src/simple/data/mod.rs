mod number;

pub use number::*;

use crate::{DataError, ExpressionDataType};

#[derive(Debug, Eq, PartialEq)]
pub struct SimpleDataList {
    list: Vec<SimpleData>,
}

impl Default for SimpleDataList {
    fn default() -> Self {
        SimpleDataList::new()
            .append(SimpleData::Unit)
            .append(SimpleData::False)
            .append(SimpleData::True)
    }
}

impl SimpleDataList {
    pub fn new() -> Self {
        SimpleDataList { list: vec![] }
    }

    pub fn append(mut self, item: SimpleData) -> Self {
        self.list.push(item);
        self
    }

    pub fn push(&mut self, item: SimpleData) {
        self.list.push(item);
    }

    pub(crate) fn get(&self, index: usize) -> Option<&SimpleData> {
        self.list.get(index)
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }
}

pub type DataCastResult<T> = Result<T, DataError>;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum SimpleData {
    Unit,
    True,
    False,
    Type(ExpressionDataType),
    Number(i32),
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
}

impl SimpleData {
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

    pub fn as_number(&self) -> DataCastResult<i32> {
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
    use crate::simple::data::SimpleData;
    use crate::ExpressionDataType;

    #[test]
    fn get_data_type() {
        assert_eq!(SimpleData::Unit.get_data_type(), ExpressionDataType::Unit);
        assert_eq!(SimpleData::True.get_data_type(), ExpressionDataType::True);
        assert_eq!(SimpleData::False.get_data_type(), ExpressionDataType::False);
        assert_eq!(SimpleData::Number(0).get_data_type(), ExpressionDataType::Number);
        assert_eq!(SimpleData::Char('a').get_data_type(), ExpressionDataType::Char);
        assert_eq!(SimpleData::Byte(0).get_data_type(), ExpressionDataType::Byte);
        assert_eq!(SimpleData::Symbol(0).get_data_type(), ExpressionDataType::Symbol);
        assert_eq!(SimpleData::Expression(0).get_data_type(), ExpressionDataType::Expression);
        assert_eq!(SimpleData::External(0).get_data_type(), ExpressionDataType::External);
        assert_eq!(SimpleData::CharList(String::new()).get_data_type(), ExpressionDataType::CharList);
        assert_eq!(SimpleData::ByteList(vec![]).get_data_type(), ExpressionDataType::ByteList);
        assert_eq!(SimpleData::Pair(0, 0).get_data_type(), ExpressionDataType::Pair);
        assert_eq!(SimpleData::Range(0, 0).get_data_type(), ExpressionDataType::Range);
        assert_eq!(SimpleData::Slice(0, 0).get_data_type(), ExpressionDataType::Slice);
        assert_eq!(SimpleData::Link(0, 0, true).get_data_type(), ExpressionDataType::Link);
        assert_eq!(SimpleData::List(vec![], vec![]).get_data_type(), ExpressionDataType::List);
    }

    #[test]
    fn is_unit() {
        assert!(SimpleData::Unit.is_unit());
    }

    #[test]
    fn is_unit_not_unit() {
        assert!(!SimpleData::True.is_unit());
    }

    #[test]
    fn is_true() {
        assert!(SimpleData::True.is_true());
    }

    #[test]
    fn is_true_not_true() {
        assert!(!SimpleData::Unit.is_true());
    }

    #[test]
    fn is_false() {
        assert!(SimpleData::False.is_false());
    }

    #[test]
    fn is_false_not_false() {
        assert!(!SimpleData::Unit.is_false());
    }

    #[test]
    fn is_type() {
        assert_eq!(SimpleData::Type(ExpressionDataType::Unit).as_type().unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn as_type_not_type() {
        assert!(SimpleData::Unit.as_type().is_err());
    }

    #[test]
    fn as_number() {
        assert_eq!(SimpleData::Number(10).as_number().unwrap(), 10);
    }

    #[test]
    fn as_number_not_number() {
        assert!(SimpleData::Unit.as_number().is_err());
    }

    #[test]
    fn as_char() {
        assert_eq!(SimpleData::Char('a').as_char().unwrap(), 'a');
    }

    #[test]
    fn as_char_not_char() {
        assert!(SimpleData::Unit.as_char().is_err());
    }

    #[test]
    fn as_byte() {
        assert_eq!(SimpleData::Byte(10).as_byte().unwrap(), 10);
    }

    #[test]
    fn as_byte_not_byte() {
        assert!(SimpleData::Unit.as_byte().is_err());
    }

    #[test]
    fn as_symbol() {
        assert_eq!(SimpleData::Symbol(10).as_symbol().unwrap(), 10);
    }

    #[test]
    fn as_symbol_not_symbol() {
        assert!(SimpleData::Unit.as_symbol().is_err());
    }

    #[test]
    fn as_expression() {
        assert_eq!(SimpleData::Expression(10).as_expression().unwrap(), 10);
    }

    #[test]
    fn as_expression_not_expression() {
        assert!(SimpleData::Unit.as_expression().is_err());
    }

    #[test]
    fn as_external() {
        assert_eq!(SimpleData::External(10).as_external().unwrap(), 10);
    }

    #[test]
    fn as_external_not_external() {
        assert!(SimpleData::Unit.as_external().is_err());
    }

    #[test]
    fn as_char_list() {
        assert_eq!(SimpleData::CharList(String::new()).as_char_list().unwrap(), "");
    }

    #[test]
    fn as_char_list_not_char_list() {
        assert!(SimpleData::Unit.as_char_list().is_err());
    }

    #[test]
    fn as_byte_list() {
        assert_eq!(SimpleData::ByteList(vec![10]).as_byte_list().unwrap(), vec![10]);
    }

    #[test]
    fn as_byte_list_not_byte_list() {
        assert!(SimpleData::Unit.as_byte_list().is_err());
    }

    #[test]
    fn as_pair() {
        assert_eq!(SimpleData::Pair(10, 20).as_pair().unwrap(), (10, 20));
    }

    #[test]
    fn as_pair_not_pair() {
        assert!(SimpleData::Unit.as_pair().is_err());
    }

    #[test]
    fn as_range() {
        assert_eq!(SimpleData::Range(10, 20).as_range().unwrap(), (10, 20));
    }

    #[test]
    fn as_range_not_range() {
        assert!(SimpleData::Unit.as_range().is_err());
    }

    #[test]
    fn as_link() {
        assert_eq!(SimpleData::Link(10, 20, true).as_link().unwrap(), (10, 20, true));
    }

    #[test]
    fn as_link_not_link() {
        assert!(SimpleData::Unit.as_link().is_err());
    }

    #[test]
    fn as_slice() {
        assert_eq!(SimpleData::Slice(10, 20).as_slice().unwrap(), (10, 20));
    }

    #[test]
    fn as_slice_not_slice() {
        assert!(SimpleData::Unit.as_slice().is_err());
    }

    #[test]
    fn as_list() {
        assert_eq!(
            SimpleData::List(vec![10, 20], vec![20, 10]).as_list().unwrap(),
            (vec![10, 20], vec![20, 10])
        );
    }

    #[test]
    fn as_list_not_list() {
        assert!(SimpleData::Unit.as_list().is_err());
    }
}
