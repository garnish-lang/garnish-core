use crate::data::DataCastResult;
use crate::data::SimpleNumber::*;
use crate::DataError;
use garnish_traits::{GarnishNumber, TypeConstants};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Add, Div, Mul, Rem, Sub};

#[derive(Copy, Clone, Debug)]
pub enum SimpleNumber {
    Integer(i32),
    Float(f64),
}

impl Eq for SimpleNumber {}

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

    pub fn to_integer(&self) -> Self {
        match self {
            SimpleNumber::Integer(v) => Integer(*v),
            SimpleNumber::Float(v) => Float(*v as f64),
        }
    }

    pub fn to_float(&self) -> Self {
        match self {
            SimpleNumber::Integer(v) => Integer(*v as i32),
            SimpleNumber::Float(v) => Float(*v),
        }
    }
}

impl Display for SimpleNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Integer(v) => f.write_str(v.to_string().as_str()),
            Float(v) => f.write_str(v.to_string().as_str()),
        }
    }
}

impl Hash for SimpleNumber {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Integer(v) => v.hash(state),
            Float(v) => format!("{}", v).hash(state),
        }
    }
}

impl From<i32> for SimpleNumber {
    fn from(x: i32) -> Self {
        SimpleNumber::Integer(x)
    }
}

impl From<SimpleNumber> for i32 {
    fn from(x: SimpleNumber) -> Self {
        match x {
            Integer(v) => v,
            Float(v) => v as i32,
        }
    }
}

impl From<usize> for SimpleNumber {
    fn from(x: usize) -> Self {
        SimpleNumber::Integer(x as i32)
    }
}

impl From<SimpleNumber> for usize {
    fn from(x: SimpleNumber) -> Self {
        match x {
            Integer(v) => v as usize,
            Float(v) => v as i32 as usize,
        }
    }
}

impl From<f64> for SimpleNumber {
    fn from(x: f64) -> Self {
        SimpleNumber::Float(x)
    }
}

impl From<SimpleNumber> for f64 {
    fn from(x: SimpleNumber) -> Self {
        match x {
            Integer(v) => f64::from(v),
            Float(v) => v,
        }
    }
}

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

fn do_op<IntOp, FloatOp>(left: &SimpleNumber, right: &SimpleNumber, int_op: IntOp, float_op: FloatOp) -> Option<SimpleNumber>
where
    IntOp: Fn(i32, i32) -> (i32, bool),
    FloatOp: Fn(f64, f64) -> f64,
{
    Some(match (left, right) {
        (Integer(v1), Integer(v2)) => {
            let (v, o) = int_op(*v1, *v2);
            if o {
                return None;
            }

            Integer(v)
        }
        (Float(v1), Float(v2)) => {
            let f = float_op(*v1, *v2);
            if f.is_infinite() {
                return None;
            } else {
                Float(f)
            }
        }
        (Integer(v1), Float(v2)) => {
            let f = float_op(f64::from(*v1), *v2);
            if f.is_infinite() {
                return None;
            } else {
                Float(f)
            }
        }
        (Float(v1), Integer(v2)) => {
            let f = float_op(*v1, f64::from(*v2));
            if f.is_infinite() {
                return None;
            } else {
                Float(f)
            }
        }
    })
}

impl TypeConstants for SimpleNumber {
    fn one() -> Self {
        Integer(1)
    }

    fn zero() -> Self {
        Integer(0)
    }

    fn max_value() -> Self {
        Float(f64::MAX)
    }
}

impl Add<i32> for SimpleNumber {
    type Output = Self;

    fn add(self, rhs: i32) -> Self::Output {
        self.plus(rhs.into()).unwrap()
    }
}

impl Add<SimpleNumber> for i32 {
    type Output = SimpleNumber;

    fn add(self, rhs: SimpleNumber) -> Self::Output {
        rhs.plus(self.into()).unwrap()
    }
}

