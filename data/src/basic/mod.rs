mod garnish;
mod merge_to_symbol_list;

use garnish_lang_traits::GarnishDataType;
use crate::{DataError, data::SimpleNumber, error::DataErrorType};

type BasicNumber = SimpleNumber;

pub trait BasicDataCustom: Clone {}

impl BasicDataCustom for () {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum BasicData<T> 
where T: BasicDataCustom {
    Unit,
    True,
    False,
    Type(GarnishDataType),
    Number(BasicNumber),
    Char(char),
    Byte(u8),
    Symbol(u64),
    SymbolList(usize),
    Expression(usize),
    External(usize),
    CharList(usize),
    ByteList(usize),
    Pair(usize, usize),
    Range(usize, usize),
    Slice(usize, usize),
    Partial(usize, usize),
    List(usize),
    ListItem(usize),
    Concatenation(usize, usize),
    Custom(T),
}

impl<T> BasicData<T>
where T: BasicDataCustom {
    pub fn get_data_type(&self) -> GarnishDataType {
        match self {
            BasicData::Unit => GarnishDataType::Unit,
            BasicData::True => GarnishDataType::True,
            BasicData::False => GarnishDataType::False,
            BasicData::Type(_) => GarnishDataType::Type,
            BasicData::Number(_) => GarnishDataType::Number,
            BasicData::Char(_) => GarnishDataType::Char,
            BasicData::Byte(_) => GarnishDataType::Byte,
            BasicData::Symbol(_) => GarnishDataType::Symbol,
            BasicData::SymbolList(_) => GarnishDataType::SymbolList,
            BasicData::Expression(_) => GarnishDataType::Expression,
            BasicData::External(_) => GarnishDataType::External,
            BasicData::CharList(_) => GarnishDataType::CharList,
            BasicData::ByteList(_) => GarnishDataType::ByteList,
            BasicData::Pair(_, _) => GarnishDataType::Pair,
            BasicData::Range(_, _) => GarnishDataType::Range,
            BasicData::Slice(_, _) => GarnishDataType::Slice,
            BasicData::Partial(_, _) => GarnishDataType::Partial,
            BasicData::List(_) => GarnishDataType::List,
            BasicData::ListItem(_) => GarnishDataType::List,
            BasicData::Concatenation(_, _) => GarnishDataType::Concatenation,
            BasicData::Custom(_) => GarnishDataType::Custom,
        }
    }

    pub fn as_type(&self) -> Result<GarnishDataType, DataError> {
        match self {
            BasicData::Type(type_) => Ok(*type_),
            _ => Err(DataError::not_type_error(GarnishDataType::Type)),
        }
    }

    pub fn as_number(&self) -> Result<BasicNumber, DataError> {
        match self {
            BasicData::Number(number) => Ok(*number),
            _ => Err(DataError::not_type_error(GarnishDataType::Number)),
        }
    }

    pub fn as_char(&self) -> Result<char, DataError> {
        match self {
            BasicData::Char(c) => Ok(*c),
            _ => Err(DataError::not_type_error(GarnishDataType::Char)),
        }
    }

    pub fn as_byte(&self) -> Result<u8, DataError> {
        match self {
            BasicData::Byte(b) => Ok(*b),
            _ => Err(DataError::not_type_error(GarnishDataType::Byte)),
        }
    }

    pub fn as_symbol(&self) -> Result<u64, DataError> {
        match self {
            BasicData::Symbol(s) => Ok(*s),
            _ => Err(DataError::not_type_error(GarnishDataType::Symbol)),
        }
    }

    pub fn as_expression(&self) -> Result<usize, DataError> {
        match self {
            BasicData::Expression(e) => Ok(*e),
            _ => Err(DataError::not_type_error(GarnishDataType::Expression)),
        }
    }

    pub fn as_external(&self) -> Result<usize, DataError> {
        match self {
            BasicData::External(e) => Ok(*e),
            _ => Err(DataError::not_type_error(GarnishDataType::External)),
        }
    }

    pub fn as_pair(&self) -> Result<(usize, usize), DataError> {
        match self {
            BasicData::Pair(left, right) => Ok((*left, *right)),
            _ => Err(DataError::not_type_error(GarnishDataType::Pair)),
        }
    }

    pub fn as_partial(&self) -> Result<(usize, usize), DataError> {
        match self {
            BasicData::Partial(left, right) => Ok((*left, *right)),
            _ => Err(DataError::not_type_error(GarnishDataType::Partial)),
        }
    }

    pub fn as_concatenation(&self) -> Result<(usize, usize), DataError> {
        match self {
            BasicData::Concatenation(left, right) => Ok((*left, *right)),
            _ => Err(DataError::not_type_error(GarnishDataType::Concatenation)),
        }
    }

    pub fn as_range(&self) -> Result<(usize, usize), DataError> {
        match self {
            BasicData::Range(start, end) => Ok((*start, *end)),
            _ => Err(DataError::not_type_error(GarnishDataType::Range)),
        }
    }

    pub fn as_slice(&self) -> Result<(usize, usize), DataError> {
        match self {
            BasicData::Slice(value, range) => Ok((*value, *range)),
            _ => Err(DataError::not_type_error(GarnishDataType::Slice)),
        }
    }

    pub fn as_custom(&self) -> Result<&T, DataError> {
        match self {
            BasicData::Custom(c) => Ok(c),
            _ => Err(DataError::not_type_error(GarnishDataType::Custom)),
        }
    }
}

pub type BasicDataUnitCustom = BasicData<()>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicGarnishData<T>
where T: BasicDataCustom {
    data: Vec<BasicData<T>>,
}

pub type BasicGarnishDataUnit = BasicGarnishData<()>;

impl<T> BasicGarnishData<T>
where T: BasicDataCustom {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn new_full(data: Vec<BasicData<T>>) -> Self {
        Self { data }
    }

    pub fn push_basic_data(&mut self, data: BasicData<T>) -> usize {
        let index = self.data.len();
        self.data.push(data);
        index
    }

    pub fn get_basic_data(&self, index: usize) -> Option<&BasicData<T>> {
        self.data.get(index)
    }

    pub fn get_basic_data_mut(&mut self, index: usize) -> Option<&mut BasicData<T>> {
        self.data.get_mut(index)
    }

    pub(crate) fn get_data_ensure_index(&self, index: usize) -> Result<&BasicData<T>, DataError> {
        match self.get_basic_data(index) {
            Some(data) => Ok(data),
            None => Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(index))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_garnish_data() {
        BasicGarnishData::<()>::new();
    }

    #[test]
    fn add_basic_data() {
        let mut data = BasicGarnishDataUnit::new();

        let index = data.push_basic_data(BasicData::Unit);
        assert_eq!(index, 0);

        let index = data.push_basic_data(BasicData::True);
        assert_eq!(index, 1);
        
        assert_eq!(data.data, vec![BasicData::Unit, BasicData::True]);
    }

    #[test]
    fn get_basic_data() {
        let mut data = BasicGarnishDataUnit::new();
        let index1 = data.push_basic_data(BasicData::Unit);
        let index2 = data.push_basic_data(BasicData::True);

        assert_eq!(data.get_basic_data(index1), Some(&BasicData::Unit));
        assert_eq!(data.get_basic_data(index2), Some(&BasicData::True));
    }

    #[test]
    fn get_basic_data_mut() {
        let mut data = BasicGarnishDataUnit::new();
        let index1 = data.push_basic_data(BasicData::Unit);
        let index2 = data.push_basic_data(BasicData::True);

        assert_eq!(data.get_basic_data_mut(index1), Some(&mut BasicData::Unit));
        assert_eq!(data.get_basic_data_mut(index2), Some(&mut BasicData::True));
    }

    #[test]
    fn get_data_ensure_index_ok() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Number(100.into()));
        let result = data.get_data_ensure_index(0);
        assert_eq!(result, Ok(&BasicData::Number(100.into())));
    }

