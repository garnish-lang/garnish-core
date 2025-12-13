mod garnish;

use garnish_lang_traits::GarnishDataType;
use crate::data::SimpleNumber;

type BasicNumber = SimpleNumber;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum BasicData<T> {
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

impl<T> BasicData<T> {
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
}

pub type BasicDataUnitCustom = BasicData<()>;

pub struct BasicGarnishData<T> {
    data: Vec<BasicData<T>>,
}

pub type BasicGarnishDataUnit = BasicGarnishData<()>;

impl BasicGarnishDataUnit {
    pub fn new_unit() -> Self {
        Self { data: vec![] }
    }
}

impl<T> BasicGarnishData<T> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_garnish_data() {
        BasicGarnishData::<()>::new();
    }

    #[test]
    fn test_basic_garnish_data_new_unit() {
        BasicGarnishData::new_unit();
    }

    #[test]
    fn add_basic_data() {
        let mut data = BasicGarnishData::new_unit();

        let index = data.push_basic_data(BasicData::Unit);
        assert_eq!(index, 0);

        let index = data.push_basic_data(BasicData::True);
        assert_eq!(index, 1);
        
        assert_eq!(data.data, vec![BasicData::Unit, BasicData::True]);
    }

    #[test]
    fn get_basic_data() {
        let mut data = BasicGarnishData::new_unit();
        let index1 = data.push_basic_data(BasicData::Unit);
        let index2 = data.push_basic_data(BasicData::True);

        assert_eq!(data.get_basic_data(index1), Some(&BasicData::Unit));
        assert_eq!(data.get_basic_data(index2), Some(&BasicData::True));
    }

    #[test]
    fn get_basic_data_mut() {
        let mut data = BasicGarnishData::new_unit();
        let index1 = data.push_basic_data(BasicData::Unit);
        let index2 = data.push_basic_data(BasicData::True);

        assert_eq!(data.get_basic_data_mut(index1), Some(&mut BasicData::Unit));
        assert_eq!(data.get_basic_data_mut(index2), Some(&mut BasicData::True));
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
}