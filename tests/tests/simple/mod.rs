mod apply;
mod arithmetic;
mod bitwise;
mod casting;
mod comparison;
mod concat;
mod equality;
mod jumps;
mod list;
mod logical;
mod pair;
mod put;
mod range;
mod resolve;
mod sideeffect;
mod access;

#[cfg(test)]
pub mod testing_utilities {
    use garnish_lang_simple_data::{DataError, SimpleDataRuntimeNC, SimpleGarnishData};
    use garnish_lang_runtime::SimpleGarnishRuntime;
    use garnish_lang_traits::{GarnishDataType, GarnishContext, GarnishData, GarnishRuntime, Instruction, RuntimeError};

    pub const DEFERRED_VALUE: usize = 1000;

    pub struct DeferOpTestContext {}

    impl DeferOpTestContext {
        pub fn new() -> Self {
            DeferOpTestContext {}
        }
    }

    impl GarnishContext<SimpleGarnishData> for DeferOpTestContext {
        fn defer_op(
            &mut self,
            data: &mut SimpleGarnishData,
            _operation: Instruction,
            _left: (GarnishDataType, usize),
            _right: (GarnishDataType, usize),
        ) -> Result<bool, RuntimeError<DataError>> {
            // add simple value that is produced by any op
            data.add_external(DEFERRED_VALUE).and_then(|r| data.push_register(r))?;
            Ok(true)
        }
    }

    pub fn deferred_op<F>(func: F)
    where
        F: Fn(&mut SimpleGarnishRuntime<SimpleGarnishData>, &mut DeferOpTestContext),
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
        F: Fn(&mut SimpleGarnishRuntime<SimpleGarnishData>, &mut DeferOpTestContext),
    {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_expression(10).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();

        let mut context = DeferOpTestContext::new();

        func(&mut runtime, &mut context);

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_external(i).unwrap(), DEFERRED_VALUE);
    }

    pub fn create_simple_runtime() -> SimpleGarnishRuntime<SimpleGarnishData> {
        SimpleGarnishRuntime::new(SimpleGarnishData::new())
    }

    pub fn add_pair(runtime: &mut SimpleGarnishData, key: &str, value: i32) -> usize {
        let sym_value = SimpleDataRuntimeNC::parse_symbol(key).unwrap();
        let i1 = runtime.add_symbol(sym_value).unwrap();
        let i2 = runtime.add_number(value.into()).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();

        return i3;
    }

    pub fn add_list(runtime: &mut SimpleGarnishData, count: usize) -> usize {
        runtime.start_list(count).unwrap();
        for i in 0..count {
            let d = add_pair(runtime, format!("val{}", i).as_str(), (i as i32 + 1) * 10);
            runtime.add_to_list(d, true).unwrap();
        }
        runtime.end_list().unwrap()
    }

    pub fn add_list_with_start(runtime: &mut SimpleGarnishData, count: usize, start_value: i32) -> usize {
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

    pub fn add_integer_list_with_start(runtime: &mut SimpleGarnishData, count: usize, start_value: i32) -> usize {
        runtime.start_list(count).unwrap();
        for i in 0..count {
            let v = start_value + i as i32;
            let d = runtime.add_number(v.into()).unwrap();
            runtime.add_to_list(d, false).unwrap();
        }
        runtime.end_list().unwrap()
    }

    pub fn add_integer_list(runtime: &mut SimpleGarnishData, count: usize) -> usize {
        runtime.start_list(count).unwrap();
        for i in 0..count {
            let d = runtime.add_number(((i as i32 + 1) * 10).into()).unwrap();
            runtime.add_to_list(d, false).unwrap();
        }
        runtime.end_list().unwrap()
    }

    pub fn add_range(runtime: &mut SimpleGarnishData, start: i32, end: i32) -> usize {
        let d1 = runtime.add_number(start.into()).unwrap();
        let d2 = runtime.add_number(end.into()).unwrap();
        let d3 = runtime.add_range(d1, d2).unwrap();
        return d3;
    }

    pub fn add_concatenation_with_start(runtime: &mut SimpleGarnishData, count: usize, start: i32) -> usize {
        let v = start as i32;
        let mut left = add_pair(runtime, format!("val{}", v).as_str(), v);

        for i in 1..count {
            let v = start + i as i32;
            let right = add_pair(runtime, format!("val{}", v).as_str(), v);
            left = runtime.add_concatenation(left, right).unwrap();
        }

        left
    }

    pub fn add_char_list(runtime: &mut SimpleGarnishData, s: &str) -> usize {
        let chars = SimpleDataRuntimeNC::parse_char_list(s).unwrap();

        runtime.start_char_list().unwrap();
        for c in chars {
            runtime.add_to_char_list(c).unwrap();
        }

        runtime.end_char_list().unwrap()
    }

    pub fn slice_of_char_list(runtime: &mut SimpleGarnishData, s: &str, start: i32, end: i32) -> usize {
        let list = add_char_list(runtime, s);
        let range = add_range(runtime, start, end);
        runtime.add_slice(list, range).unwrap()
    }

    pub fn add_byte_list(runtime: &mut SimpleGarnishData, s: &str) -> usize {
        let bytes = SimpleDataRuntimeNC::parse_byte_list(s).unwrap();

        runtime.start_byte_list().unwrap();
        for b in bytes {
            runtime.add_to_byte_list(b).unwrap();
        }

        runtime.end_byte_list().unwrap()
    }

    pub fn slice_of_byte_list(runtime: &mut SimpleGarnishData, s: &str, start: i32, end: i32) -> usize {
        let list = add_byte_list(runtime, s);
        let range = add_range(runtime, start, end);
        runtime.add_slice(list, range).unwrap()
    }
}