impl Mul<i32> for SimpleNumber {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        self.multiply(rhs.into()).unwrap()
    }
}

impl Add<f64> for SimpleNumber {
    type Output = Self;

    fn add(self, rhs: f64) -> Self::Output {
        self.plus(rhs.into()).unwrap()
    }
}

impl GarnishNumber for SimpleNumber {
    fn plus(self, rhs: Self) -> Option<Self> {
        do_op(&self, &rhs, i32::overflowing_add, f64::add)
    }

    fn subtract(self, rhs: Self) -> Option<Self> {
        do_op(&self, &rhs, i32::overflowing_sub, f64::sub)
    }

    fn multiply(self, rhs: Self) -> Option<Self> {
        do_op(&self, &rhs, i32::overflowing_mul, f64::mul)
    }

    fn divide(self, rhs: Self) -> Option<Self> {
        if rhs == Integer(0) || rhs == Float(0.0) {
            return None;
        }

        do_op(&self, &rhs, i32::overflowing_div, f64::div)
    }

    fn power(self, rhs: Self) -> Option<Self> {
        Some(match (self, rhs) {
            (Integer(v1), Integer(v2)) => {
                if v2 < 0 {
                    return None;
                }

                let (v, o) = v1.overflowing_pow(v2 as u32);
                if o {
                    return None;
                }

                Integer(v)
            }
            (Float(v1), Float(v2)) => {
                if v2 < 0.0 {
                    return None;
                }

                let f = v1.powf(v2);
                if f.is_infinite() {
                    return None;
                } else {
                    Float(f)
                }
            }
            (Integer(v1), Float(v2)) => {
                if v2 < 0.0 {
                    return None;
                }

                let f = f64::from(v1).powf(v2);
                if f.is_infinite() {
                    return None;
                } else {
                    Float(f)
                }
            }
            (Float(v1), Integer(v2)) => {
                if v2 < 0 {
                    return None;
                }

                let f = v1.powf(f64::from(v2));
                if f.is_infinite() {
                    return None;
                } else {
                    Float(f)
                }
            }
        })
    }

    fn integer_divide(self, rhs: Self) -> Option<Self> {
        if rhs == Integer(0) || rhs == Float(0.0) {
            return None;
        }

        Some(match (self, rhs) {
            (Integer(v1), Integer(v2)) => {
                let (v, o) = v1.overflowing_div(v2);
                if o {
                    return None;
                }

                Integer(v)
            }
            (Float(v1), Float(v2)) => {
                let (v, o) = (v1 as i32).overflowing_div(v2 as i32);
                if o {
                    return None;
                }

                Integer(v)
            }
            (Integer(v1), Float(v2)) => {
                let (v, o) = v1.overflowing_div(v2 as i32);
                if o {
                    return None;
                }

                Integer(v)
            }
            (Float(v1), Integer(v2)) => {
                let (v, o) = (v1 as i32).overflowing_div(v2);
                if o {
                    return None;
                }

                Integer(v)
            }
        })
    }

    fn remainder(self, rhs: Self) -> Option<Self> {
        if rhs == Integer(0) || rhs == Float(0.0) {
            return None;
        }

        do_op(&self, &rhs, i32::overflowing_rem, f64::rem)
    }

    fn absolute_value(self) -> Option<Self> {
        Some(match self {
            Integer(v) => {
                let (v, o) = v.overflowing_abs();
                if o {
                    return None;
                }

                Integer(v)
            }
            Float(v) => Float(v.abs()),
        })
    }

    fn opposite(self) -> Option<Self> {
        Some(match self {
            Integer(v) => {
                let (v, o) = v.overflowing_neg();
                if o {
                    return None;
                }

                Integer(v)
            }
            Float(v) => Float(-v),
        })
    }

    fn increment(self) -> Option<Self> {
        Some(match self {
            Integer(v) => {
                let (v, o) = v.overflowing_add(1);
                if o {
                    return None;
                }

                Integer(v)
            }
            Float(v) => Float(v + 1.0),
        })
    }

