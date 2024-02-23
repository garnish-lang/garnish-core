#[cfg(test)]
mod tests {
    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang_simple_data::data::SimpleNumber;
    use garnish_lang_traits::{ExpressionDataType, GarnishLangRuntimeData, GarnishRuntime, Instruction, NO_CONTEXT};

    #[test]
    fn make_list() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i2 = runtime.get_data_mut().add_number(20.into()).unwrap();
        let i3 = runtime.get_data_mut().add_number(20.into()).unwrap();
        let start = runtime.get_data_mut().get_data_len();

        runtime.get_data_mut().push_register(i1).unwrap();
        runtime.get_data_mut().push_register(i2).unwrap();
        runtime.get_data_mut().push_register(i3).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::MakeList, Some(3)).unwrap();

        runtime.make_list(3).unwrap();

        assert_eq!(runtime.get_data_mut().get_list_len(start).unwrap(), 3);
        assert_eq!(runtime.get_data_mut().get_list_item(start, 0.into()).unwrap(), i1);
        assert_eq!(runtime.get_data_mut().get_list_item(start, 1.into()).unwrap(), i2);
        assert_eq!(runtime.get_data_mut().get_list_item(start, 2.into()).unwrap(), i3);
    }

    #[test]
    fn make_list_no_refs_is_err() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().add_number(20.into()).unwrap();
        runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::MakeList, Some(3)).unwrap();

        let result = runtime.make_list(3);

        assert!(result.is_err());
    }

    #[test]
    fn make_list_with_associations() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_symbol(1).unwrap();
        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i3 = runtime.get_data_mut().add_symbol(2).unwrap();
        let i4 = runtime.get_data_mut().add_number(20.into()).unwrap();
        let i5 = runtime.get_data_mut().add_symbol(3).unwrap();
        let i6 = runtime.get_data_mut().add_number(30.into()).unwrap();
        // 6
        let i7 = runtime.get_data_mut().add_pair((i1, i2)).unwrap();
        let i8 = runtime.get_data_mut().add_pair((i3, i4)).unwrap();
        let i9 = runtime.get_data_mut().add_pair((i5, i6)).unwrap();

        let start = runtime.get_data_mut().get_data_len();

        runtime.get_data_mut().push_register(i7).unwrap();
        runtime.get_data_mut().push_register(i8).unwrap();
        runtime.get_data_mut().push_register(i9).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::MakeList, Some(3)).unwrap();

        runtime.make_list(3).unwrap();

        assert_eq!(runtime.get_data_mut().get_list_len(start).unwrap(), 3);
        assert_eq!(runtime.get_data_mut().get_list_item(start, 0.into()).unwrap(), i7);
        assert_eq!(runtime.get_data_mut().get_list_item(start, 1.into()).unwrap(), i8);
        assert_eq!(runtime.get_data_mut().get_list_item(start, 2.into()).unwrap(), i9);

        assert_eq!(runtime.get_data_mut().get_list_associations_len(start).unwrap(), 3);
        assert_eq!(runtime.get_data_mut().get_list_association(start, 0.into()).unwrap(), i7);
        assert_eq!(runtime.get_data_mut().get_list_association(start, 1.into()).unwrap(), i8);
        assert_eq!(runtime.get_data_mut().get_list_association(start, 2.into()).unwrap(), i9);
    }

    #[test]
    fn apply() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_symbol(1).unwrap();
        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i3 = runtime.get_data_mut().add_pair((i1, i2)).unwrap();
        runtime.get_data_mut().start_list(1).unwrap();
        runtime.get_data_mut().add_to_list(i3, true).unwrap();
        let i4 = runtime.get_data_mut().end_list().unwrap();
        let i5 = runtime.get_data_mut().add_symbol(1).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::Access, None).unwrap();

        runtime.get_data_mut().push_register(i4).unwrap();
        runtime.get_data_mut().push_register(i5).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_mut().get_register(0).unwrap(), i2);
    }

    #[test]
    fn apply_with_integer() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_symbol(1).unwrap();
        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i3 = runtime.get_data_mut().add_pair((i1, i2)).unwrap();
        runtime.get_data_mut().start_list(1).unwrap();
        runtime.get_data_mut().add_to_list(i3, true).unwrap();
        let i4 = runtime.get_data_mut().end_list().unwrap();
        let i5 = runtime.get_data_mut().add_number(0.into()).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::Access, None).unwrap();

        runtime.get_data_mut().push_register(i4).unwrap();
        runtime.get_data_mut().push_register(i5).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_mut().get_register(0).unwrap(), i3);
    }

    #[test]
    fn apply_char_list_with_integer() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().start_char_list().unwrap();
        runtime.get_data_mut().add_to_char_list('a').unwrap();
        runtime.get_data_mut().add_to_char_list('b').unwrap();
        runtime.get_data_mut().add_to_char_list('c').unwrap();
        let d1 = runtime.get_data_mut().end_char_list().unwrap();
        let d2 = runtime.get_data_mut().add_number(2.into()).unwrap();
        let start = runtime.get_data_mut().get_data_len();

        runtime.get_data_mut().push_instruction(Instruction::Access, None).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_mut().get_register(0).unwrap(), start);
        assert_eq!(runtime.get_data_mut().get_char(start).unwrap(), 'c');
    }

    #[test]
    fn apply_byte_list_with_integer() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().start_byte_list().unwrap();
        runtime.get_data_mut().add_to_byte_list(10).unwrap();
        runtime.get_data_mut().add_to_byte_list(20).unwrap();
        runtime.get_data_mut().add_to_byte_list(30).unwrap();
        let d1 = runtime.get_data_mut().end_byte_list().unwrap();
        let d2 = runtime.get_data_mut().add_number(2.into()).unwrap();
        let start = runtime.get_data_mut().get_data_len();

        runtime.get_data_mut().push_instruction(Instruction::Access, None).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_mut().get_register(0).unwrap(), start);
        assert_eq!(runtime.get_data_mut().get_byte(start).unwrap(), 30.into());
    }

    #[test]
    fn apply_with_integer_out_of_bounds_is_unit() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_symbol(1).unwrap();
        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i3 = runtime.get_data_mut().add_pair((i1, i2)).unwrap();
        runtime.get_data_mut().start_list(1).unwrap();
        runtime.get_data_mut().add_to_list(i3, true).unwrap();
        let i4 = runtime.get_data_mut().end_list().unwrap();
        let i5 = runtime.get_data_mut().add_number(10.into()).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::Access, None).unwrap();

        runtime.get_data_mut().push_register(i4).unwrap();
        runtime.get_data_mut().push_register(i5).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn apply_with_number_negative_is_unit() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_symbol(1).unwrap();
        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i3 = runtime.get_data_mut().add_pair((i1, i2)).unwrap();
        runtime.get_data_mut().start_list(1).unwrap();
        runtime.get_data_mut().add_to_list(i3, true).unwrap();
        let i4 = runtime.get_data_mut().end_list().unwrap();
        let i5 = runtime.get_data_mut().add_number(SimpleNumber::Integer(-1)).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::Access, None).unwrap();

        runtime.get_data_mut().push_register(i4).unwrap();
        runtime.get_data_mut().push_register(i5).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn apply_non_list_on_left_is_unit() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i2 = runtime.get_data_mut().add_symbol(1).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::Access, None).unwrap();

        runtime.get_data_mut().push_register(i1).unwrap();
        runtime.get_data_mut().push_register(i2).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn apply_non_symbol_on_right_is_unit() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_symbol(1).unwrap();
        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i3 = runtime.get_data_mut().add_pair((i1, i2)).unwrap();
        runtime.get_data_mut().start_list(1).unwrap();
        runtime.get_data_mut().add_to_list(i3, true).unwrap();
        let i4 = runtime.get_data_mut().end_list().unwrap();
        let i5 = runtime.get_data_mut().add_expression(10).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::Access, None).unwrap();

        runtime.get_data_mut().push_register(i4).unwrap();
        runtime.get_data_mut().push_register(i5).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn apply_no_refs_is_err() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_symbol(1).unwrap();
        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i3 = runtime.get_data_mut().add_pair((i1, i2)).unwrap();
        runtime.get_data_mut().start_list(1).unwrap();
        runtime.get_data_mut().add_to_list(i3, true).unwrap();
        let _i4 = runtime.get_data_mut().end_list().unwrap();
        let _i5 = runtime.get_data_mut().add_symbol(1).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::Access, None).unwrap();

        let result = runtime.apply(NO_CONTEXT);

        assert!(result.is_err());
    }

    #[test]
    fn apply_with_non_existent_key() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_symbol(1).unwrap();
        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i3 = runtime.get_data_mut().add_pair((i1, i2)).unwrap();
        runtime.get_data_mut().start_list(1).unwrap();
        runtime.get_data_mut().add_to_list(i3, true).unwrap();
        let i4 = runtime.get_data_mut().end_list().unwrap();
        let i5 = runtime.get_data_mut().add_symbol(2).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::Access, None).unwrap();

        runtime.get_data_mut().push_register(i4).unwrap();
        runtime.get_data_mut().push_register(i5).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::Unit);
    }
}

