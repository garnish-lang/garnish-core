#[cfg(test)]
mod general {

    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang_traits::{ExpressionDataType, GarnishLangRuntimeData, GarnishRuntime};

    #[test]
    fn equality_no_references_is_err() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().add_number(10.into()).unwrap();
        let result = runtime.equal();

        assert!(result.is_err());
    }

    #[test]
    fn equality_of_unsupported_comparison_is_false() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let exp1 = runtime.get_data_mut().add_expression(10).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(exp1).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn not_equal_true() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let d2 = runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.not_equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn not_equal_false() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let d2 = runtime.get_data_mut().add_number(10.into()).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.not_equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn type_equal_true() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_number(20.into()).unwrap();
        let d2 = runtime.get_data_mut().add_number(10.into()).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.type_equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn type_equal_true_with_type_on_right() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_number(20.into()).unwrap();
        let d2 = runtime.get_data_mut().add_type(ExpressionDataType::Number).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.type_equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn type_equal_false() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let d2 = runtime.get_data_mut().add_true().unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.type_equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }
}

#[cfg(test)]
mod simple_types {

    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang_traits::{ExpressionDataType, GarnishLangRuntimeData, GarnishRuntime};

    #[test]
    fn equality_units_equal() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_unit().unwrap();
        let int2 = runtime.get_data_mut().add_unit().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_true_equal() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_true().unwrap();
        let int2 = runtime.get_data_mut().add_true().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_false_equal() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_false().unwrap();
        let int2 = runtime.get_data_mut().add_false().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn types_equal() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_type(ExpressionDataType::Number).unwrap();
        let int2 = runtime.get_data_mut().add_type(ExpressionDataType::Number).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn types_not_equal() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_type(ExpressionDataType::Number).unwrap();
        let int2 = runtime.get_data_mut().add_type(ExpressionDataType::Char).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }
}

#[cfg(test)]
mod numbers {

    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang_traits::{ExpressionDataType, GarnishLangRuntimeData, GarnishRuntime};

    #[test]
    fn equality_integers_equal() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let int2 = runtime.get_data_mut().add_number(10.into()).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_integers_not_equal() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let int2 = runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }
}

#[cfg(test)]
mod chars {

    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang_traits::{ExpressionDataType, GarnishLangRuntimeData, GarnishRuntime};

    #[test]
    fn equality_chars_equal() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_char('a').unwrap();
        let int2 = runtime.get_data_mut().add_char('a').unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_chars_not_equal() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_char('a').unwrap();
        let int2 = runtime.get_data_mut().add_char('b').unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn equality_char_lists_equal() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().start_char_list().unwrap();
        runtime.get_data_mut().add_to_char_list('a').unwrap();
        runtime.get_data_mut().add_to_char_list('b').unwrap();
        runtime.get_data_mut().add_to_char_list('c').unwrap();
        let int1 = runtime.get_data_mut().end_char_list().unwrap();

        runtime.get_data_mut().start_char_list().unwrap();
        runtime.get_data_mut().add_to_char_list('a').unwrap();
        runtime.get_data_mut().add_to_char_list('b').unwrap();
        runtime.get_data_mut().add_to_char_list('c').unwrap();
        let int2 = runtime.get_data_mut().end_char_list().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_char_lists_not_equal() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().start_char_list().unwrap();
        runtime.get_data_mut().add_to_char_list('a').unwrap();
        runtime.get_data_mut().add_to_char_list('b').unwrap();
        runtime.get_data_mut().add_to_char_list('c').unwrap();
        let int1 = runtime.get_data_mut().end_char_list().unwrap();

        runtime.get_data_mut().start_char_list().unwrap();
        runtime.get_data_mut().add_to_char_list('a').unwrap();
        runtime.get_data_mut().add_to_char_list('b').unwrap();
        runtime.get_data_mut().add_to_char_list('d').unwrap();
        let int2 = runtime.get_data_mut().end_char_list().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn equality_char_char_list_equal() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_char('a').unwrap();

