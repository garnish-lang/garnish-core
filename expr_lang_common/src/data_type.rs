use crate::Error;
use std::convert::{Infallible, TryFrom, TryInto};
use std::{fmt, mem};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum DataType {
    Unit = 1,
    Integer,
    Float,
    Pair,
    Symbol,
    Reference,
    Range,
    Slice,
    Link,
    Expression,
    ExternalMethod,
    Partial,
    Character,
    CharacterList,
    List,
}

const LAST_DATA_TYPE_VALUE: u8 = DataType::List as u8;

impl TryInto<u8> for DataType {
    type Error = Infallible;

    fn try_into(self) -> Result<u8, Self::Error> {
        Ok(self as u8)
    }
}

impl TryFrom<u8> for DataType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Err(format!("Invalid DataType code: {}", value).into()),
            x if x > 0 && x <= LAST_DATA_TYPE_VALUE => {
                Ok(unsafe { mem::transmute::<u8, DataType>(value) })
            }
            _ => Err(format!("Invalid DataType code: {}", value).into()),
        }
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::LAST_DATA_TYPE_VALUE;
    use crate::DataType;
    use std::convert::TryFrom;

    #[test]
    fn data_type_from_valid_u8() {
        let data_type = DataType::try_from(1).unwrap();
        assert_eq!(data_type, DataType::Unit);
    }

    #[test]
    fn data_type_from_zero_results_in_error() {
        let data_type = DataType::try_from(0);
        assert!(data_type.is_err());
    }

    #[test]
    fn data_type_from_list() {
        let data_type = DataType::try_from(LAST_DATA_TYPE_VALUE).unwrap();
        assert_eq!(data_type, DataType::List);
    }

    #[test]
    fn data_type_from_value_larger_than_list() {
        let data_type = DataType::try_from(LAST_DATA_TYPE_VALUE + 1);
        assert!(data_type.is_err());
    }
}