    fn decrement(self) -> Option<Self> {
        Some(match self {
            Integer(v) => {
                let (v, o) = v.overflowing_sub(1);
                if o {
                    return None;
                }

                Integer(v)
            }
            Float(v) => Float(v - 1.0),
        })
    }

    fn bitwise_not(self) -> Option<Self> {
        Some(match self {
            Integer(v) => Integer(!v),
            Float(_) => return None,
        })
    }

    fn bitwise_and(self, rhs: Self) -> Option<Self> {
        Some(match (self, rhs) {
            (Integer(v1), Integer(v2)) => Integer(v1 & v2),
            _ => return None,
        })
    }

    fn bitwise_or(self, rhs: Self) -> Option<Self> {
        Some(match (self, rhs) {
            (Integer(v1), Integer(v2)) => Integer(v1 | v2),
            _ => return None,
        })
    }

    fn bitwise_xor(self, rhs: Self) -> Option<Self> {
        Some(match (self, rhs) {
            (Integer(v1), Integer(v2)) => Integer(v1 ^ v2),
            _ => return None,
        })
    }

    fn bitwise_shift_left(self, rhs: Self) -> Option<Self> {
        Some(match (self, rhs) {
            (Integer(v1), Integer(v2)) => Integer(v1 << v2),
            _ => return None,
        })
    }

