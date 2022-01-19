use log::trace;

use crate::{next_two_raw_ref, push_number, push_unit, ExpressionDataType, GarnishLangRuntimeData, GarnishNumber, RuntimeError, next_ref};

pub fn add<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, "addition", Data::Number::add)
}

pub fn subtract<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, "subtraction", Data::Number::subtract)
}

pub fn multiply<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, "multiplication", Data::Number::multiply)
}

pub fn power<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, "power of", Data::Number::power)
}

pub fn divide<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, "division", Data::Number::divide)
}

pub fn integer_divide<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, "integer division", Data::Number::integer_divide)
}

pub fn remainder<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, "remainder", Data::Number::remainder)
}

pub fn absolute_value<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    perform_unary_op(this, "absolute value", Data::Number::absolute_value)
}

pub fn opposite<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    perform_unary_op(this, "opposite", Data::Number::opposite)
}

fn perform_unary_op<Data: GarnishLangRuntimeData, Op>(this: &mut Data, op_name: &str, op: Op) -> Result<(), RuntimeError<Data::Error>>
    where
        Op: FnOnce(Data::Number) -> Option<Data::Number>,
{
    let addr = next_ref(this)?;

    let t = this.get_data_type(addr)?;
    trace!(
        "Attempting {:?} on {:?} at {:?}",
        op_name,
        t,
        addr,
    );

    match t {
        ExpressionDataType::Number => {
            let value = this.get_number(addr)?;

            match op(value) {
                Some(result) => push_number(this, result),
                None => push_unit(this),
            }
        }
        _ => {
            trace!("Unsupported types pushing unit");
            push_unit(this)
        }
    }
}

fn perform_op<Data: GarnishLangRuntimeData, Op>(this: &mut Data, op_name: &str, op: Op) -> Result<(), RuntimeError<Data::Error>>
where
    Op: FnOnce(Data::Number, Data::Number) -> Option<Data::Number>,
{
    let (right_addr, left_addr) = next_two_raw_ref(this)?;

    let types = (this.get_data_type(left_addr)?, this.get_data_type(right_addr)?);
    trace!(
        "Attempting {:?} between {:?} at {:?} and {:?} at {:?}",
        op_name,
        types.0,
        left_addr,
        types.1,
        right_addr
    );

    match types {
        (ExpressionDataType::Number, ExpressionDataType::Number) => {
            let left = this.get_number(left_addr)?;
            let right = this.get_number(right_addr)?;

            match op(left, right) {
                Some(result) => push_number(this, result),
                None => push_unit(this),
            }
        }
        _ => {
            trace!("Unsupported types pushing unit");
            push_unit(this)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn perform_addition() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10).unwrap();
        let int2 = runtime.add_number(20).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.add().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), 30);
    }

    #[test]
    fn perform_addition_no_refs_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_number(10).unwrap();
        runtime.add_number(20).unwrap();

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
    fn subtract() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10).unwrap();
        let int2 = runtime.add_number(20).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.subtract().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), -10);
    }

    #[test]
    fn multiply() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10).unwrap();
        let int2 = runtime.add_number(20).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.multiply().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), 200);
    }

    #[test]
    fn divide() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(20).unwrap();
        let int2 = runtime.add_number(10).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.divide().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), 2);
    }

    #[test]
    fn integer_divide() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(20).unwrap();
        let int2 = runtime.add_number(10).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.integer_divide().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), 2);
    }

    #[test]
    fn power() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10).unwrap();
        let int2 = runtime.add_number(3).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.power().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), 1000);
    }

    #[test]
    fn remainder() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(23).unwrap();
        let int2 = runtime.add_number(20).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.remainder().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), 3);
    }

    #[test]
    fn absolute_value() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(-10).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();

        runtime.absolute_value().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), 10);
    }

    #[test]
    fn opposite() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();

        runtime.opposite().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), -10);
    }
}
