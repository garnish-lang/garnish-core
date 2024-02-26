#[cfg(test)]
mod general {

    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang_traits::{GarnishDataType, GarnishData, GarnishRuntime};

    #[test]
    fn less_than_no_references_is_err() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().add_number(10.into()).unwrap();
        let result = runtime.less_than();

        assert!(result.is_err());
    }

    #[test]
    fn less_than_of_unsupported_comparison_is_false() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let exp1 = runtime.get_data_mut().add_expression(10).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(exp1).unwrap();

        runtime.less_than().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::False);
    }
}

#[cfg(test)]
mod less_than {
    use crate::simple::testing_utilities::{add_byte_list, add_char_list, create_simple_runtime, slice_of_byte_list, slice_of_char_list};
    use garnish_lang_simple_data::{DataError, SimpleGarnishData};
    use garnish_lang_runtime::runtime_impls::SimpleGarnishRuntime;
    use garnish_lang_traits::{GarnishDataType, GarnishData, GarnishRuntime, RuntimeError};

    fn perform_compare<Setup, Op>(expected: bool, op_name: &str, op: Op, setup: Setup)
    where
        Op: Fn(&mut SimpleGarnishRuntime<SimpleGarnishData>) -> Result<Option<usize>, RuntimeError<DataError>>,
        Setup: Copy + Fn(&mut SimpleGarnishData) -> (usize, usize),
    {
        let mut runtime = create_simple_runtime();

        let registers = setup(runtime.get_data_mut());

        runtime.get_data_mut().push_register(registers.0).unwrap();
        runtime.get_data_mut().push_register(registers.1).unwrap();

        op(&mut runtime).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        let result = runtime.get_data_mut().get_data_type(i).unwrap();
        let expected_type = match expected {
            true => GarnishDataType::True,
            false => GarnishDataType::False,
        };

        assert_eq!(result, expected_type, "For op {:?}", op_name);
    }

    fn perform_all_compare<Setup>(less_than: bool, less_than_equal: bool, greater_than: bool, greater_than_equal: bool, setup: Setup)
    where
        Setup: Copy + Fn(&mut SimpleGarnishData) -> (usize, usize),
    {
        perform_compare(less_than, "less than", SimpleGarnishRuntime::less_than, setup);
        perform_compare(less_than_equal, "less than or equal", SimpleGarnishRuntime::less_than_or_equal, setup);
        perform_compare(greater_than, "greater than", SimpleGarnishRuntime::greater_than, setup);
        perform_compare(
            greater_than_equal,
            "greater than or equal",
            SimpleGarnishRuntime::greater_than_or_equal,
            setup,
        );
    }

    #[test]
    fn units_are_false() {
        perform_all_compare(false, false, false, false, |data| (data.add_unit().unwrap(), data.add_unit().unwrap()));
    }

    #[test]
    fn trues_are_false() {
        perform_all_compare(false, false, false, false, |data| (data.add_true().unwrap(), data.add_true().unwrap()));
    }

    #[test]
    fn falses_are_false() {
        perform_all_compare(false, false, false, false, |data| (data.add_false().unwrap(), data.add_false().unwrap()));
    }

    #[test]
    fn numbers_less_than() {
        perform_all_compare(true, true, false, false, |data| {
            (data.add_number(10.into()).unwrap(), data.add_number(20.into()).unwrap())
        });
    }

    #[test]
    fn numbers_equal() {
        perform_all_compare(false, true, false, true, |data| {
            (data.add_number(20.into()).unwrap(), data.add_number(20.into()).unwrap())
        });
    }

    #[test]
    fn numbers_greater_than() {
        perform_all_compare(false, false, true, true, |data| {
            (data.add_number(20.into()).unwrap(), data.add_number(10.into()).unwrap())
        });
    }

    #[test]
    fn chars_less_than() {
        perform_all_compare(true, true, false, false, |data| {
            (data.add_char('d').unwrap(), data.add_char('f').unwrap())
        });
    }

    #[test]
    fn chars_equal() {
        perform_all_compare(false, true, false, true, |data| {
            (data.add_char('d').unwrap(), data.add_char('d').unwrap())
        });
    }

    #[test]
    fn chars_greater_than() {
        perform_all_compare(false, false, true, true, |data| {
            (data.add_char('d').unwrap(), data.add_char('b').unwrap())
        });
    }

    #[test]
    fn bytes_less_than() {
        perform_all_compare(true, true, false, false, |data| (data.add_byte(10).unwrap(), data.add_byte(20).unwrap()));
    }

    #[test]
    fn bytes_equal() {
        perform_all_compare(false, true, false, true, |data| (data.add_byte(20).unwrap(), data.add_byte(20).unwrap()));
    }

    #[test]
    fn bytes_greater_than() {
        perform_all_compare(false, false, true, true, |data| (data.add_byte(20).unwrap(), data.add_byte(10).unwrap()));
    }

