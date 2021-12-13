use log::trace;

use crate::{ExpressionData, GarnishLangRuntime, GarnishLangRuntimeResult};

use super::context::GarnishLangRuntimeContext;

impl GarnishLangRuntime {
    pub fn resolve<T: GarnishLangRuntimeContext>(&mut self, context: Option<&mut T>) -> GarnishLangRuntimeResult {
        trace!("Instruction - Resolve");
        let addr = self.next_raw_ref()?;

        // check result
        match self.current_result {
            None => (),
            Some(result_ref) => match self.get_access_addr(addr, result_ref)? {
                None => (),
                Some(i) => {
                    self.add_reference_data(i)?;
                    return Ok(());
                }
            },
        }

        // check input
        match self.inputs.last() {
            None => (),
            Some(list_ref) => match self.get_access_addr(addr, *list_ref)? {
                None => (),
                Some(i) => {
                    self.add_reference_data(i)?;
                    return Ok(());
                }
            },
        }

        // check context
        match context {
            None => (),
            Some(c) => match c.resolve(addr, self)? {
                true => return Ok(()), // context resovled end look up
                false => (),           // not resolved fall through
            },
        }

        // default to unit
        self.add_data_ref(ExpressionData::unit())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        error,
        runtime::context::{EmptyContext, GarnishLangRuntimeContext},
        ExpressionData, ExpressionDataType, GarnishLangRuntime, GarnishLangRuntimeResult, Instruction,
    };

    #[test]
    fn resolve_no_ref_is_err() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();
        runtime.add_data(ExpressionData::list(vec![3], vec![3])).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();

        runtime.add_instruction(Instruction::Resolve, None).unwrap();

        runtime.current_result = Some(4);

        let result = runtime.resolve::<EmptyContext>(None);

        assert!(result.is_err());
    }

    #[test]
    fn resolve_from_result() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();
        runtime.add_data(ExpressionData::list(vec![3], vec![3])).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();

        runtime.add_instruction(Instruction::Resolve, None).unwrap();

        runtime.reference_stack.push(5);

        runtime.current_result = Some(4);

        runtime.resolve::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_data(6).unwrap().as_reference().unwrap(), 2);
    }

    #[test]
    fn resolve_from_input() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();
        runtime.add_data(ExpressionData::list(vec![3], vec![3])).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();

        runtime.add_instruction(Instruction::Resolve, None).unwrap();

        runtime.reference_stack.push(5);

        runtime.add_input_reference(4).unwrap();

        runtime.resolve::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_data(6).unwrap().as_reference().unwrap(), 2);
    }

    #[test]
    fn resolve_not_found_is_unit() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();
        runtime.add_data(ExpressionData::list(vec![3], vec![3])).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"two".to_string())).unwrap();

        runtime.add_instruction(Instruction::Resolve, None).unwrap();

        runtime.reference_stack.push(5);

        runtime.resolve::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_data(6).unwrap().get_type(), ExpressionDataType::Unit);
    }

    #[test]
    fn resolve_from_context() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();

        runtime.add_instruction(Instruction::Resolve, None).unwrap();

        runtime.reference_stack.push(1);

        struct MyContext {}

        impl GarnishLangRuntimeContext for MyContext {
            fn resolve(&mut self, sym_addr: usize, runtime: &mut GarnishLangRuntime) -> GarnishLangRuntimeResult<bool> {
                match runtime.get_data(sym_addr) {
                    None => Err(error(format!("Symbol address, {:?}, given to resolve not found in runtime.", sym_addr)))?,
                    Some(data) => match data.get_type() {
                        ExpressionDataType::Symbol => {
                            let addr = runtime.get_data_len();
                            runtime.add_data(ExpressionData::integer(100))?;
                            let raddr = runtime.get_data_len();
                            runtime.add_reference_data(addr)?;
                            runtime.reference_stack.push(raddr);
                            Ok(true)
                        }
                        t => Err(error(format!("Address given to resolve is of type {:?}. Expected symbol type.", t)))?,
                    },
                }
            }

            fn apply(&mut self, _: usize, _: usize, _: &mut GarnishLangRuntime) -> GarnishLangRuntimeResult<bool> {
                Ok(false)
            }
        }

        let mut context = MyContext {};

        runtime.resolve(Some(&mut context)).unwrap();

        assert_eq!(runtime.get_data(2).unwrap().as_integer().unwrap(), 100);
        assert_eq!(runtime.reference_stack.get(0).unwrap(), &3);
        assert_eq!(runtime.get_data(3).unwrap().as_reference().unwrap(), 2);
    }
}
