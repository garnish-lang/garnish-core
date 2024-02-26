#[cfg(test)]
mod deferring {
    use crate::simple::testing_utilities::deferred_op;
    use garnish_lang_traits::GarnishRuntime;

    #[test]
    fn type_cast() {
        deferred_op(|runtime, context| {
            runtime.type_cast(Some(context)).unwrap();
        })
    }
}

#[cfg(test)]
mod type_of {

    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang_traits::{GarnishDataType, GarnishData, GarnishRuntime};

    #[test]
    fn type_of_number() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_number(10.into()).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();

        runtime.type_of().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_type(i).unwrap(), GarnishDataType::Number);
    }
}

#[cfg(test)]
mod simple {

    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang_traits::{GarnishDataType, GarnishData, GarnishRuntime, NO_CONTEXT};

    #[test]
    fn no_op_cast_expression() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_expression(10).unwrap();
        let d2 = runtime.get_data_mut().add_expression(10).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_expression(i).unwrap(), 10);
    }

    #[test]
    fn cast_to_unit() {
        let mut runtime = create_simple_runtime();

        let int = runtime.get_data_mut().add_number(10.into()).unwrap();
        let unit = runtime.get_data_mut().add_unit().unwrap();

        runtime.get_data_mut().push_register(int).unwrap();
        runtime.get_data_mut().push_register(unit).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::Unit);
    }

    #[test]
    fn cast_to_true() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let d2 = runtime.get_data_mut().add_true().unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::True);
    }

    #[test]
    fn cast_to_true_with_type() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let d2 = runtime.get_data_mut().add_type(GarnishDataType::True).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::True);
    }

    #[test]
    fn cast_unit_to_true() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_unit().unwrap();
        let d2 = runtime.get_data_mut().add_true().unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::False);
    }

    #[test]
    fn cast_false_to_true() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_false().unwrap();
        let d2 = runtime.get_data_mut().add_true().unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::False);
    }

    #[test]
    fn cast_to_false() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let d2 = runtime.get_data_mut().add_false().unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::False);
    }

    #[test]
    fn cast_unit_to_false() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_unit().unwrap();
        let d2 = runtime.get_data_mut().add_false().unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::True);
    }

    #[test]
    fn cast_true_to_false() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_true().unwrap();
        let d2 = runtime.get_data_mut().add_false().unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::False);
    }
}

#[cfg(test)]
mod primitive {
    use crate::simple::testing_utilities::{add_char_list, create_simple_runtime};
    use garnish_lang_simple_data::SimpleDataRuntimeNC;
    use garnish_lang_traits::{GarnishData, GarnishRuntime, NO_CONTEXT};

    #[test]
    fn integer_to_char() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_number(('a' as i32).into()).unwrap();
        let d2 = runtime.get_data_mut().add_char('\0').unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let expected = SimpleDataRuntimeNC::number_to_char(('a' as i32).into()).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_char(i).unwrap(), expected);
    }

    #[test]
    fn integer_to_byte() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let d2 = runtime.get_data_mut().add_byte(0).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let expected = SimpleDataRuntimeNC::number_to_byte(10.into()).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_byte(i).unwrap(), expected);
    }

    #[test]
    fn char_to_integer() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_char('a').unwrap();
        let d2 = runtime.get_data_mut().add_number(0.into()).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let expected = SimpleDataRuntimeNC::char_to_number('a').unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), expected);
    }

    #[test]
    fn char_to_byte() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_char('a').unwrap();
        let d2 = runtime.get_data_mut().add_byte(0).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let expected = SimpleDataRuntimeNC::char_to_byte('a').unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_byte(i).unwrap(), expected);
    }

    #[test]
    fn byte_to_integer() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_byte('a' as u8).unwrap();
        let d2 = runtime.get_data_mut().add_number(0.into()).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let expected = SimpleDataRuntimeNC::byte_to_number('a' as u8).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), expected);
    }

    #[test]
    fn byte_to_char() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_byte('a' as u8).unwrap();
        let d2 = runtime.get_data_mut().add_char('a').unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let expected = SimpleDataRuntimeNC::byte_to_char('a' as u8).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_char(i).unwrap(), expected);
    }

    #[test]
    fn char_list_to_byte() {
        let mut runtime = create_simple_runtime();

        let d1 = add_char_list(runtime.get_data_mut(), "100");
        let d2 = runtime.get_data_mut().add_byte(0).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_byte(i).unwrap(), 100.into());
    }

    #[test]
    fn char_list_to_char() {
        let mut runtime = create_simple_runtime();

        let d1 = add_char_list(runtime.get_data_mut(), "c");
        let d2 = runtime.get_data_mut().add_char('a').unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_char(i).unwrap(), 'c');
    }

    #[test]
    fn char_list_to_number() {
        let mut runtime = create_simple_runtime();

        let d1 = add_char_list(runtime.get_data_mut(), "100");
        let d2 = runtime.get_data_mut().add_number(0.into()).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 100.into());
    }
}

