#[cfg(test)]
mod deferring {
    use crate::simple::testing_utilities::{deferred_op, deferred_unary_op};
    use garnish_traits::GarnishRuntime;

    #[test]
    fn add() {
        deferred_op(|data, context| {
            data.add(Some(context)).unwrap();
        })
    }

    #[test]
    fn subtract() {
        deferred_op(|data, context| {
            data.subtract(Some(context)).unwrap();
        })
    }

    #[test]
    fn multiply() {
        deferred_op(|data, context| {
            data.multiply(Some(context)).unwrap();
        });
    }

    #[test]
    fn divide() {
        deferred_op(|data, context| {
            data.divide(Some(context)).unwrap();
        });
    }

    #[test]
    fn integer_divide() {
        deferred_op(|data, context| {
            data.integer_divide(Some(context)).unwrap();
        });
    }

    #[test]
    fn remainder() {
        deferred_op(|data, context| {
            data.remainder(Some(context)).unwrap();
        });
    }

    #[test]
    fn power() {
        deferred_op(|data, context| {
            data.power(Some(context)).unwrap();
        });
    }

    #[test]
    fn absolute_value() {
        deferred_unary_op(|data, context| {
            data.absolute_value(Some(context)).unwrap();
        });
    }

    #[test]
    fn opposite() {
        deferred_unary_op(|data, context| {
            data.opposite(Some(context)).unwrap();
        });
    }
}

#[cfg(test)]
mod tests {
    use garnish_data::data::SimpleNumber;
    use garnish_data::SimpleDataRuntimeNC;
    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang_runtime::{EmptyContext, ExpressionDataType, GarnishLangRuntimeData, GarnishRuntime};

    #[test]
    fn add() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let int2 = runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.add::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 30.into());
    }

    #[test]
    fn add_no_refs_is_err() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().add_number(20.into()).unwrap();

        let result = runtime.add::<EmptyContext>(None);

        assert!(result.is_err());
    }

    #[test]
    fn add_with_non_numbers() {
        let mut runtime = create_simple_runtime();

        runtime
            .get_data_mut()
            .add_symbol(SimpleDataRuntimeNC::parse_symbol("sym1").unwrap())
            .unwrap();
        runtime
            .get_data_mut()
            .add_symbol(SimpleDataRuntimeNC::parse_symbol("sym2").unwrap())
            .unwrap();

        runtime.get_data_mut().push_register(1).unwrap();
        runtime.get_data_mut().push_register(2).unwrap();

        runtime.add::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn subtract() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let int2 = runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.subtract::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), SimpleNumber::Integer(-10));
    }

    #[test]
    fn multiply() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let int2 = runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.multiply::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 200.into());
    }

    #[test]
    fn divide() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(20.into()).unwrap();
        let int2 = runtime.get_data_mut().add_number(10.into()).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.divide::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 2.into());
    }

    #[test]
    fn integer_divide() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(20.into()).unwrap();
        let int2 = runtime.get_data_mut().add_number(10.into()).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.integer_divide::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 2.into());
    }

    #[test]
    fn power() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let int2 = runtime.get_data_mut().add_number(3.into()).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.power::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 1000.into());
    }

    #[test]
    fn remainder() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(23.into()).unwrap();
        let int2 = runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.remainder::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 3.into());
    }

    #[test]
    fn absolute_value() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(SimpleNumber::Integer(-10)).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();

        runtime.absolute_value::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 10.into());
    }

    #[test]
    fn opposite() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();

        runtime.opposite::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), SimpleNumber::Integer(-10));
    }
}
