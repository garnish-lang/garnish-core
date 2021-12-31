#[cfg(test)]
mod tests {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

    #[test]
    fn make_pair() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_symbol("my_symbol").unwrap();
        let start = runtime.get_data_len();

        runtime.push_register(i1).unwrap();
        runtime.push_register(i2).unwrap();

        runtime.push_instruction(Instruction::MakePair, None).unwrap();

        runtime.make_pair().unwrap();

        assert_eq!(runtime.get_data_type(start).unwrap(), ExpressionDataType::Pair);
        assert_eq!(runtime.get_pair(start).unwrap(), (i1, i2));

        assert_eq!(runtime.get_register().len(), 1);
        assert_eq!(*runtime.get_register().get(0).unwrap(), start);
    }

    #[test]
    fn make_pair_no_refs_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_integer(10).unwrap();
        runtime.add_symbol("my_symbol").unwrap();

        runtime.push_instruction(Instruction::MakePair, None).unwrap();

        let result = runtime.make_pair();

        assert!(result.is_err());
    }
}
