

#[cfg(test)]
mod tests {
    use garnish_lang_simple_data::SimpleDataRuntimeNC;
    use garnish_lang_traits::{EmptyContext, GarnishData, GarnishRuntime, Instruction};
    use crate::simple::testing_utilities::create_simple_runtime;

    #[test]
    fn access_integer_to_list() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let d2 = runtime.get_data_mut().add_number(20.into()).unwrap();
        let d3 = runtime.get_data_mut().add_number(30.into()).unwrap();
        runtime.get_data_mut().start_list(3).unwrap();
        runtime.get_data_mut().add_to_list(d1, false).unwrap();
        runtime.get_data_mut().add_to_list(d2, false).unwrap();
        runtime.get_data_mut().add_to_list(d3, false).unwrap();
        let d4 = runtime.get_data_mut().end_list().unwrap();
        let d5 = runtime.get_data_mut().add_number(2.into()).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::Access, None).unwrap();

        runtime.get_data_mut().push_register(d4).unwrap();
        runtime.get_data_mut().push_register(d5).unwrap();

        let next = runtime.access::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 30.into());
        assert!(next.is_none())
    }

    #[test]
    fn access_integer_to_char_list() {
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

        let next = runtime.access::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_data_mut().get_register(0).unwrap(), start);
        assert_eq!(runtime.get_data_mut().get_char(start).unwrap(), 'c');
        assert!(next.is_none())
    }

    #[test]
    fn access_integer_to_byte_list() {
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

        let next = runtime.access::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_data_mut().get_register(0).unwrap(), start);
        assert_eq!(runtime.get_data_mut().get_byte(start).unwrap(), 30.into());
        assert!(next.is_none())
    }

    #[test]
    fn range_with_integer() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i2 = runtime.get_data_mut().add_number(20.into()).unwrap();
        let i3 = runtime.get_data_mut().add_range(i1, i2).unwrap();
        let i4 = runtime.get_data_mut().add_number(5.into()).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::Access, None).unwrap();

        runtime.get_data_mut().push_register(i3).unwrap();
        runtime.get_data_mut().push_register(i4).unwrap();

        runtime.access::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 15.into());
    }

    #[test]
    fn access_symbol_to_list() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime
            .get_data_mut()
            .add_symbol(SimpleDataRuntimeNC::parse_symbol("val1").unwrap())
            .unwrap();
        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i3 = runtime.get_data_mut().add_pair((i1, i2)).unwrap();

        let i4 = runtime
            .get_data_mut()
            .add_symbol(SimpleDataRuntimeNC::parse_symbol("val2").unwrap())
            .unwrap();
        let i5 = runtime.get_data_mut().add_number(20.into()).unwrap();
        let i6 = runtime.get_data_mut().add_pair((i4, i5)).unwrap();

        let i7 = runtime
            .get_data_mut()
            .add_symbol(SimpleDataRuntimeNC::parse_symbol("val3").unwrap())
            .unwrap();
        let i8 = runtime.get_data_mut().add_number(30.into()).unwrap();
        let i9 = runtime.get_data_mut().add_pair((i7, i8)).unwrap();

        runtime.get_data_mut().start_list(3).unwrap();
        runtime.get_data_mut().add_to_list(i3, true).unwrap();
        runtime.get_data_mut().add_to_list(i6, true).unwrap();
        runtime.get_data_mut().add_to_list(i9, true).unwrap();
        let i10 = runtime.get_data_mut().end_list().unwrap();

        let i11 = runtime
            .get_data_mut()
            .add_symbol(SimpleDataRuntimeNC::parse_symbol("val2").unwrap())
            .unwrap();

        runtime.get_data_mut().push_register(i10).unwrap();
        runtime.get_data_mut().push_register(i11).unwrap();

        runtime.access::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 20.into());
    }

    #[test]
    fn access_with_unsupported_left_is_unit() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let int2 = runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();
        runtime.access::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_data_mut().get_register(0).unwrap(), 0usize);
    }

    #[test]
    fn access_no_references_is_err() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().add_expression(0).unwrap();
        runtime.get_data_mut().add_number(20.into()).unwrap();

        // 1
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::PutValue, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::EndExpression, None).unwrap();

        // 5
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(2)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(3)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Apply, None).unwrap();

        runtime.get_data_mut().push_jump_point(1).unwrap();

        runtime.get_data_mut().set_instruction_cursor(7).unwrap();

        let result = runtime.access::<EmptyContext>(None);

        assert!(result.is_err());
    }
}