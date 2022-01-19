use log::trace;

use crate::{next_two_raw_ref, push_integer, push_unit, ExpressionDataType, GarnishLangRuntimeData, GarnishNumber, RuntimeError};

pub fn add<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    let (right_addr, left_addr) = next_two_raw_ref(this)?;

    let types = (this.get_data_type(left_addr)?, this.get_data_type(right_addr)?);
    trace!(
        "Attempting addition between {:?} at {:?} and {:?} at {:?}",
        types.0,
        left_addr,
        types.1,
        right_addr
    );

    match types {
        (ExpressionDataType::Number, ExpressionDataType::Number) => {
            let left = this.get_integer(left_addr)?;
            let right = this.get_integer(right_addr)?;

            match left.add(right) {
                Some(result) => push_integer(this, result),
                None => push_unit(this),
            }
        }
        _ => {
            trace!("Unsupported types pushing unit");
            push_unit(this)
        }
    }
}

pub fn subtract<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    let (right_addr, left_addr) = next_two_raw_ref(this)?;

    let types = (this.get_data_type(left_addr)?, this.get_data_type(right_addr)?);
    trace!(
        "Attempting addition between {:?} at {:?} and {:?} at {:?}",
        types.0,
        left_addr,
        types.1,
        right_addr
    );

    match types {
        (ExpressionDataType::Number, ExpressionDataType::Number) => {
            let left = this.get_integer(left_addr)?;
            let right = this.get_integer(right_addr)?;

            match left.subtract(right) {
                Some(result) => push_integer(this, result),
                None => push_unit(this),
            }
        }
        _ => {
            trace!("Unsupported types pushing unit");
            push_unit(this)
        }
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