        runtime.get_data_mut().start_char_list().unwrap();
        runtime.get_data_mut().add_to_char_list('a').unwrap();
        let int2 = runtime.get_data_mut().end_char_list().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_char_char_list_not_equal() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_char('a').unwrap();

        runtime.get_data_mut().start_char_list().unwrap();
        runtime.get_data_mut().add_to_char_list('a').unwrap();
        runtime.get_data_mut().add_to_char_list('b').unwrap();
        let int2 = runtime.get_data_mut().end_char_list().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn equality_char_list_char_equal() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().start_char_list().unwrap();
        runtime.get_data_mut().add_to_char_list('a').unwrap();
        let int1 = runtime.get_data_mut().end_char_list().unwrap();

        let int2 = runtime.get_data_mut().add_char('a').unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_char_list_char_not_equal() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().start_char_list().unwrap();
        runtime.get_data_mut().add_to_char_list('a').unwrap();
        runtime.get_data_mut().add_to_char_list('b').unwrap();
        let int1 = runtime.get_data_mut().end_char_list().unwrap();

        let int2 = runtime.get_data_mut().add_char('a').unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }
}

#[cfg(test)]
mod bytes {

    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang_traits::{ExpressionDataType, GarnishLangRuntimeData, GarnishRuntime};

    #[test]
    fn equality_bytes_equal() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_byte(10).unwrap();
        let int2 = runtime.get_data_mut().add_byte(10).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_bytes_not_equal() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_byte(10).unwrap();
        let int2 = runtime.get_data_mut().add_byte(20).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn equality_byte_lists_equal() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().start_byte_list().unwrap();
        runtime.get_data_mut().add_to_byte_list(10).unwrap();
        runtime.get_data_mut().add_to_byte_list(20).unwrap();
        runtime.get_data_mut().add_to_byte_list(30).unwrap();
        let int1 = runtime.get_data_mut().end_byte_list().unwrap();

        runtime.get_data_mut().start_byte_list().unwrap();
        runtime.get_data_mut().add_to_byte_list(10).unwrap();
        runtime.get_data_mut().add_to_byte_list(20).unwrap();
        runtime.get_data_mut().add_to_byte_list(30).unwrap();
        let int2 = runtime.get_data_mut().end_byte_list().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_byte_lists_not_equal() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().start_byte_list().unwrap();
        runtime.get_data_mut().add_to_byte_list(10).unwrap();
        runtime.get_data_mut().add_to_byte_list(20).unwrap();
        runtime.get_data_mut().add_to_byte_list(30).unwrap();
        let int1 = runtime.get_data_mut().end_byte_list().unwrap();

        runtime.get_data_mut().start_byte_list().unwrap();
        runtime.get_data_mut().add_to_byte_list(10).unwrap();
        runtime.get_data_mut().add_to_byte_list(20).unwrap();
        runtime.get_data_mut().add_to_byte_list(40).unwrap();
        let int2 = runtime.get_data_mut().end_byte_list().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn equality_byte_byte_list_equal() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_byte(10).unwrap();

        runtime.get_data_mut().start_byte_list().unwrap();
        runtime.get_data_mut().add_to_byte_list(10).unwrap();
        let int2 = runtime.get_data_mut().end_byte_list().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_byte_byte_list_not_equal() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_byte(10).unwrap();

        runtime.get_data_mut().start_byte_list().unwrap();
        runtime.get_data_mut().add_to_byte_list(10).unwrap();
        runtime.get_data_mut().add_to_byte_list(20).unwrap();
        let int2 = runtime.get_data_mut().end_byte_list().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn equality_byte_list_byte_equal() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().start_byte_list().unwrap();
        runtime.get_data_mut().add_to_byte_list(10).unwrap();
        let int1 = runtime.get_data_mut().end_byte_list().unwrap();

        let int2 = runtime.get_data_mut().add_byte(10).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_byte_list_byte_not_equal() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().start_byte_list().unwrap();
        runtime.get_data_mut().add_to_byte_list(10).unwrap();
        runtime.get_data_mut().add_to_byte_list(20).unwrap();
        let int1 = runtime.get_data_mut().end_byte_list().unwrap();

