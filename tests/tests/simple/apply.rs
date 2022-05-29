#[cfg(test)]
mod deferring {
    use crate::simple::testing_utilities::{create_simple_runtime, DeferOpTestContext, DEFERRED_VALUE};
    use garnish_traits::{GarnishLangRuntimeData, GarnishRuntime};

    #[test]
    fn apply() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let int2 = runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        let mut context = DeferOpTestContext::new();

        runtime.apply(Some(&mut context)).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_external(i).unwrap(), DEFERRED_VALUE);
    }

    #[test]
    fn empty_apply() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();

        let mut context = DeferOpTestContext::new();

        runtime.empty_apply(Some(&mut context)).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_external(i).unwrap(), DEFERRED_VALUE);
    }
}

#[cfg(test)]
mod tests {
    use garnish_data::{DataError, SimpleDataRuntimeNC, SimpleRuntimeData, symbol_value};
    use crate::simple::testing_utilities::{create_simple_runtime, DeferOpTestContext, DEFERRED_VALUE};
    use garnish_traits::{
        EmptyContext, ExpressionDataType, GarnishLangRuntimeContext, GarnishLangRuntimeData, GarnishRuntime, Instruction, RuntimeError,
    };

    #[test]
    fn deferred() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let int2 = runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        let mut context = DeferOpTestContext::new();
        runtime.apply(Some(&mut context)).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_external(i).unwrap(), DEFERRED_VALUE);
    }

    #[test]
    fn apply() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let exp1 = runtime.get_data_mut().add_expression(0).unwrap();
        let int2 = runtime.get_data_mut().add_number(20.into()).unwrap();

        // 1
        let i1 = runtime.get_data_mut().push_instruction(Instruction::Put, Some(int1)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::PutValue, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::EndExpression, None).unwrap();

        // 5
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(exp1)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(int2)).unwrap();
        let i2 = runtime.get_data_mut().push_instruction(Instruction::Apply, None).unwrap();
        let i3 = runtime.get_data_mut().push_instruction(Instruction::EndExpression, None).unwrap();

        runtime.get_data_mut().push_jump_point(i1).unwrap();

        runtime.get_data_mut().push_register(exp1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.get_data_mut().set_instruction_cursor(i2).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_data_mut().get_value(0).unwrap(), int2);
        assert_eq!(runtime.get_data_mut().get_instruction_cursor(), i1);
        assert_eq!(runtime.get_data_mut().get_jump_path(0).unwrap(), i3);
    }

    #[test]
    fn apply_integer_to_list() {
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

        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        let i1 = runtime.get_data_mut().push_instruction(Instruction::Apply, None).unwrap();
        let i2 = runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();

        runtime.get_data_mut().set_instruction_cursor(i1).unwrap();

        runtime.get_data_mut().push_register(d4).unwrap();
        runtime.get_data_mut().push_register(d5).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 30.into());
        assert_eq!(runtime.get_data_mut().get_instruction_cursor(), i2);
    }

    #[test]
    fn apply_integer_to_char_list() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().start_char_list().unwrap();
        runtime.get_data_mut().add_to_char_list('a').unwrap();
        runtime.get_data_mut().add_to_char_list('b').unwrap();
        runtime.get_data_mut().add_to_char_list('c').unwrap();
        let d1 = runtime.get_data_mut().end_char_list().unwrap();
        let d2 = runtime.get_data_mut().add_number(2.into()).unwrap();
        let start = runtime.get_data_mut().get_data_len();

        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        let i1 = runtime.get_data_mut().push_instruction(Instruction::Apply, None).unwrap();
        let i2 = runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();

        runtime.get_data_mut().set_instruction_cursor(i1).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_data_mut().get_register(0).unwrap(), start);
        assert_eq!(runtime.get_data_mut().get_char(start).unwrap(), 'c');
        assert_eq!(runtime.get_data_mut().get_instruction_cursor(), i2);
    }

    #[test]
    fn apply_integer_to_byte_list() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().start_byte_list().unwrap();
        runtime.get_data_mut().add_to_byte_list(10).unwrap();
        runtime.get_data_mut().add_to_byte_list(20).unwrap();
        runtime.get_data_mut().add_to_byte_list(30).unwrap();
        let d1 = runtime.get_data_mut().end_byte_list().unwrap();
        let d2 = runtime.get_data_mut().add_number(2.into()).unwrap();
        let start = runtime.get_data_mut().get_data_len();

        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        let i1 = runtime.get_data_mut().push_instruction(Instruction::Apply, None).unwrap();
        let i2 = runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();

        runtime.get_data_mut().set_instruction_cursor(i1).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_data_mut().get_register(0).unwrap(), start);
        assert_eq!(runtime.get_data_mut().get_byte(start).unwrap(), 30.into());
        assert_eq!(runtime.get_data_mut().get_instruction_cursor(), i2);
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

        runtime.apply::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 15.into());
    }

    #[test]
    fn apply_symbol_to_list() {
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

        runtime.apply::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 20.into());
    }

    #[test]
    fn apply_list_of_sym_or_integer_to_list() {
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

        let i11 = runtime.get_data_mut().add_number(2.into()).unwrap(); // integer access
        let i12 = runtime
            .get_data_mut()
            .add_symbol(SimpleDataRuntimeNC::parse_symbol("val2").unwrap())
            .unwrap(); // symbol access
        let i13 = runtime.get_data_mut().add_number(5.into()).unwrap(); // access out of bounds
        let i14 = runtime.get_data_mut().add_expression(10).unwrap(); // invalid access type

        let i15 = runtime
            .get_data_mut()
            .add_symbol(SimpleDataRuntimeNC::parse_symbol("new_key").unwrap())
            .unwrap();
        let i16 = runtime
            .get_data_mut()
            .add_symbol(SimpleDataRuntimeNC::parse_symbol("val1").unwrap())
            .unwrap();
        let i17 = runtime.get_data_mut().add_pair((i15, i16)).unwrap(); // pair mapping
        runtime.get_data_mut().start_list(2).unwrap();
        runtime.get_data_mut().add_to_list(i11, false).unwrap();
        runtime.get_data_mut().add_to_list(i12, false).unwrap();
        runtime.get_data_mut().add_to_list(i13, false).unwrap();
        runtime.get_data_mut().add_to_list(i14, false).unwrap();
        runtime.get_data_mut().add_to_list(i17, true).unwrap();
        let i18 = runtime.get_data_mut().end_list().unwrap();

        runtime.get_data_mut().push_register(i10).unwrap();
        runtime.get_data_mut().push_register(i18).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        let addr = runtime.get_data_mut().pop_register().unwrap();
        let len = runtime.get_data_mut().get_list_len(addr).unwrap();
        let association_len = runtime.get_data_mut().get_list_associations_len(addr).unwrap();
        assert_eq!(len, 5);
        assert_eq!(association_len, 2);

        let pair_addr = runtime.get_data_mut().get_list_item(addr, 0.into()).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(pair_addr).unwrap(), ExpressionDataType::Pair);

        let (pair_left, pair_right) = runtime.get_data_mut().get_pair(pair_addr).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(pair_left).unwrap(), ExpressionDataType::Symbol);
        assert_eq!(runtime.get_data_mut().get_symbol(pair_left).unwrap(), symbol_value("val3"));
        assert_eq!(runtime.get_data_mut().get_data_type(pair_right).unwrap(), ExpressionDataType::Number);
        assert_eq!(runtime.get_data_mut().get_number(pair_right).unwrap(), 30.into());

        let association_value1 = runtime
            .get_data_mut()
            .get_list_item_with_symbol(addr, symbol_value("val3"))
            .unwrap()
            .unwrap();
        assert_eq!(runtime.get_data_mut().get_number(association_value1).unwrap(), 30.into());

        let int_addr = runtime.get_data_mut().get_list_item(addr, 1.into()).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(int_addr).unwrap(), ExpressionDataType::Number);
        assert_eq!(runtime.get_data_mut().get_number(int_addr).unwrap(), 20.into());

        let unit1 = runtime.get_data_mut().get_list_item(addr, 2.into()).unwrap();
        let unit2 = runtime.get_data_mut().get_list_item(addr, 3.into()).unwrap();

        assert_eq!(runtime.get_data_mut().get_data_type(unit1).unwrap(), ExpressionDataType::Unit);
        assert_eq!(runtime.get_data_mut().get_data_type(unit2).unwrap(), ExpressionDataType::Unit);

        let map_pair_addr = runtime.get_data_mut().get_list_item(addr, 4.into()).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(map_pair_addr).unwrap(), ExpressionDataType::Pair);

        let (map_pair_left, map_pair_right) = runtime.get_data_mut().get_pair(map_pair_addr).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(map_pair_left).unwrap(), ExpressionDataType::Symbol);
        assert_eq!(runtime.get_data_mut().get_symbol(map_pair_left).unwrap(), symbol_value("new_key"));
        assert_eq!(runtime.get_data_mut().get_data_type(map_pair_right).unwrap(), ExpressionDataType::Number);
        assert_eq!(runtime.get_data_mut().get_number(map_pair_right).unwrap(), 10.into());
    }

    #[test]
    fn apply_with_unsupported_left_is_unit() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let int2 = runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_data_mut().get_register(0).unwrap(), 0usize);
    }

    #[test]
    fn apply_no_references_is_err() {
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

        let result = runtime.apply::<EmptyContext>(None);

        assert!(result.is_err());
    }

    #[test]
    fn empty_apply() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let exp1 = runtime.get_data_mut().add_expression(0).unwrap();

        // 1
        let i1 = runtime.get_data_mut().push_instruction(Instruction::Put, Some(int1)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::PutValue, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::EndExpression, None).unwrap();

        // 5
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(exp1)).unwrap();
        let i2 = runtime.get_data_mut().push_instruction(Instruction::EmptyApply, None).unwrap();
        let i3 = runtime.get_data_mut().push_instruction(Instruction::EndExpression, None).unwrap();

        runtime.get_data_mut().push_jump_point(i1).unwrap();

        runtime.get_data_mut().push_register(exp1).unwrap();

        runtime.get_data_mut().set_instruction_cursor(i2).unwrap();

        runtime.empty_apply::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_value(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::Unit);
        assert_eq!(runtime.get_data_mut().get_instruction_cursor(), i1);
        assert_eq!(runtime.get_data_mut().get_jump_path(0).unwrap(), i3);
    }

    #[test]
    fn empty_apply_no_references_is_err() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().add_expression(0).unwrap();

        // 1
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::PutValue, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::EndExpression, None).unwrap();

        // 5
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(2)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::EmptyApply, None).unwrap();

        runtime.get_data_mut().push_jump_point(1).unwrap();

        runtime.get_data_mut().set_instruction_cursor(6).unwrap();

        let result = runtime.empty_apply::<EmptyContext>(None);

        assert!(result.is_err());
    }

    #[test]
    fn reapply_if_true() {
        let mut runtime = create_simple_runtime();

        let true1 = runtime.get_data_mut().add_true().unwrap();
        let _exp1 = runtime.get_data_mut().add_expression(0).unwrap();
        let int1 = runtime.get_data_mut().add_number(20.into()).unwrap();
        let _int2 = runtime.get_data_mut().add_number(30.into()).unwrap();
        let int3 = runtime.get_data_mut().add_number(40.into()).unwrap();

        // 1
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(2)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Apply, None).unwrap();

        // 4
        let i1 = runtime.get_data_mut().push_instruction(Instruction::Put, Some(0)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::PutValue, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::PutValue, None).unwrap();
        let i2 = runtime.get_data_mut().push_instruction(Instruction::Reapply, Some(0)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::EndExpression, None).unwrap();

        runtime.get_data_mut().push_jump_point(i1).unwrap();

        runtime.get_data_mut().push_register(true1).unwrap();
        runtime.get_data_mut().push_register(int3).unwrap();

        runtime.get_data_mut().push_value_stack(int1).unwrap();

        runtime.get_data_mut().set_instruction_cursor(i2).unwrap();

        runtime.reapply(0).unwrap();

        assert_eq!(runtime.get_data_mut().get_value_stack_len(), 1);
        assert_eq!(runtime.get_data_mut().get_value(0).unwrap(), int3);
        assert_eq!(runtime.get_data_mut().get_instruction_cursor(), i1);
    }

    #[test]
    fn reapply_if_false() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().add_false().unwrap();
        runtime.get_data_mut().add_expression(0).unwrap();
        runtime.get_data_mut().add_number(20.into()).unwrap();
        runtime.get_data_mut().add_number(30.into()).unwrap();
        runtime.get_data_mut().add_number(40.into()).unwrap();

        // 1
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(2)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Apply, None).unwrap();

        // 4
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(0)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::PutValue, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::PutValue, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Reapply, Some(0)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::EndExpression, None).unwrap();

        runtime.get_data_mut().push_jump_point(4).unwrap();

        runtime.get_data_mut().push_register(1).unwrap();
        runtime.get_data_mut().push_register(4).unwrap();

        runtime.get_data_mut().push_value_stack(2).unwrap();
        runtime.get_data_mut().push_jump_path(9).unwrap();

        runtime.get_data_mut().set_instruction_cursor(8).unwrap();

        runtime.reapply(0).unwrap();

        assert_eq!(runtime.get_data_mut().get_value_stack_len(), 1);
        assert_eq!(runtime.get_data_mut().get_value(0).unwrap(), 2);
        assert_eq!(runtime.get_data_mut().get_instruction_cursor(), 8);
        assert_eq!(runtime.get_data_mut().get_jump_path(0).unwrap(), 9);
    }

    #[test]
    fn apply_from_context() {
        let mut runtime = create_simple_runtime();

        let ext1 = runtime.get_data_mut().add_external(3).unwrap();
        let int1 = runtime.get_data_mut().add_number(100.into()).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        let i1 = runtime.get_data_mut().push_instruction(Instruction::Apply, None).unwrap();
        let i2 = runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();

        runtime.get_data_mut().push_register(ext1).unwrap();
        runtime.get_data_mut().push_register(int1).unwrap();

        runtime.get_data_mut().set_instruction_cursor(i1).unwrap();

        struct MyContext {
            new_addr: usize,
        }

        impl GarnishLangRuntimeContext<SimpleRuntimeData> for MyContext {
            fn resolve(&mut self, _: u64, _: &mut SimpleRuntimeData) -> Result<bool, RuntimeError<DataError>> {
                Ok(false)
            }

            fn apply(&mut self, external_value: usize, input_addr: usize, data: &mut SimpleRuntimeData) -> Result<bool, RuntimeError<DataError>> {
                assert_eq!(external_value, 3);

                let value = match data.get_data_type(input_addr)? {
                    ExpressionDataType::Number => data.get_number(input_addr)?,
                    _ => return Ok(false),
                };

                self.new_addr = data.add_number(value * 2.into())?;
                data.push_register(self.new_addr)?;

                Ok(true)
            }
        }

        let mut context = MyContext { new_addr: 0 };

        runtime.apply(Some(&mut context)).unwrap();

        assert_eq!(runtime.get_data_mut().get_number(context.new_addr).unwrap(), 200.into());
        assert_eq!(runtime.get_data_mut().get_register(0).unwrap(), context.new_addr);
        assert_eq!(runtime.get_data_mut().get_instruction_cursor(), i2);
    }
}

