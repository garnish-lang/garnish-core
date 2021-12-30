use crate::runtime::types::ExpressionDataType;
use std::any::Any;
use std::fmt::Debug;
use std::slice::Iter;

pub type DataCoersionResult<T> = Result<T, String>;

pub trait SimpleData: Any + Debug {
    fn get_type(&self) -> ExpressionDataType;
}

pub(crate) type AnyData = Box<dyn Any>;

pub(crate) trait AsAnyData {
    fn as_any_data(self) -> AnyData;
}

#[derive(Debug)]
pub struct SimpleDataList {
    list: Vec<AnyData>,
}

impl Default for SimpleDataList {
    fn default() -> Self {
        SimpleDataList::new().append(UnitData::new())
    }
}

impl SimpleDataList {
    pub fn new() -> Self {
        SimpleDataList { list: vec![] }
    }

    pub fn append<T: SimpleData>(mut self, item: T) -> Self {
        self.list.push(item.as_any_data());
        self
    }

    pub fn push<T: SimpleData>(&mut self, item: T) {
        self.list.push(item.as_any_data());
    }

    pub fn get(&self, index: usize) -> Option<&AnyData> {
        self.list.get(index)
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn iter(&self) -> Iter<'_, AnyData>{
        self.list.iter()
    }
}

impl PartialEq<SimpleDataList> for SimpleDataList {
    fn eq(&self, other: &SimpleDataList) -> bool {
        if self.list.len() != other.list.len() {
            return false;
        }

        let mut equal = true;
        for i in 0..self.list.len() {
            match (self.list.get(i), other.list.get(i)) {
                (Some(left), Some(right)) => {
                    if !data_equal(left, right) {
                        equal = false;
                        break;
                    }
                }
                _ => {
                    equal = false;
                    break;
                }
            }
        }

        equal
    }
}

impl<T: SimpleData> AsAnyData for T {
    fn as_any_data(self) -> AnyData {
        Box::new(self)
    }
}

pub trait DataCoersion {
    fn as_unit(&self) -> DataCoersionResult<UnitData>;
    fn as_true(&self) -> DataCoersionResult<TrueData>;
    fn as_false(&self) -> DataCoersionResult<FalseData>;
    fn as_integer(&self) -> DataCoersionResult<IntegerData>;
    fn as_symbol(&self) -> DataCoersionResult<SymbolData>;
    fn as_expression(&self) -> DataCoersionResult<ExpressionData>;
    fn as_external(&self) -> DataCoersionResult<ExternalData>;
    fn as_pair(&self) -> DataCoersionResult<PairData>;
    fn as_list(&self) -> DataCoersionResult<ListData>;
}

fn downcast_result<T: SimpleData + Clone>(b: &AnyData) -> DataCoersionResult<T> {
    match b.downcast_ref::<T>() {
        Some(value) => Ok(value.clone()),
        None => Err(format!("Could not cast from {:?}.", b)),
    }
}

impl DataCoersion for AnyData {
    fn as_unit(&self) -> DataCoersionResult<UnitData> {
        downcast_result(self)
    }

    fn as_true(&self) -> DataCoersionResult<TrueData> {
        downcast_result(self)
    }

    fn as_false(&self) -> DataCoersionResult<FalseData> {
        downcast_result(self)
    }

    fn as_integer(&self) -> DataCoersionResult<IntegerData> {
        downcast_result(self)
    }

    fn as_symbol(&self) -> DataCoersionResult<SymbolData> {
        downcast_result(self)
    }

    fn as_expression(&self) -> DataCoersionResult<ExpressionData> {
        downcast_result(self)
    }

    fn as_external(&self) -> DataCoersionResult<ExternalData> {
        downcast_result(self)
    }

    fn as_pair(&self) -> DataCoersionResult<PairData> {
        downcast_result(self)
    }