    #[test]
    fn get_data_ensure_index_error() {
        let mut data = BasicGarnishDataUnit::new();
        data.push_basic_data(BasicData::Number(100.into()));
        let result = data.get_data_ensure_index(1);
        assert_eq!(result, Err(DataError::new("Invalid data index", DataErrorType::InvalidDataIndex(1))));
    }
}

#[cfg(test)]
mod basic_data {
    use super::*;

    #[test]
    fn get_data_type() {
        let scenarios = vec![
            (BasicDataUnitCustom::Unit, GarnishDataType::Unit),
            (BasicDataUnitCustom::True, GarnishDataType::True),
            (BasicDataUnitCustom::False, GarnishDataType::False),
            (BasicDataUnitCustom::Type(GarnishDataType::Number), GarnishDataType::Type),
            (BasicDataUnitCustom::Number(100.into()), GarnishDataType::Number),
            (BasicDataUnitCustom::Char('a'), GarnishDataType::Char),
            (BasicDataUnitCustom::Byte(100), GarnishDataType::Byte),
            (BasicDataUnitCustom::Symbol(100), GarnishDataType::Symbol),
            (BasicDataUnitCustom::SymbolList(3), GarnishDataType::SymbolList),
            (BasicDataUnitCustom::Expression(100), GarnishDataType::Expression),
            (BasicDataUnitCustom::External(100), GarnishDataType::External),
            (BasicDataUnitCustom::CharList(3), GarnishDataType::CharList),
            (BasicDataUnitCustom::ByteList(3), GarnishDataType::ByteList),
            (BasicDataUnitCustom::Pair(100, 200), GarnishDataType::Pair),
            (BasicDataUnitCustom::Range(100, 200), GarnishDataType::Range),
            (BasicDataUnitCustom::Slice(100, 200), GarnishDataType::Slice),
            (BasicDataUnitCustom::Partial(100, 200), GarnishDataType::Partial),
            (BasicDataUnitCustom::List(100), GarnishDataType::List),
            (BasicDataUnitCustom::ListItem(100), GarnishDataType::List),
            (BasicDataUnitCustom::Concatenation(100, 200), GarnishDataType::Concatenation),
            (BasicDataUnitCustom::Custom(()), GarnishDataType::Custom),
        ];

        for (data, expected) in scenarios {
            assert_eq!(data.get_data_type(), expected, "Got {:?} expected {:?}", data.get_data_type(), expected);
        }
    }

