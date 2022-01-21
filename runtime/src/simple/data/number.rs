use crate::{DataCastResult, DataError, GarnishNumber};
use std::cmp::Ordering;
use crate::SimpleNumber::{Float, Integer};
use std::ops::{Add, Sub, Mul, Div, Rem};

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

fn do_op<IntOp, FloatOp>(left: &SimpleNumber, right: &SimpleNumber, int_op: IntOp, float_op: FloatOp) -> Option<SimpleNumber>
where IntOp: Fn(i32, i32) -> i32, FloatOp: Fn(f64, f64) -> f64
{
    Some(match (left, right) {
        (SimpleNumber::Integer(v1), SimpleNumber::Integer(v2)) => Integer(int_op(*v1, *v2)),
        (SimpleNumber::Float(v1), SimpleNumber::Float(v2)) => Float(float_op(*v1, *v2)),
        (SimpleNumber::Integer(v1), SimpleNumber::Float(v2)) => Float(float_op(f64::from(*v1), *v2)),
        (SimpleNumber::Float(v1), SimpleNumber::Integer(v2)) => Float(float_op(*v1, f64::from(*v2))),
    })
}

impl GarnishNumber for SimpleNumber {
    fn plus(self, rhs: Self) -> Option<Self> {
        do_op(&self, &rhs, i32::add, f64::add)
    }

    fn subtract(self, rhs: Self) -> Option<Self> {
        do_op(&self, &rhs, i32::sub, f64::sub)
    }

    fn multiply(self, rhs: Self) -> Option<Self> {
        do_op(&self, &rhs, i32::mul, f64::mul)
    }

    fn divide(self, rhs: Self) -> Option<Self> {
        do_op(&self, &rhs, i32::div, f64::div)
    }

    fn power(self, rhs: Self) -> Option<Self> {
        Some(match (self, rhs) {
            (SimpleNumber::Integer(v1), SimpleNumber::Integer(v2)) => {
                if v2 < 0 {
                    return None;
                }

                Integer(v1.pow(v2 as u32))
            },
            (SimpleNumber::Float(v1), SimpleNumber::Float(v2)) => {
                if v2 < 0.0 {
                    return None;
                }

                Float(v1.powf(v2))
            },
            (SimpleNumber::Integer(v1), SimpleNumber::Float(v2)) => {
                if v2 < 0.0 {
                    return None;
                }

                Float(f64::from(v1).powf(v2))
            },
            (SimpleNumber::Float(v1), SimpleNumber::Integer(v2)) => {
                if v2 < 0 {
                    return None;
                }

                Float(v1.powf(f64::from(v2)))
            }
        })
    }

    fn integer_divide(self, rhs: Self) -> Option<Self> {
        Some(match (self, rhs) {
            (SimpleNumber::Integer(v1), SimpleNumber::Integer(v2)) => Integer(v1 / v2),
            (SimpleNumber::Float(v1), SimpleNumber::Float(v2)) => Integer(v1 as i32 / v2 as i32),
            (SimpleNumber::Integer(v1), SimpleNumber::Float(v2)) => Integer(v1 / v2 as i32),
            (SimpleNumber::Float(v1), SimpleNumber::Integer(v2)) => Integer(v1 as i32 / v2),
        })
    }

    fn remainder(self, rhs: Self) -> Option<Self> {
        do_op(&self, &rhs, i32::rem, f64::rem)
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
        assert_eq!(Integer(10).plus(Integer(20)).unwrap(), Integer(30));
        assert_eq!(Float(10.0).plus(Float(20.0)).unwrap(), Float(30.0));
        assert_eq!(Integer(10).plus(Float(20.0)).unwrap(), Float(30.0));
        assert_eq!(Float(10.0).plus(Integer(20)).unwrap(), Float(30.0));
    }

    #[test]
    fn subtract() {
        assert_eq!(Integer(10).subtract(Integer(20)).unwrap(), Integer(-10));
        assert_eq!(Float(10.0).subtract(Float(20.0)).unwrap(), Float(-10.0));
        assert_eq!(Integer(10).subtract(Float(20.0)).unwrap(), Float(-10.0));
        assert_eq!(Float(10.0).subtract(Integer(20)).unwrap(), Float(-10.0));
    }

    #[test]
    fn multiply() {
        assert_eq!(Integer(10).multiply(Integer(20)).unwrap(), Integer(200));
        assert_eq!(Float(10.0).multiply(Float(20.0)).unwrap(), Float(200.0));
        assert_eq!(Integer(10).multiply(Float(20.0)).unwrap(), Float(200.0));
        assert_eq!(Float(10.0).multiply(Integer(20)).unwrap(), Float(200.0));
    }

    #[test]
    fn divide() {
        assert_eq!(Integer(10).divide(Integer(20)).unwrap(), Integer(0));
        assert_eq!(Float(10.0).divide(Float(20.0)).unwrap(), Float(0.5));
        assert_eq!(Integer(10).divide(Float(20.0)).unwrap(), Float(0.5));
        assert_eq!(Float(10.0).divide(Integer(20)).unwrap(), Float(0.5));
    }

    #[test]
    fn power() {
        assert_eq!(Integer(10).power(Integer(3)).unwrap(), Integer(1000));
        assert_eq!(Float(10.0).power(Float(3.0)).unwrap(), Float(1000.0));
        assert_eq!(Integer(10).power(Float(3.0)).unwrap(), Float(1000.0));
        assert_eq!(Float(10.0).power(Integer(3)).unwrap(), Float(1000.0));
    }

    #[test]
    fn negative_power_none() {
        assert_eq!(Integer(10).power(Integer(-3)), None);
        assert_eq!(Float(10.0).power(Float(-3.0)), None);
        assert_eq!(Integer(10).power(Float(-3.0)), None);
        assert_eq!(Float(10.0).power(Integer(-3)), None);
    }

    #[test]
    fn integer_divide() {
        assert_eq!(Integer(10).integer_divide(Integer(20)).unwrap(), Integer(0));
        assert_eq!(Float(10.0).integer_divide(Float(20.0)).unwrap(), Integer(0));
        assert_eq!(Integer(10).integer_divide(Float(20.0)).unwrap(), Integer(0));
        assert_eq!(Float(10.0).integer_divide(Integer(20)).unwrap(), Integer(0));
    }

    #[test]
    fn remainder() {
        assert_eq!(Integer(10).remainder(Integer(20)).unwrap(), Integer(10));
        assert_eq!(Float(10.0).remainder(Float(20.0)).unwrap(), Float(10.0));
        assert_eq!(Integer(10).remainder(Float(20.0)).unwrap(), Float(10.0));
        assert_eq!(Float(10.0).remainder(Integer(20)).unwrap(), Float(10.0));
    }
}