#[cfg(test)]
mod ranges {

    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang_traits::{ExpressionDataType, GarnishLangRuntimeData, GarnishRuntime, Instruction, NO_CONTEXT};

    #[test]
    fn apply_with_integer() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i2 = runtime.get_data_mut().add_number(20.into()).unwrap();
        let i3 = runtime.get_data_mut().add_range(i1, i2).unwrap();
        let i4 = runtime.get_data_mut().add_number(5.into()).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::Access, None).unwrap();

        runtime.get_data_mut().push_register(i3).unwrap();
        runtime.get_data_mut().push_register(i4).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 15.into());
    }

    #[test]
    fn apply_with_integer_out_of_range() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i2 = runtime.get_data_mut().add_number(20.into()).unwrap();
        let i3 = runtime.get_data_mut().add_range(i1, i2).unwrap();
        let i4 = runtime.get_data_mut().add_number(30.into()).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::Access, None).unwrap();

        runtime.get_data_mut().push_register(i3).unwrap();
        runtime.get_data_mut().push_register(i4).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::Unit);
    }
}

#[cfg(test)]
mod slice {
    use crate::simple::testing_utilities::{add_integer_list, add_list, add_pair, add_range, create_simple_runtime};
    use garnish_lang_simple_data::SimpleDataRuntimeNC;
    use garnish_lang_traits::{ExpressionDataType, GarnishLangRuntimeData, GarnishRuntime, NO_CONTEXT};