#[cfg(test)]
mod lists {
    use garnish_lang_simple_data::{symbol_value, SimpleDataRuntimeNC};

    use crate::simple::testing_utilities::{
        add_byte_list, add_char_list, add_list_with_start, add_range, create_simple_runtime,
    };
    use garnish_lang_traits::{GarnishData, GarnishRuntime, NO_CONTEXT};

    #[test]
    fn range_to_list() {
        let mut runtime = create_simple_runtime();

        let d1 = add_range(runtime.get_data_mut(), 10, 20);
        let list = add_list_with_start(runtime.get_data_mut(), 1, 0);

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_data_mut().get_register(0).unwrap();
        let len = runtime.get_data_mut().get_list_len(addr).unwrap();
        assert_eq!(len, 11);

        for i in 0..10 {
            let item_addr = runtime.get_data_mut().get_list_item(addr, i.into()).unwrap();
            assert_eq!(runtime.get_data_mut().get_number(item_addr).unwrap(), (10 + i).into());
        }
    }

    #[test]
    fn char_list_to_list() {
        let mut runtime = create_simple_runtime();

        let input = "characters";
        let d1 = add_char_list(runtime.get_data_mut(), input);
        let list = add_list_with_start(runtime.get_data_mut(), 1, 0);

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_data_mut().get_register(0).unwrap();
        let expected = SimpleDataRuntimeNC::parse_char_list(input).unwrap();
        let mut result = vec![];

        for i in 0..input.len() {
            let item_addr = runtime.get_data_mut().get_list_item(addr, i.into()).unwrap();
            let item = runtime.get_data_mut().get_char(item_addr).unwrap();
            result.push(item);
        }

        assert_eq!(result, expected);
    }

    #[test]
    fn byte_list_to_list() {
        let mut runtime = create_simple_runtime();

        let input = "characters";
        let d1 = add_byte_list(runtime.get_data_mut(), input);
        let list = add_list_with_start(runtime.get_data_mut(), 1, 0);

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_data_mut().get_register(0).unwrap();
        let expected = SimpleDataRuntimeNC::parse_byte_list(input).unwrap();
        let mut result = vec![];

        for i in 0..input.len() {
            let item_addr = runtime.get_data_mut().get_list_item(addr, i.into()).unwrap();
            let item = runtime.get_data_mut().get_byte(item_addr).unwrap();
            result.push(item);
        }

        assert_eq!(result, expected);
    }

    #[test]
    fn slice_of_list_to_list() {
        let mut runtime = create_simple_runtime();

        let d1 = add_list_with_start(runtime.get_data_mut(), 10, 20);
        let d2 = add_range(runtime.get_data_mut(), 2, 7);
        let d3 = runtime.get_data_mut().add_slice(d1, d2).unwrap();
        let list = add_list_with_start(runtime.get_data_mut(), 1, 0);

        runtime.get_data_mut().push_register(d3).unwrap();
        runtime.get_data_mut().push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_data_mut().get_register(0).unwrap();
        let len = runtime.get_data_mut().get_list_len(addr).unwrap();
        assert_eq!(len, 6);

        for i in 0..6 {
            let value = 22 + i;
            let item_addr = runtime.get_data_mut().get_list_item(addr, i.into()).unwrap();
            let (left, right) = runtime.get_data_mut().get_pair(item_addr).unwrap();
            let s = symbol_value(format!("val{}", value).as_ref());
            assert_eq!(runtime.get_data_mut().get_symbol(left).unwrap(), s);
            assert_eq!(runtime.get_data_mut().get_number(right).unwrap(), value.into());

            let association = runtime.get_data_mut().get_list_item_with_symbol(addr, s).unwrap().unwrap();
            assert_eq!(runtime.get_data_mut().get_number(association).unwrap(), value.into())
        }
    }