    fn as_list(&self) -> DataCoersionResult<ListData> {
        downcast_result(self)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
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

#[derive(Clone, PartialEq, Eq, Debug)]
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

#[derive(Clone, PartialEq, Eq, Debug)]
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

#[derive(Clone, PartialEq, Eq, Debug)]
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

#[derive(Clone, PartialEq, Eq, Debug)]
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

#[derive(Clone, PartialEq, Eq, Debug)]
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

#[derive(Clone, PartialEq, Eq, Debug)]
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

#[derive(Clone, PartialEq, Eq, Debug)]
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

#[derive(Clone, PartialEq, Eq, Debug)]
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

// Comparison utilities

fn cmp_any<T: 'static + PartialEq + Eq>(left: &Box<dyn Any>, right: &Box<dyn Any>) -> bool {
    match (left.downcast_ref::<T>(), right.downcast_ref::<T>()) {
        (Some(left), Some(right)) => left == right,
        _ => false,
    }
}

pub fn data_equal(left: &Box<dyn Any>, right: &Box<dyn Any>) -> bool {
    cmp_any::<UnitData>(left, right)
        || cmp_any::<TrueData>(left, right)
        || cmp_any::<FalseData>(left, right)
        || cmp_any::<IntegerData>(left, right)
        || cmp_any::<SymbolData>(left, right)
        || cmp_any::<ExpressionData>(left, right)
        || cmp_any::<ExternalData>(left, right)
        || cmp_any::<PairData>(left, right)
        || cmp_any::<ListData>(left, right)
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

#[cfg(test)]
mod comparisons {
    use crate::simple::expression_data::cmp_any;
    use crate::{data_equal, AsAnyData, ExpressionData, ExternalData, FalseData, IntegerData, ListData, PairData, SymbolData, TrueData, UnitData};

    #[test]
    fn same_type_equal() {
        let left = IntegerData::from(10).as_any_data();
        let right = IntegerData::from(10).as_any_data();

        assert!(cmp_any::<IntegerData>(&left, &right));
    }

    #[test]
    fn same_type_not_equal() {
        let left = IntegerData::from(10).as_any_data();
        let right = IntegerData::from(20).as_any_data();

        assert!(!cmp_any::<IntegerData>(&left, &right));
    }

    #[test]
    fn different_type_not_equal() {
        let left = IntegerData::from(10).as_any_data();
        let right = UnitData::new().as_any_data();

        assert!(!cmp_any::<IntegerData>(&left, &right));
    }

    #[test]
    fn data_equal_all_supported_types() {
        let cases = vec![
            (UnitData::new().as_any_data(), UnitData::new().as_any_data(), true),
            (TrueData::new().as_any_data(), TrueData::new().as_any_data(), true),
            (FalseData::new().as_any_data(), FalseData::new().as_any_data(), true),
            (UnitData::new().as_any_data(), FalseData::new().as_any_data(), false),
            (TrueData::new().as_any_data(), UnitData::new().as_any_data(), false),
            (TrueData::new().as_any_data(), FalseData::new().as_any_data(), false),
            // Integer
            (IntegerData::from(10).as_any_data(), IntegerData::from(10).as_any_data(), true),
            (IntegerData::from(10).as_any_data(), IntegerData::from(20).as_any_data(), false),
            // Symbol
            (SymbolData::from(10).as_any_data(), SymbolData::from(10).as_any_data(), true),
            (SymbolData::from(10).as_any_data(), SymbolData::from(20).as_any_data(), false),
            // Expression
            (ExpressionData::from(10).as_any_data(), ExpressionData::from(10).as_any_data(), true),
            (ExpressionData::from(10).as_any_data(), ExpressionData::from(20).as_any_data(), false),
            // External
            (ExternalData::from(10).as_any_data(), ExternalData::from(10).as_any_data(), true),
            (ExternalData::from(10).as_any_data(), ExternalData::from(20).as_any_data(), false),
            // Pair
            (PairData::from((10, 20)).as_any_data(), PairData::from((10, 20)).as_any_data(), true),
            (PairData::from((10, 20)).as_any_data(), PairData::from((10, 10)).as_any_data(), false),
            // List
            (
                ListData::from_items(vec![1, 2, 3], vec![4, 5, 6]).as_any_data(),
                ListData::from_items(vec![1, 2, 3], vec![4, 5, 6]).as_any_data(),
                true,
            ),
            (
                ListData::from_items(vec![1, 2, 3], vec![4, 5, 6]).as_any_data(),
                ListData::from_items(vec![1, 2], vec![4, 5]).as_any_data(),
                false,
            ),
        ];

        for (left, right, expected_result) in cases {
            assert_eq!(data_equal(&left, &right), expected_result);
        }
    }
}
