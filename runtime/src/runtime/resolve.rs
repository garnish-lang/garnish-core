use log::trace;

use crate::{GarnishLangRuntime, GarnishLangRuntimeResult};

use super::{context::GarnishLangRuntimeContext, data::GarnishLangRuntimeData};

impl<Data> GarnishLangRuntime<Data>
where
    Data: GarnishLangRuntimeData,
{
    pub fn resolve<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - Resolve");
        let addr = self.next_ref()?;

        // check result
        match self.data.get_result() {
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
        match self.data.get_current_input() {
            None => (),
            Some(list_ref) => match self.get_access_addr(addr, list_ref)? {
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
        self.push_unit()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        error,
        runtime::{
            context::{EmptyContext, GarnishLangRuntimeContext},
            data::GarnishLangRuntimeData,
        },
        ExpressionData, ExpressionDataType, GarnishLangRuntime, GarnishLangRuntimeResult, Instruction, NestInto, SimpleRuntimeData,
    };

    #[test]
    fn resolve_no_ref_is_err() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();
        runtime.add_data(ExpressionData::list(vec![3], vec![3])).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();

        runtime.add_instruction(Instruction::Resolve, None).unwrap();

        runtime.data.set_result(Some(4)).unwrap();

        let result = runtime.resolve::<EmptyContext>(None);

        assert!(result.is_err());
    }

    #[test]
    fn resolve_from_result() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();
        runtime.add_data(ExpressionData::list(vec![3], vec![3])).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();

        runtime.add_instruction(Instruction::Resolve, None).unwrap();

        runtime.data.push_register(5).unwrap();

        runtime.data.set_result(Some(4)).unwrap();

        runtime.resolve::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.data.get_reference(6).unwrap(), 2);
    }

    #[test]
    fn resolve_from_input() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();
        runtime.add_data(ExpressionData::list(vec![3], vec![3])).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();

        runtime.add_instruction(Instruction::Resolve, None).unwrap();

        runtime.data.push_register(5).unwrap();

        runtime.data.push_input(4).unwrap();

        runtime.resolve::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.data.get_reference(6).unwrap(), 2);
    }

    #[test]
    fn resolve_not_found_is_unit() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();
        runtime.add_data(ExpressionData::list(vec![3], vec![3])).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"two".to_string())).unwrap();

        runtime.add_instruction(Instruction::Resolve, None).unwrap();

        runtime.data.push_register(5).unwrap();

        runtime.resolve::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.data.get_data_type(6).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn resolve_from_context() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();

        runtime.add_instruction(Instruction::Resolve, None).unwrap();

        runtime.data.push_register(1).unwrap();

        struct MyContext {}

        impl GarnishLangRuntimeContext<SimpleRuntimeData> for MyContext {
            fn resolve(&mut self, sym_addr: usize, runtime: &mut GarnishLangRuntime<SimpleRuntimeData>) -> GarnishLangRuntimeResult<String, bool> {
                match runtime.data.get_data_type(sym_addr).nest_into()? {
                    ExpressionDataType::Symbol => {
                        let addr = runtime.data.get_data_len();
                        runtime.data.add_integer(100).nest_into()?;
                        let raddr = runtime.data.get_data_len();
                        runtime.push_reference(addr)?;
                        runtime.data.push_register(raddr).nest_into()?;
                        Ok(true)
                    }
                    t => Err(error(format!("Address given to resolve is of type {:?}. Expected symbol type.", t)))?,
                }
            }

            fn apply(&mut self, _: usize, _: usize, _: &mut GarnishLangRuntime<SimpleRuntimeData>) -> GarnishLangRuntimeResult<String, bool> {
                Ok(false)
            }
        }

        let mut context = MyContext {};

        runtime.resolve(Some(&mut context)).unwrap();

        assert_eq!(runtime.data.get_integer(2).unwrap(), 100);
        assert_eq!(runtime.data.get_register().get(0).unwrap(), &3);
        assert_eq!(runtime.data.get_reference(3).unwrap(), 2);
    }
}