        let int2 = runtime.get_data_mut().add_byte(10).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }
}

#[cfg(test)]
mod symbols {

    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang_traits::{ExpressionDataType, GarnishLangRuntimeData, GarnishRuntime, Instruction};

    #[test]
    fn equality_equal() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_symbol(1).unwrap();
        let int2 = runtime.get_data_mut().add_symbol(1).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_not_equal() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_symbol(0).unwrap();
        let int2 = runtime.get_data_mut().add_symbol(1).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::Equal, None).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }
}

#[cfg(test)]
mod expression {

    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang_traits::{ExpressionDataType, GarnishLangRuntimeData, GarnishRuntime, Instruction};

    #[test]
    fn equality_equal() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_expression(10).unwrap();
        let int2 = runtime.get_data_mut().add_expression(10).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_not_equal() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_expression(10).unwrap();
        let int2 = runtime.get_data_mut().add_expression(20).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::Equal, None).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }
}

#[cfg(test)]
mod external {

    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang_traits::{ExpressionDataType, GarnishLangRuntimeData, GarnishRuntime, Instruction};

    #[test]
    fn equality_equal() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_external(10).unwrap();
        let int2 = runtime.get_data_mut().add_external(10).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_not_equal() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_external(10).unwrap();
        let int2 = runtime.get_data_mut().add_external(20).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::Equal, None).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }
}

#[cfg(test)]
mod pairs {

    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang_traits::{ExpressionDataType, GarnishLangRuntimeData, GarnishRuntime, Instruction};

    #[test]
    fn equality_equal() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i3 = runtime.get_data_mut().add_pair((i1, i2)).unwrap();

        let i4 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i5 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i6 = runtime.get_data_mut().add_pair((i4, i5)).unwrap();

        runtime.get_data_mut().push_register(i3).unwrap();
        runtime.get_data_mut().push_register(i6).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_not_equal() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i2 = runtime.get_data_mut().add_number(20.into()).unwrap();
        let i3 = runtime.get_data_mut().add_pair((i1, i2)).unwrap();

        let i4 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i5 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i6 = runtime.get_data_mut().add_pair((i4, i5)).unwrap();

        runtime.get_data_mut().push_register(i3).unwrap();
        runtime.get_data_mut().push_register(i6).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::Equal, None).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }
}

#[cfg(test)]
mod ranges {

    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang_traits::{ExpressionDataType, GarnishLangRuntimeData, GarnishRuntime};

    #[test]
    fn equality_equal() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i3 = runtime.get_data_mut().add_range(i1, i2).unwrap();

        let i4 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i5 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i6 = runtime.get_data_mut().add_range(i4, i5).unwrap();

        runtime.get_data_mut().push_register(i3).unwrap();
        runtime.get_data_mut().push_register(i6).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_open_start_equal() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_unit().unwrap();
        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i3 = runtime.get_data_mut().add_range(i1, i2).unwrap();

        let i4 = runtime.get_data_mut().add_unit().unwrap();
        let i5 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i6 = runtime.get_data_mut().add_range(i4, i5).unwrap();

        runtime.get_data_mut().push_register(i3).unwrap();
        runtime.get_data_mut().push_register(i6).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_open_start_integer_not_equal() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_unit().unwrap();
        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i3 = runtime.get_data_mut().add_range(i1, i2).unwrap();

        let i4 = runtime.get_data_mut().add_number(5.into()).unwrap();
        let i5 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i6 = runtime.get_data_mut().add_range(i4, i5).unwrap();

        runtime.get_data_mut().push_register(i3).unwrap();
        runtime.get_data_mut().push_register(i6).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn equality_integer_open_start_not_equal() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_number(5.into()).unwrap();
        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i3 = runtime.get_data_mut().add_range(i1, i2).unwrap();

