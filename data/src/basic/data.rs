use garnish_lang_traits::{GarnishDataType};
use crate::{DataError};

type BasicNumber = crate::data::SimpleNumber;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum BasicData<T> 
where T: crate::basic::BasicDataCustom {
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
    List(usize, usize),
    Concatenation(usize, usize),
    Custom(T),
    // non garnish data
    Empty,
    UninitializedList(usize, usize),
    ListItem(usize),
    AssociativeItem(u64, usize),
}

impl<T> BasicData<T>
where T: crate::basic::BasicDataCustom {
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
            BasicData::List(_, _) => GarnishDataType::List,
            BasicData::Concatenation(_, _) => GarnishDataType::Concatenation,
            BasicData::Custom(_) => GarnishDataType::Custom,
            // non garnish data
            BasicData::Empty => GarnishDataType::Invalid,
            BasicData::UninitializedList(_, _) => GarnishDataType::Invalid,
            BasicData::ListItem(_) => GarnishDataType::Invalid,
            BasicData::AssociativeItem(_, _) => GarnishDataType::Invalid,
        }
    }

    pub fn as_type(&self) -> Result<GarnishDataType, DataError> {
        match self {
            BasicData::Type(type_) => Ok(*type_),
            _ => Err(DataError::not_type_error(GarnishDataType::Type, self.get_data_type())),
        }
    }

    pub fn as_type_mut(&mut self) -> Result<&mut GarnishDataType, DataError> {
        match self {
            BasicData::Type(type_) => Ok(type_),
            _ => Err(DataError::not_type_error(GarnishDataType::Type, self.get_data_type())),
        }
    }

    pub fn as_number(&self) -> Result<BasicNumber, DataError> {
        match self {
            BasicData::Number(number) => Ok(*number),
            _ => Err(DataError::not_type_error(GarnishDataType::Number, self.get_data_type())),
        }
    }

    pub fn as_number_mut(&mut self) -> Result<&mut BasicNumber, DataError> {
        match self {
            BasicData::Number(number) => Ok(number),
            _ => Err(DataError::not_type_error(GarnishDataType::Number, self.get_data_type())),
        }
    }

    pub fn as_char(&self) -> Result<char, DataError> {
        match self {
            BasicData::Char(c) => Ok(*c),
            _ => Err(DataError::not_type_error(GarnishDataType::Char, self.get_data_type())),
        }
    }

    pub fn as_char_mut(&mut self) -> Result<&mut char, DataError> {
        match self {
            BasicData::Char(c) => Ok(c),
            _ => Err(DataError::not_type_error(GarnishDataType::Char, self.get_data_type())),
        }
    }

    pub fn as_byte(&self) -> Result<u8, DataError> {
        match self {
            BasicData::Byte(b) => Ok(*b),
            _ => Err(DataError::not_type_error(GarnishDataType::Byte, self.get_data_type())),
        }
    }

    pub fn as_byte_mut(&mut self) -> Result<&mut u8, DataError> {
        match self {
            BasicData::Byte(b) => Ok(b),
            _ => Err(DataError::not_type_error(GarnishDataType::Byte, self.get_data_type())),
        }
    }

    pub fn as_symbol(&self) -> Result<u64, DataError> {
        match self {
            BasicData::Symbol(s) => Ok(*s),
            _ => Err(DataError::not_type_error(GarnishDataType::Symbol, self.get_data_type())),
        }
    }

    pub fn as_symbol_mut(&mut self) -> Result<&mut u64, DataError> {
        match self {
            BasicData::Symbol(s) => Ok(s),
            _ => Err(DataError::not_type_error(GarnishDataType::Symbol, self.get_data_type())),
        }
    }

    pub fn as_expression(&self) -> Result<usize, DataError> {
        match self {
            BasicData::Expression(e) => Ok(*e),
            _ => Err(DataError::not_type_error(GarnishDataType::Expression, self.get_data_type())),
        }
    }

    pub fn as_expression_mut(&mut self) -> Result<&mut usize, DataError> {
        match self {
            BasicData::Expression(e) => Ok(e),
            _ => Err(DataError::not_type_error(GarnishDataType::Expression, self.get_data_type())),
        }
    }

    pub fn as_external(&self) -> Result<usize, DataError> {
        match self {
            BasicData::External(e) => Ok(*e),
            _ => Err(DataError::not_type_error(GarnishDataType::External, self.get_data_type())),
        }
    }

    pub fn as_external_mut(&mut self) -> Result<&mut usize, DataError> {
        match self {
            BasicData::External(e) => Ok(e),
            _ => Err(DataError::not_type_error(GarnishDataType::External, self.get_data_type())),
        }
    }

    pub fn as_char_list(&self) -> Result<usize, DataError> {
        match self {
            BasicData::CharList(l) => Ok(*l),
            _ => Err(DataError::not_type_error(GarnishDataType::CharList, self.get_data_type())),
        }
    }

    pub fn as_char_list_mut(&mut self) -> Result<&mut usize, DataError> {
        match self {
            BasicData::CharList(l) => Ok(l),
            _ => Err(DataError::not_type_error(GarnishDataType::CharList, self.get_data_type())),
        }
    }

    pub fn as_byte_list(&self) -> Result<usize, DataError> {
        match self {
            BasicData::ByteList(l) => Ok(*l),
            _ => Err(DataError::not_type_error(GarnishDataType::ByteList, self.get_data_type())),
        }
    }

    pub fn as_byte_list_mut(&mut self) -> Result<&mut usize, DataError> {
        match self {
            BasicData::ByteList(l) => Ok(l),
            _ => Err(DataError::not_type_error(GarnishDataType::ByteList, self.get_data_type())),
        }
    }

    pub fn as_pair(&self) -> Result<(usize, usize), DataError> {
        match self {
            BasicData::Pair(left, right) => Ok((*left, *right)),
            _ => Err(DataError::not_type_error(GarnishDataType::Pair, self.get_data_type())),
        }
    }

    pub fn as_pair_mut(&mut self) -> Result<(&mut usize, &mut usize), DataError> {
        match self {
            BasicData::Pair(left, right) => Ok((left, right)),
            _ => Err(DataError::not_type_error(GarnishDataType::Pair, self.get_data_type())),
        }
    }

    pub fn as_partial(&self) -> Result<(usize, usize), DataError> {
        match self {
            BasicData::Partial(left, right) => Ok((*left, *right)),
            _ => Err(DataError::not_type_error(GarnishDataType::Partial, self.get_data_type())),
        }
    }

    pub fn as_partial_mut(&mut self) -> Result<(&mut usize, &mut usize), DataError> {
        match self {
            BasicData::Partial(left, right) => Ok((left, right)),
            _ => Err(DataError::not_type_error(GarnishDataType::Partial, self.get_data_type())),
        }
    }

    pub fn as_concatenation(&self) -> Result<(usize, usize), DataError> {
        match self {
            BasicData::Concatenation(left, right) => Ok((*left, *right)),
            _ => Err(DataError::not_type_error(GarnishDataType::Concatenation, self.get_data_type())),
        }
    }

    pub fn as_concatenation_mut(&mut self) -> Result<(&mut usize, &mut usize), DataError> {
        match self {
            BasicData::Concatenation(left, right) => Ok((left, right)),
            _ => Err(DataError::not_type_error(GarnishDataType::Concatenation, self.get_data_type())),
        }
    }

    pub fn as_range(&self) -> Result<(usize, usize), DataError> {
        match self {
            BasicData::Range(start, end) => Ok((*start, *end)),
            _ => Err(DataError::not_type_error(GarnishDataType::Range, self.get_data_type())),
        }
    }

    pub fn as_range_mut(&mut self) -> Result<(&mut usize, &mut usize), DataError> {
        match self {
            BasicData::Range(start, end) => Ok((start, end)),
            _ => Err(DataError::not_type_error(GarnishDataType::Range, self.get_data_type())),
        }
    }

    pub fn as_slice(&self) -> Result<(usize, usize), DataError> {
        match self {
            BasicData::Slice(value, range) => Ok((*value, *range)),
            _ => Err(DataError::not_type_error(GarnishDataType::Slice, self.get_data_type())),
        }
    }

    pub fn as_slice_mut(&mut self) -> Result<(&mut usize, &mut usize), DataError> {
        match self {
            BasicData::Slice(value, range) => Ok((value, range)),
            _ => Err(DataError::not_type_error(GarnishDataType::Slice, self.get_data_type())),
        }
    }

    pub fn as_list(&self) -> Result<(usize, usize), DataError> {
        match self {
            BasicData::List(len, count) => Ok((*len, *count)),
            _ => Err(DataError::not_type_error(GarnishDataType::List, self.get_data_type())),
        }
    }

    pub fn as_list_mut(&mut self) -> Result<(&mut usize, &mut usize), DataError> {
        match self {
            BasicData::List(len, count) => Ok((len, count)),
            _ => Err(DataError::not_type_error(GarnishDataType::List, self.get_data_type())),
        }
    }

    pub fn as_custom(&self) -> Result<&T, DataError> {
        match self {
            BasicData::Custom(c) => Ok(c),
            _ => Err(DataError::not_type_error(GarnishDataType::Custom, self.get_data_type())),
        }
    }

    pub fn as_custom_mut(&mut self) -> Result<&mut T, DataError> {
        match self {
            BasicData::Custom(c) => Ok(c),
            _ => Err(DataError::not_type_error(GarnishDataType::Custom, self.get_data_type())),
        }
    }

    pub fn as_uninitialized_list(&self) -> Result<(usize, usize), DataError> {
        match self {
            BasicData::UninitializedList(len, index) => Ok((*len, *index)),
            _ => Err(DataError::not_basic_type_error()),
        }
    }

    pub fn as_uninitialized_list_mut(&mut self) -> Result<(&mut usize, &mut usize), DataError> {
        match self {
            BasicData::UninitializedList(len, index) => Ok((len, index)),
            _ => Err(DataError::not_basic_type_error()),
        }
    }

    pub fn as_associative_item(&self) -> Result<(u64, usize), DataError> {
        match self {
            BasicData::AssociativeItem(sym, index) => Ok((*sym, *index)),
            _ => Err(DataError::not_basic_type_error()),
        }
    }
    
    pub fn as_associative_item_mut(&mut self) -> Result<(&mut u64, &mut usize), DataError> {
        match self {
            BasicData::AssociativeItem(sym, index) => Ok((sym, index)),
            _ => Err(DataError::not_basic_type_error()),
        }
    }

    pub fn as_list_item(&self) -> Result<usize, DataError> {
        match self {
            BasicData::ListItem(item) => Ok(*item),
            _ => Err(DataError::not_basic_type_error()),
        }
    }

    pub fn as_list_item_mut(&mut self) -> Result<&mut usize, DataError> {
        match self {
            BasicData::ListItem(item) => Ok(item),
            _ => Err(DataError::not_basic_type_error()),
        }
    }
}

