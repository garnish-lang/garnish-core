use crate::runtime::types::ExpressionDataType;
use crate::simple::data::SimpleData;
use std::hash::{Hash, Hasher};

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct UnitData {
    t: ExpressionDataType,
}

impl UnitData {
    pub fn new() -> Self {
        UnitData { t: ExpressionDataType::Unit }
    }
}

impl SimpleData for UnitData {
    fn get_type(&self) -> ExpressionDataType {
        ExpressionDataType::Unit
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct TrueData {}

impl TrueData {
    pub fn new() -> Self {
        TrueData {}
    }
}

impl SimpleData for TrueData {
    fn get_type(&self) -> ExpressionDataType {
        ExpressionDataType::True
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct FalseData {}

impl FalseData {
    pub fn new() -> Self {
        FalseData {}
    }
}

impl SimpleData for FalseData {
    fn get_type(&self) -> ExpressionDataType {
        ExpressionDataType::False
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct IntegerData {
    value: i32,
}

impl IntegerData {
    pub fn value(&self) -> i32 {
        self.value
    }
}

impl From<i32> for IntegerData {
    fn from(value: i32) -> Self {
        IntegerData { value }
    }
}

impl SimpleData for IntegerData {
    fn get_type(&self) -> ExpressionDataType {
        ExpressionDataType::Integer
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct FloatData {
    value: f64,
}

impl FloatData {
    pub fn value(&self) -> f64 {
        self.value
    }
}

impl From<f64> for FloatData {
    fn from(value: f64) -> Self {
        FloatData { value }
    }
}

impl Hash for FloatData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(&self.value.to_le_bytes());
    }
}

impl SimpleData for FloatData {
    fn get_type(&self) -> ExpressionDataType {
        ExpressionDataType::Float
    }
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct CharData {
    value: char,
}

impl CharData {
    pub fn value(&self) -> char {
        self.value
    }
}

impl From<char> for CharData {
    fn from(value: char) -> Self {
        CharData { value }
    }
}

impl SimpleData for CharData {
    fn get_type(&self) -> ExpressionDataType {
        ExpressionDataType::Char
    }
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct CharListData {
    value: String,
}

impl CharListData {
    pub fn value(&self) -> String {
        self.value.clone()
    }
}

impl From<&str> for CharListData {
    fn from(value: &str) -> Self {
        CharListData { value: value.to_string() }
    }
}

impl From<String> for CharListData {
    fn from(value: String) -> Self {
        CharListData { value }
    }
}

impl SimpleData for CharListData {
    fn get_type(&self) -> ExpressionDataType {
        ExpressionDataType::CharList
    }
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct ByteData {
    value: u8,
}

impl ByteData {
    pub fn value(&self) -> u8 {
        self.value
    }
}

impl From<u8> for ByteData {
    fn from(value: u8) -> Self {
        ByteData { value }
    }
}

impl SimpleData for ByteData {
    fn get_type(&self) -> ExpressionDataType {
        ExpressionDataType::Byte
    }
}

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct ByteListData {
    value: Vec<u8>,
}

impl ByteListData {
    pub fn value(&self) -> &Vec<u8> {
        &self.value
    }
}

impl From<Vec<u8>> for ByteListData {
    fn from(value: Vec<u8>) -> Self {
        ByteListData { value: value }
    }
}

impl SimpleData for ByteListData {
    fn get_type(&self) -> ExpressionDataType {
        ExpressionDataType::ByteList
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct SymbolData {
    value: u64,
}

impl SymbolData {
    pub fn value(&self) -> u64 {
        self.value
    }
}

impl From<u64> for SymbolData {
    fn from(value: u64) -> Self {
        SymbolData { value }
    }
}

impl SimpleData for SymbolData {
    fn get_type(&self) -> ExpressionDataType {
        ExpressionDataType::Symbol
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct ExpressionData {
    value: usize,
}

impl ExpressionData {
    pub fn value(&self) -> usize {
        self.value
    }
}

impl From<usize> for ExpressionData {
    fn from(value: usize) -> Self {
        ExpressionData { value }
    }
}

impl SimpleData for ExpressionData {
    fn get_type(&self) -> ExpressionDataType {
        ExpressionDataType::Expression
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct ExternalData {
    value: usize,
}

impl ExternalData {
    pub fn value(&self) -> usize {
        self.value
    }
}

impl From<usize> for ExternalData {
    fn from(value: usize) -> Self {
        ExternalData { value }
    }
}

impl SimpleData for ExternalData {
    fn get_type(&self) -> ExpressionDataType {
        ExpressionDataType::External
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct PairData {
    left: usize,
    right: usize,
}

impl PairData {
    pub fn left(&self) -> usize {
        self.left
    }

    pub fn right(&self) -> usize {
        self.right
    }
}

impl From<(usize, usize)> for PairData {
    fn from((left, right): (usize, usize)) -> Self {
        PairData { left, right }
    }
}

impl SimpleData for PairData {
    fn get_type(&self) -> ExpressionDataType {
        ExpressionDataType::Pair
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct RangeData {
    start: usize,
    end: usize,
}

impl RangeData {
    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }
}

impl From<(usize, usize)> for RangeData {
    fn from((start, end): (usize, usize)) -> Self {
        RangeData { start, end }
    }
}

impl SimpleData for RangeData {
    fn get_type(&self) -> ExpressionDataType {
        ExpressionDataType::Range
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct SliceData {
    list: usize,
    range: usize,
}

impl SliceData {
    pub fn list(&self) -> usize {
        self.list
    }

    pub fn range(&self) -> usize {
        self.range
    }
}

impl From<(usize, usize)> for SliceData {
    fn from((list, range): (usize, usize)) -> Self {
        SliceData { list, range }
    }
}

impl SimpleData for SliceData {
    fn get_type(&self) -> ExpressionDataType {
        ExpressionDataType::Slice
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct LinkData {
    value: usize,
    linked: usize,
    is_append: bool
}

impl LinkData {
    pub fn value(&self) -> usize {
        self.value
    }

    pub fn linked(&self) -> usize {
        self.linked
    }

    pub fn is_append(&self) -> bool {
        self.is_append
    }
}

impl From<(usize, usize, bool)> for LinkData {
    fn from((value, linked, is_append): (usize, usize, bool)) -> Self {
        LinkData { value, linked, is_append }
    }
}

impl SimpleData for LinkData {
    fn get_type(&self) -> ExpressionDataType {
        ExpressionDataType::Link
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct ListData {
    items: Vec<usize>,
    associations: Vec<usize>,
}

impl ListData {
    pub fn items(&self) -> &Vec<usize> {
        &self.items
    }

    pub fn associations(&self) -> &Vec<usize> {
        &self.associations
    }
}

impl ListData {
    pub fn from_items(items: Vec<usize>, associations: Vec<usize>) -> Self {
        ListData { items, associations }
    }
}

impl SimpleData for ListData {
    fn get_type(&self) -> ExpressionDataType {
        ExpressionDataType::List
    }
}

#[cfg(test)]
mod simple_tests {
    use crate::{AnyData, AsAnyData, ByteData, ByteListData, CharData, CharListData, DataCoersion, ExpressionData, ExpressionDataType, ExternalData, FalseData, FloatData, IntegerData, LinkData, ListData, PairData, RangeData, SimpleData, SliceData, SymbolData, TrueData, UnitData};

    fn all_data_list(remove: usize) -> Vec<AnyData> {
        let mut data: Vec<AnyData> = vec![
            UnitData::new().as_any_data(),
            TrueData::new().as_any_data(), // 1
            FalseData::new().as_any_data(),
            IntegerData::from(10).as_any_data(), // 3
            SymbolData::from(1).as_any_data(),
            ExternalData::from(1).as_any_data(), // 5
            ExpressionData::from(1).as_any_data(),
            PairData::from((1, 2)).as_any_data(), // 7
            ListData::from_items(vec![1, 2, 3], vec![1, 2, 3]).as_any_data(),
            FloatData::from(3.14).as_any_data(), // 9
            CharData::from('a').as_any_data(),
            CharListData::from("abc").as_any_data(), // 11
            ByteData::from(10).as_any_data(),
            ByteListData::from(vec![10u8, 15u8, 20u8]).as_any_data(), // 13
            RangeData::from((1, 2)).as_any_data(),
            SliceData::from((1, 2)).as_any_data(), // 15
            LinkData::from((1, 2, true)).as_any_data(),
        ];

        data.remove(remove);

        data
    }

    #[test]
    fn unit() {
        let data = UnitData::new();

        assert_eq!(data.get_type(), ExpressionDataType::Unit);
    }

    #[test]
    fn unit_coersion() {
        let data = UnitData::new().as_any_data().as_unit();
        assert!(data.is_ok(), "{:?}", data);
    }

    #[test]
    fn not_unit_coersion_fails() {
        let data: Vec<AnyData> = all_data_list(0);

        for d in data {
            assert!(d.as_unit().is_err());
        }
    }

    #[test]
    fn true_boolean() {
        let data = TrueData::new();

        assert_eq!(data.get_type(), ExpressionDataType::True);
    }

    #[test]
    fn true_boolean_coersion() {
        let data = TrueData::new().as_any_data().as_true();
        assert!(data.is_ok(), "{:?}", data);
    }

    #[test]
    fn not_true_boolean_coersion_fails() {
        let data: Vec<AnyData> = all_data_list(1);

        for d in data {
            assert!(d.as_true().is_err());
        }
    }

    #[test]
    fn false_boolean() {
        let data = FalseData::new();

        assert_eq!(data.get_type(), ExpressionDataType::False);
    }

    #[test]
    fn false_boolean_coersion() {
        let data = FalseData::new().as_any_data().as_false();
        assert!(data.is_ok(), "{:?}", data);
    }

    #[test]
    fn not_false_boolean_coersion_fails() {
        let data: Vec<AnyData> = all_data_list(2);

        for d in data {
            assert!(d.as_false().is_err());
        }
    }

    #[test]
    fn integer() {
        assert_eq!(IntegerData::from(10).get_type(), ExpressionDataType::Integer);
    }

    #[test]
    fn integer_value() {
        assert_eq!(IntegerData::from(10).value(), 10);
    }

    #[test]
    fn integer_coersion() {
        let data = IntegerData::from(10).as_any_data().as_integer();
        assert!(data.is_ok(), "{:?}", data);
    }

    #[test]
    fn not_integer_coersion_fails() {
        let data: Vec<AnyData> = all_data_list(3);

        for d in data {
            assert!(d.as_integer().is_err());
        }
    }

    #[test]
    fn float() {
        assert_eq!(FloatData::from(3.14).get_type(), ExpressionDataType::Float);
    }

    #[test]
    fn float_value() {
        assert_eq!(FloatData::from(3.14).value(), 3.14);
    }

    #[test]
    fn float_coersion() {
        let data = FloatData::from(3.14).as_any_data().as_float();
        assert!(data.is_ok(), "{:?}", data);
    }

    #[test]
    fn not_float_coersion_fails() {
        let data: Vec<AnyData> = all_data_list(9);

        for d in data {
            assert!(d.as_float().is_err());
        }
    }

    #[test]
    fn char() {
        assert_eq!(CharData::from('a').get_type(), ExpressionDataType::Char);
    }

    #[test]
    fn char_value() {
        assert_eq!(CharData::from('a').value(), 'a');
    }

    #[test]
    fn char_coersion() {
        let data = CharData::from('a').as_any_data().as_char();
        assert!(data.is_ok(), "{:?}", data);
    }

    #[test]
    fn not_char_coersion_fails() {
        let data: Vec<AnyData> = all_data_list(10);

        for d in data {
            assert!(d.as_char().is_err());
        }
    }

    #[test]
    fn char_list() {
        assert_eq!(CharListData::from("test").get_type(), ExpressionDataType::CharList);
    }

    #[test]
    fn char_list_value() {
        assert_eq!(CharListData::from("test").value(), "test");
    }

    #[test]
    fn char_list_coersion() {
        let data = CharListData::from("test").as_any_data().as_char_list();
        assert!(data.is_ok(), "{:?}", data);
    }

    #[test]
    fn not_char_list_coersion_fails() {
        let data: Vec<AnyData> = all_data_list(11);

        for d in data {
            assert!(d.as_char_list().is_err());
        }
    }

    #[test]
    fn byte() {
        assert_eq!(ByteData::from(10).get_type(), ExpressionDataType::Byte);
    }

    #[test]
    fn byte_value() {
        assert_eq!(ByteData::from(10).value(), 10);
    }

    #[test]
    fn byte_coersion() {
        let data = ByteData::from(10).as_any_data().as_byte();
        assert!(data.is_ok(), "{:?}", data);
    }

    #[test]
    fn not_byte_coersion_fails() {
        let data: Vec<AnyData> = all_data_list(12);

        for d in data {
            assert!(d.as_byte().is_err());
        }
    }

    #[test]
    fn byte_list() {
        assert_eq!(ByteListData::from(vec![10, 15, 20]).get_type(), ExpressionDataType::ByteList);
    }

    #[test]
    fn byte_list_value() {
        assert_eq!(ByteListData::from(vec![10, 15, 20]).value(), &[10, 15, 20]);
    }

    #[test]
    fn byte_list_coersion() {
        let data = ByteListData::from(vec![10, 15, 20]).as_any_data().as_byte_list();
        assert!(data.is_ok(), "{:?}", data);
    }

    #[test]
    fn not_byte_list_coersion_fails() {
        let data: Vec<AnyData> = all_data_list(13);

        for d in data {
            assert!(d.as_byte_list().is_err());
        }
    }

    #[test]
    fn symbol() {
        assert_eq!(SymbolData::from(10).get_type(), ExpressionDataType::Symbol);
    }

    #[test]
    fn symbol_value() {
        assert_eq!(SymbolData::from(10).value(), 10);
    }

    #[test]
    fn symbol_coersion() {
        let data = SymbolData::from(10).as_any_data().as_symbol();
        assert!(data.is_ok(), "{:?}", data);
    }

    #[test]
    fn not_symbol_coersion_fails() {
        let data: Vec<AnyData> = all_data_list(4);

        for d in data {
            assert!(d.as_symbol().is_err());
        }
    }

    #[test]
    fn expression() {
        assert_eq!(ExpressionData::from(10).get_type(), ExpressionDataType::Expression);
    }

    #[test]
    fn expression_value() {
        assert_eq!(ExpressionData::from(10).value(), 10);
    }

    #[test]
    fn expression_coersion() {
        let data = ExpressionData::from(10).as_any_data().as_expression();
        assert!(data.is_ok(), "{:?}", data);
    }

    #[test]
    fn not_expression_coersion_fails() {
        let data: Vec<AnyData> = all_data_list(6);

        for d in data {
            assert!(d.as_expression().is_err());
        }
    }

    #[test]
    fn external() {
        assert_eq!(ExternalData::from(10).get_type(), ExpressionDataType::External);
    }

    #[test]
    fn external_value() {
        assert_eq!(ExternalData::from(10).value(), 10);
    }

    #[test]
    fn external_coersion() {
        let data = ExternalData::from(10).as_any_data().as_external();
        assert!(data.is_ok(), "{:?}", data);
    }

    #[test]
    fn not_external_coersion_fails() {
        let data: Vec<AnyData> = all_data_list(5);

        for d in data {
            assert!(d.as_external().is_err());
        }
    }

    #[test]
    fn pair() {
        assert_eq!(PairData::from((5, 10)).get_type(), ExpressionDataType::Pair);
    }

    #[test]
    fn pair_left() {
        assert_eq!(PairData::from((5, 10)).left(), 5);
    }

    #[test]
    fn pair_right() {
        assert_eq!(PairData::from((5, 10)).right(), 10);
    }

    #[test]
    fn pair_coersion() {
        let data = PairData::from((5, 10)).as_any_data().as_pair();
        assert!(data.is_ok(), "{:?}", data);
    }

    #[test]
    fn not_pair_coersion_fails() {
        let data: Vec<AnyData> = all_data_list(7);

        for d in data {
            assert!(d.as_pair().is_err());
        }
    }

    #[test]
    fn range() {
        assert_eq!(RangeData::from((5, 10)).get_type(), ExpressionDataType::Range);
    }

    #[test]
    fn range_start() {
        assert_eq!(RangeData::from((5, 10)).start(), 5);
    }

    #[test]
    fn range_end() {
        assert_eq!(RangeData::from((5, 10)).end(), 10);
    }

    #[test]
    fn range_coersion() {
        let data = RangeData::from((5, 10)).as_any_data().as_range();
        assert!(data.is_ok(), "{:?}", data);
    }

    #[test]
    fn not_range_coersion_fails() {
        let data: Vec<AnyData> = all_data_list(14);

        for d in data {
            assert!(d.as_range().is_err());
        }
    }

    #[test]
    fn slice() {
        assert_eq!(SliceData::from((5, 10)).get_type(), ExpressionDataType::Slice);
    }

    #[test]
    fn slice_list() {
        assert_eq!(SliceData::from((5, 10)).list(), 5);
    }

    #[test]
    fn slice_range() {
        assert_eq!(SliceData::from((5, 10)).range(), 10);
    }

    #[test]
    fn slice_coersion() {
        let data = SliceData::from((5, 10)).as_any_data().as_slice();
        assert!(data.is_ok(), "{:?}", data);
    }

    #[test]
    fn not_slice_coersion_fails() {
        let data: Vec<AnyData> = all_data_list(15);

        for d in data {
            assert!(d.as_slice().is_err());
        }
    }

    #[test]
    fn link() {
        assert_eq!(LinkData::from((5, 10, true)).get_type(), ExpressionDataType::Link);
    }

    #[test]
    fn link_value() {
        assert_eq!(LinkData::from((5, 10, true)).value(), 5);
    }

    #[test]
    fn link_linked() {
        assert_eq!(LinkData::from((5, 10, true)).linked(), 10);
    }

    #[test]
    fn link_is_append() {
        assert_eq!(LinkData::from((5, 10, true)).is_append(), true);
    }

    #[test]
    fn link_coersion() {
        let data = LinkData::from((5, 10, true)).as_any_data().as_link();
        assert!(data.is_ok(), "{:?}", data);
    }

    #[test]
    fn not_link_coersion_fails() {
        let data: Vec<AnyData> = all_data_list(16);

        for d in data {
            assert!(d.as_link().is_err());
        }
    }

    #[test]
    fn list() {
        assert_eq!(ListData::from_items(vec![1, 2, 3], vec![4, 5, 6]).get_type(), ExpressionDataType::List);
    }

    #[test]
    fn list_items() {
        assert_eq!(ListData::from_items(vec![1, 2, 3], vec![4, 5, 6]).items(), &vec![1, 2, 3]);
    }

    #[test]
    fn list_associations() {
        assert_eq!(ListData::from_items(vec![1, 2, 3], vec![4, 5, 6]).associations(), &vec![4, 5, 6]);
    }

    #[test]
    fn list_coersion() {
        let data = ListData::from_items(vec![1, 2, 3], vec![4, 5, 6]).as_any_data().as_list();
        assert!(data.is_ok(), "{:?}", data);
    }

    #[test]
    fn not_list_coersion_fails() {
        let data: Vec<AnyData> = all_data_list(8);

        for d in data {
            assert!(d.as_list().is_err());
        }
    }
}