        let i4 = runtime.get_data_mut().add_unit().unwrap();
        let i5 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i6 = runtime.get_data_mut().add_range(i4, i5).unwrap();

        runtime.get_data_mut().push_register(i3).unwrap();
        runtime.get_data_mut().push_register(i6).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn equality_open_end_equal() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i2 = runtime.get_data_mut().add_unit().unwrap();
        let i3 = runtime.get_data_mut().add_range(i1, i2).unwrap();

        let i4 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i5 = runtime.get_data_mut().add_unit().unwrap();
        let i6 = runtime.get_data_mut().add_range(i4, i5).unwrap();

        runtime.get_data_mut().push_register(i3).unwrap();
        runtime.get_data_mut().push_register(i6).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_open_end_integer_not_equal() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i2 = runtime.get_data_mut().add_unit().unwrap();
        let i3 = runtime.get_data_mut().add_range(i1, i2).unwrap();

        let i4 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i5 = runtime.get_data_mut().add_number(20.into()).unwrap();
        let i6 = runtime.get_data_mut().add_range(i4, i5).unwrap();

        runtime.get_data_mut().push_register(i3).unwrap();
        runtime.get_data_mut().push_register(i6).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn equality_integer_open_end_not_equal() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i2 = runtime.get_data_mut().add_number(20.into()).unwrap();
        let i3 = runtime.get_data_mut().add_range(i1, i2).unwrap();

        let i4 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i5 = runtime.get_data_mut().add_unit().unwrap();
        let i6 = runtime.get_data_mut().add_range(i4, i5).unwrap();

        runtime.get_data_mut().push_register(i3).unwrap();
        runtime.get_data_mut().push_register(i6).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn equality_start_not_equal() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i3 = runtime.get_data_mut().add_range(i1, i2).unwrap();

        let i4 = runtime.get_data_mut().add_number(5.into()).unwrap();
        let i5 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i6 = runtime.get_data_mut().add_range(i4, i5).unwrap();

        runtime.get_data_mut().push_register(i3).unwrap();
        runtime.get_data_mut().push_register(i6).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn equality_end_not_equal() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i3 = runtime.get_data_mut().add_range(i1, i2).unwrap();

        let i4 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i5 = runtime.get_data_mut().add_number(20.into()).unwrap();
        let i6 = runtime.get_data_mut().add_range(i4, i5).unwrap();

        runtime.get_data_mut().push_register(i3).unwrap();
        runtime.get_data_mut().push_register(i6).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }
}

#[cfg(test)]
mod lists {

    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang_traits::{ExpressionDataType, GarnishLangRuntimeData, GarnishRuntime, Instruction};

    #[test]
    fn equality_only_items_equal() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i3 = runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().start_list(3).unwrap();
        runtime.get_data_mut().add_to_list(i1, false).unwrap();
        runtime.get_data_mut().add_to_list(i2, false).unwrap();
        runtime.get_data_mut().add_to_list(i3, false).unwrap();
        let i4 = runtime.get_data_mut().end_list().unwrap();

        let i5 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i6 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i7 = runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().start_list(3).unwrap();
        runtime.get_data_mut().add_to_list(i5, false).unwrap();
        runtime.get_data_mut().add_to_list(i6, false).unwrap();
        runtime.get_data_mut().add_to_list(i7, false).unwrap();
        let i8 = runtime.get_data_mut().end_list().unwrap();

        runtime.get_data_mut().push_register(i4).unwrap();
        runtime.get_data_mut().push_register(i8).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_only_items_not_equal() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i3 = runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().start_list(3).unwrap();
        runtime.get_data_mut().add_to_list(i1, false).unwrap();
        runtime.get_data_mut().add_to_list(i2, false).unwrap();
        runtime.get_data_mut().add_to_list(i3, false).unwrap();
        let i4 = runtime.get_data_mut().end_list().unwrap();