    fn bitwise_shift_right(self, rhs: Self) -> Option<Self> {
        Some(match (self, rhs) {
            (Integer(v1), Integer(v2)) => Integer(v1 >> v2),
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::data::SimpleNumber;
    use crate::data::SimpleNumber::*;
    use garnish_traits::GarnishNumber;
    use std::usize;

    #[test]
    fn from_i32() {
        assert_eq!(SimpleNumber::from(10i32), Integer(10));
    }

    #[test]
    fn number_integer_into_i32() {
        assert_eq!(i32::from(Integer(10)), 10);
    }

    #[test]
    fn number_float_into_i32() {
        assert_eq!(i32::from(Float(10.0)), 10);
    }

    #[test]
    fn from_usize() {
        assert_eq!(SimpleNumber::from(10usize), Integer(10));
    }

    #[test]
    fn number_integer_into_usize() {
        assert_eq!(usize::from(Integer(10)), 10);
    }

    #[test]
    fn number_float_into_usize() {
        assert_eq!(usize::from(Float(10.0)), 10);
    }

    #[test]
    fn from_f64() {
        assert_eq!(SimpleNumber::from(10.0f64), Float(10.0));
    }

    #[test]
    fn number_integer_into_f64() {
        assert_eq!(f64::from(Integer(10)), 10.0);
    }

    #[test]
    fn number_float_into_f64() {
        assert_eq!(f64::from(Float(10.0)), 10.0);
    }

    #[test]
    fn as_integer() {
        assert_eq!(SimpleNumber::Integer(10).as_integer().unwrap(), 10.into());
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
    fn add_overflow() {
        assert_eq!(Integer(i32::MAX).plus(Integer(1)), None);
        assert_eq!(Float(f64::MAX).plus(Float(f64::MAX)), None);
        // following can't be infinite
        // assert_eq!(Float(f64::MAX).plus(Integer(i32::MAX)), None);
        // assert_eq!(Integer(i32::MAX).plus(Float(f64::MAX)), None);
    }

    #[test]
    fn subtract() {
        assert_eq!(Integer(10).subtract(Integer(20)).unwrap(), Integer(-10));
        assert_eq!(Float(10.0).subtract(Float(20.0)).unwrap(), Float(-10.0));
        assert_eq!(Integer(10).subtract(Float(20.0)).unwrap(), Float(-10.0));
        assert_eq!(Float(10.0).subtract(Integer(20)).unwrap(), Float(-10.0));
    }

    #[test]
    fn subtract_overflow() {
        assert_eq!(Integer(i32::MIN).subtract(Integer(1)), None);
        assert_eq!(Float(f64::MIN).subtract(Float(f64::MAX)), None);
        // following can't be infinite
        // assert_eq!(Float(f64::MIN).subtract(Integer(i32::MAX)), None);
        // assert_eq!(Integer(i32::MIN).subtract(Float(f64::MAX)), None);
    }

    #[test]
    fn multiply() {
        assert_eq!(Integer(10).multiply(Integer(20)).unwrap(), Integer(200));
        assert_eq!(Float(10.0).multiply(Float(20.0)).unwrap(), Float(200.0));
        assert_eq!(Integer(10).multiply(Float(20.0)).unwrap(), Float(200.0));
        assert_eq!(Float(10.0).multiply(Integer(20)).unwrap(), Float(200.0));
    }

    #[test]
    fn multiply_overflow() {
        assert_eq!(Integer(i32::MAX).multiply(Integer(2)), None);
        assert_eq!(Float(f64::MAX).multiply(Float(f64::MAX)), None);
        assert_eq!(Integer(i32::MAX).multiply(Float(f64::MAX)), None);
        assert_eq!(Float(f64::MAX).multiply(Integer(i32::MAX)), None);
    }

    #[test]
    fn divide() {
        assert_eq!(Integer(10).divide(Integer(20)).unwrap(), Integer(0));
        assert_eq!(Float(10.0).divide(Float(20.0)).unwrap(), Float(0.5));
        assert_eq!(Integer(10).divide(Float(20.0)).unwrap(), Float(0.5));
        assert_eq!(Float(10.0).divide(Integer(20)).unwrap(), Float(0.5));
    }

    #[test]
    fn division_overflow() {
        assert_eq!(Integer(i32::MIN).divide(Integer(-1)), None);
        assert_eq!(Float(f64::MIN).divide(Float(-f64::MIN_POSITIVE)), None);
        assert_eq!(Integer(i32::MIN).divide(Float(-f64::MIN_POSITIVE)), None);
        // following can't be infinite
        // assert_eq!(Float(-f64::MIN_POSITIVE).divide(Integer(i32::MIN)).unwrap(), Float(0.5));
    }

    #[test]
    fn division_by_zero() {
        assert_eq!(Integer(i32::MAX).divide(Integer(0)), None);
        assert_eq!(Float(10.0).divide(Float(0.0)), None);
        assert_eq!(Integer(10).divide(Float(0.0)), None);
        assert_eq!(Float(10.0).divide(Integer(0)), None);
    }

    #[test]
    fn power() {
        assert_eq!(Integer(10).power(Integer(3)).unwrap(), Integer(1000));
        assert_eq!(Float(10.0).power(Float(3.0)).unwrap(), Float(1000.0));
        assert_eq!(Integer(10).power(Float(3.0)).unwrap(), Float(1000.0));
        assert_eq!(Float(10.0).power(Integer(3)).unwrap(), Float(1000.0));
    }

    #[test]
    fn power_overflow() {
        assert_eq!(Integer(i32::MAX).power(Integer(2)), None);
        assert_eq!(Float(f64::MAX).power(Float(f64::MAX)), None);
        assert_eq!(Integer(i32::MAX).power(Float(f64::MAX)), None);
        assert_eq!(Float(f64::MAX).power(Integer(i32::MAX)), None);
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
    fn integer_division_overflow() {
        assert_eq!(Integer(i32::MIN).integer_divide(Integer(-1)), None);
        assert_eq!(Float(f64::MIN).integer_divide(Float(-1.0)), None);
        assert_eq!(Integer(i32::MIN).integer_divide(Float(-1.0)), None);
        assert_eq!(Float(f64::MIN).integer_divide(Integer(-1)), None);
    }

    #[test]
    fn integer_division_by_zero() {
        assert_eq!(Integer(i32::MIN).integer_divide(Integer(0)), None);
        assert_eq!(Float(10.0).integer_divide(Float(0.0)), None);
        assert_eq!(Integer(10).integer_divide(Float(0.0)), None);
        assert_eq!(Float(10.0).integer_divide(Integer(0)), None);
    }

    #[test]
    fn remainder() {
        assert_eq!(Integer(10).remainder(Integer(20)).unwrap(), Integer(10));
        assert_eq!(Float(10.0).remainder(Float(20.0)).unwrap(), Float(10.0));
        assert_eq!(Integer(10).remainder(Float(20.0)).unwrap(), Float(10.0));
        assert_eq!(Float(10.0).remainder(Integer(20)).unwrap(), Float(10.0));
    }

    #[test]
    fn remainder_overflow() {
        assert_eq!(Integer(i32::MIN).remainder(Integer(-1)), None);
    }

    #[test]
    fn remainder_by_zero() {
        assert_eq!(Integer(i32::MIN).remainder(Integer(0)), None);
    }

    #[test]
    fn absolute_value() {
        assert_eq!(Integer(-10).absolute_value().unwrap(), Integer(10));
        assert_eq!(Float(-10.0).absolute_value().unwrap(), Float(10.0));
    }

    #[test]
    fn absolute_value_overflow() {
        assert_eq!(Integer(i32::MIN).absolute_value(), None);
    }

    #[test]
    fn opposite() {
        assert_eq!(Integer(10).opposite().unwrap(), Integer(-10));
        assert_eq!(Float(10.0).opposite().unwrap(), Float(-10.0));
    }

    #[test]
    fn opposite_overflow() {
        assert_eq!(Integer(i32::MIN).opposite(), None);
    }

    #[test]
    fn increment() {
        assert_eq!(Integer(10).increment().unwrap(), Integer(11));
        assert_eq!(Float(10.0).increment().unwrap(), Float(11.0));
    }

    #[test]
    fn increment_overflow() {
        assert_eq!(Integer(i32::MAX).increment(), None);
    }

    #[test]
    fn decrement() {
        assert_eq!(Integer(10).decrement().unwrap(), Integer(9));
        assert_eq!(Float(10.0).decrement().unwrap(), Float(9.0));
    }

    #[test]
    fn decrement_overflow() {
        assert_eq!(Integer(i32::MIN).decrement(), None);
    }

    #[test]
    fn bitwise_not() {
        assert_eq!(Integer(10).bitwise_not().unwrap(), Integer(!10));
        assert!(Float(10.0).bitwise_not().is_none());
    }

    #[test]
    fn bitwise_and() {
        assert_eq!(Integer(10).bitwise_and(Integer(20)).unwrap(), Integer(10 & 20));
        assert!(Float(10.0).bitwise_and(Float(1.0)).is_none());
    }

    #[test]
    fn bitwise_or() {
        assert_eq!(Integer(10).bitwise_or(Integer(20)).unwrap(), Integer(10 | 20));
        assert!(Float(10.0).bitwise_or(Float(1.0)).is_none());
    }

    #[test]
    fn bitwise_xor() {
        assert_eq!(Integer(10).bitwise_xor(Integer(20)).unwrap(), Integer(10 ^ 20));
        assert!(Float(10.0).bitwise_xor(Float(1.0)).is_none());
    }

    #[test]
    fn bitwise_left_shift() {
        assert_eq!(Integer(10).bitwise_shift_left(Integer(2)).unwrap(), Integer(10 << 2));
        assert!(Float(10.0).bitwise_shift_left(Float(1.0)).is_none());
    }

    #[test]
    fn bitwise_right_shift() {
        assert_eq!(Integer(10).bitwise_shift_right(Integer(2)).unwrap(), Integer(10 >> 2));
        assert!(Float(10.0).bitwise_shift_right(Float(1.0)).is_none());
    }
}
