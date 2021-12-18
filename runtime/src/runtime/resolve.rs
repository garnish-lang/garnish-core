#[cfg(test)]
mod tests {
    use crate::{
        error,
        runtime::{
            context::{EmptyContext, GarnishLangRuntimeContext},
            utilities::push_integer,
            GarnishRuntime,
        },
        ExpressionData, ExpressionDataType, GarnishLangRuntimeData, GarnishLangRuntimeResult, Instruction, NestInto, SimpleRuntimeData,
    };

    #[test]
    fn resolve_no_ref_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();
        runtime.add_data(ExpressionData::list(vec![3], vec![3])).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();

        runtime.push_instruction(Instruction::Resolve, None).unwrap();

        runtime.set_result(Some(4)).unwrap();

        let result = runtime.resolve::<EmptyContext>(None);

        assert!(result.is_err());
    }

    #[test]
    fn resolve_from_result() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();
        runtime.add_data(ExpressionData::list(vec![3], vec![3])).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();

        runtime.push_instruction(Instruction::Resolve, None).unwrap();

        runtime.push_register(5).unwrap();

        runtime.set_result(Some(4)).unwrap();

        runtime.resolve::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_register().get(0).unwrap(), &2);
    }

    #[test]
    fn resolve_from_input() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();
        runtime.add_data(ExpressionData::list(vec![3], vec![3])).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();

        runtime.push_instruction(Instruction::Resolve, None).unwrap();

        runtime.push_register(5).unwrap();

        runtime.push_input_stack(4).unwrap();

        runtime.resolve::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_register().get(0).unwrap(), &2);
    }

    #[test]
    fn resolve_not_found_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();
        runtime.add_data(ExpressionData::list(vec![3], vec![3])).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"two".to_string())).unwrap();

        runtime.push_instruction(Instruction::Resolve, None).unwrap();

        runtime.push_register(5).unwrap();

        runtime.resolve::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_data_type(6).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn resolve_from_context() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();

        runtime.push_instruction(Instruction::Resolve, None).unwrap();

        runtime.push_register(1).unwrap();

        struct MyContext {}

        impl GarnishLangRuntimeContext<SimpleRuntimeData> for MyContext {
            fn resolve(&mut self, sym_addr: usize, runtime: &mut SimpleRuntimeData) -> GarnishLangRuntimeResult<String, bool> {
                match runtime.get_data_type(sym_addr).nest_into()? {
                    ExpressionDataType::Symbol => {
                        push_integer(runtime, 100)?;
                        Ok(true)
                    }
                    t => Err(error(format!("Address given to resolve is of type {:?}. Expected symbol type.", t)))?,
                }
            }

            fn apply(&mut self, _: usize, _: usize, _: &mut SimpleRuntimeData) -> GarnishLangRuntimeResult<String, bool> {
                Ok(false)
            }
        }

        let mut context = MyContext {};

        runtime.resolve(Some(&mut context)).unwrap();

        assert_eq!(runtime.get_register().get(0).unwrap(), &2);
    }
}