        let i5 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i6 = runtime.get_data_mut().add_number(20.into()).unwrap();
        let i7 = runtime.get_data_mut().add_number(30.into()).unwrap();
        runtime.get_data_mut().start_list(3).unwrap();
        runtime.get_data_mut().add_to_list(i5, false).unwrap();
        runtime.get_data_mut().add_to_list(i6, false).unwrap();
        runtime.get_data_mut().add_to_list(i7, false).unwrap();
        let i8 = runtime.get_data_mut().end_list().unwrap();

        runtime.get_data_mut().push_register(i4).unwrap();
        runtime.get_data_mut().push_register(i8).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::Equal, None).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn equality_associations_equal() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_symbol(1).unwrap();
        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i3 = runtime.get_data_mut().add_pair((i1, i2)).unwrap();

        let i4 = runtime.get_data_mut().add_symbol(2).unwrap();
        let i5 = runtime.get_data_mut().add_number(20.into()).unwrap();
        let i6 = runtime.get_data_mut().add_pair((i4, i5)).unwrap();

        let i7 = runtime.get_data_mut().add_symbol(3).unwrap();
        let i8 = runtime.get_data_mut().add_number(30.into()).unwrap();
        let i9 = runtime.get_data_mut().add_pair((i7, i8)).unwrap();

        runtime.get_data_mut().start_list(3).unwrap();
        runtime.get_data_mut().add_to_list(i3, true).unwrap();
        runtime.get_data_mut().add_to_list(i6, true).unwrap();
        runtime.get_data_mut().add_to_list(i9, true).unwrap();
        let i10 = runtime.get_data_mut().end_list().unwrap();

        let i11 = runtime.get_data_mut().add_symbol(3).unwrap();
        let i12 = runtime.get_data_mut().add_number(30.into()).unwrap();
        let i13 = runtime.get_data_mut().add_pair((i11, i12)).unwrap();

        let i14 = runtime.get_data_mut().add_symbol(1).unwrap();
        let i15 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i16 = runtime.get_data_mut().add_pair((i14, i15)).unwrap();

        let i17 = runtime.get_data_mut().add_symbol(2).unwrap();
        let i18 = runtime.get_data_mut().add_number(20.into()).unwrap();
        let i19 = runtime.get_data_mut().add_pair((i17, i18)).unwrap();

        runtime.get_data_mut().start_list(3).unwrap();
        runtime.get_data_mut().add_to_list(i13, true).unwrap();
        runtime.get_data_mut().add_to_list(i16, true).unwrap();
        runtime.get_data_mut().add_to_list(i19, true).unwrap();
        let i20 = runtime.get_data_mut().end_list().unwrap();

        runtime.get_data_mut().push_register(i10).unwrap();
        runtime.get_data_mut().push_register(i20).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_associations_not_equal() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_symbol(1).unwrap();
        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i3 = runtime.get_data_mut().add_pair((i1, i2)).unwrap();

        let i4 = runtime.get_data_mut().add_symbol(2).unwrap();
        let i5 = runtime.get_data_mut().add_number(20.into()).unwrap();
        let i6 = runtime.get_data_mut().add_pair((i4, i5)).unwrap();

        let i7 = runtime.get_data_mut().add_symbol(3).unwrap();
        let i8 = runtime.get_data_mut().add_number(30.into()).unwrap();
        let i9 = runtime.get_data_mut().add_pair((i7, i8)).unwrap();

        runtime.get_data_mut().start_list(3).unwrap();
        runtime.get_data_mut().add_to_list(i3, true).unwrap();
        runtime.get_data_mut().add_to_list(i6, true).unwrap();
        runtime.get_data_mut().add_to_list(i9, true).unwrap();
        let i10 = runtime.get_data_mut().end_list().unwrap();

        let i11 = runtime.get_data_mut().add_symbol(3).unwrap();
        let i12 = runtime.get_data_mut().add_number(30.into()).unwrap();
        let i13 = runtime.get_data_mut().add_pair((i11, i12)).unwrap();

        let i14 = runtime.get_data_mut().add_symbol(1).unwrap();
        let i15 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i16 = runtime.get_data_mut().add_pair((i14, i15)).unwrap();

        let i17 = runtime.get_data_mut().add_symbol(2).unwrap();
        let i18 = runtime.get_data_mut().add_number(100.into()).unwrap();
        let i19 = runtime.get_data_mut().add_pair((i17, i18)).unwrap();

        runtime.get_data_mut().start_list(3).unwrap();
        runtime.get_data_mut().add_to_list(i13, true).unwrap();
        runtime.get_data_mut().add_to_list(i16, true).unwrap();
        runtime.get_data_mut().add_to_list(i19, true).unwrap();
        let i20 = runtime.get_data_mut().end_list().unwrap();

        runtime.get_data_mut().push_register(i10).unwrap();
        runtime.get_data_mut().push_register(i20).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn equality_mixed_values_equal() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_symbol(1).unwrap();
        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i3 = runtime.get_data_mut().add_pair((i1, i2)).unwrap();

        let i4 = runtime.get_data_mut().add_number(40.into()).unwrap();

        let i5 = runtime.get_data_mut().add_symbol(3).unwrap();
        let i6 = runtime.get_data_mut().add_number(30.into()).unwrap();
        let i7 = runtime.get_data_mut().add_pair((i5, i6)).unwrap();

        runtime.get_data_mut().start_list(3).unwrap();
        runtime.get_data_mut().add_to_list(i3, true).unwrap();
        runtime.get_data_mut().add_to_list(i4, false).unwrap();
        runtime.get_data_mut().add_to_list(i7, true).unwrap();
        let i8 = runtime.get_data_mut().end_list().unwrap();

        let i9 = runtime.get_data_mut().add_symbol(1).unwrap();
        let i10 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i11 = runtime.get_data_mut().add_pair((i9, i10)).unwrap();

        let i12 = runtime.get_data_mut().add_number(40.into()).unwrap();

        let i13 = runtime.get_data_mut().add_symbol(3).unwrap();
        let i14 = runtime.get_data_mut().add_number(30.into()).unwrap();
        let i15 = runtime.get_data_mut().add_pair((i13, i14)).unwrap();

        runtime.get_data_mut().start_list(3).unwrap();
        runtime.get_data_mut().add_to_list(i11, true).unwrap();
        runtime.get_data_mut().add_to_list(i12, false).unwrap();
        runtime.get_data_mut().add_to_list(i15, true).unwrap();
        let i16 = runtime.get_data_mut().end_list().unwrap();

        runtime.get_data_mut().push_register(i8).unwrap();
        runtime.get_data_mut().push_register(i16).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_mixed_values_not_equal() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_symbol(1).unwrap();
        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i3 = runtime.get_data_mut().add_pair((i1, i2)).unwrap();

        let i4 = runtime.get_data_mut().add_number(40.into()).unwrap();

        let i5 = runtime.get_data_mut().add_symbol(3).unwrap();
        let i6 = runtime.get_data_mut().add_number(30.into()).unwrap();
        let i7 = runtime.get_data_mut().add_pair((i5, i6)).unwrap();

        runtime.get_data_mut().start_list(3).unwrap();
        runtime.get_data_mut().add_to_list(i3, true).unwrap();
        runtime.get_data_mut().add_to_list(i4, false).unwrap();
        runtime.get_data_mut().add_to_list(i7, true).unwrap();
        let i8 = runtime.get_data_mut().end_list().unwrap();

        let i9 = runtime.get_data_mut().add_symbol(1).unwrap();
        let i10 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i11 = runtime.get_data_mut().add_pair((i9, i10)).unwrap();

        let i12 = runtime.get_data_mut().add_number(20.into()).unwrap();

        let i13 = runtime.get_data_mut().add_symbol(3).unwrap();
        let i14 = runtime.get_data_mut().add_number(30.into()).unwrap();
        let i15 = runtime.get_data_mut().add_pair((i13, i14)).unwrap();

        runtime.get_data_mut().start_list(3).unwrap();
        runtime.get_data_mut().add_to_list(i11, true).unwrap();
        runtime.get_data_mut().add_to_list(i12, false).unwrap();
        runtime.get_data_mut().add_to_list(i15, true).unwrap();
        let i16 = runtime.get_data_mut().end_list().unwrap();

        runtime.get_data_mut().push_register(i8).unwrap();
        runtime.get_data_mut().push_register(i16).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }
}

