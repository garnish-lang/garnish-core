use log::trace;

use crate::runtime::data::Overflowable;
use crate::{next_two_raw_ref, push_integer, push_pair, push_unit, ExpressionDataType, GarnishLangRuntimeData, RuntimeError};

pub fn perform_addition<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    trace!("Instruction - Addition");

    let (right_addr, left_addr) = next_two_raw_ref(this)?;

    match (this.get_data_type(left_addr)?, this.get_data_type(right_addr)?) {
        (ExpressionDataType::Integer, ExpressionDataType::Integer) => {
            let left = this.get_integer(left_addr)?;
            let right = this.get_integer(right_addr)?;

            trace!("Performing {:?} + {:?}", left, right);

            let (sum, overflowed) = left.overflowable_addition(right);

            if overflowed {
                let left = this.add_integer(sum)?;
                let right = this.add_true()?;
                push_pair(this, left, right)
            } else {
                push_integer(this, sum)
            }
        }
        _ => push_unit(this),
    }
}

#[cfg(test)]
mod tests {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn perform_addition() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_integer(10).unwrap();
        let int2 = runtime.add_integer(20).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.perform_addition().unwrap();

        assert_eq!(runtime.get_registers(), &vec![new_data_start]);
        assert_eq!(runtime.get_integer(new_data_start).unwrap(), 30);
    }

    #[test]
    fn addition_with_overflow_is_pair() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_integer(11).unwrap();
        let int2 = runtime.add_integer(i32::MAX).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.perform_addition().unwrap();

        assert_eq!(runtime.get_registers(), &vec![new_data_start + 1]);
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

        runtime.perform_addition().unwrap();

        assert_eq!(runtime.get_registers(), &vec![new_data_start + 1]);
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

        let result = runtime.perform_addition();

        assert!(result.is_err());
    }

    #[test]
    fn perform_addition_with_non_integers() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_symbol("sym1").unwrap();
        runtime.add_symbol("sym2").unwrap();

        runtime.push_register(1).unwrap();
        runtime.push_register(2).unwrap();

        runtime.perform_addition().unwrap();

        assert_eq!(runtime.get_registers(), &vec![0]);
    }
}