    #[test]
    fn index_slice_of_list() {
        let mut runtime = create_simple_runtime();

        let d1 = add_integer_list(runtime.get_data_mut(), 10);
        let d2 = runtime.get_data_mut().add_number(1.into()).unwrap();
        let d3 = runtime.get_data_mut().add_number(4.into()).unwrap();
        let d4 = runtime.get_data_mut().add_range(d2, d3).unwrap();
        let d5 = runtime.get_data_mut().add_slice(d1, d4).unwrap();
        let d6 = runtime.get_data_mut().add_number(2.into()).unwrap();

        runtime.get_data_mut().push_register(d5).unwrap();
        runtime.get_data_mut().push_register(d6).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 40.into());
    }

    #[test]
    fn index_slice_of_char_list() {
        let mut runtime = create_simple_runtime();

        let chars = SimpleDataRuntimeNC::parse_char_list("abcde").unwrap();

        runtime.get_data_mut().start_char_list().unwrap();
        for c in chars {
            runtime.get_data_mut().add_to_char_list(c).unwrap();
        }
        let list = runtime.get_data_mut().end_char_list().unwrap();

        let range = add_range(runtime.get_data_mut(), 1, 3);
        let slice = runtime.get_data_mut().add_slice(list, range).unwrap();
        let d6 = runtime.get_data_mut().add_number(2.into()).unwrap();

        runtime.get_data_mut().push_register(slice).unwrap();
        runtime.get_data_mut().push_register(d6).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_char(i).unwrap(), 'd');
    }

    #[test]
    fn index_slice_of_byte_list() {
        let mut runtime = create_simple_runtime();

        let bytes = SimpleDataRuntimeNC::parse_byte_list("abcde").unwrap();

        runtime.get_data_mut().start_byte_list().unwrap();
        for b in bytes {
            runtime.get_data_mut().add_to_byte_list(b).unwrap();
        }
        let list = runtime.get_data_mut().end_byte_list().unwrap();

        let range = add_range(runtime.get_data_mut(), 1, 3);
        let slice = runtime.get_data_mut().add_slice(list, range).unwrap();
        let d6 = runtime.get_data_mut().add_number(2.into()).unwrap();

        runtime.get_data_mut().push_register(slice).unwrap();
        runtime.get_data_mut().push_register(d6).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_byte(i).unwrap(), 'd' as u8);
    }

    #[test]
    fn sym_index_slice_of_list() {
        let mut runtime = create_simple_runtime();

        let d1 = add_list(runtime.get_data_mut(), 10);
        let d2 = runtime.get_data_mut().add_number(1.into()).unwrap();
        let d3 = runtime.get_data_mut().add_number(4.into()).unwrap();
        let d4 = runtime.get_data_mut().add_range(d2, d3).unwrap();
        let d5 = runtime.get_data_mut().add_slice(d1, d4).unwrap();
        let d6 = runtime
            .get_data_mut()
            .add_symbol(SimpleDataRuntimeNC::parse_symbol("val4").unwrap())
            .unwrap();

        runtime.get_data_mut().push_register(d5).unwrap();
        runtime.get_data_mut().push_register(d6).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 50.into());
    }

    #[test]
    fn sym_index_slice_of_list_duplicate() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().start_list(10).unwrap();
        for i in 0..10 {
            let d = add_pair(runtime.get_data_mut(), format!("val{}", i).as_str(), (i + 1) * 10);
            runtime.get_data_mut().add_to_list(d, true).unwrap();
        }

        let d = add_pair(runtime.get_data_mut(), format!("val{}", 5).as_str(), 123);
        runtime.get_data_mut().add_to_list(d, true).unwrap();

        let d1 = runtime.get_data_mut().end_list().unwrap();

        let d2 = runtime.get_data_mut().add_number(2.into()).unwrap();
        let d3 = runtime.get_data_mut().add_number(11.into()).unwrap();
        let d4 = runtime.get_data_mut().add_range(d2, d3).unwrap();
        let d5 = runtime.get_data_mut().add_slice(d1, d4).unwrap();
        let d6 = runtime
            .get_data_mut()
            .add_symbol(SimpleDataRuntimeNC::parse_symbol("val5").unwrap())
            .unwrap();

        runtime.get_data_mut().push_register(d5).unwrap();
        runtime.get_data_mut().push_register(d6).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 123.into());
    }

    #[test]
    fn sym_index_slice_of_list_sym_not_in_slice_before() {
        let mut runtime = create_simple_runtime();

        let d1 = add_list(runtime.get_data_mut(), 10);
        let d2 = runtime.get_data_mut().add_number(2.into()).unwrap();
        let d3 = runtime.get_data_mut().add_number(4.into()).unwrap();
        let d4 = runtime.get_data_mut().add_range(d2, d3).unwrap();
        let d5 = runtime.get_data_mut().add_slice(d1, d4).unwrap();
        let d6 = runtime.get_data_mut().add_symbol(1).unwrap();

        runtime.get_data_mut().push_register(d5).unwrap();
        runtime.get_data_mut().push_register(d6).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn sym_index_slice_of_list_sym_not_in_slice() {
        let mut runtime = create_simple_runtime();

        let d1 = add_list(runtime.get_data_mut(), 10);
        let d2 = runtime.get_data_mut().add_number(2.into()).unwrap();
        let d3 = runtime.get_data_mut().add_number(4.into()).unwrap();
        let d4 = runtime.get_data_mut().add_range(d2, d3).unwrap();
        let d5 = runtime.get_data_mut().add_slice(d1, d4).unwrap();
        let d6 = runtime.get_data_mut().add_symbol(8).unwrap();

        runtime.get_data_mut().push_register(d5).unwrap();
        runtime.get_data_mut().push_register(d6).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::Unit);
    }
}

