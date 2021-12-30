use log::trace;

use crate::runtime::data::Overflowable;
use crate::{next_two_raw_ref, push_integer, push_pair, push_unit, ExpressionDataType, GarnishLangRuntimeData, GarnishLangRuntimeResult, NestInto};

pub fn perform_addition<Data: GarnishLangRuntimeData>(this: &mut Data) -> GarnishLangRuntimeResult<Data::Error> {
    trace!("Instruction - Addition");

    let (right_addr, left_addr) = next_two_raw_ref(this)?;

    match (this.get_data_type(left_addr).nest_into()?, this.get_data_type(right_addr).nest_into()?) {
        (ExpressionDataType::Integer, ExpressionDataType::Integer) => {
            let left = this.get_integer(left_addr).nest_into()?;
            let right = this.get_integer(right_addr).nest_into()?;

            trace!("Performing {:?} + {:?}", left, right);

            let (sum, overflowed) = left.overflowable_addition(right);

            if overflowed {
                let left = this.add_integer(sum).nest_into()?;
                let right = this.add_true().nest_into()?;
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
    use crate::{runtime::GarnishRuntime, ExpressionData, ExpressionDataType, GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn perform_addition() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.push_register(1).unwrap();
        runtime.push_register(2).unwrap();

        runtime.perform_addition().unwrap();

        assert_eq!(runtime.get_register(), &vec![3]);
        assert_eq!(runtime.get_integer(3).unwrap(), 30);
    }

    #[test]
    fn addition_with_overflow_is_pair() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(11)).unwrap();
        runtime.add_data(ExpressionData::integer(i32::MAX)).unwrap();

        runtime.push_register(1).unwrap();
        runtime.push_register(2).unwrap();

        runtime.perform_addition().unwrap();

        assert_eq!(runtime.get_register(), &vec![5]);
        let (left, right) = runtime.get_pair(5).unwrap();
        assert_eq!(left, 3);
        assert_eq!(right, 4);
        assert_eq!(runtime.get_integer(left).unwrap(), i32::MIN + 10);
        assert_eq!(runtime.get_data_type(right).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn perform_addition_no_refs_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        let result = runtime.perform_addition();

        assert!(result.is_err());
    }

    #[test]
    fn perform_addition_with_non_integers() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::symbol(&"sym1".to_string(), 1)).unwrap();
        runtime.add_data(ExpressionData::symbol(&"sym2".to_string(), 2)).unwrap();

        runtime.push_register(1).unwrap();
        runtime.push_register(2).unwrap();

        runtime.perform_addition().unwrap();

        assert_eq!(runtime.get_register(), &vec![3]);
        assert_eq!(runtime.get_data_type(3).unwrap(), ExpressionDataType::Unit);
    }
}
