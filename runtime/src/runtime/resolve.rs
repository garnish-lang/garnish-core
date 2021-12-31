use log::trace;

use crate::{
     push_unit, runtime::list::get_access_addr, GarnishLangRuntimeContext, GarnishLangRuntimeData, GarnishLangRuntimeResult, NestInto,
};

pub fn resolve<Data: GarnishLangRuntimeData, T: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    data: Data::Size,
    context: Option<&mut T>,
) -> GarnishLangRuntimeResult<Data::Error> {
    trace!("Instruction - Resolve");

    // check input
    match this.get_current_value() {
        None => (),
        Some(list_ref) => match get_access_addr(this, data, list_ref)? {
            None => (),
            Some(i) => {
                this.push_register(i).nest_into()?;
                return Ok(());
            }
        },
    }

    // check context
    match context {
        None => (),
        Some(c) => match c.resolve(this.get_symbol(data).nest_into()?, this)? {
            true => return Ok(()), // context resovled end look up
            false => (),           // not resolved fall through
        },
    }

    // default to unit
    push_unit(this)
}

#[cfg(test)]
mod tests {
    use crate::{
        runtime::{
            context::{EmptyContext, GarnishLangRuntimeContext},
            utilities::push_integer,
            GarnishRuntime,
        },
        symbol_value, GarnishLangRuntimeData, GarnishLangRuntimeResult, Instruction, SimpleRuntimeData,
    };

    #[test]
    fn resolve_from_input() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_symbol("one").unwrap();

        runtime.push_instruction(Instruction::Resolve, None).unwrap();

        runtime.push_value_stack(i4).unwrap();

        runtime.resolve::<EmptyContext>(i5, None).unwrap();

        assert_eq!(runtime.get_register().get(0).unwrap(), &i2);
    }

    #[test]
    fn resolve_not_found_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let _i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_symbol("two").unwrap();

        runtime.push_instruction(Instruction::Resolve, None).unwrap();

        runtime.resolve::<EmptyContext>(i5, None).unwrap();

        assert_eq!(runtime.get_register().get(0).unwrap(), &0);
    }

    #[test]
    fn resolve_from_context() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let start = runtime.get_data_len();

        runtime.push_instruction(Instruction::Resolve, None).unwrap();

        struct MyContext {}

        impl GarnishLangRuntimeContext<SimpleRuntimeData> for MyContext {
            fn resolve(&mut self, sym_val: u64, runtime: &mut SimpleRuntimeData) -> GarnishLangRuntimeResult<String, bool> {
                assert_eq!(symbol_value("one"), sym_val);

                push_integer(runtime, 100)?;
                Ok(true)
            }

            fn apply(&mut self, _: usize, _: usize, _: &mut SimpleRuntimeData) -> GarnishLangRuntimeResult<String, bool> {
                Ok(false)
            }
        }

        let mut context = MyContext {};

        runtime.resolve(i1, Some(&mut context)).unwrap();

        assert_eq!(runtime.get_register().get(0).unwrap(), &start);
    }
}
