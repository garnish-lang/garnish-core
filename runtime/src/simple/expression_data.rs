use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::{collections::HashMap, convert::TryInto, hash::Hasher};

use crate::runtime::types::ExpressionDataType;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExpressionData {
    pub(crate) data_type: ExpressionDataType,
    pub(crate) bytes: Vec<u8>,
    pub(crate) symbols: HashMap<String, u64>,
}

impl ExpressionData {
    fn new(data_type: ExpressionDataType, bytes: Vec<u8>) -> ExpressionData {
        ExpressionData {
            data_type,
            bytes,
            symbols: HashMap::new(),
        }
    }

    pub fn unit() -> ExpressionData {
        ExpressionData::new(ExpressionDataType::Unit, vec![])
    }

    pub fn boolean_true() -> ExpressionData {
        ExpressionData::new(ExpressionDataType::True, vec![])
    }

    pub fn boolean_false() -> ExpressionData {
        ExpressionData::new(ExpressionDataType::False, vec![])
    }

    pub fn integer(i: i64) -> ExpressionData {
        ExpressionData::new(ExpressionDataType::Integer, i.to_le_bytes().to_vec())
    }

    pub fn symbol(_: &String, value: u64) -> ExpressionData {
        let d = ExpressionData::new(ExpressionDataType::Symbol, value.to_le_bytes().to_vec());
        // d.symbols.insert(name.clone(), value);
        d
    }

    pub fn pair(left: usize, right: usize) -> ExpressionData {
        ExpressionData::new(ExpressionDataType::Pair, [left.to_le_bytes(), right.to_le_bytes()].concat())
    }

    pub fn list(refs: Vec<usize>, association_refs: Vec<usize>) -> ExpressionData {
        let mut vec_of_bytes: Vec<[u8; std::mem::size_of::<usize>()]> = vec![];
        vec_of_bytes.push(refs.len().to_le_bytes());

        for r in refs {
            vec_of_bytes.push(r.to_le_bytes());
        }

        vec_of_bytes.push(association_refs.len().to_le_bytes());

        for r in association_refs {
            vec_of_bytes.push(r.to_le_bytes());
        }

        ExpressionData::new(ExpressionDataType::List, vec_of_bytes.concat())
    }

    pub fn expression(i: usize) -> ExpressionData {
        ExpressionData::new(ExpressionDataType::Expression, i.to_le_bytes().to_vec())
    }

    pub fn external(i: usize) -> ExpressionData {
        ExpressionData::new(ExpressionDataType::External, i.to_le_bytes().to_vec())
    }

    pub fn symbol_from_string(s: &str) -> ExpressionData {
        let mut h = DefaultHasher::new();
        s.hash(&mut h);
        let hv = h.finish();

        let mut d = ExpressionData::new(ExpressionDataType::Symbol, hv.to_le_bytes().to_vec());
        d.symbols.insert(s.to_string(), hv);
        d
    }

    pub fn get_type(&self) -> ExpressionDataType {
        self.data_type
    }

    pub fn get_symbols(&self) -> &HashMap<String, u64> {
        &self.symbols
    }

    pub fn as_boolean(&self) -> Result<bool, String> {
        match self.get_type() {
            ExpressionDataType::True => Ok(true),
            ExpressionDataType::False => Ok(false),
            _ => Err(format!("Not of boolean type.")),
        }
    }

    pub fn as_integer(&self) -> Result<i64, String> {
        let (bytes, _) = self.bytes.split_at(std::mem::size_of::<i64>());
        Ok(i64::from_le_bytes(match bytes.try_into() {
            Ok(v) => v,
            Err(e) => Result::Err(e.to_string())?,
        }))
    }

    pub fn as_reference(&self) -> Result<usize, String> {
        let (bytes, _) = self.bytes.split_at(std::mem::size_of::<usize>());
        Ok(usize::from_le_bytes(match bytes.try_into() {
            Ok(v) => v,
            Err(e) => Result::Err(e.to_string())?,
        }))
    }

    pub fn as_symbol_value(&self) -> Result<u64, String> {
        let (bytes, _) = self.bytes.split_at(std::mem::size_of::<u64>());
        Ok(u64::from_le_bytes(match bytes.try_into() {
            Ok(v) => v,
            Err(e) => Result::Err(e.to_string())?,
        }))
    }

    pub fn as_symbol_name(&self) -> Result<String, String> {
        // calling this method assumes only one symbol available
        match self.symbols.keys().next() {
            None => Err("No symbols in expression data".to_string()),
            Some(v) => Ok(v.clone()),
        }
    }

    pub fn as_pair(&self) -> Result<(usize, usize), String> {
        let (left_bytes, remaining) = self.bytes.split_at(std::mem::size_of::<usize>());
        let (right_bytes, _) = remaining.split_at(std::mem::size_of::<usize>());
        Ok((
            usize::from_le_bytes(match left_bytes.try_into() {
                Ok(v) => v,
                Err(e) => Result::Err(e.to_string())?,
            }),
            usize::from_le_bytes(match right_bytes.try_into() {
                Ok(v) => v,
                Err(e) => Result::Err(e.to_string())?,
            }),
        ))
    }

