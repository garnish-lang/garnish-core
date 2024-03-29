#[cfg(test)]
mod tests {

    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang_traits::{GarnishDataType, GarnishData, GarnishRuntime};

    #[test]
    fn start_side_effect() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().add_number(10.into()).unwrap();

        runtime.start_side_effect().unwrap();

        let i = runtime.get_data_mut().get_current_value().unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::Unit);
    }

    #[test]
    fn start_side_effect_with_value() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_number(10.into()).unwrap();

        runtime.get_data_mut().push_value_stack(d1).unwrap();

        runtime.start_side_effect().unwrap();

        assert_eq!(runtime.get_data_mut().get_current_value().unwrap(), d1);
    }

    #[test]
    fn end_side_effect() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_number(10.into()).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_value_stack(d1).unwrap();

        runtime.end_side_effect().unwrap();

        assert_eq!(runtime.get_data_mut().get_register_len(), 0);
        assert_eq!(runtime.get_data_mut().get_current_value(), None);
    }

    #[test]
    fn end_side_effect_no_value_is_err() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_number(10.into()).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();

        let result = runtime.end_side_effect();

        assert!(result.is_err());
    }

    #[test]
    fn end_side_effect_no_register_is_err() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_number(10.into()).unwrap();

        runtime.get_data_mut().push_value_stack(d1).unwrap();

        let result = runtime.end_side_effect();

        assert!(result.is_err());
    }
}
