#[cfg(test)]
mod deferring {
    use crate::simple::testing_utilities::{DEFERRED_VALUE, DeferOpTestContext, create_simple_runtime};
    use garnish_lang::{GarnishData, GarnishRuntime};

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
    use crate::simple::testing_utilities::{DEFERRED_VALUE, DeferOpTestContext, create_simple_runtime};
    use garnish_lang::simple::{DataError, SimpleGarnishData};
    use garnish_lang::{EmptyContext, GarnishContext, GarnishData, GarnishDataType, GarnishRuntime, Instruction, RuntimeError};

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
        runtime.get_data_mut().push_instruction(Instruction::EndExpression, None).unwrap();

        runtime.get_data_mut().push_jump_point(i1).unwrap();

        runtime.get_data_mut().push_register(exp1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.get_data_mut().set_instruction_cursor(i2).unwrap();

        let next = runtime.apply::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_data_mut().get_value(0).unwrap(), int2);
        assert_eq!(next.unwrap(), i1);
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
        runtime.get_data_mut().push_instruction(Instruction::EndExpression, None).unwrap();

        runtime.get_data_mut().push_jump_point(i1).unwrap();

        runtime.get_data_mut().push_register(exp1).unwrap();

        runtime.get_data_mut().set_instruction_cursor(i2).unwrap();

        let next = runtime.empty_apply::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_value(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::Unit);
        assert_eq!(next.unwrap(), i1);
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
    fn reapply() {
        let mut runtime = create_simple_runtime();

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

        runtime.get_data_mut().push_register(int3).unwrap();

        runtime.get_data_mut().push_value_stack(int1).unwrap();

        runtime.get_data_mut().set_instruction_cursor(i2).unwrap();

        let next = runtime.reapply(0).unwrap();

        assert_eq!(runtime.get_data_mut().get_value_stack_len(), 1);
        assert_eq!(runtime.get_data_mut().get_value(0).unwrap(), int3);
        assert_eq!(next.unwrap(), i1);
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

        impl GarnishContext<SimpleGarnishData> for MyContext {
            fn resolve(&mut self, _: u64, _: &mut SimpleGarnishData) -> Result<bool, RuntimeError<DataError>> {
                Ok(false)
            }

            fn apply(&mut self, external_value: usize, input_addr: usize, data: &mut SimpleGarnishData) -> Result<bool, RuntimeError<DataError>> {
                assert_eq!(external_value, 3);

                let value = match data.get_data_type(input_addr)? {
                    GarnishDataType::Number => data.get_number(input_addr)?,
                    _ => return Ok(false),
                };

                self.new_addr = data.add_number(value * 2.into())?;
                data.push_register(self.new_addr)?;

                Ok(true)
            }
        }

        let mut context = MyContext { new_addr: 0 };

        let next = runtime.apply(Some(&mut context)).unwrap();

        assert_eq!(runtime.get_data_mut().get_number(context.new_addr).unwrap(), 200.into());
        assert_eq!(runtime.get_data_mut().get_register(0).unwrap(), context.new_addr);
        assert_eq!(next.unwrap(), i2);
    }
}

#[cfg(test)]
mod slices {
    use crate::simple::testing_utilities::{add_concatenation_with_start, add_list, add_range, create_simple_runtime};
    use garnish_lang::{EmptyContext, GarnishData, GarnishRuntime};

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
}