pub type BasicDataUnitCustom = BasicData<()>;

#[cfg(test)]
mod tests {
    use crate::error::DataErrorType;

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
            (BasicDataUnitCustom::List(100, 200), GarnishDataType::List),
            (BasicDataUnitCustom::Concatenation(100, 200), GarnishDataType::Concatenation),
            (BasicDataUnitCustom::Custom(()), GarnishDataType::Custom),
            // non garnish data
            (BasicDataUnitCustom::Empty, GarnishDataType::Invalid),
            (BasicDataUnitCustom::UninitializedList(100, 200), GarnishDataType::Invalid),
            (BasicDataUnitCustom::ListItem(100), GarnishDataType::Invalid),
            (BasicDataUnitCustom::AssociativeItem(100, 200), GarnishDataType::Invalid),
        ];

        for (data, expected) in scenarios {
            assert_eq!(data.get_data_type(), expected, "Got {:?} expected {:?}", data.get_data_type(), expected);
        }
    }

    #[test]
    fn as_type() {
        let data = BasicDataUnitCustom::Type(GarnishDataType::Number);
        assert_eq!(data.as_type(), Ok(GarnishDataType::Number));
    }

    #[test]
    fn as_type_not_type() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_type(), Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Type, GarnishDataType::Number))));
    }

    #[test]
    fn as_type_mut() {
        let mut data = BasicDataUnitCustom::Type(GarnishDataType::Number);
        *data.as_type_mut().unwrap() = GarnishDataType::Char;
        assert_eq!(data.as_type(), Ok(GarnishDataType::Char));
    }

    #[test]
    fn as_type_mut_not_type() {
        let mut data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_type_mut(), Err(DataError::not_type_error(GarnishDataType::Type, GarnishDataType::Number)));
    }

    #[test]
    fn as_number() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_number(), Ok(100.into()));
    }

    #[test]
    fn as_number_not_number() {
        let data = BasicDataUnitCustom::Symbol(100);
        assert_eq!(data.as_number(), Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Number, GarnishDataType::Symbol))));
    }

    #[test]
    fn as_number_mut() {
        let mut data = BasicDataUnitCustom::Number(100.into());
        *data.as_number_mut().unwrap() = 200.into();
        assert_eq!(data.as_number(), Ok(200.into()));
    }

    #[test]
    fn as_number_mut_not_number() {
        let mut data = BasicDataUnitCustom::Symbol(100);
        assert_eq!(data.as_number_mut(), Err(DataError::not_type_error(GarnishDataType::Number, GarnishDataType::Symbol)));
    }

    #[test]
    fn as_char() {
        let data = BasicDataUnitCustom::Char('a');
        assert_eq!(data.as_char(), Ok('a'));
    }

    #[test]
    fn as_char_not_char() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_char(), Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Char, GarnishDataType::Number))));
    }

    #[test]
    fn as_char_mut() {
        let mut data = BasicDataUnitCustom::Char('a');
        *data.as_char_mut().unwrap() = 'b';
        assert_eq!(data.as_char(), Ok('b'));
    }

    #[test]
    fn as_char_mut_not_char() {
        let mut data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_char_mut(), Err(DataError::not_type_error(GarnishDataType::Char, GarnishDataType::Number)));
    }

    #[test]
    fn as_byte() {
        let data = BasicDataUnitCustom::Byte(100);
        assert_eq!(data.as_byte(), Ok(100));
    }

    #[test]
    fn as_byte_not_byte() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_byte(), Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Byte, GarnishDataType::Number))));
    }

    #[test]
    fn as_byte_mut() {
        let mut data = BasicDataUnitCustom::Byte(100);
        *data.as_byte_mut().unwrap() = 200;
        assert_eq!(data.as_byte(), Ok(200));
    }

    #[test]
    fn as_byte_mut_not_byte() {
        let mut data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_byte_mut(), Err(DataError::not_type_error(GarnishDataType::Byte, GarnishDataType::Number)));
    }

    #[test]
    fn as_symbol() {
        let data = BasicDataUnitCustom::Symbol(100);
        assert_eq!(data.as_symbol(), Ok(100));
    }

    #[test]
    fn as_symbol_not_symbol() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_symbol(), Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Symbol, GarnishDataType::Number))));
    }

    #[test]
    fn as_symbol_mut() {
        let mut data = BasicDataUnitCustom::Symbol(100);
        *data.as_symbol_mut().unwrap() = 200;
        assert_eq!(data.as_symbol(), Ok(200));
    }

    #[test]
    fn as_symbol_mut_not_symbol() {
        let mut data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_symbol_mut(), Err(DataError::not_type_error(GarnishDataType::Symbol, GarnishDataType::Number)));
    }

    #[test]
    fn as_expression() {
        let data = BasicDataUnitCustom::Expression(100);
        assert_eq!(data.as_expression(), Ok(100));
    }

    #[test]
    fn as_expression_not_expression() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_expression(), Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Expression, GarnishDataType::Number))));
    }

    #[test]
    fn as_expression_mut() {
        let mut data = BasicDataUnitCustom::Expression(100);
        *data.as_expression_mut().unwrap() = 200;
        assert_eq!(data.as_expression(), Ok(200));
    }

    #[test]
    fn as_expression_mut_not_expression() {
        let mut data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_expression_mut(), Err(DataError::not_type_error(GarnishDataType::Expression, GarnishDataType::Number)));
    }

    #[test]
    fn as_external() {
        let data = BasicDataUnitCustom::External(100);
        assert_eq!(data.as_external(), Ok(100));
    }

    #[test]
    fn as_external_not_external() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_external(), Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::External, GarnishDataType::Number))));
    }

    #[test]
    fn as_external_mut() {
        let mut data = BasicDataUnitCustom::External(100);
        *data.as_external_mut().unwrap() = 200;
        assert_eq!(data.as_external(), Ok(200));
    }

    #[test]
    fn as_external_mut_not_external() {
        let mut data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_external_mut(), Err(DataError::not_type_error(GarnishDataType::External, GarnishDataType::Number)));
    }

    #[test]
    fn as_pair() {
        let data = BasicDataUnitCustom::Pair(100, 200);
        assert_eq!(data.as_pair(), Ok((100, 200)));
    }

    #[test]
    fn as_pair_not_pair() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_pair(), Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Pair, GarnishDataType::Number))));
    }

    #[test]
    fn as_pair_mut() {
        let mut data = BasicDataUnitCustom::Pair(100, 200);
        let (left, right) = data.as_pair_mut().unwrap();
        *left = 300;
        *right = 400;
        assert_eq!(data.as_pair(), Ok((300, 400)));
    }

    #[test]
    fn as_pair_mut_not_pair() {
        let mut data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_pair_mut(), Err(DataError::not_type_error(GarnishDataType::Pair, GarnishDataType::Number)));
    }

    #[test]
    fn as_partial() {
        let data = BasicDataUnitCustom::Partial(100, 200);
        assert_eq!(data.as_partial(), Ok((100, 200)));
    }

    #[test]
    fn as_partial_not_partial() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_partial(), Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Partial, GarnishDataType::Number))));
    }

    #[test]
    fn as_partial_mut() {
        let mut data = BasicDataUnitCustom::Partial(100, 200);
        let (left, right) = data.as_partial_mut().unwrap();
        *left = 300;
        *right = 400;
        assert_eq!(data.as_partial(), Ok((300, 400)));
    }

    #[test]
    fn as_partial_mut_not_partial() {
        let mut data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_partial_mut(), Err(DataError::not_type_error(GarnishDataType::Partial, GarnishDataType::Number)));
    }

    #[test]
    fn as_concatenation() {
        let data = BasicDataUnitCustom::Concatenation(100, 200);
        assert_eq!(data.as_concatenation(), Ok((100, 200)));
    }

    #[test]
    fn as_concatenation_not_concatenation() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_concatenation(), Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Concatenation, GarnishDataType::Number))));
    }

    #[test]
    fn as_concatenation_mut() {
        let mut data = BasicDataUnitCustom::Concatenation(100, 200);
        let (left, right) = data.as_concatenation_mut().unwrap();
        *left = 300;
        *right = 400;
        assert_eq!(data.as_concatenation(), Ok((300, 400)));
    }

    #[test]
    fn as_concatenation_mut_not_concatenation() {
        let mut data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_concatenation_mut(), Err(DataError::not_type_error(GarnishDataType::Concatenation, GarnishDataType::Number)));
    }

    #[test]
    fn as_range() {
        let data = BasicDataUnitCustom::Range(100, 200);
        assert_eq!(data.as_range(), Ok((100, 200)));
    }

    #[test]
    fn as_range_not_range() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_range(), Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Range, GarnishDataType::Number))));
    }

    #[test]
    fn as_range_mut() {
        let mut data = BasicDataUnitCustom::Range(100, 200);
        let (start, end) = data.as_range_mut().unwrap();
        *start = 300;
        *end = 400;
        assert_eq!(data.as_range(), Ok((300, 400)));
    }

    #[test]
    fn as_range_mut_not_range() {
        let mut data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_range_mut(), Err(DataError::not_type_error(GarnishDataType::Range, GarnishDataType::Number)));
    }

    #[test]
    fn as_slice() {
        let data = BasicDataUnitCustom::Slice(100, 200);
        assert_eq!(data.as_slice(), Ok((100, 200)));
    }

    #[test]
    fn as_slice_not_slice() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_slice(), Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Slice, GarnishDataType::Number))));
    }

    #[test]
    fn as_slice_mut() {
        let mut data = BasicDataUnitCustom::Slice(100, 200);
        let (value, range) = data.as_slice_mut().unwrap();
        *value = 300;
        *range = 400;
        assert_eq!(data.as_slice(), Ok((300, 400)));
    }

    #[test]
    fn as_slice_mut_not_slice() {
        let mut data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_slice_mut(), Err(DataError::not_type_error(GarnishDataType::Slice, GarnishDataType::Number)));
    }

    #[test]
    fn as_list() {
        let data = BasicDataUnitCustom::List(100, 200);
        assert_eq!(data.as_list(), Ok((100, 200)));
    }

    #[test]
    fn as_list_not_list() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_list(), Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::List, GarnishDataType::Number))));
    }

    #[test]
    fn as_list_mut() {
        let mut data = BasicDataUnitCustom::List(100, 200);
        let (len, count) = data.as_list_mut().unwrap();
        *len = 300;
        *count = 400;
        assert_eq!(data.as_list(), Ok((300, 400)));
    }

    #[test]
    fn as_list_mut_not_list() {
        let mut data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_list_mut(), Err(DataError::not_type_error(GarnishDataType::List, GarnishDataType::Number)));
    }

    #[test]
    fn as_custom() {
        let data = BasicDataUnitCustom::Custom(());
        assert_eq!(data.as_custom(), Ok(&()));
    }

    #[test]
    fn as_custom_not_custom() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_custom(), Err(DataError::new("Not of type", DataErrorType::NotType(GarnishDataType::Custom, GarnishDataType::Number))));
    }

    #[test]
    fn as_custom_mut() {
        let mut data = BasicDataUnitCustom::Custom(());
        let _ = data.as_custom_mut().unwrap();
    }

    #[test]
    fn as_custom_mut_not_custom() {
        let mut data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_custom_mut(), Err(DataError::not_type_error(GarnishDataType::Custom, GarnishDataType::Number)));
    }

    #[test]
    fn as_uninitialized_list() {
        let data = BasicDataUnitCustom::UninitializedList(100, 200);
        assert_eq!(data.as_uninitialized_list(), Ok((100, 200)));
    }
    
    #[test]
    fn as_uninitialized_list_not_uninitialized_list() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_uninitialized_list(), Err(DataError::not_basic_type_error()));
    }

    #[test]
    fn as_uninitialized_list_mut() {
        let mut data = BasicDataUnitCustom::UninitializedList(100, 200);
        let (len, index) = data.as_uninitialized_list_mut().unwrap();
        *len = 300;
        *index = 400;
        assert_eq!(data.as_uninitialized_list(), Ok((300, 400)));
    }

    #[test]
    fn as_uninitialized_list_mut_not_uninitialized_list() {
        let mut data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_uninitialized_list_mut(), Err(DataError::not_basic_type_error()));
    }

    #[test]
    fn as_associative_item() {
        let data = BasicDataUnitCustom::AssociativeItem(100, 200);
        assert_eq!(data.as_associative_item(), Ok((100, 200)));
    }
    
    #[test]
    fn as_associative_item_not_associative_item() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_associative_item(), Err(DataError::not_basic_type_error()));
    }

    #[test]
    fn as_associative_item_mut() {
        let mut data = BasicDataUnitCustom::AssociativeItem(100, 200);
        let (sym, index) = data.as_associative_item_mut().unwrap();
        *sym = 200;
        *index = 300;
        assert_eq!(data.as_associative_item(), Ok((200, 300)));
    }
    
    #[test]
    fn as_associative_item_mut_not_associative_item() {
        let mut data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_associative_item_mut(), Err(DataError::not_basic_type_error()));
    }

    #[test]
    fn as_list_item() {
        let data = BasicDataUnitCustom::ListItem(100);
        assert_eq!(data.as_list_item(), Ok(100));
    }

    #[test]
    fn as_list_item_not_list_item() {
        let data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_list_item(), Err(DataError::not_basic_type_error()));
    }

    #[test]
    fn as_list_item_mut() {
        let mut data = BasicDataUnitCustom::ListItem(100);
        *data.as_list_item_mut().unwrap() = 200;
        assert_eq!(data.as_list_item(), Ok(200));
    }
    
    #[test]
    fn as_list_item_mut_not_list_item() {
        let mut data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_list_item_mut(), Err(DataError::not_basic_type_error()));
    }

    #[test]
    fn as_char_list() {
        let data = BasicDataUnitCustom::CharList(100);
        assert_eq!(data.as_char_list(), Ok(100));
    }

    #[test]
    fn as_char_list_not_char_list() {
        let mut data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_char_list_mut(), Err(DataError::not_type_error(GarnishDataType::CharList, GarnishDataType::Number)));
    }

    #[test]
    fn as_char_list_mut() {
        let mut data = BasicDataUnitCustom::CharList(100);
        *data.as_char_list_mut().unwrap() = 200;
        assert_eq!(data.as_char_list(), Ok(200));
    }
    
    #[test]
    fn as_char_list_mut_not_char_list() {
        let mut data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_char_list_mut(), Err(DataError::not_type_error(GarnishDataType::CharList, GarnishDataType::Number)));
    }

    #[test]
    fn as_byte_list() {
        let data = BasicDataUnitCustom::ByteList(100);
        assert_eq!(data.as_byte_list(), Ok(100));
    }

    #[test]
    fn as_byte_list_not_byte_list() {
        let mut data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_byte_list_mut(), Err(DataError::not_type_error(GarnishDataType::ByteList, GarnishDataType::Number)));
    }

    #[test]
    fn as_byte_list_mut() {
        let mut data = BasicDataUnitCustom::ByteList(100);
        *data.as_byte_list_mut().unwrap() = 200;
        assert_eq!(data.as_byte_list(), Ok(200));
    }

    #[test]
    fn as_byte_list_mut_not_byte_list() {
        let mut data = BasicDataUnitCustom::Number(100.into());
        assert_eq!(data.as_byte_list_mut(), Err(DataError::not_type_error(GarnishDataType::ByteList, GarnishDataType::Number)));
    }
}
