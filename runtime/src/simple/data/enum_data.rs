
use crate::{DataError, ExpressionDataType};


pub type DataCastResult<T> = Result<T, DataError>;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum SimpleDataEnum {
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
    List(Vec<usize>, Vec<usize>)
}

impl SimpleDataEnum {
    pub fn get_data_type(&self) -> ExpressionDataType {
        match self {
            SimpleDataEnum::Unit => ExpressionDataType::Unit,
            SimpleDataEnum::True => ExpressionDataType::True,
            SimpleDataEnum::False => ExpressionDataType::False,
            SimpleDataEnum::Number(_) => ExpressionDataType::Number,
            SimpleDataEnum::Char(_) => ExpressionDataType::Char,
            SimpleDataEnum::Byte(_) => ExpressionDataType::Byte,
            SimpleDataEnum::Symbol(_) => ExpressionDataType::Symbol,
            SimpleDataEnum::Expression(_) => ExpressionDataType::Expression,
            SimpleDataEnum::External(_) => ExpressionDataType::External,
            SimpleDataEnum::CharList(_) => ExpressionDataType::CharList,
            SimpleDataEnum::ByteList(_) => ExpressionDataType::ByteList,
            SimpleDataEnum::Pair(_, _) => ExpressionDataType::Pair,
            SimpleDataEnum::Range(_, _) => ExpressionDataType::Range,
            SimpleDataEnum::Slice(_, _) => ExpressionDataType::Slice,
            SimpleDataEnum::Link(_, _, _) => ExpressionDataType::Link,
            SimpleDataEnum::List(_, _) => ExpressionDataType::List,
        }
    }

    pub fn is_unit(&self) -> bool {
        match self {
            SimpleDataEnum::Unit => true,
            _ => false
        }
    }

    pub fn is_true(&self) -> bool {
        match self {
            SimpleDataEnum::True => true,
            _ => false
        }
    }

    pub fn is_false(&self) -> bool {
        match self {
            SimpleDataEnum::False => true,
            _ => false
        }
    }

    pub fn as_number(&self) -> DataCastResult<i32> {
        match self {
            SimpleDataEnum::Number(v) => Ok(*v),
            _ => Err(DataError::from(format!("{:?} is not a Number", self)))
        }
    }

    pub fn as_char(&self) -> DataCastResult<char> {
        match self {
            SimpleDataEnum::Char(v) => Ok(*v),
            _ => Err(DataError::from(format!("{:?} is not a Char", self)))
        }
    }

    pub fn as_byte(&self) -> DataCastResult<u8> {
        match self {
            SimpleDataEnum::Byte(v) => Ok(*v),
            _ => Err(DataError::from(format!("{:?} is not a Byte", self)))
        }
    }

    pub fn as_symbol(&self) -> DataCastResult<u64> {
        match self {
            SimpleDataEnum::Symbol(v) => Ok(*v),
            _ => Err(DataError::from(format!("{:?} is not a Symbol", self)))
        }
    }

    pub fn as_expression(&self) -> DataCastResult<usize> {
        match self {
            SimpleDataEnum::Expression(v) => Ok(*v),
            _ => Err(DataError::from(format!("{:?} is not an Expression", self)))
        }
    }

    pub fn as_external(&self) -> DataCastResult<usize> {
        match self {
            SimpleDataEnum::External(v) => Ok(*v),
            _ => Err(DataError::from(format!("{:?} is not an External", self)))
        }
    }

    pub fn as_char_list(&self) -> DataCastResult<String> {
        match self {
            SimpleDataEnum::CharList(v) => Ok(v.clone()),
            _ => Err(DataError::from(format!("{:?} is not a CharList", self)))
        }
    }

    pub fn as_byte_list(&self) -> DataCastResult<Vec<u8>> {
        match self {
            SimpleDataEnum::ByteList(v) => Ok(v.clone()),
            _ => Err(DataError::from(format!("{:?} is not a ByteList", self)))
        }
    }

