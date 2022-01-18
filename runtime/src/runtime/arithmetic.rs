use log::trace;

use crate::runtime::data::Overflowable;
use crate::{next_two_raw_ref, push_integer, push_pair, push_unit, ExpressionDataType, GarnishLangRuntimeData, RuntimeError};

pub fn add<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    let (right_addr, left_addr) = next_two_raw_ref(this)?;

    let types = (this.get_data_type(left_addr)?, this.get_data_type(right_addr)?);
    trace!("Attempting addition between {:?} at {:?} and {:?} at {:?}", types.0, left_addr, types.1, right_addr);

    match types {
        (ExpressionDataType::Integer, ExpressionDataType::Integer) => {
            let left = this.get_integer(left_addr)?;
            let right = this.get_integer(right_addr)?;

            let (sum, overflowed) = left.overflowable_addition(right);

            if overflowed {
                let left = this.add_integer(sum)?;
                let right = this.add_true()?;
                push_pair(this, left, right)
            } else {
                push_integer(this, sum)
            }
        }
        (ExpressionDataType::Integer, ExpressionDataType::Float) => {
            let left = this.get_integer(left_addr)?;
            let right = this.get_float(right_addr)?;

            match Data::integer_to_float(left) {
                Some(left) => {
                    let sum = left + right;
                    this.add_float(sum).and_then(|r| this.push_register(r))?;
                    Ok(())
                }
                None => push_unit(this)
            }
        }
        (ExpressionDataType::Float, ExpressionDataType::Integer) => {
            let left = this.get_float(left_addr)?;
            let right = this.get_integer(right_addr)?;

            match Data::integer_to_float(right) {
                Some(right) => {
                    let sum = left + right;
                    this.add_float(sum).and_then(|r| this.push_register(r))?;
                    Ok(())
                }
                None => push_unit(this)
            }
        }
        (ExpressionDataType::Float, ExpressionDataType::Float) => {
            let left = this.get_float(left_addr)?;
            let right = this.get_float(right_addr)?;

            let sum = left + right;

            this.add_float(sum).and_then(|r| this.push_register(r))?;

            Ok(())
        }
        _ => {
            trace!("Unsupported types pushing unit");
            push_unit(this)
        },
    }
}

pub fn subtract<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    let (right_addr, left_addr) = next_two_raw_ref(this)?;

    let types = (this.get_data_type(left_addr)?, this.get_data_type(right_addr)?);
    trace!("Attempting addition between {:?} at {:?} and {:?} at {:?}", types.0, left_addr, types.1, right_addr);

    match types {
        (ExpressionDataType::Integer, ExpressionDataType::Integer) => {
            let left = this.get_integer(left_addr)?;
            let right = this.get_integer(right_addr)?;

            let result= left - right;

            push_integer(this, result)
        }
        _ => {
            trace!("Unsupported types pushing unit");
            push_unit(this)
        },
    }
}

pub fn multiply<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    todo!()
}

pub fn power<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    todo!()
}

pub fn divide<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    todo!()
}

pub fn integer_divide<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    todo!()
}

pub fn remainder<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    todo!()
}

pub fn absolute_value<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    todo!()
}

pub fn opposite<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn perform_addition_integer_integer() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_integer(10).unwrap();
        let int2 = runtime.add_integer(20).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.add().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_integer(new_data_start).unwrap(), 30);
    }

    #[test]
    fn perform_addition_integer_float() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_integer(10).unwrap();
        let int2 = runtime.add_float(20.5).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.add().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_float(new_data_start).unwrap(), 30.5);
    }

    #[test]
    fn perform_addition_float_integer() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_float(10.5).unwrap();
        let int2 = runtime.add_integer(20).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.add().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_float(new_data_start).unwrap(), 30.5);
    }

    #[test]
    fn perform_addition_float_float() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_float(10.5).unwrap();
        let int2 = runtime.add_float(20.5).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.add().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_float(new_data_start).unwrap(), 31.0);
    }

    #[test]
    fn addition_with_overflow_is_pair() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_integer(11).unwrap();
        let int2 = runtime.add_integer(i32::MAX).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.add().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start + 1);
        let (left, right) = runtime.get_pair(new_data_start + 1).unwrap();
        assert_eq!(left, new_data_start);
        assert_eq!(right, 2);
        assert_eq!(runtime.get_integer(left).unwrap(), i32::MIN + 10);
        assert_eq!(runtime.get_data_type(right).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn addition_with_underflow_is_pair() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_integer(-11).unwrap();
        let int2 = runtime.add_integer(i32::MIN).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.add().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start + 1);
        let (left, right) = runtime.get_pair(new_data_start + 1).unwrap();
        assert_eq!(left, new_data_start);
        assert_eq!(right, 2);
        assert_eq!(runtime.get_integer(left).unwrap(), i32::MAX - 10);
        assert_eq!(runtime.get_data_type(right).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn perform_addition_no_refs_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_integer(10).unwrap();
        runtime.add_integer(20).unwrap();

        let result = runtime.add();

        assert!(result.is_err());
    }

    #[test]
    fn perform_addition_with_non_integers() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_symbol("sym1").unwrap();
        runtime.add_symbol("sym2").unwrap();

        runtime.push_register(1).unwrap();
        runtime.push_register(2).unwrap();

        runtime.add().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn subtract_integer_integer() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_integer(10).unwrap();
        let int2 = runtime.add_integer(20).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.subtract().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_integer(new_data_start).unwrap(), -10);
    }
}
