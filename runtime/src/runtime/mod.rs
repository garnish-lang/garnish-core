mod apply;
mod arithmetic;
mod bitwise;
mod casting;
mod comparison;
mod concat;
mod context;
mod data;
mod equality;
mod error;
pub mod instruction;
mod internals;
mod jumps;
mod link;
mod list;
mod logical;
mod pair;
mod put;
mod range;
mod resolve;
pub mod result;
pub mod runtime_impls;
mod sideeffect;
pub mod types;
mod utilities;

pub use utilities::{iterate_link, link_count};

pub use context::*;
pub use data::{GarnishLangRuntimeData, GarnishNumber, TypeConstants};
pub use error::*;

pub(crate) use utilities::*;

use crate::GarnishLangRuntimeInfo;

pub use garnish_traits::GarnishRuntime;

#[cfg(test)]
pub mod testing_utilites {
    use crate::{DataError, ExpressionDataType, GarnishLangRuntimeContext, GarnishLangRuntimeData, Instruction, RuntimeError, SimpleDataRuntimeNC};
    use crate::runtime_impls::SimpleGarnishRuntime;
    use crate::simple::SimpleRuntimeData;

    pub const DEFERRED_VALUE: usize = 1000;

    pub struct DeferOpTestContext {}

    impl DeferOpTestContext {
        pub fn new() -> Self {
            DeferOpTestContext {}
        }
    }

    impl GarnishLangRuntimeContext<SimpleRuntimeData> for DeferOpTestContext {
        fn defer_op(
            &mut self,
            data: &mut SimpleRuntimeData,
            _operation: Instruction,
            _left: (ExpressionDataType, usize),
            _right: (ExpressionDataType, usize),
        ) -> Result<bool, RuntimeError<DataError>> {
            // add simple value that is produced by any op
            data.add_external(DEFERRED_VALUE).and_then(|r| data.push_register(r))?;
            Ok(true)
        }
    }

