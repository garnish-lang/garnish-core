#[cfg(test)]
mod deferring {
    use crate::simple::testing_utilities::{DeferOpTestContext, create_simple_runtime};
    use garnish_lang::{GarnishData, GarnishDataType, GarnishRuntime};

    #[test]
    fn resolve() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();

        let mut context = DeferOpTestContext::new();

        runtime.resolve(int1).unwrap();

        // resolve never passes to defer make sure default unit is place when not resolved
        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::Unit);
    }
}

#[cfg(test)]
mod tests {
    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang::{ GarnishData, GarnishDataType, GarnishRuntime, Instruction};

    #[allow(const_item_mutation)]
    #[test]
    fn resolve_non_symbol() {
        let mut runtime = create_simple_runtime();

        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();

        runtime.resolve(i2).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::Unit);
    }

    #[test]
    fn resolve_from_input() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_symbol(1).unwrap();
        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i3 = runtime.get_data_mut().add_pair((i1, i2)).unwrap();
        let l1 = runtime.get_data_mut().start_list(1).unwrap();
        let l1 = runtime.get_data_mut().add_to_list(l1, i3).unwrap();
        let i4 = runtime.get_data_mut().end_list(l1).unwrap();
        let i5 = runtime.get_data_mut().add_symbol(1).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::Resolve, None).unwrap();

        runtime.get_data_mut().push_value_stack(i4).unwrap();

        runtime.resolve(i5).unwrap();

        assert_eq!(runtime.get_data_mut().get_register(0).unwrap(), i2);
    }

    #[test]
    fn resolve_not_found_is_unit() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_symbol(1).unwrap();
        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i3 = runtime.get_data_mut().add_pair((i1, i2)).unwrap();
        let l1 = runtime.get_data_mut().start_list(1).unwrap();
        let l1 = runtime.get_data_mut().add_to_list(l1, i3).unwrap();
        let _i4 = runtime.get_data_mut().end_list(l1).unwrap();
        let i5 = runtime.get_data_mut().add_symbol(2).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::Resolve, None).unwrap();

        runtime.resolve(i5).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::Unit);
    }
}
