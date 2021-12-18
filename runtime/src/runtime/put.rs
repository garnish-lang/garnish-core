#[cfg(test)]
mod tests {
    use crate::{runtime::GarnishRuntime, ExpressionData, ExpressionDataType, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

    #[test]
    fn put() {
        let mut runtime = SimpleRuntimeData::new();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.set_end_of_constant(runtime.get_data_len()).unwrap();

        runtime.put(1).unwrap();

        assert_eq!(*runtime.get_register().get(0).unwrap(), 1);
    }

    #[test]
    fn put_outside_of_constant_data() {
        let mut runtime = SimpleRuntimeData::new();
        runtime.add_data(ExpressionData::integer(10)).unwrap();

        let result = runtime.put(0);

        assert!(result.is_err());
    }

    #[test]
    fn put_input() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.push_input_stack(2).unwrap();

        runtime.put_input().unwrap();

        assert_eq!(*runtime.get_register().get(0).unwrap(), 2);
    }

    #[test]
    fn put_input_is_unit_if_no_input() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.put_input().unwrap();

        assert_eq!(runtime.get_data_type(3).unwrap(), ExpressionDataType::Unit);
        assert_eq!(*runtime.get_register().get(0).unwrap(), 3);
    }

    #[test]
    fn push_input() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.push_register(2).unwrap();

        runtime.push_input().unwrap();

        assert_eq!(runtime.get_input(0).unwrap(), 2usize);
        assert_eq!(runtime.get_result().unwrap(), 2usize);
    }

    #[test]
    fn push_input_no_register_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        assert!(runtime.push_input().is_err());
    }

    #[test]
    fn push_result() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.push_instruction(Instruction::PushResult, None).unwrap();

        runtime.push_register(1).unwrap();

        runtime.push_result().unwrap();

        assert_eq!(runtime.get_result().unwrap(), 1usize);
        assert_eq!(runtime.get_integer(runtime.get_result().unwrap()).unwrap(), 10i64);
    }

    #[test]
    fn push_result_no_register_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.push_instruction(Instruction::PushResult, None).unwrap();

        assert!(runtime.push_result().is_err());
    }

    #[test]
    fn put_result() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.set_result(Some(2)).unwrap();

        runtime.put_result().unwrap();

        assert_eq!(*runtime.get_register().get(0).unwrap(), 2);
    }

    #[test]
    fn put_result_is_unit_if_no_result() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.put_result().unwrap();

        assert_eq!(runtime.get_data_type(3).unwrap(), ExpressionDataType::Unit);
        assert_eq!(*runtime.get_register().get(0).unwrap(), 3);
    }
}
