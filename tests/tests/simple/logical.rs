#[cfg(test)]
mod and {
    use garnish_lang_traits::{ExpressionDataType, GarnishLangRuntimeData, GarnishRuntime};

    use crate::simple::testing_utilities::create_simple_runtime;

    #[test]
    fn and_true_booleans() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_true().unwrap();
        let int2 = runtime.get_data_mut().add_true().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.and().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn and_false_on_left() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_false().unwrap();
        let int2 = runtime.get_data_mut().add_true().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.and().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn and_false_on_right() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_true().unwrap();
        let int2 = runtime.get_data_mut().add_false().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.and().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn and_false_booleans() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_false().unwrap();
        let int2 = runtime.get_data_mut().add_false().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.and().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn and_false_unit() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_unit().unwrap();
        let int2 = runtime.get_data_mut().add_unit().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.and().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }
}

#[cfg(test)]
mod or {
    use garnish_lang_traits::{ExpressionDataType, GarnishLangRuntimeData, GarnishRuntime};

    use crate::simple::testing_utilities::create_simple_runtime;

    #[test]
    fn or_true_booleans() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_true().unwrap();
        let int2 = runtime.get_data_mut().add_true().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.or().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn or_false_on_left() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_false().unwrap();
        let int2 = runtime.get_data_mut().add_true().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.or().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn or_false_on_right() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_true().unwrap();
        let int2 = runtime.get_data_mut().add_false().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.or().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn or_false_booleans() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_false().unwrap();
        let int2 = runtime.get_data_mut().add_false().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.or().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn or_false_unit() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_unit().unwrap();
        let int2 = runtime.get_data_mut().add_unit().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.or().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }
}

#[cfg(test)]
mod xor {
    use garnish_lang_traits::{ExpressionDataType, GarnishLangRuntimeData, GarnishRuntime};

    use crate::simple::testing_utilities::create_simple_runtime;

    #[test]
    fn xor_true_booleans() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_true().unwrap();
        let int2 = runtime.get_data_mut().add_true().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.xor().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn xor_false_on_left() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_false().unwrap();
        let int2 = runtime.get_data_mut().add_true().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.xor().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn xor_false_on_right() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_true().unwrap();
        let int2 = runtime.get_data_mut().add_false().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.xor().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn xor_false_booleans() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_false().unwrap();
        let int2 = runtime.get_data_mut().add_false().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.xor().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn xor_false_unit() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_unit().unwrap();
        let int2 = runtime.get_data_mut().add_unit().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.xor().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }
}

#[cfg(test)]
mod not {
    use garnish_lang_traits::{ExpressionDataType, GarnishLangRuntimeData, GarnishRuntime};

    use crate::simple::testing_utilities::create_simple_runtime;

    #[test]
    fn not_true_is_false() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_true().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();

        runtime.not().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn not_any_is_false() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();

        runtime.not().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn not_false_is_true() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_false().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();

        runtime.not().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn not_unit_is_true() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_unit().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();

        runtime.not().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }
}

#[cfg(test)]
mod tis {
    use garnish_lang_traits::{ExpressionDataType, GarnishLangRuntimeData, GarnishRuntime};

    use crate::simple::testing_utilities::create_simple_runtime;

    #[test]
    fn tis_false_is_false() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_false().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();

        runtime.tis().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn tis_any_is_true() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();

        runtime.tis().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn tis_true_is_true() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_true().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();

        runtime.tis().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn tis_unit_is_false() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_unit().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();

        runtime.tis().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }
}
