#[cfg(test)]
mod deferring {

    use crate::simple::testing_utilities::{create_simple_runtime, DeferOpTestContext};
    use garnish_traits::{ExpressionDataType, GarnishLangRuntimeData, GarnishRuntime};

    #[test]
    fn resolve() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();

        let mut context = DeferOpTestContext::new();

        runtime.resolve(int1, Some(&mut context)).unwrap();

        // resolve never passes to defer make sure default unit is place when not resolved
        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::Unit);
    }
}

#[cfg(test)]
mod tests {
    use garnish_data::{DataError, SimpleRuntimeData};

    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_traits::{
        EmptyContext, ExpressionDataType, GarnishLangRuntimeContext, GarnishLangRuntimeData, GarnishRuntime, Instruction, RuntimeError, EMPTY_CONTEXT,
    };

    #[allow(const_item_mutation)]
    #[test]
    fn resolve_non_symbol() {
        let mut runtime = create_simple_runtime();

        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();

        runtime.resolve(i2, Some(&mut EMPTY_CONTEXT)).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn resolve_from_input() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_symbol(1).unwrap();
        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i3 = runtime.get_data_mut().add_pair((i1, i2)).unwrap();
        runtime.get_data_mut().start_list(1).unwrap();
        runtime.get_data_mut().add_to_list(i3, true).unwrap();
        let i4 = runtime.get_data_mut().end_list().unwrap();
        let i5 = runtime.get_data_mut().add_symbol(1).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::Resolve, None).unwrap();

        runtime.get_data_mut().push_value_stack(i4).unwrap();

        runtime.resolve::<EmptyContext>(i5, None).unwrap();

        assert_eq!(runtime.get_data_mut().get_register(0).unwrap(), i2);
    }

    #[test]
    fn resolve_not_found_is_unit() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_symbol(1).unwrap();
        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i3 = runtime.get_data_mut().add_pair((i1, i2)).unwrap();
        runtime.get_data_mut().start_list(1).unwrap();
        runtime.get_data_mut().add_to_list(i3, true).unwrap();
        let _i4 = runtime.get_data_mut().end_list().unwrap();
        let i5 = runtime.get_data_mut().add_symbol(2).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::Resolve, None).unwrap();

        runtime.resolve::<EmptyContext>(i5, None).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn resolve_from_context() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_symbol(1).unwrap();
        let start = runtime.get_data_mut().get_data_len();

        runtime.get_data_mut().push_instruction(Instruction::Resolve, None).unwrap();

        struct MyContext {}

        impl GarnishLangRuntimeContext<SimpleRuntimeData> for MyContext {
            fn resolve(&mut self, sym_val: u64, runtime: &mut SimpleRuntimeData) -> Result<bool, RuntimeError<DataError>> {
                assert_eq!(sym_val, 1);

                let addr = runtime.add_number(100.into())?;
                runtime.push_register(addr)?;
                Ok(true)
            }

            fn apply(&mut self, _: usize, _: usize, _: &mut SimpleRuntimeData) -> Result<bool, RuntimeError<DataError>> {
                Ok(false)
            }
        }

        let mut context = MyContext {};

        runtime.resolve(i1, Some(&mut context)).unwrap();

        assert_eq!(runtime.get_data_mut().get_register(0).unwrap(), start);
    }
}
