use crate::value::ExpressionValueRef;
use crate::{DataType, Result};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ExpressionResult<'a> {
    value: ExpressionValueRef<'a>,
}

impl<'a> ExpressionResult<'a> {
    pub fn new(data: &'a [u8], symbol_table: Option<&'a HashMap<String, usize>>) -> Result<Self> {
        Ok(ExpressionResult {
            value: ExpressionValueRef::new(data, symbol_table)?,
        })
    }

    pub fn new_with_start(
        data: &'a [u8],
        symbol_table: Option<&'a HashMap<String, usize>>,
        start: usize,
    ) -> Result<Self> {
        Ok(ExpressionResult {
            value: ExpressionValueRef::new_with_start(data, symbol_table, start)?,
        })
    }

    pub fn get_type(&self) -> Result<DataType> {
        self.value.get_type()
    }

    pub fn as_integer(&self) -> Result<i32> {
        self.value.as_integer()
    }

    pub fn as_float(&self) -> Result<f32> {
        self.value.as_float()
    }

    pub fn as_string(&self) -> Result<String> {
        self.value.as_string()
    }

    pub fn as_char(&self) -> Result<char> {
        self.value.as_char()
    }

    pub fn as_symbol(&self) -> Result<usize> {
        self.value.as_symbol()
    }

    pub fn is_unit(&self) -> bool {
        self.value.is_unit()
    }

    pub fn is_range(&self) -> bool {
        self.value.is_range()
    }

    pub fn get_range_flags(&self) -> Result<u8> {
        self.value.get_range_flags()
    }

    pub fn get_range_min(&self) -> Result<ExpressionValueRef> {
        self.value.get_range_start()
    }

    pub fn get_range_max(&self) -> Result<ExpressionValueRef> {
        self.value.get_range_end()
    }

    pub fn get_range_step(&self) -> Result<ExpressionValueRef> {
        self.value.get_range_step()
    }

    pub fn is_pair(&self) -> bool {
        self.value.is_pair()
    }

    pub fn get_pair_left(&self) -> Result<ExpressionValueRef> {
        self.value.get_pair_left()
    }

    pub fn get_pair_right(&self) -> Result<ExpressionValueRef> {
        self.value.get_pair_right()
    }

    pub fn is_partial(&self) -> bool {
        self.value.is_partial()
    }

    pub fn get_partial_base(&self) -> Result<ExpressionValueRef> {
        self.value.get_partial_base()
    }

    pub fn get_partial_value(&self) -> Result<ExpressionValueRef> {
        self.value.get_partial_value()
    }
    pub fn is_slice(&self) -> bool {
        self.value.is_slice()
    }

    pub fn get_slice_source(&self) -> Result<ExpressionValueRef> {
        self.value.get_slice_source()
    }

    pub fn get_slice_range(&self) -> Result<ExpressionValueRef> {
        self.value.get_slice_range()
    }

    pub fn is_link(&self) -> bool {
        self.value.is_link()
    }

    pub fn get_link_head(&self) -> Result<ExpressionValueRef> {
        self.value.get_link_head()
    }

    pub fn get_link_value(&self) -> Result<ExpressionValueRef> {
        self.value.get_link_value()
    }

    pub fn get_link_next(&self) -> Result<ExpressionValueRef> {
        self.value.get_link_next()
    }

    pub fn is_list(&self) -> bool {
        self.value.is_list()
    }

    pub fn list_len(&self) -> Result<usize> {
        self.value.list_len()
    }

    pub fn get_list_item(&self, index: usize) -> Result<ExpressionValueRef> {
        self.value.get_list_item(index)
    }
}

#[cfg(test)]
mod tests {
    use crate::byte_vec::DataVecWriter;
    use crate::data_type::DataType;
    use crate::{ExpressionResult, ExpressionValue};
    use std::convert::TryInto;

    #[test]
    fn as_integer() {
        let v = ExpressionValue::from(ExpressionValue::integer(1000000000));
        let result = ExpressionResult::new(&v.get_data()[..], None).unwrap();

        assert_eq!(result.as_integer().unwrap(), 1000000000);
    }
    #[test]
    fn as_float() {
        let v = ExpressionValue::from(ExpressionValue::float(3.14));
        let result = ExpressionResult::new(&v.get_data()[..], None).unwrap();

        assert_eq!(result.as_float().unwrap(), 3.14);
    }

    #[test]
    fn is_unit_true() {
        let v = vec![DataType::Unit.try_into().unwrap()];
        let result = ExpressionResult::new(&v[..], None).unwrap();

        assert!(result.is_unit());
    }

    #[test]
    fn is_unit_false() {
        let v = ExpressionValue::from(ExpressionValue::integer(1000000000));
        let result = ExpressionResult::new(&v.get_data()[..], None).unwrap();

        assert!(!result.is_unit());
    }

    #[test]
    fn as_symbol() {
        let mut v = vec![];
        DataVecWriter::new(&mut v)
            .push_data_type(DataType::Symbol)
            .push_size(1000);

        let result = ExpressionResult::new(&v[..], None).unwrap();

        assert_eq!(result.as_symbol().unwrap(), 1000);
    }
}