#[cfg(test)]
mod concatenation {

    use crate::simple::testing_utilities::{add_concatenation_with_start, add_list_with_start, create_simple_runtime};
    use garnish_lang_traits::{ExpressionDataType, GarnishLangRuntimeData, GarnishRuntime};

    #[test]
    fn concatenation_concatenation_equal() {
        let mut runtime = create_simple_runtime();

        let d1 = add_concatenation_with_start(runtime.get_data_mut(), 10, 10);
        let d2 = add_concatenation_with_start(runtime.get_data_mut(), 10, 10);

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn concatenation_concatenation_not_equal() {
        let mut runtime = create_simple_runtime();

        let d1 = add_concatenation_with_start(runtime.get_data_mut(), 10, 10);
        let d2 = add_concatenation_with_start(runtime.get_data_mut(), 10, 20);

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn concatenation_of_list_concatenation_equal() {
        let mut runtime = create_simple_runtime();

        let d1 = add_list_with_start(runtime.get_data_mut(), 5, 10);
        let d2 = add_list_with_start(runtime.get_data_mut(), 5, 15);
        let d3 = runtime.get_data_mut().add_concatenation(d1, d2).unwrap();
        let d4 = add_concatenation_with_start(runtime.get_data_mut(), 10, 10);

        runtime.get_data_mut().push_register(d3).unwrap();
        runtime.get_data_mut().push_register(d4).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn concatenation_of_list_concatenation_not_equal() {
        let mut runtime = create_simple_runtime();

        let d1 = add_list_with_start(runtime.get_data_mut(), 5, 10);
        let d2 = add_list_with_start(runtime.get_data_mut(), 5, 15);
        let d3 = runtime.get_data_mut().add_concatenation(d1, d2).unwrap();
        let d4 = add_concatenation_with_start(runtime.get_data_mut(), 11, 10);

        runtime.get_data_mut().push_register(d3).unwrap();
        runtime.get_data_mut().push_register(d4).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn concatenation_list_equal() {
        let mut runtime = create_simple_runtime();

        let d1 = add_concatenation_with_start(runtime.get_data_mut(), 10, 10);
        let d2 = add_list_with_start(runtime.get_data_mut(), 10, 10);

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn concatenation_list_not_equal() {
        let mut runtime = create_simple_runtime();

        let d1 = add_concatenation_with_start(runtime.get_data_mut(), 10, 10);
        let d2 = add_list_with_start(runtime.get_data_mut(), 11, 10);

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn list_concatenation_equal() {
        let mut runtime = create_simple_runtime();

        let d1 = add_list_with_start(runtime.get_data_mut(), 10, 10);
        let d2 = add_concatenation_with_start(runtime.get_data_mut(), 10, 10);

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn lsit_concatenation_not_equal() {
        let mut runtime = create_simple_runtime();

        let d1 = add_list_with_start(runtime.get_data_mut(), 10, 10);
        let d2 = add_concatenation_with_start(runtime.get_data_mut(), 11, 10);

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }
}

#[cfg(test)]
mod slices {

    use crate::simple::testing_utilities::{
        add_byte_list, add_char_list, add_list_with_start, add_range, create_simple_runtime,
    };
    use garnish_lang_traits::{ExpressionDataType, GarnishLangRuntimeData, GarnishRuntime};

    #[test]
    fn slice_of_list_slice_of_list() {
        let mut runtime = create_simple_runtime();

        let d1 = add_list_with_start(runtime.get_data_mut(), 10, 15);
        let d2 = add_range(runtime.get_data_mut(), 0, 4);
        let d3 = runtime.get_data_mut().add_slice(d1, d2).unwrap();

        let d4 = add_list_with_start(runtime.get_data_mut(), 10, 10);
        let d5 = add_range(runtime.get_data_mut(), 5, 9);
        let d6 = runtime.get_data_mut().add_slice(d4, d5).unwrap();

        runtime.get_data_mut().push_register(d3).unwrap();
        runtime.get_data_mut().push_register(d6).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn slice_of_char_list_slice_of_char_list() {
        let mut runtime = create_simple_runtime();

        let list1 = add_char_list(runtime.get_data_mut(), "abcde");
        let range1 = add_range(runtime.get_data_mut(), 1, 3);
        let slice1 = runtime.get_data_mut().add_slice(list1, range1).unwrap();

        let list2 = add_char_list(runtime.get_data_mut(), "abcde");
        let range2 = add_range(runtime.get_data_mut(), 1, 3);
        let slice2 = runtime.get_data_mut().add_slice(list2, range2).unwrap();

        runtime.get_data_mut().push_register(slice1).unwrap();
        runtime.get_data_mut().push_register(slice2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn slice_of_byte_list_slice_of_byte_list() {
        let mut runtime = create_simple_runtime();

        let list1 = add_byte_list(runtime.get_data_mut(), "abcde");
        let range1 = add_range(runtime.get_data_mut(), 1, 3);
        let slice1 = runtime.get_data_mut().add_slice(list1, range1).unwrap();

        let list2 = add_byte_list(runtime.get_data_mut(), "abcde");
        let range2 = add_range(runtime.get_data_mut(), 1, 3);
        let slice2 = runtime.get_data_mut().add_slice(list2, range2).unwrap();

        runtime.get_data_mut().push_register(slice1).unwrap();
        runtime.get_data_mut().push_register(slice2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn slice_of_list_slice_of_incomplete_list_equal() {
        let mut runtime = create_simple_runtime();

        let unit = runtime.get_data_mut().add_unit().unwrap();
        let d1 = runtime.get_data_mut().add_number(10.into()).unwrap();

        runtime.get_data_mut().start_list(3).unwrap();
        runtime.get_data_mut().add_to_list(d1, false).unwrap();
        runtime.get_data_mut().add_to_list(unit, false).unwrap();
        runtime.get_data_mut().add_to_list(unit, false).unwrap();
        let list1 = runtime.get_data_mut().end_list().unwrap();

        runtime.get_data_mut().start_list(1).unwrap();
        runtime.get_data_mut().add_to_list(d1, false).unwrap();
        let list2 = runtime.get_data_mut().end_list().unwrap();

        let range = add_range(runtime.get_data_mut(), 0, 2);
        let slice1 = runtime.get_data_mut().add_slice(list1, range).unwrap();
        let slice2 = runtime.get_data_mut().add_slice(list2, range).unwrap();

        runtime.get_data_mut().push_register(slice1).unwrap();
        runtime.get_data_mut().push_register(slice2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn slice_of_list_slice_of_incomplete_list_not_equal() {
        let mut runtime = create_simple_runtime();

        let unit = runtime.get_data_mut().add_unit().unwrap();
        let d1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let d2 = runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.get_data_mut().start_list(3).unwrap();
        runtime.get_data_mut().add_to_list(d1, false).unwrap();
        runtime.get_data_mut().add_to_list(d2, false).unwrap();
        runtime.get_data_mut().add_to_list(unit, false).unwrap();
        let list1 = runtime.get_data_mut().end_list().unwrap();

        runtime.get_data_mut().start_list(1).unwrap();
        runtime.get_data_mut().add_to_list(d1, false).unwrap();
        let list2 = runtime.get_data_mut().end_list().unwrap();

        let range = add_range(runtime.get_data_mut(), 0, 2);
        let slice1 = runtime.get_data_mut().add_slice(list1, range).unwrap();
        let slice2 = runtime.get_data_mut().add_slice(list2, range).unwrap();

        runtime.get_data_mut().push_register(slice1).unwrap();
        runtime.get_data_mut().push_register(slice2).unwrap();

        runtime.equal().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::False);
    }
}
