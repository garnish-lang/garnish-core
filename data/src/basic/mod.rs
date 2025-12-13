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