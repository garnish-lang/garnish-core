use crate::{DataError, ExpressionDataType};
use crate::ExpressionDataType::CharList;

pub type DataCastResult<T> = Result<T, DataError>;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum SimpleData {
    Unit,
    True,
    False,
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
    List(Vec<usize>)
}

impl SimpleData {
    pub fn get_data_type(&self) -> ExpressionDataType {
        match self {
            SimpleData::Unit => ExpressionDataType::Unit,
            SimpleData::True => ExpressionDataType::True,
            SimpleData::False => ExpressionDataType::False,
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
            SimpleData::List(_) => ExpressionDataType::List,
        }
    }

    pub fn is_unit(self) -> bool {
        match self {
            SimpleData::Unit => true,
            _ => false
        }
    }

    pub fn is_true(self) -> bool {
        match self {
            SimpleData::True => true,
            _ => false
        }
    }

    pub fn is_false(self) -> bool {
        match self {
            SimpleData::False => true,
            _ => false
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ExpressionDataType;
    use crate::simple::data::enum_data::SimpleData;

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
        assert_eq!(SimpleData::List(vec![]).get_data_type(), ExpressionDataType::List);
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
}