use log::trace;

use crate::{next_two_raw_ref, push_boolean, ExpressionDataType, GarnishLangRuntimeData, GarnishLangRuntimeResult, NestInto};

pub(crate) fn equality_comparison<Data: GarnishLangRuntimeData>(this: &mut Data) -> GarnishLangRuntimeResult<Data::Error> {
    trace!("Instruction - Equality Comparison");

    let (right_addr, left_addr) = next_two_raw_ref(this)?;

    let result = match (this.get_data_type(left_addr).nest_into()?, this.get_data_type(right_addr).nest_into()?) {
        (ExpressionDataType::Integer, ExpressionDataType::Integer) => {
            let left = this.get_integer(left_addr).nest_into()?;
            let right = this.get_integer(right_addr).nest_into()?;

            trace!("Comparing {:?} == {:?}", left, right);

            left == right
        }
        _ => false,
    };

    push_boolean(this, result)
}

#[cfg(test)]
mod tests {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

    #[test]
    fn equality_true() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_integer(10).unwrap();
        runtime.add_integer(10).unwrap();

        runtime.push_register(1).unwrap();
        runtime.push_register(2).unwrap();

        // runtime.add_instruction(Instruction::EqualityComparison, None).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_register(), &vec![3]);
        assert_eq!(runtime.get_data_type(3).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_false() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_integer(10).unwrap();
        runtime.add_integer(20).unwrap();

        runtime.push_register(1).unwrap();
        runtime.push_register(2).unwrap();

        runtime.push_instruction(Instruction::EqualityComparison, None).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_register(), &vec![3]);
        assert_eq!(runtime.get_data_type(3).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn equality_no_references_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_integer(10).unwrap();
        runtime.add_integer(10).unwrap();

        runtime.push_instruction(Instruction::EqualityComparison, None).unwrap();

        let result = runtime.equality_comparison();

        assert!(result.is_err());
    }

    #[test]
    fn equality_of_unsupported_comparison_is_false() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_integer(10).unwrap();
        runtime.add_expression(10).unwrap();

        runtime.push_instruction(Instruction::EqualityComparison, None).unwrap();

        runtime.push_register(1).unwrap();
        runtime.push_register(2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_register(), &vec![3]);
        assert_eq!(runtime.get_data_type(3).unwrap(), ExpressionDataType::False);
    }
}