    #[test]
    fn slice_of_char_list_to_list() {
        let mut runtime = create_simple_runtime();

        let input = "characters";
        let d1 = add_char_list(runtime.get_data_mut(), input);
        let d2 = add_range(runtime.get_data_mut(), 2, 7); // "aracte"
        let d3 = runtime.get_data_mut().add_slice(d1, d2).unwrap();
        let list = add_list_with_start(runtime.get_data_mut(), 1, 0);

        runtime.get_data_mut().push_register(d3).unwrap();
        runtime.get_data_mut().push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_data_mut().get_register(0).unwrap();
        let expected: Vec<char> = SimpleDataRuntimeNC::parse_char_list(input)
            .unwrap()
            .iter()
            .skip(2)
            .take(6)
            .map(|c| *c)
            .collect();
        let mut result = vec![];

        for i in 0..expected.len() {
            let item_addr = runtime.get_data_mut().get_list_item(addr, i.into()).unwrap();
            let item = runtime.get_data_mut().get_char(item_addr).unwrap();
            result.push(item);
        }

        assert_eq!(result, expected);
    }

    #[test]
    fn slice_of_byte_list_to_list() {
        let mut runtime = create_simple_runtime();

        let input = "characters";
        let d1 = add_byte_list(runtime.get_data_mut(), input);
        let d2 = add_range(runtime.get_data_mut(), 2, 7);
        let d3 = runtime.get_data_mut().add_slice(d1, d2).unwrap();
        let list = add_list_with_start(runtime.get_data_mut(), 1, 0);

        runtime.get_data_mut().push_register(d3).unwrap();
        runtime.get_data_mut().push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_data_mut().get_register(0).unwrap();
        let expected: Vec<u8> = SimpleDataRuntimeNC::parse_byte_list(input)
            .unwrap()
            .iter()
            .skip(2)
            .take(6)
            .map(|c| *c)
            .collect();
        let mut result = vec![];

        for i in 0..expected.len() {
            let item_addr = runtime.get_data_mut().get_list_item(addr, i.into()).unwrap();
            let item = runtime.get_data_mut().get_byte(item_addr).unwrap();
            result.push(item);
        }

        assert_eq!(result, expected);
    }
}

#[cfg(test)]
mod concatenation {
    use garnish_lang_simple_data::symbol_value;

    use crate::simple::testing_utilities::{add_concatenation_with_start, add_list_with_start, add_range, create_simple_runtime};
    use garnish_lang_traits::{GarnishData, GarnishRuntime, NO_CONTEXT};

    #[test]
    fn concatenation_to_list() {
        let mut runtime = create_simple_runtime();

        let start_value = 20;
        let d1 = add_concatenation_with_start(runtime.get_data_mut(), 10, start_value);
        let d2 = add_list_with_start(runtime.get_data_mut(), 0, 0);

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_data_mut().get_register(0).unwrap();
        let len = runtime.get_data_mut().get_list_len(addr).unwrap();
        assert_eq!(len, 10);

        for i in 0..10 {
            let item_addr = runtime.get_data_mut().get_list_item(addr, i.into()).unwrap();
            let (left, right) = runtime.get_data_mut().get_pair(item_addr).unwrap();
            let s = symbol_value(format!("val{}", start_value + i).as_ref());
            assert_eq!(runtime.get_data_mut().get_symbol(left).unwrap(), s);
            assert_eq!(runtime.get_data_mut().get_number(right).unwrap(), (start_value + i).into());

            let association = runtime.get_data_mut().get_list_item_with_symbol(addr, s).unwrap().unwrap();
            assert_eq!(runtime.get_data_mut().get_number(association).unwrap(), (start_value + i).into())
        }
    }

    #[test]
    fn slice_of_concatenation_to_list() {
        let mut runtime = create_simple_runtime();

        let start_value = 20;
        let d1 = add_concatenation_with_start(runtime.get_data_mut(), 10, start_value);
        let d2 = add_range(runtime.get_data_mut(), 2, 8);
        let d3 = runtime.get_data_mut().add_slice(d1, d2).unwrap();
        let d4 = add_list_with_start(runtime.get_data_mut(), 0, 0);

        runtime.get_data_mut().push_register(d3).unwrap();
        runtime.get_data_mut().push_register(d4).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_data_mut().get_register(0).unwrap();
        let len = runtime.get_data_mut().get_list_len(addr).unwrap();
        assert_eq!(len, 7);

        for i in 0..6 {
            let item_addr = runtime.get_data_mut().get_list_item(addr, i.into()).unwrap();
            let (left, right) = runtime.get_data_mut().get_pair(item_addr).unwrap();
            let v = start_value + 2 + i;
            let s = symbol_value(format!("val{}", v).as_ref());
            assert_eq!(runtime.get_data_mut().get_symbol(left).unwrap(), s);
            assert_eq!(runtime.get_data_mut().get_number(right).unwrap(), v.into());

            let association = runtime.get_data_mut().get_list_item_with_symbol(addr, s).unwrap().unwrap();
            assert_eq!(runtime.get_data_mut().get_number(association).unwrap(), v.into())
        }
    }