    pub fn deferred_op<F>(func: F)
    where
        F: Fn(&mut SimpleGarnishRuntime<SimpleRuntimeData>, &mut DeferOpTestContext),
    {
        let mut runtime = create_simple_runtime();
        

        let int1 = runtime.get_data_mut().add_external(10).unwrap();
        let int2 = runtime.get_data_mut().add_expression(20).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        let mut context = DeferOpTestContext::new();

        func(&mut runtime, &mut context);

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_external(i).unwrap(), DEFERRED_VALUE);
    }

    pub fn deferred_unary_op<F>(func: F)
    where
        F: Fn(&mut SimpleGarnishRuntime<SimpleRuntimeData>, &mut DeferOpTestContext),
    {
        let mut runtime = create_simple_runtime();
        

        let int1 = runtime.get_data_mut().add_expression(10).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();

        let mut context = DeferOpTestContext::new();

        func(&mut runtime, &mut context);

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_external(i).unwrap(), DEFERRED_VALUE);
    }

    pub fn create_simple_runtime() -> SimpleGarnishRuntime<SimpleRuntimeData> {
        SimpleGarnishRuntime::new(SimpleRuntimeData::new())
    }

    pub fn add_pair(runtime: &mut SimpleRuntimeData, key: &str, value: i32) -> usize {
        let sym_value = SimpleDataRuntimeNC::parse_symbol(key).unwrap();
        let i1 = runtime.add_symbol(sym_value).unwrap();
        let i2 = runtime.add_number(value.into()).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();

        return i3;
    }

    pub fn add_list(runtime: &mut SimpleRuntimeData, count: usize) -> usize {
        runtime.start_list(count).unwrap();
        for i in 0..count {
            let d = add_pair(runtime, format!("val{}", i).as_str(), (i as i32 + 1) * 10);
            runtime.add_to_list(d, true).unwrap();
        }
        runtime.end_list().unwrap()
    }

    pub fn add_list_with_start(runtime: &mut SimpleRuntimeData, count: usize, start_value: i32) -> usize {
        runtime.start_list(count).unwrap();
        for i in 0..count {
            let v = start_value + i as i32;

            // use crate::symbol_value;
            // let sym = format!("val{}", v);
            // println!("{} = {}", sym, symbol_value(sym.as_str()));

            let d = add_pair(runtime, format!("val{}", v).as_str(), v);
            runtime.add_to_list(d, true).unwrap();
        }
        runtime.end_list().unwrap()
    }

    pub fn add_integer_list_with_start(runtime: &mut SimpleRuntimeData, count: usize, start_value: i32) -> usize {
        runtime.start_list(count).unwrap();
        for i in 0..count {
            let v = start_value + i as i32;
            let d = runtime.add_number(v.into()).unwrap();
            runtime.add_to_list(d, false).unwrap();
        }
        runtime.end_list().unwrap()
    }

    pub fn add_integer_list(runtime: &mut SimpleRuntimeData, count: usize) -> usize {
        runtime.start_list(count).unwrap();
        for i in 0..count {
            let d = runtime.add_number(((i as i32 + 1) * 10).into()).unwrap();
            runtime.add_to_list(d, false).unwrap();
        }
        runtime.end_list().unwrap()
    }

    pub fn add_range(runtime: &mut SimpleRuntimeData, start: i32, end: i32) -> usize {
        let d1 = runtime.add_number(start.into()).unwrap();
        let d2 = runtime.add_number(end.into()).unwrap();
        let d3 = runtime.add_range(d1, d2).unwrap();
        return d3;
    }

    pub fn add_links(runtime: &mut SimpleRuntimeData, count: usize, is_append: bool) -> usize {
        let mut last = runtime.add_unit().unwrap();
        for i in 0..count {
            // Append:  10 -> 20 -> 30 | 30 is the current value
            // Prepend: 10 <- 20 <- 30 | 10 is the current value
            // if not append we make reversed the creation to match ex above
            let i = if is_append { i } else { count - 1 - i };

            // use crate::symbol_value;
            // let sym = format!("val{}", i);
            // println!("{} = {}", sym, symbol_value(sym.as_str()));

            let v = add_pair(runtime, format!("val{}", i).as_str(), i as i32 + 1);
            last = runtime.add_link(v, last, is_append).unwrap();
        }
        last
    }

    pub fn add_links_with_start(runtime: &mut SimpleRuntimeData, count: usize, is_append: bool, start: i32) -> usize {
        let mut last = runtime.add_unit().unwrap();
        for i in 0..count {
            // Append:  10 -> 20 -> 30 | 30 is the current value
            // Prepend: 10 <- 20 <- 30 | 10 is the current value
            // if not append we make reversed the creation to match ex above
            let i = if is_append { i } else { count - 1 - i };
            let v = start + i as i32;

            // use crate::symbol_value;
            // let sym = format!("val{}", i);
            // println!("{} = {}", sym, symbol_value(sym.as_str()));

            let v = add_pair(runtime, format!("val{}", v).as_str(), v);
            last = runtime.add_link(v, last, is_append).unwrap();
        }
        last
    }

    pub fn add_concatenation_with_start(runtime: &mut SimpleRuntimeData, count: usize, start: i32) -> usize {
        let v = start as i32;
        let mut left = add_pair(runtime, format!("val{}", v).as_str(), v);

        for i in 1..count {
            let v = start + i as i32;
            let right = add_pair(runtime, format!("val{}", v).as_str(), v);
            left = runtime.add_concatenation(left, right).unwrap();
        }

        left
    }

    pub fn add_char_list(runtime: &mut SimpleRuntimeData, s: &str) -> usize {
        let chars = SimpleDataRuntimeNC::parse_char_list(s).unwrap();

        runtime.start_char_list().unwrap();
        for c in chars {
            runtime.add_to_char_list(c).unwrap();
        }

        runtime.end_char_list().unwrap()
    }

    pub fn slice_of_char_list(runtime: &mut SimpleRuntimeData, s: &str, start: i32, end: i32) -> usize {
        let list = add_char_list(runtime, s);
        let range = add_range(runtime, start, end);
        runtime.add_slice(list, range).unwrap()
    }

    pub fn add_byte_list(runtime: &mut SimpleRuntimeData, s: &str) -> usize {
        let bytes = SimpleDataRuntimeNC::parse_byte_list(s).unwrap();

        runtime.start_byte_list().unwrap();
        for b in bytes {
            runtime.add_to_byte_list(b).unwrap();
        }

        runtime.end_byte_list().unwrap()
    }

    pub fn slice_of_byte_list(runtime: &mut SimpleRuntimeData, s: &str, start: i32, end: i32) -> usize {
        let list = add_byte_list(runtime, s);
        let range = add_range(runtime, start, end);
        runtime.add_slice(list, range).unwrap()
    }
}
