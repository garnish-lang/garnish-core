#[cfg(test)]
mod tests {

    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang_traits::{GarnishDataType, GarnishData, GarnishRuntime, Instruction};

    #[test]
    fn put() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().add_number(10.into()).unwrap();
        let max = runtime.get_data_mut().get_data_len();
        runtime.get_data_mut().set_end_of_constant(max).unwrap();

        runtime.put(1).unwrap();

        assert_eq!(runtime.get_data_mut().get_register(0).unwrap(), 1);
    }

    #[test]
    fn put_outside_of_constant_data() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().add_number(10.into()).unwrap();

        let result = runtime.put(10);

        assert!(result.is_err());
    }

    #[test]
    fn put_input() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.get_data_mut().push_value_stack(2).unwrap();

        runtime.put_value().unwrap();

        assert_eq!(runtime.get_data_mut().get_register(0).unwrap(), 2);
    }

    #[test]
    fn put_input_is_unit_if_no_input() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.put_value().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::Unit);
    }

    #[test]
    fn push_input() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.get_data_mut().push_register(2).unwrap();

        runtime.push_value().unwrap();

        assert_eq!(runtime.get_data_mut().get_value_stack_len(), 1);
        assert_eq!(runtime.get_data_mut().get_value(0).unwrap(), 2usize);
        assert_eq!(runtime.get_data_mut().get_current_value().unwrap(), 2usize);
    }

    #[test]
    fn push_input_no_register_is_err() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().add_number(20.into()).unwrap();

        assert!(runtime.push_value().is_err());
    }

    #[test]
    fn push_result() {
        let mut runtime = create_simple_runtime();

        let i1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::UpdateValue, None).unwrap();

        runtime.get_data_mut().push_register(i1).unwrap();

        runtime.get_data_mut().push_value_stack(i1).unwrap();

        runtime.update_value().unwrap();

        let i = runtime.get_data_mut().get_current_value().unwrap();
        assert_eq!(runtime.get_data_mut().get_value_stack_len(), 1);
        assert_eq!(runtime.get_data_mut().get_current_value().unwrap(), i1);
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 10.into());
    }

    #[test]
    fn push_result_no_register_is_err() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::UpdateValue, None).unwrap();

        assert!(runtime.update_value().is_err());
    }
}
