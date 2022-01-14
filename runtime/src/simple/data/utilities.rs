use crate::simple::data::{
    AnyData, ByteData, ByteListData, CharData, CharListData, ExpressionData, ExternalData, FalseData, FloatData, IntegerData, ListData, PairData,
    RangeData, SimpleData, SliceData, SymbolData, TrueData, UnitData,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

use std::any::Any;
use crate::LinkData;

pub type DataCoersionResult<T> = Result<T, String>;

pub trait DataCoersion {
    fn as_unit(&self) -> DataCoersionResult<UnitData>;
    fn as_true(&self) -> DataCoersionResult<TrueData>;
    fn as_false(&self) -> DataCoersionResult<FalseData>;
    fn as_integer(&self) -> DataCoersionResult<IntegerData>;
    fn as_float(&self) -> DataCoersionResult<FloatData>;
    fn as_char(&self) -> DataCoersionResult<CharData>;
    fn as_char_list(&self) -> DataCoersionResult<CharListData>;
    fn as_byte(&self) -> DataCoersionResult<ByteData>;
    fn as_byte_list(&self) -> DataCoersionResult<ByteListData>;
    fn as_symbol(&self) -> DataCoersionResult<SymbolData>;
    fn as_expression(&self) -> DataCoersionResult<ExpressionData>;
    fn as_external(&self) -> DataCoersionResult<ExternalData>;
    fn as_pair(&self) -> DataCoersionResult<PairData>;
    fn as_range(&self) -> DataCoersionResult<RangeData>;
    fn as_slice(&self) -> DataCoersionResult<SliceData>;
    fn as_link(&self) -> DataCoersionResult<LinkData>;
    fn as_list(&self) -> DataCoersionResult<ListData>;
}

fn downcast_result<T: SimpleData + Clone>(b: &AnyData) -> DataCoersionResult<T> {
    match b.data.downcast_ref::<T>() {
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

    fn as_float(&self) -> DataCoersionResult<FloatData> {
        downcast_result(self)
    }

    fn as_char(&self) -> DataCoersionResult<CharData> {
        downcast_result(self)
    }

    fn as_char_list(&self) -> DataCoersionResult<CharListData> {
        downcast_result(self)
    }

    fn as_byte(&self) -> DataCoersionResult<ByteData> {
        downcast_result(self)
    }

    fn as_byte_list(&self) -> DataCoersionResult<ByteListData> {
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

    fn as_range(&self) -> DataCoersionResult<RangeData> {
        downcast_result(self)
    }

    fn as_slice(&self) -> DataCoersionResult<SliceData> {
        downcast_result(self)
    }

    fn as_link(&self) -> DataCoersionResult<LinkData> {
        downcast_result(self)
    }

    fn as_list(&self) -> DataCoersionResult<ListData> {
        downcast_result(self)
    }
}

// Comparison utilities

fn cmp_any<T: 'static + PartialEq + PartialEq>(left: &Box<dyn Any>, right: &Box<dyn Any>) -> bool {
    match (left.downcast_ref::<T>(), right.downcast_ref::<T>()) {
        (Some(left), Some(right)) => left == right,
        _ => false,
    }
}

pub(crate) fn data_equal(left: &Box<dyn Any>, right: &Box<dyn Any>) -> bool {
    cmp_any::<UnitData>(left, right)
        || cmp_any::<TrueData>(left, right)
        || cmp_any::<FalseData>(left, right)
        || cmp_any::<IntegerData>(left, right)
        || cmp_any::<FloatData>(left, right)
        || cmp_any::<SymbolData>(left, right)
        || cmp_any::<ExpressionData>(left, right)
        || cmp_any::<ExternalData>(left, right)
        || cmp_any::<PairData>(left, right)
        || cmp_any::<RangeData>(left, right)
        || cmp_any::<SliceData>(left, right)
        || cmp_any::<LinkData>(left, right)
        || cmp_any::<ListData>(left, right)
        || cmp_any::<CharData>(left, right)
        || cmp_any::<CharListData>(left, right)
        || cmp_any::<ByteData>(left, right)
        || cmp_any::<ByteListData>(left, right)
}

pub fn symbol_value(value: &str) -> u64 {
    let mut h = DefaultHasher::new();
    value.hash(&mut h);
    let hv = h.finish();

    hv
}

#[cfg(test)]
mod comparisons {
    use crate::simple::data::utilities::cmp_any;
    use crate::{data_equal, AsAnyData, ByteData, ByteListData, CharData, CharListData, ExpressionData, ExternalData, FalseData, FloatData, IntegerData, ListData, PairData, RangeData, SliceData, SymbolData, TrueData, UnitData, LinkData};

    #[test]
    fn same_type_equal() {
        let left = IntegerData::from(10).as_any_data();
        let right = IntegerData::from(10).as_any_data();

        assert!(cmp_any::<IntegerData>(&left.data, &right.data));
    }

    #[test]
    fn same_type_not_equal() {
        let left = IntegerData::from(10).as_any_data();
        let right = IntegerData::from(20).as_any_data();

        assert!(!cmp_any::<IntegerData>(&left.data, &right.data));
    }

    #[test]
    fn different_type_not_equal() {
        let left = IntegerData::from(10).as_any_data();
        let right = UnitData::new().as_any_data();

        assert!(!cmp_any::<IntegerData>(&left.data, &right.data));
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
            // Float
            (FloatData::from(10.5).as_any_data(), FloatData::from(10.5).as_any_data(), true),
            (FloatData::from(10.5).as_any_data(), FloatData::from(20.5).as_any_data(), false),
            // Char
            (CharData::from('a').as_any_data(), CharData::from('a').as_any_data(), true),
            (CharData::from('a').as_any_data(), CharData::from('b').as_any_data(), false),
            // Byte
            (ByteData::from(10).as_any_data(), ByteData::from(10).as_any_data(), true),
            (ByteData::from(10).as_any_data(), ByteData::from(20).as_any_data(), false),
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
            // Pair
            (LinkData::from((10, 20, true)).as_any_data(), LinkData::from((10, 20, true)).as_any_data(), true),
            (LinkData::from((10, 20, true)).as_any_data(), LinkData::from((10, 10, true)).as_any_data(), false),
            // Range
            (
                RangeData::from((10, 20)).as_any_data(),
                RangeData::from((10, 20)).as_any_data(),
                true,
            ),
            (
                RangeData::from((10, 20)).as_any_data(),
                RangeData::from((10, 10)).as_any_data(),
                false,
            ),
            // Slice
            (SliceData::from((10, 20)).as_any_data(), SliceData::from((10, 20)).as_any_data(), true),
            (SliceData::from((10, 20)).as_any_data(), SliceData::from((10, 10)).as_any_data(), false),
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
            // Char List
            (CharListData::from("abc").as_any_data(), CharListData::from("abc").as_any_data(), true),
            (CharListData::from("abc").as_any_data(), CharListData::from("abd").as_any_data(), false),
            // Byte List
            (
                ByteListData::from(vec![1, 2, 3]).as_any_data(),
                ByteListData::from(vec![1, 2, 3]).as_any_data(),
                true,
            ),
            (
                ByteListData::from(vec![1, 2, 3]).as_any_data(),
                ByteListData::from(vec![1, 2, 4]).as_any_data(),
                false,
            ),
        ];

        for (left, right, expected_result) in cases {
            assert_eq!(
                data_equal(&left.data, &right.data),
                expected_result,
                "{:?} == {:?}",
                left.get_data_type(),
                right.get_data_type()
            );
        }
    }
}
