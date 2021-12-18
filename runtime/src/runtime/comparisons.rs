#[cfg(test)]
mod tests {
    use crate::{
        runtime::{data::GarnishLangRuntimeData, GarnishRuntime},
        ExpressionData, ExpressionDataType, Instruction, SimpleRuntimeData,
    };

    #[test]
    fn equality_true() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();

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

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

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

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();

        runtime.push_instruction(Instruction::EqualityComparison, None).unwrap();

        let result = runtime.equality_comparison();

        assert!(result.is_err());
    }
}