    pub fn as_pair(&self) -> DataCastResult<(usize, usize)> {
        match self {
            SimpleDataEnum::Pair(l, r) => Ok((*l, *r)),
            _ => Err(DataError::from(format!("{:?} is not a Pair", self)))
        }
    }

    pub fn as_range(&self) -> DataCastResult<(usize, usize)> {
        match self {
            SimpleDataEnum::Range(s, e) => Ok((*s, *e)),
            _ => Err(DataError::from(format!("{:?} is not a Range", self)))
        }
    }

    pub fn as_link(&self) -> DataCastResult<(usize, usize, bool)> {
        match self {
            SimpleDataEnum::Link(v, l, a) => Ok((*v, *l, *a)),
            _ => Err(DataError::from(format!("{:?} is not a Link", self)))
        }
    }

    pub fn as_slice(&self) -> DataCastResult<(usize, usize)> {
        match self {
            SimpleDataEnum::Slice(v, r) => Ok((*v, *r)),
            _ => Err(DataError::from(format!("{:?} is not a Slice", self)))
        }
    }

    pub fn as_list(&self) -> DataCastResult<(Vec<usize>, Vec<usize>)> {
        match self {
            SimpleDataEnum::List(v, a) => Ok((v.clone(), a.clone())),
            _ => Err(DataError::from(format!("{:?} is not a List", self)))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ExpressionDataType;
    use crate::simple::data::enum_data::SimpleDataEnum;

    #[test]
    fn get_data_type() {
        assert_eq!(SimpleDataEnum::Unit.get_data_type(), ExpressionDataType::Unit);
        assert_eq!(SimpleDataEnum::True.get_data_type(), ExpressionDataType::True);
        assert_eq!(SimpleDataEnum::False.get_data_type(), ExpressionDataType::False);
        assert_eq!(SimpleDataEnum::Number(0).get_data_type(), ExpressionDataType::Number);
        assert_eq!(SimpleDataEnum::Char('a').get_data_type(), ExpressionDataType::Char);
        assert_eq!(SimpleDataEnum::Byte(0).get_data_type(), ExpressionDataType::Byte);
        assert_eq!(SimpleDataEnum::Symbol(0).get_data_type(), ExpressionDataType::Symbol);
        assert_eq!(SimpleDataEnum::Expression(0).get_data_type(), ExpressionDataType::Expression);
        assert_eq!(SimpleDataEnum::External(0).get_data_type(), ExpressionDataType::External);
        assert_eq!(SimpleDataEnum::CharList(String::new()).get_data_type(), ExpressionDataType::CharList);
        assert_eq!(SimpleDataEnum::ByteList(vec![]).get_data_type(), ExpressionDataType::ByteList);
        assert_eq!(SimpleDataEnum::Pair(0, 0).get_data_type(), ExpressionDataType::Pair);
        assert_eq!(SimpleDataEnum::Range(0, 0).get_data_type(), ExpressionDataType::Range);
        assert_eq!(SimpleDataEnum::Slice(0, 0).get_data_type(), ExpressionDataType::Slice);
        assert_eq!(SimpleDataEnum::Link(0, 0, true).get_data_type(), ExpressionDataType::Link);
        assert_eq!(SimpleDataEnum::List(vec![], vec![]).get_data_type(), ExpressionDataType::List);
    }

    #[test]
    fn is_unit() {
        assert!(SimpleDataEnum::Unit.is_unit());
    }

    #[test]
    fn is_unit_not_unit() {
        assert!(!SimpleDataEnum::True.is_unit());
    }

    #[test]
    fn is_true() {
        assert!(SimpleDataEnum::True.is_true());
    }

    #[test]
    fn is_true_not_true() {
        assert!(!SimpleDataEnum::Unit.is_true());
    }

    #[test]
    fn is_false() {
        assert!(SimpleDataEnum::False.is_false());
    }

    #[test]
    fn is_false_not_false() {
        assert!(!SimpleDataEnum::Unit.is_false());
    }

    #[test]
    fn as_number() {
        assert_eq!(SimpleDataEnum::Number(10).as_number().unwrap(), 10);
    }

    #[test]
    fn as_number_not_number() {
        assert!(SimpleDataEnum::Unit.as_number().is_err());
    }

    #[test]
    fn as_char() {
        assert_eq!(SimpleDataEnum::Char('a').as_char().unwrap(), 'a');
    }

    #[test]
    fn as_char_not_char() {
        assert!(SimpleDataEnum::Unit.as_char().is_err());
    }

    #[test]
    fn as_byte() {
        assert_eq!(SimpleDataEnum::Byte(10).as_byte().unwrap(), 10);
    }

    #[test]
    fn as_byte_not_byte() {
        assert!(SimpleDataEnum::Unit.as_byte().is_err());
    }

    #[test]
    fn as_symbol() {
        assert_eq!(SimpleDataEnum::Symbol(10).as_symbol().unwrap(), 10);
    }

    #[test]
    fn as_symbol_not_symbol() {
        assert!(SimpleDataEnum::Unit.as_symbol().is_err());
    }

    #[test]
    fn as_expression() {
        assert_eq!(SimpleDataEnum::Expression(10).as_expression().unwrap(), 10);
    }

    #[test]
    fn as_expression_not_expression() {
        assert!(SimpleDataEnum::Unit.as_expression().is_err());
    }

    #[test]
    fn as_external() {
        assert_eq!(SimpleDataEnum::External(10).as_external().unwrap(), 10);
    }

    #[test]
    fn as_external_not_external() {
        assert!(SimpleDataEnum::Unit.as_external().is_err());
    }

    #[test]
    fn as_char_list() {
        assert_eq!(SimpleDataEnum::CharList(String::new()).as_char_list().unwrap(), "");
    }

    #[test]
    fn as_char_list_not_char_list() {
        assert!(SimpleDataEnum::Unit.as_char_list().is_err());
    }

    #[test]
    fn as_byte_list() {
        assert_eq!(SimpleDataEnum::ByteList(vec![10]).as_byte_list().unwrap(), vec![10]);
    }

    #[test]
    fn as_byte_list_not_byte_list() {
        assert!(SimpleDataEnum::Unit.as_byte_list().is_err());
    }

    #[test]
    fn as_pair() {
        assert_eq!(SimpleDataEnum::Pair(10, 20).as_pair().unwrap(), (10, 20));
    }

    #[test]
    fn as_pair_not_pair() {
        assert!(SimpleDataEnum::Unit.as_pair().is_err());
    }

    #[test]
    fn as_range() {
        assert_eq!(SimpleDataEnum::Range(10, 20).as_range().unwrap(), (10, 20));
    }

    #[test]
    fn as_range_not_range() {
        assert!(SimpleDataEnum::Unit.as_range().is_err());
    }

    #[test]
    fn as_link() {
        assert_eq!(SimpleDataEnum::Link(10, 20, true).as_link().unwrap(), (10, 20, true));
    }

    #[test]
    fn as_link_not_link() {
        assert!(SimpleDataEnum::Unit.as_link().is_err());
    }

    #[test]
    fn as_slice() {
        assert_eq!(SimpleDataEnum::Slice(10, 20).as_slice().unwrap(), (10, 20));
    }

    #[test]
    fn as_slice_not_slice() {
        assert!(SimpleDataEnum::Unit.as_slice().is_err());
    }

    #[test]
    fn as_list() {
        assert_eq!(SimpleDataEnum::List(vec![10, 20], vec![20, 10]).as_list().unwrap(), (vec![10, 20], vec![20, 10]));
    }

    #[test]
    fn as_list_not_list() {
        assert!(SimpleDataEnum::Unit.as_list().is_err());
    }
}