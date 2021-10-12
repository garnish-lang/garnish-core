use std::{convert::TryInto};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ExpressionDataType {
    Unit = 1,
    Reference,
    Integer
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct ExpressionData {
    pub(crate) data_type: ExpressionDataType,
    pub(crate) bytes: Vec<u8>
}

impl ExpressionData {
    pub fn unit() -> ExpressionData {
        ExpressionData { data_type: ExpressionDataType::Unit, bytes: vec![] }
    }

    pub fn integer(i: i64) -> ExpressionData {
        ExpressionData { data_type: ExpressionDataType::Integer, bytes: i.to_le_bytes()[..].to_vec() }
    }

    pub fn reference(r: usize) -> ExpressionData {
        ExpressionData { data_type: ExpressionDataType::Reference, bytes: r.to_le_bytes()[..].to_vec() }
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
    use crate::{ExpressionData, ExpressionDataType};

    #[test]
    fn get_data_type_expression_data() {
        let d = ExpressionData::unit();
        assert_eq!(d.get_type(), ExpressionDataType::Unit);
    }

    #[test]
    fn integer_expression_data() {
        let d = ExpressionData::integer(1234567890);
        assert_eq!(d.bytes, 1234567890i64.to_le_bytes());
    }

    #[test]
    fn reference_expression_data() {
        let d = ExpressionData::reference(1234567890);
        assert_eq!(d.bytes, 1234567890usize.to_le_bytes());
    }

    #[test]
    fn expression_data_as_integer() {
        assert_eq!(ExpressionData::reference(1234567890).as_integer().unwrap(), 1234567890)
    }
}