    #[test]
    fn concatenation_of_lists_to_list() {
        let mut runtime = create_simple_runtime();

        let start_value = 10;
        let d1 = add_list_with_start(runtime.get_data_mut(), 10, start_value);
        let d2 = add_list_with_start(runtime.get_data_mut(), 10, start_value + 10);
        let d3 = runtime.get_data_mut().add_concatenation(d1, d2).unwrap();
        let d4 = add_list_with_start(runtime.get_data_mut(), 0, 0);

        runtime.get_data_mut().push_register(d3).unwrap();
        runtime.get_data_mut().push_register(d4).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_data_mut().get_register(0).unwrap();
        let len = runtime.get_data_mut().get_list_len(addr).unwrap();
        assert_eq!(len, 20);

        for i in 0..20 {
            let item_addr = runtime.get_data_mut().get_list_item(addr, i.into()).unwrap();
            let (left, right) = runtime.get_data_mut().get_pair(item_addr).unwrap();
            let s = symbol_value(format!("val{}", start_value + i).as_ref());
            assert_eq!(runtime.get_data_mut().get_symbol(left).unwrap(), s);
            assert_eq!(runtime.get_data_mut().get_number(right).unwrap(), (start_value + i).into());

            let association = runtime.get_data_mut().get_list_item_with_symbol(addr, s).unwrap().unwrap();
            assert_eq!(runtime.get_data_mut().get_number(association).unwrap(), (start_value + i).into())
        }
    }

    #[test]
    fn slice_of_concatenation_of_lists_to_list() {
        let mut runtime = create_simple_runtime();

        let start_value = 20;
        let d1 = add_list_with_start(runtime.get_data_mut(), 10, start_value);
        let d2 = add_list_with_start(runtime.get_data_mut(), 10, start_value + 10);
        let d3 = runtime.get_data_mut().add_concatenation(d1, d2).unwrap();
        let d4 = add_range(runtime.get_data_mut(), 8, 12);
        let d5 = runtime.get_data_mut().add_slice(d3, d4).unwrap();
        let d6 = add_list_with_start(runtime.get_data_mut(), 0, 0);

        runtime.get_data_mut().push_register(d5).unwrap();
        runtime.get_data_mut().push_register(d6).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_data_mut().get_register(0).unwrap();
        let len = runtime.get_data_mut().get_list_len(addr).unwrap();
        assert_eq!(len, 5);

        for i in 0..5 {
            let item_addr = runtime.get_data_mut().get_list_item(addr, i.into()).unwrap();
            let (left, right) = runtime.get_data_mut().get_pair(item_addr).unwrap();
            let v = start_value + 8 + i;
            let s = symbol_value(format!("val{}", v).as_ref());
            assert_eq!(runtime.get_data_mut().get_symbol(left).unwrap(), s);
            assert_eq!(runtime.get_data_mut().get_number(right).unwrap(), v.into());

            let association = runtime.get_data_mut().get_list_item_with_symbol(addr, s).unwrap().unwrap();
            assert_eq!(runtime.get_data_mut().get_number(association).unwrap(), v.into())
        }
    }
}

#[cfg(test)]
mod deferred {
    use garnish_lang_simple_data::SimpleDataRuntimeNC;

    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang_traits::{GarnishDataType, GarnishData, GarnishRuntime, NO_CONTEXT};

    #[test]
    fn char_list() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_unit().unwrap();

        runtime.get_data_mut().start_char_list().unwrap();
        let s = runtime.get_data_mut().end_char_list().unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(s).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_data_mut().get_register(0).unwrap();
        let len = runtime.get_data_mut().get_char_list_len(addr).unwrap();

        let expected = "()";
        let mut chars = String::new();

        for i in 0..len {
            let c = runtime.get_data_mut().get_char_list_item(addr, i.into()).unwrap();
            chars.push(c);
        }

        assert_eq!(chars, expected);
    }

    #[test]
    fn byte_list() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_unit().unwrap();

        runtime.get_data_mut().start_byte_list().unwrap();
        let s = runtime.get_data_mut().end_byte_list().unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(s).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(addr).unwrap(), GarnishDataType::ByteList);
    }

    #[test]
    fn symbols() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_unit().unwrap();

        let s = runtime
            .get_data_mut()
            .add_symbol(SimpleDataRuntimeNC::parse_symbol("sym").unwrap())
            .unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(s).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(addr).unwrap(), GarnishDataType::Symbol);
    }
}
