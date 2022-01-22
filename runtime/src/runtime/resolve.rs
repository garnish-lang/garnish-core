use crate::{push_unit, runtime::list::get_access_addr, ErrorType, GarnishLangRuntimeContext, GarnishLangRuntimeData, RuntimeError, ExpressionDataType};

pub fn resolve<Data: GarnishLangRuntimeData, T: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    data: Data::Size,
    context: Option<&mut T>,
) -> Result<(), RuntimeError<Data::Error>> {
    // check input
    match this.get_current_value() {
        None => (),
        Some(list_ref) => match get_access_addr(this, data, list_ref) {
            Err(e) => {
                // ignore unsupported op type, will be handled by below resolve
                if e.get_type() != ErrorType::UnsupportedOpTypes {
                    Err(e)?;
                }
            }
            Ok(v) => match v {
                None => (),
                Some(i) => {
                    this.push_register(i)?;
                    return Ok(());
                }
            },
        },
    }

    // check context
    match context {
        None => (),
        Some(c) => match this.get_data_type(data)? {
            ExpressionDataType::Symbol => {
                match c.resolve(this.get_symbol(data)?, this)? {
                    true => return Ok(()), // context resovled end look up
                    false => (),           // not resolved fall through
                }
            }
            _ => (), // not a symbol push unit below
        }
    }

    // default to unit
    push_unit(this)
}

#[cfg(test)]
mod deferring {
    use crate::runtime::GarnishRuntime;
    use crate::testing_utilites::DeferOpTestContext;
    use crate::{ExpressionDataType, GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn resolve() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10.into()).unwrap();

        let mut context = DeferOpTestContext::new();

        runtime.resolve(int1, Some(&mut context)).unwrap();

        // resolve never passes to defer make sure default unit is place when not resolved
        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }
}

#[cfg(test)]
mod tests {
    use crate::runtime::context::EMPTY_CONTEXT;
    use crate::simple::DataError;
    use crate::{
        runtime::{
            context::{EmptyContext, GarnishLangRuntimeContext},
            utilities::push_number,
            GarnishRuntime,
        },
        ExpressionDataType, GarnishLangRuntimeData, Instruction, RuntimeError, SimpleRuntimeData,
    };

    #[allow(const_item_mutation)]
    #[test]
    fn resolve_non_symbol() {
        let mut runtime = SimpleRuntimeData::new();

        let i2 = runtime.add_number(10.into()).unwrap();

        runtime.resolve(i2, Some(&mut EMPTY_CONTEXT)).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn resolve_from_input() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol(1).unwrap();
        let i2 = runtime.add_number(10.into()).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_symbol(1).unwrap();

        runtime.push_instruction(Instruction::Resolve, None).unwrap();

        runtime.push_value_stack(i4).unwrap();

        runtime.resolve::<EmptyContext>(i5, None).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), i2);
    }

    #[test]
    fn resolve_not_found_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol(1).unwrap();
        let i2 = runtime.add_number(10.into()).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let _i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_symbol(2).unwrap();

        runtime.push_instruction(Instruction::Resolve, None).unwrap();

        runtime.resolve::<EmptyContext>(i5, None).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn resolve_from_context() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol(1).unwrap();
        let start = runtime.get_data_len();

        runtime.push_instruction(Instruction::Resolve, None).unwrap();

        struct MyContext {}

        impl GarnishLangRuntimeContext<SimpleRuntimeData> for MyContext {
            fn resolve(&mut self, sym_val: u64, runtime: &mut SimpleRuntimeData) -> Result<bool, RuntimeError<DataError>> {
                assert_eq!(sym_val, 1);

                push_number(runtime, 100.into())?;
                Ok(true)
            }

            fn apply(&mut self, _: usize, _: usize, _: &mut SimpleRuntimeData) -> Result<bool, RuntimeError<DataError>> {
                Ok(false)
            }
        }

        let mut context = MyContext {};

        runtime.resolve(i1, Some(&mut context)).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), start);
    }
}
