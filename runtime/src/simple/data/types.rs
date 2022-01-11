use std::hash::{Hash, Hasher};
use crate::runtime::types::ExpressionDataType;
use crate::simple::data::SimpleData;

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
        state.finish();
    }
}

impl SimpleData for FloatData {
    fn get_type(&self) -> ExpressionDataType {
        ExpressionDataType::Float
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
    use crate::{
        AnyData, AsAnyData, DataCoersion, ExpressionData, ExpressionDataType, ExternalData, FalseData, IntegerData, ListData, PairData, SimpleData,
        SymbolData, TrueData, UnitData,
    };

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