    pub fn as_list(&self) -> Result<(Vec<usize>, Vec<usize>), String> {
        let (len_bytes, mut remaining) = self.bytes.split_at(std::mem::size_of::<usize>());
        let mut list = vec![];
        let mut associative_list = vec![];

        let len = usize::from_le_bytes(match len_bytes.try_into() {
            Ok(v) => v,
            Err(e) => Result::Err(e.to_string())?,
        });

        for _ in 0..len {
            let (value, next) = remaining.split_at(std::mem::size_of::<usize>());

            list.push(usize::from_le_bytes(match value.try_into() {
                Ok(v) => v,
                Err(e) => Result::Err(e.to_string())?,
            }));

            remaining = next
        }

        // associative refs
        let (len_bytes, mut remaining) = remaining.split_at(std::mem::size_of::<usize>());

        let len = usize::from_le_bytes(match len_bytes.try_into() {
            Ok(v) => v,
            Err(e) => Result::Err(e.to_string())?,
        });

        for _ in 0..len {
            let (value, next) = remaining.split_at(std::mem::size_of::<usize>());

            associative_list.push(usize::from_le_bytes(match value.try_into() {
                Ok(v) => v,
                Err(e) => Result::Err(e.to_string())?,
            }));

            remaining = next
        }

        Ok((list, associative_list))
    }

    pub fn as_expression(&self) -> Result<usize, String> {
        let (bytes, _) = self.bytes.split_at(std::mem::size_of::<usize>());
        Ok(usize::from_le_bytes(match bytes.try_into() {
            Ok(v) => v,
            Err(e) => Result::Err(e.to_string())?,
        }))
    }

    pub fn as_external(&self) -> Result<usize, String> {
        let (bytes, _) = self.bytes.split_at(std::mem::size_of::<usize>());
        Ok(usize::from_le_bytes(match bytes.try_into() {
            Ok(v) => v,
            Err(e) => Result::Err(e.to_string())?,
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::vec;

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
    fn boolean_true() {
        let d = ExpressionData::boolean_true();

        assert_eq!(d.data_type, ExpressionDataType::True);
    }

    #[test]
    fn boolean_false() {
        let d = ExpressionData::boolean_false();

        assert_eq!(d.data_type, ExpressionDataType::False);
    }

    #[test]
    fn as_boolean() {
        let d = ExpressionData::boolean_true();
        assert!(d.as_boolean().unwrap());

        let d = ExpressionData::boolean_false();
        assert!(!d.as_boolean().unwrap());
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
    }

    #[test]
    fn symbol_from_string() {
        let d = ExpressionData::symbol_from_string(&"my_symbol".to_string());

        let mut h = DefaultHasher::new();
        "my_symbol".hash(&mut h);
        let hv = h.finish();

        assert!(d.symbols.contains_key("my_symbol"));
        assert_eq!(d.symbols.get("my_symbol").unwrap(), &hv);
        assert_eq!(d.data_type, ExpressionDataType::Symbol);
        assert_eq!(d.bytes, hv.to_le_bytes());
    }

    #[test]
    fn as_symbol_value() {
        let d = ExpressionData::symbol_from_string(&"my_symbol".to_string());

        let mut h = DefaultHasher::new();
        "my_symbol".hash(&mut h);
        let hv = h.finish();

        assert_eq!(d.as_symbol_value().unwrap(), hv);
    }

    #[test]
    fn pair() {
        let d = ExpressionData::pair(2, 5);

        let two = 2usize.to_le_bytes();
        let five = 5usize.to_le_bytes();
        let expected = [two, five].concat();

        assert_eq!(d.get_type(), ExpressionDataType::Pair);
        assert_eq!(d.bytes[..], expected[..]);
    }

    #[test]
    fn as_pair() {
        let d = ExpressionData::pair(2, 5);

        let (left, right) = d.as_pair().unwrap();

        assert_eq!(left, 2);
        assert_eq!(right, 5);
    }

    #[test]
    fn list() {
        let d = ExpressionData::list(vec![1, 2, 3], vec![]);

        let one = 1usize.to_le_bytes();
        let two = 2usize.to_le_bytes();
        let three = 3usize.to_le_bytes();
        let zero = 0usize.to_le_bytes();
        let expected = [three, one, two, three, zero].concat();

        assert_eq!(d.get_type(), ExpressionDataType::List);
        assert_eq!(d.bytes[..], expected[..]);
    }

    #[test]
    fn as_list() {
        let d = ExpressionData::list(vec![1, 2, 3], vec![]);

        let (list, associations) = d.as_list().unwrap();

        assert_eq!(list.len(), 3);
        assert_eq!(list[0], 1);
        assert_eq!(list[1], 2);
        assert_eq!(list[2], 3);
        assert_eq!(associations.len(), 0);
    }

    #[test]
    fn list_with_associations() {
        let d = ExpressionData::list(vec![1, 2, 3], vec![4, 5, 6]);

        let one = 1usize.to_le_bytes();
        let two = 2usize.to_le_bytes();
        let three = 3usize.to_le_bytes();
        let four = 4usize.to_le_bytes();
        let five = 5usize.to_le_bytes();
        let six = 6usize.to_le_bytes();
        let expected = [three, one, two, three, three, four, five, six].concat();

        assert_eq!(d.get_type(), ExpressionDataType::List);
        assert_eq!(d.bytes[..], expected[..]);
    }

    #[test]
    fn expression() {
        let d = ExpressionData::expression(1);

        assert_eq!(d.data_type, ExpressionDataType::Expression);
        assert_eq!(d.bytes, 1usize.to_le_bytes());
    }

    #[test]
    fn as_expression() {
        let d = ExpressionData::expression(1);

        assert_eq!(d.as_expression().unwrap(), 1usize)
    }

    #[test]
    fn external() {
        let d = ExpressionData::external(1);

        assert_eq!(d.data_type, ExpressionDataType::External);
        assert_eq!(d.bytes, 1usize.to_le_bytes());
    }

    #[test]
    fn as_external() {
        let d = ExpressionData::external(1);

        assert_eq!(d.as_external().unwrap(), 1usize)
    }
}