#[cfg(test)]
mod concatenation {
    use garnish_lang_simple_data::data::SimpleNumber::Integer;
    use crate::simple::testing_utilities::{
        add_concatenation_with_start, add_integer_list_with_start, add_list_with_start, add_range, create_simple_runtime,
    };
    use garnish_lang_simple_data::SimpleDataRuntimeNC;
    use garnish_lang_traits::{ExpressionDataType, GarnishLangRuntimeData, GarnishRuntime, NO_CONTEXT};

    #[test]
    fn index_concat_of_items_with_number() {
        let mut runtime = create_simple_runtime();

        let d1 = add_concatenation_with_start(runtime.get_data_mut(), 10, 20);
        let d2 = runtime.get_data_mut().add_number(3.into()).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        let (_left, right) = runtime.get_data_mut().get_pair(i).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(right).unwrap(), 23.into());
    }

    #[test]
    fn index_concat_of_items_with_symbol() {
        let mut runtime = create_simple_runtime();

        let d1 = add_concatenation_with_start(runtime.get_data_mut(), 10, 20);
        let d2 = runtime
            .get_data_mut()
            .add_symbol(SimpleDataRuntimeNC::parse_symbol("val23").unwrap())
            .unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 23.into());
    }

    #[test]
    fn index_concat_of_lists_with_number() {
        let mut runtime = create_simple_runtime();

        let d1 = add_integer_list_with_start(runtime.get_data_mut(), 10, 20);
        let d2 = add_integer_list_with_start(runtime.get_data_mut(), 10, 40);
        let d3 = runtime.get_data_mut().add_concatenation(d1, d2).unwrap();
        let d4 = runtime.get_data_mut().add_number(13.into()).unwrap();

        runtime.get_data_mut().push_register(d3).unwrap();
        runtime.get_data_mut().push_register(d4).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 43.into());
    }

    #[test]
    fn index_concat_of_lists_with_symbol() {
        let mut runtime = create_simple_runtime();

        let d1 = add_list_with_start(runtime.get_data_mut(), 10, 20);
        let d2 = add_list_with_start(runtime.get_data_mut(), 10, 40);
        let d3 = runtime.get_data_mut().add_concatenation(d1, d2).unwrap();
        let d4 = runtime
            .get_data_mut()
            .add_symbol(SimpleDataRuntimeNC::parse_symbol("val43").unwrap())
            .unwrap();

        runtime.get_data_mut().push_register(d3).unwrap();
        runtime.get_data_mut().push_register(d4).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 43.into());
    }

    #[test]
    fn index_concat_of_lists_with_duplicate_symbol() {
        let mut runtime = create_simple_runtime();

        let d1 = add_list_with_start(runtime.get_data_mut(), 10, 20);
        let k1 = runtime.get_data_mut()
            .add_symbol(SimpleDataRuntimeNC::parse_symbol("val25").unwrap())
            .unwrap();
        let v1 = runtime.get_data_mut().add_number(Integer(123)).unwrap();
        let d2 = runtime.get_data_mut().add_pair((k1, v1)).unwrap();
        let d3 = runtime.get_data_mut().add_concatenation(d1, d2).unwrap();
        let d4 = runtime
            .get_data_mut()
            .add_symbol(SimpleDataRuntimeNC::parse_symbol("val25").unwrap())
            .unwrap();

        runtime.get_data_mut().push_register(d3).unwrap();
        runtime.get_data_mut().push_register(d4).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 123.into());
    }

    #[test]
    fn index_slice_of_concat_of_items_with_number() {
        let mut runtime = create_simple_runtime();

        let d1 = add_concatenation_with_start(runtime.get_data_mut(), 10, 20);
        let d2 = add_range(runtime.get_data_mut(), 2, 5);
        let d3 = runtime.get_data_mut().add_slice(d1, d2).unwrap();
        let d4 = runtime.get_data_mut().add_number(1.into()).unwrap();

        runtime.get_data_mut().push_register(d3).unwrap();
        runtime.get_data_mut().push_register(d4).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        let (_left, right) = runtime.get_data_mut().get_pair(i).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(right).unwrap(), 23.into());
    }

    #[test]
    fn index_slice_of_concat_of_items_with_symbol() {
        let mut runtime = create_simple_runtime();

        let d1 = add_concatenation_with_start(runtime.get_data_mut(), 10, 20);
        let d2 = add_range(runtime.get_data_mut(), 2, 5);
        let d3 = runtime.get_data_mut().add_slice(d1, d2).unwrap();
        let d4 = runtime
            .get_data_mut()
            .add_symbol(SimpleDataRuntimeNC::parse_symbol("val23").unwrap())
            .unwrap();

        runtime.get_data_mut().push_register(d3).unwrap();
        runtime.get_data_mut().push_register(d4).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 23.into());
    }

    #[test]
    fn index_slice_of_concat_of_items_with_duplicate_symbol() {
        let mut runtime = create_simple_runtime();

        let d1 = add_concatenation_with_start(runtime.get_data_mut(), 10, 20);
        let d2 = runtime
            .get_data_mut()
            .add_symbol(SimpleDataRuntimeNC::parse_symbol("val25").unwrap())
            .unwrap();
        let d3 = runtime.get_data_mut().add_number(Integer(123)).unwrap();
        let d4 = runtime.get_data_mut().add_pair((d2, d3)).unwrap();

        let d5 = runtime.get_data_mut().add_concatenation(d1, d4).unwrap();
        let d6 = add_range(runtime.get_data_mut(), 2, 11);
        let d7 = runtime.get_data_mut().add_slice(d5, d6).unwrap();

        let d8 = runtime
            .get_data_mut()
            .add_symbol(SimpleDataRuntimeNC::parse_symbol("val25").unwrap())
            .unwrap();

        runtime.get_data_mut().push_register(d7).unwrap();
        runtime.get_data_mut().push_register(d8).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 123.into());
    }

    #[test]
    fn index_slice_of_concat_of_lists_with_number() {
        let mut runtime = create_simple_runtime();

        let d1 = add_integer_list_with_start(runtime.get_data_mut(), 10, 20);
        let d2 = add_integer_list_with_start(runtime.get_data_mut(), 10, 40);
        let d3 = runtime.get_data_mut().add_concatenation(d1, d2).unwrap();
        let d4 = add_range(runtime.get_data_mut(), 12, 15);
        let d5 = runtime.get_data_mut().add_slice(d3, d4).unwrap();
        let d6 = runtime.get_data_mut().add_number(1.into()).unwrap();

        runtime.get_data_mut().push_register(d5).unwrap();
        runtime.get_data_mut().push_register(d6).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 43.into());
    }

    #[test]
    fn index_slice_of_concat_of_lists_with_symbol() {
        let mut runtime = create_simple_runtime();

        let d1 = add_list_with_start(runtime.get_data_mut(), 10, 20);
        let d2 = add_list_with_start(runtime.get_data_mut(), 10, 40);
        let d3 = runtime.get_data_mut().add_concatenation(d1, d2).unwrap();
        let d4 = add_range(runtime.get_data_mut(), 12, 15);
        let d5 = runtime.get_data_mut().add_slice(d3, d4).unwrap();
        let d6 = runtime
            .get_data_mut()
            .add_symbol(SimpleDataRuntimeNC::parse_symbol("val43").unwrap())
            .unwrap();

        runtime.get_data_mut().push_register(d5).unwrap();
        runtime.get_data_mut().push_register(d6).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 43.into());
    }

    #[test]
    fn index_slice_of_concat_of_lists_with_symbol_range_across_lists() {
        let mut runtime = create_simple_runtime();

        let d1 = add_list_with_start(runtime.get_data_mut(), 10, 20);
        let d2 = add_list_with_start(runtime.get_data_mut(), 10, 40);
        let d3 = runtime.get_data_mut().add_concatenation(d1, d2).unwrap();
        let d4 = add_range(runtime.get_data_mut(), 8, 12);
        let d5 = runtime.get_data_mut().add_slice(d3, d4).unwrap();
        let d6 = runtime
            .get_data_mut()
            .add_symbol(SimpleDataRuntimeNC::parse_symbol("val40").unwrap())
            .unwrap();

        runtime.get_data_mut().push_register(d5).unwrap();
        runtime.get_data_mut().push_register(d6).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 40.into());
    }

    #[test]
    fn index_slice_of_concat_of_lists_with_symbol_out_of_bounds() {
        let mut runtime = create_simple_runtime();

        let d1 = add_list_with_start(runtime.get_data_mut(), 10, 20);
        let d2 = add_list_with_start(runtime.get_data_mut(), 10, 40);
        let d3 = runtime.get_data_mut().add_concatenation(d1, d2).unwrap();
        let d4 = add_range(runtime.get_data_mut(), 12, 15);
        let d5 = runtime.get_data_mut().add_slice(d3, d4).unwrap();
        let d6 = runtime
            .get_data_mut()
            .add_symbol(SimpleDataRuntimeNC::parse_symbol("val23").unwrap())
            .unwrap();

        runtime.get_data_mut().push_register(d5).unwrap();
        runtime.get_data_mut().push_register(d6).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn index_slice_of_concat_of_lists_with_symbol_out_of_bounds_same_list() {
        let mut runtime = create_simple_runtime();

        let d1 = add_list_with_start(runtime.get_data_mut(), 10, 20);
        let d2 = add_list_with_start(runtime.get_data_mut(), 10, 40);
        let d3 = runtime.get_data_mut().add_concatenation(d1, d2).unwrap();
        let d4 = add_range(runtime.get_data_mut(), 12, 15);
        let d5 = runtime.get_data_mut().add_slice(d3, d4).unwrap();
        let d6 = runtime
            .get_data_mut()
            .add_symbol(SimpleDataRuntimeNC::parse_symbol("val48").unwrap())
            .unwrap();

        runtime.get_data_mut().push_register(d5).unwrap();
        runtime.get_data_mut().push_register(d6).unwrap();

        runtime.apply(NO_CONTEXT).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::Unit);
    }
}