    #[test]
    fn char_list_less_than() {
        perform_all_compare(true, true, false, false, |data| (add_char_list(data, "aaa"), add_char_list(data, "bbb")));
    }

    #[test]
    fn char_list_less_than_dif_len() {
        perform_all_compare(true, true, false, false, |data| {
            (add_char_list(data, "aaa"), add_char_list(data, "aaaaa"))
        });
    }

    #[test]
    fn char_list_equal() {
        perform_all_compare(false, true, false, true, |data| (add_char_list(data, "aaa"), add_char_list(data, "aaa")));
    }

    #[test]
    fn char_list_greater_than() {
        perform_all_compare(false, false, true, true, |data| (add_char_list(data, "bbb"), add_char_list(data, "aaa")));
    }

    #[test]
    fn char_list_greater_than_dif_len() {
        perform_all_compare(false, false, true, true, |data| {
            (add_char_list(data, "aaaaa"), add_char_list(data, "aaa"))
        });
    }

    #[test]
    fn byte_list_less_than() {
        perform_all_compare(true, true, false, false, |data| (add_byte_list(data, "aaa"), add_byte_list(data, "bbb")));
    }

    #[test]
    fn byte_list_less_than_dif_len() {
        perform_all_compare(true, true, false, false, |data| {
            (add_byte_list(data, "aaa"), add_byte_list(data, "aaaaa"))
        });
    }

    #[test]
    fn byte_list_equal() {
        perform_all_compare(false, true, false, true, |data| (add_byte_list(data, "aaa"), add_byte_list(data, "aaa")));
    }

    #[test]
    fn byte_list_greater_than() {
        perform_all_compare(false, false, true, true, |data| (add_byte_list(data, "bbb"), add_byte_list(data, "aaa")));
    }

    #[test]
    fn byte_list_greater_than_dif_len() {
        perform_all_compare(false, false, true, true, |data| {
            (add_byte_list(data, "aaaaa"), add_byte_list(data, "aaa"))
        });
    }

    #[test]
    fn slice_of_char_list_less_than() {
        perform_all_compare(true, true, false, false, |data| {
            (slice_of_char_list(data, "aaaaaa", 0, 3), slice_of_char_list(data, "bbbbbb", 1, 4))
        });
    }

    #[test]
    fn slice_of_char_list_less_than_dif_len() {
        perform_all_compare(true, true, false, false, |data| {
            (slice_of_char_list(data, "aaaaaa", 1, 4), slice_of_char_list(data, "aaaaaaaaa", 1, 6))
        });
    }

    #[test]
    fn slice_of_char_list_equal() {
        perform_all_compare(false, true, false, true, |data| {
            (slice_of_char_list(data, "aaaaaa", 0, 3), slice_of_char_list(data, "aaaaaa", 1, 4))
        });
    }

    #[test]
    fn slice_of_char_list_greater_than() {
        perform_all_compare(false, false, true, true, |data| {
            (slice_of_char_list(data, "bbbbbb", 0, 3), slice_of_char_list(data, "aaaaaa", 1, 4))
        });
    }

    #[test]
    fn slice_of_char_list_greater_than_dif_len() {
        perform_all_compare(false, false, true, true, |data| {
            (slice_of_char_list(data, "aaaaaaaaa", 2, 7), slice_of_char_list(data, "aaaaaa", 1, 4))
        });
    }

    #[test]
    fn slice_of_byte_list_less_than() {
        perform_all_compare(true, true, false, false, |data| {
            (slice_of_byte_list(data, "aaaaaa", 0, 3), slice_of_byte_list(data, "bbbbbb", 1, 4))
        });
    }

    #[test]
    fn slice_of_byte_list_less_than_dif_len() {
        perform_all_compare(true, true, false, false, |data| {
            (slice_of_byte_list(data, "aaaaaa", 1, 4), slice_of_byte_list(data, "aaaaaaaaa", 1, 6))
        });
    }

    #[test]
    fn slice_of_byte_list_equal() {
        perform_all_compare(false, true, false, true, |data| {
            (slice_of_byte_list(data, "aaaaaa", 0, 3), slice_of_byte_list(data, "aaaaaa", 1, 4))
        });
    }

    #[test]
    fn slice_of_byte_list_greater_than() {
        perform_all_compare(false, false, true, true, |data| {
            (slice_of_byte_list(data, "bbbbbb", 0, 3), slice_of_byte_list(data, "aaaaaa", 1, 4))
        });
    }

    #[test]
    fn slice_of_byte_list_greater_than_dif_len() {
        perform_all_compare(false, false, true, true, |data| {
            (slice_of_byte_list(data, "aaaaaaaaa", 2, 7), slice_of_byte_list(data, "aaaaaa", 1, 4))
        });
    }
}
