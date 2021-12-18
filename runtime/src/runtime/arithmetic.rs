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
