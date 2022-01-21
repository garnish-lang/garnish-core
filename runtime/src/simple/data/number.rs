use crate::{DataCastResult, DataError, GarnishNumber};
use std::cmp::Ordering;
use crate::SimpleNumber::{Float, Integer};

#[derive(Copy, Clone, Debug)]
pub enum SimpleNumber {
    Integer(i32),
    Float(f64),
}

impl Eq for SimpleNumber {}

impl PartialEq for SimpleNumber {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SimpleNumber::Integer(v1), SimpleNumber::Integer(v2)) => v1 == v2,
            (SimpleNumber::Float(v1), SimpleNumber::Float(v2)) => v1 == v2,
            (SimpleNumber::Integer(v1), SimpleNumber::Float(v2)) => f64::from(*v1) == *v2,
            (SimpleNumber::Float(v1), SimpleNumber::Integer(v2)) => *v1 == f64::from(*v2),
        }
    }
}

impl PartialOrd for SimpleNumber {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (SimpleNumber::Integer(v1), SimpleNumber::Integer(v2)) => v1.partial_cmp(v2),
            (SimpleNumber::Float(v1), SimpleNumber::Float(v2)) => v1.partial_cmp(v2),
            (SimpleNumber::Integer(v1), SimpleNumber::Float(v2)) => f64::from(*v1).partial_cmp(v2),
            (SimpleNumber::Float(v1), SimpleNumber::Integer(v2)) => (v1).partial_cmp(&f64::from(*v2)),
        }
    }
}

impl SimpleNumber {
    pub fn as_integer(&self) -> DataCastResult<i32> {
        match self {
            SimpleNumber::Integer(v) => Ok(*v),
            _ => Err(DataError::from(format!("{:?} is not an Integer.", self))),
        }
    }

    pub fn as_float(&self) -> DataCastResult<f64> {
        match self {
            SimpleNumber::Float(v) => Ok(*v),
            _ => Err(DataError::from(format!("{:?} is not an Float.", self))),
        }
    }
}

impl GarnishNumber for SimpleNumber {
    fn add(self, rhs: Self) -> Option<Self> {
        Some(match (self, rhs) {
            (SimpleNumber::Integer(v1), SimpleNumber::Integer(v2)) => Integer(v1 + v2),
            (SimpleNumber::Float(v1), SimpleNumber::Float(v2)) => Float(v1 + v2),
            (SimpleNumber::Integer(v1), SimpleNumber::Float(v2)) => Float(f64::from(v1) + v2),
            (SimpleNumber::Float(v1), SimpleNumber::Integer(v2)) => Float(v1 + f64::from(v2)),
        })
    }

    fn subtract(self, rhs: Self) -> Option<Self> {
        todo!()
    }

    fn multiply(self, rhs: Self) -> Option<Self> {
        todo!()
    }

    fn divide(self, rhs: Self) -> Option<Self> {
        todo!()
    }

    fn power(self, rhs: Self) -> Option<Self> {
        todo!()
    }

    fn integer_divide(self, rhs: Self) -> Option<Self> {
        todo!()
    }

    fn remainder(self, rhs: Self) -> Option<Self> {
        todo!()
    }

    fn absolute_value(self) -> Option<Self> {
        todo!()
    }

    fn opposite(self) -> Option<Self> {
        todo!()
    }

    fn increment(self) -> Option<Self> {
        todo!()
    }

    fn decrement(self) -> Option<Self> {
        todo!()
    }

    fn bitwise_not(self) -> Option<Self> {
        todo!()
    }

    fn bitwise_and(self, rhs: Self) -> Option<Self> {
        todo!()
    }

    fn bitwise_or(self, rhs: Self) -> Option<Self> {
        todo!()
    }

    fn bitwise_xor(self, rhs: Self) -> Option<Self> {
        todo!()
    }

    fn bitwise_shift_left(self, rhs: Self) -> Option<Self> {
        todo!()
    }

    fn bitwise_shift_right(self, rhs: Self) -> Option<Self> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{GarnishNumber, SimpleNumber};
    use crate::SimpleNumber::{Float, Integer};

    #[test]
    fn as_integer() {
        assert_eq!(SimpleNumber::Integer(10).as_integer().unwrap(), 10);
    }

    #[test]
    fn as_integer_not_integer() {
        assert!(SimpleNumber::Float(10.0).as_integer().is_err());
    }

    #[test]
    fn as_float() {
        assert_eq!(SimpleNumber::Float(10.0).as_float().unwrap(), 10.0);
    }

    #[test]
    fn as_float_not_float() {
        assert!(SimpleNumber::Integer(10).as_float().is_err());
    }

    #[test]
    fn comparable() {
        assert!(SimpleNumber::Integer(10) > SimpleNumber::Float(5.0));
    }

    #[test]
    fn add() {
        assert_eq!(Integer(10).add(Integer(20)).unwrap(), Integer(30));
        assert_eq!(Float(10.0).add(Float(20.0)).unwrap(), Float(30.0));
        assert_eq!(Integer(10).add(Float(20.0)).unwrap(), Float(30.0));
        assert_eq!(Float(10.0).add(Integer(20)).unwrap(), Float(30.0));
    }
}