    #[test]
    fn as_number() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_number(), Ok(100.into()));
    }

    #[test]
    fn as_number_not_number() {
        let data = BasicDataUnitCustom::Symbol(100);
        assert_eq!(data.as_number(), Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Number))));
    }

    #[test]
    fn as_type() {
        let data = BasicDataUnitCustom::Type(GarnishDataType::Number);
        assert_eq!(data.as_type(), Ok(GarnishDataType::Number));
    }

    #[test]
    fn as_type_not_type() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_type(), Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Type))));
    }

    #[test]
    fn as_char() {
        let data = BasicDataUnitCustom::Char('a');
        assert_eq!(data.as_char(), Ok('a'));
    }

    #[test]
    fn as_char_not_char() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_char(), Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Char))));
    }

    #[test]
    fn as_byte() {
        let data = BasicDataUnitCustom::Byte(100);
        assert_eq!(data.as_byte(), Ok(100));
    }

    #[test]
    fn as_byte_not_byte() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_byte(), Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Byte))));
    }

    #[test]
    fn as_symbol() {
        let data = BasicDataUnitCustom::Symbol(100);
        assert_eq!(data.as_symbol(), Ok(100));
    }

    #[test]
    fn as_symbol_not_symbol() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_symbol(), Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Symbol))));
    }

    #[test]
    fn as_expression() {
        let data = BasicDataUnitCustom::Expression(100);
        assert_eq!(data.as_expression(), Ok(100));
    }

    #[test]
    fn as_expression_not_expression() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_expression(), Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Expression))));
    }

    #[test]
    fn as_external() {
        let data = BasicDataUnitCustom::External(100);
        assert_eq!(data.as_external(), Ok(100));
    }

    #[test]
    fn as_external_not_external() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_external(), Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::External))));
    }

    #[test]
    fn as_pair() {
        let data = BasicDataUnitCustom::Pair(100, 200);
        assert_eq!(data.as_pair(), Ok((100, 200)));
    }

    #[test]
    fn as_pair_not_pair() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_pair(), Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Pair))));
    }

    #[test]
    fn as_partial() {
        let data = BasicDataUnitCustom::Partial(100, 200);
        assert_eq!(data.as_partial(), Ok((100, 200)));
    }

    #[test]
    fn as_partial_not_partial() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_partial(), Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Partial))));
    }

    #[test]
    fn as_concatenation() {
        let data = BasicDataUnitCustom::Concatenation(100, 200);
        assert_eq!(data.as_concatenation(), Ok((100, 200)));
    }

    #[test]
    fn as_concatenation_not_concatenation() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_concatenation(), Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Concatenation))));
    }

    #[test]
    fn as_range() {
        let data = BasicDataUnitCustom::Range(100, 200);
        assert_eq!(data.as_range(), Ok((100, 200)));
    }

    #[test]
    fn as_range_not_range() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_range(), Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Range))));
    }

    #[test]
    fn as_slice() {
        let data = BasicDataUnitCustom::Slice(100, 200);
        assert_eq!(data.as_slice(), Ok((100, 200)));
    }

    #[test]
    fn as_slice_not_slice() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_slice(), Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Slice))));
    }

    #[test]
    fn as_custom() {
        let data = BasicDataUnitCustom::Custom(());
        assert_eq!(data.as_custom(), Ok(&()));
    }

    #[test]
    fn as_custom_not_custom() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_custom(), Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Custom))));
    }
}