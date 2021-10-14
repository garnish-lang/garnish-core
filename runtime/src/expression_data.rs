use std::{collections::HashMap, convert::TryInto, hash::Hasher};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash};
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ExpressionDataType {
    Unit = 1,
    Reference,
    Integer,
    Symbol,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExpressionData {
    pub(crate) data_type: ExpressionDataType,
    pub(crate) bytes: Vec<u8>,
    pub(crate) symbols: HashMap<String, u64>,
}

impl ExpressionData {
    fn new(data_type: ExpressionDataType, bytes: Vec<u8>) -> ExpressionData {
        ExpressionData { data_type, bytes, symbols: HashMap::new() }
    }

    pub fn unit() -> ExpressionData {
        ExpressionData::new(ExpressionDataType::Unit, vec![])
    }

    pub fn integer(i: i64) -> ExpressionData {
        ExpressionData::new(ExpressionDataType::Integer, i.to_le_bytes().to_vec())
    }

    pub fn reference(r: usize) -> ExpressionData {
        ExpressionData::new(ExpressionDataType::Reference, r.to_le_bytes().to_vec())
    }

    pub fn symbol(name: &String, value: u64) -> ExpressionData {
        let mut d = ExpressionData::new(ExpressionDataType::Symbol, value.to_le_bytes().to_vec());
        d.symbols.insert(name.clone(), value);
        d
    }

    pub fn symbol_from_string(s: &String) -> ExpressionData {
        let mut h = DefaultHasher::new();
        s.hash(&mut h);
        let hv = h.finish();

        let mut d = ExpressionData::new(ExpressionDataType::Symbol, hv.to_le_bytes().to_vec());
        d.symbols.insert(s.clone(), hv);
        d
    }

    pub fn get_type(&self) -> ExpressionDataType {
        self.data_type
    }

    pub fn as_integer(&self) -> Result<i64, String> {
        let (bytes, _) = self.bytes.split_at(std::mem::size_of::<i64>());
        Ok(i64::from_le_bytes(match bytes.try_into() {
            Ok(v) => v,
            Err(e) => Result::Err(e.to_string())?
        }))
    }

    pub fn as_reference(&self) -> Result<usize, String> {
        let (bytes, _) = self.bytes.split_at(std::mem::size_of::<usize>());
        Ok(usize::from_le_bytes(match bytes.try_into() {
            Ok(v) => v,
            Err(e) => Result::Err(e.to_string())?
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    use crate::{ExpressionData, ExpressionDataType};

    #[test]
    fn get_data_type_expression_data() {
        let d = ExpressionData::unit();

        assert_eq!(d.get_type(), ExpressionDataType::Unit);
    }

    #[test]
    fn integer_expression_data() {
        let d = ExpressionData::integer(1234567890);

        assert_eq!(d.data_type, ExpressionDataType::Integer);
        assert_eq!(d.bytes, 1234567890i64.to_le_bytes());
    }

    #[test]
    fn reference_expression_data() {
        let d = ExpressionData::reference(1234567890);

        assert_eq!(d.data_type, ExpressionDataType::Reference);
        assert_eq!(d.bytes, 1234567890usize.to_le_bytes());
    }

    #[test]
    fn expression_data_as_integer() {
        assert_eq!(ExpressionData::integer(1234567890).as_integer().unwrap(), 1234567890)
    }

    #[test]
    fn symbol() {
        let d = ExpressionData::symbol(&"false".to_string(), 0);

        assert_eq!(d.data_type, ExpressionDataType::Symbol);
        assert_eq!(d.bytes, 0u64.to_le_bytes());
        assert_eq!(*d.symbols.get("false").unwrap(), 0);
    }

    #[test]
    fn symbol_from_string() {
        let d = ExpressionData::symbol_from_string(&"my_symbol".to_string());

        let mut h = DefaultHasher::new();
        "my_symbol".hash(&mut h);
        let hv = h.finish();

        assert_eq!(d.data_type, ExpressionDataType::Symbol);
        assert_eq!(d.bytes, hv.to_le_bytes());
        assert_eq!(*d.symbols.get("my_symbol").unwrap(), hv);
    }
}