#[cfg(test)]
mod slices {
    use crate::simple::testing_utilities::{add_concatenation_with_start, add_list, add_range, create_simple_runtime};
    use garnish_traits::{EmptyContext, GarnishLangRuntimeData, GarnishRuntime};

    #[test]
    fn create_with_list() {
        let mut runtime = create_simple_runtime();

        let d1 = add_list(runtime.get_data_mut(), 10);
        let d2 = add_range(runtime.get_data_mut(), 1, 5);

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        let (list, range) = runtime.get_data_mut().get_slice(i).unwrap();
        assert_eq!(list, d1);
        assert_eq!(range, d2);
    }

    #[test]
    fn create_with_concatenation() {
        let mut runtime = create_simple_runtime();

        let d1 = add_concatenation_with_start(runtime.get_data_mut(), 10, 10);
        let d2 = add_range(runtime.get_data_mut(), 1, 5);

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        let (list, range) = runtime.get_data_mut().get_slice(i).unwrap();
        assert_eq!(list, d1);
        assert_eq!(range, d2);
    }

    #[test]
    fn create_with_slice() {
        let mut runtime = create_simple_runtime();

        let d1 = add_list(runtime.get_data_mut(), 10);
        let d2 = add_range(runtime.get_data_mut(), 1, 8);
        let d3 = runtime.get_data_mut().add_slice(d1, d2).unwrap();
        let d4 = add_range(runtime.get_data_mut(), 2, 4);

        runtime.get_data_mut().push_register(d3).unwrap();
        runtime.get_data_mut().push_register(d4).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        let (list, range) = runtime.get_data_mut().get_slice(i).unwrap();
        assert_eq!(list, d1);

        let (start, end) = runtime.get_data_mut().get_range(range).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(start).unwrap(), 3.into());
        assert_eq!(runtime.get_data_mut().get_number(end).unwrap(), 5.into());
    }

    #[test]
    fn create_with_char_list() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().start_char_list().unwrap();
        runtime.get_data_mut().add_to_char_list('a').unwrap();
        runtime.get_data_mut().add_to_char_list('b').unwrap();
        runtime.get_data_mut().add_to_char_list('c').unwrap();
        let d1 = runtime.get_data_mut().end_char_list().unwrap();
        let d2 = add_range(runtime.get_data_mut(), 1, 5);

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        let (list, range) = runtime.get_data_mut().get_slice(i).unwrap();
        assert_eq!(list, d1);
        assert_eq!(range, d2);
    }

    #[test]
    fn create_with_byte_list() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().start_byte_list().unwrap();
        runtime.get_data_mut().add_to_byte_list(10).unwrap();
        runtime.get_data_mut().add_to_byte_list(20).unwrap();
        runtime.get_data_mut().add_to_byte_list(30).unwrap();
        let d1 = runtime.get_data_mut().end_byte_list().unwrap();
        let d2 = add_range(runtime.get_data_mut(), 1, 5);

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        let (list, range) = runtime.get_data_mut().get_slice(i).unwrap();
        assert_eq!(list, d1);
        assert_eq!(range, d2);
    }

    #[test]
    fn apply_range_to_range_narrows_it() {
        let mut runtime = create_simple_runtime();

        let d1 = add_range(runtime.get_data_mut(), 5, 15);
        let d2 = add_range(runtime.get_data_mut(), 1, 9);

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        let (start, end) = runtime.get_data_mut().get_range(i).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(start).unwrap(), 6.into());
        assert_eq!(runtime.get_data_mut().get_number(end).unwrap(), 14.into());
    }

    #[test]
    fn create_with_link() {
        let mut runtime = create_simple_runtime();

        let unit = runtime.get_data_mut().add_unit().unwrap();
        let d1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let d2 = runtime.get_data_mut().add_link(d1, unit, true).unwrap();

        let d3 = add_range(runtime.get_data_mut(), 1, 5);

        runtime.get_data_mut().push_register(d2).unwrap();
        runtime.get_data_mut().push_register(d3).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        let (list, range) = runtime.get_data_mut().get_slice(i).unwrap();
        assert_eq!(list, d2);
        assert_eq!(range, d3);
    }
}
