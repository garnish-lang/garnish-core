#[cfg(test)]
mod tests {

    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_traits::{ExpressionDataType, GarnishLangRuntimeData, GarnishRuntime};

    #[test]
    fn make_pair() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i2 = runtime.get_data_mut().add_symbol(20).unwrap();
        let start = runtime.get_data_mut().get_data_len();

        runtime.get_data_mut().push_register(i1).unwrap();
        runtime.get_data_mut().push_register(i2).unwrap();

        runtime.concat().unwrap();

        assert_eq!(runtime.get_data_mut().get_data_type(start).unwrap(), ExpressionDataType::Concatenation);
        assert_eq!(runtime.get_data_mut().get_concatenation(start).unwrap(), (i1, i2));

        assert_eq!(runtime.get_data_mut().get_register(0).unwrap(), start);
    }

    #[test]
    fn make_pair_no_refs_is_err() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().add_symbol(20).unwrap();

        let result = runtime.concat();

        assert!(result.is_err());
    }
}
