#[cfg(test)]
mod tests {
    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang::simple::{SimpleDataFactory, SimpleNumber};
    use garnish_lang::{GarnishData, GarnishDataFactory, GarnishDataType, GarnishRuntime};

    #[test]
    fn add() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let int2 = runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.add().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 30.into());
    }

    #[test]
    fn add_no_refs_is_err() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().add_number(20.into()).unwrap();

        let result = runtime.add();

        assert!(result.is_err());
    }

    #[test]
    fn add_with_non_numbers() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().add_symbol(SimpleDataFactory::parse_symbol("sym1").unwrap()).unwrap();
        runtime.get_data_mut().add_symbol(SimpleDataFactory::parse_symbol("sym2").unwrap()).unwrap();

        runtime.get_data_mut().push_register(1).unwrap();
        runtime.get_data_mut().push_register(2).unwrap();

        runtime.add().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::Unit);
    }

    #[test]
    fn subtract() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let int2 = runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.subtract().unwrap();

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

        runtime.multiply().unwrap();

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

        runtime.divide().unwrap();

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

        runtime.integer_divide().unwrap();

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

        runtime.power().unwrap();

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

        runtime.remainder().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 3.into());
    }

    #[test]
    fn absolute_value() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(SimpleNumber::Integer(-10)).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();

        runtime.absolute_value().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 10.into());
    }

    #[test]
    fn opposite() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();

        runtime.opposite().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), SimpleNumber::Integer(-10));
    }
}
