#[cfg(test)]
mod tests {

    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang_traits::{GarnishData, GarnishRuntime};

    #[test]
    fn range() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let d2 = runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.make_range().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        let (start, end) = runtime.get_data_mut().get_range(i).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(start).unwrap(), 10.into());
        assert_eq!(runtime.get_data_mut().get_number(end).unwrap(), 20.into());
    }

    #[test]
    fn start_exclusive() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let d2 = runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.make_start_exclusive_range().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        let (start, end) = runtime.get_data_mut().get_range(i).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(start).unwrap(), 11.into());
        assert_eq!(runtime.get_data_mut().get_number(end).unwrap(), 20.into());
    }

    #[test]
    fn end_exclusive() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let d2 = runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.make_end_exclusive_range().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        let (start, end) = runtime.get_data_mut().get_range(i).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(start).unwrap(), 10.into());
        assert_eq!(runtime.get_data_mut().get_number(end).unwrap(), 19.into());
    }

    #[test]
    fn exclusive() {
        let mut runtime = create_simple_runtime();

        let d1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let d2 = runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.make_exclusive_range().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        let (start, end) = runtime.get_data_mut().get_range(i).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(start).unwrap(), 11.into());
        assert_eq!(runtime.get_data_mut().get_number(end).unwrap(), 19.into());
    }
}
