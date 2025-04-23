mod access;
mod apply;
mod arithmetic;
mod bitwise;
mod casting;
mod comparison;
mod concat;
mod equality;
mod error;
mod internals;
mod jumps;
pub(crate) mod list;
mod logical;
mod pair;
mod put;
mod range;
mod resolve;
mod runtime_impls;
mod sideeffect;
mod utilities;

pub use runtime_impls::{SimpleGarnishRuntime, SimpleRuntimeInfo, SimpleRuntimeState};

#[cfg(test)]
mod tests {
    use garnish_lang_traits::{GarnishData, GarnishDataType, Instruction};
    use std::error::Error;
    use std::fmt::Display;

    #[derive(Debug)]
    pub struct MockError {}

    impl Display for MockError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "MockError")
        }
    }

    impl Error for MockError {}

    pub struct MockIterator {
        current: i32,
        max: i32
    }

    impl MockIterator {
        pub fn new(count: i32) -> Self {
            Self {
                current: 0,
                max: count
            }
        }
    }

    impl Default for MockIterator {
        fn default() -> Self {
            MockIterator { current: 0, max: 0 }
        }
    }

    impl Iterator for MockIterator {
        type Item = i32;

        fn next(&mut self) -> Option<Self::Item> {
            if self.current >= self.max {
                None
            } else {
                let value = Some(self.current);
                self.current += 1;
                value
            }
        }
    }

    impl DoubleEndedIterator for MockIterator {
        fn next_back(&mut self) -> Option<Self::Item> {
            if self.current < 0 {
                None
            } else {
                let value = Some(self.current);
                self.current -= 1;
                value
            }
        }
    }

    #[derive(Default)]
    pub struct BasicData {
        type_stack: Vec<GarnishDataType>,
        registers: Vec<i32>
    }

    impl BasicData {
        pub fn new(type_stack: Vec<GarnishDataType>) -> Self {
            let registers = type_stack.iter().enumerate().map(|(i, _)| i as i32).collect();
            Self { type_stack, registers }
        }
    }

    fn stub_fn_0<D, R>(_: &D) -> R {
        unimplemented!()
    }

    fn stub_fn_0_mut<D, R>(_: &mut D) -> R {
        unimplemented!()
    }

    fn stub_fn_1<D, T, R>(_: &D, _: T) -> R {
        unimplemented!()
    }

    fn stub_fn_1_mut<D, T, R>(_: &mut D, _: T) -> R {
        unimplemented!()
    }

    // fn stub_parse_fn<R>(_: &str) -> R {
    //     unimplemented!()
    // }

    fn stub_fn_2<D, T1, T2, R>(_: &D, _: T1, _: T2) -> R {
        unimplemented!()
    }

    fn stub_fn_2_mut<D, T1, T2, R>(_: &mut D, _: T1, _: T2) -> R {
        unimplemented!()
    }

    pub struct MockGarnishData<T>
    where
        T: Default,
    {
        pub data: T,
        pub stub_get_data_len: fn(&T) -> i32,
        pub stub_get_data_iter: fn(&T) -> MockIterator,
        pub stub_get_value_stack_len: fn(&T) -> i32,
        pub stub_push_value_stack: fn(&mut T, addr: i32) -> Result<(), MockError>,
        pub stub_pop_value_stack: fn(&mut T) -> Option<i32>,
        pub stub_get_value: fn(&T, addr: i32) -> Option<i32>,
        pub stub_get_value_mut: fn(&T, addr: i32) -> Option<&'static mut i32>,
        pub stub_get_current_value: fn(&T) -> Option<i32>,
        pub stub_get_current_value_mut: fn(&T) -> Option<&'static mut i32>,
        pub stub_get_value_iter: fn(&T) -> MockIterator,
        pub stub_get_data_type: fn(&T, addr: i32) -> Result<GarnishDataType, MockError>,
        pub stub_get_number: fn(&T, addr: i32) -> Result<i32, MockError>,
        pub stub_get_type: fn(&T, addr: i32) -> Result<GarnishDataType, MockError>,
        pub stub_get_char: fn(&T, addr: i32) -> Result<char, MockError>,
        pub stub_get_byte: fn(&T, addr: i32) -> Result<u8, MockError>,
        pub stub_get_symbol: fn(&T, addr: i32) -> Result<u32, MockError>,
        pub stub_get_expression: fn(&T, addr: i32) -> Result<i32, MockError>,
        pub stub_get_external: fn(&T, addr: i32) -> Result<i32, MockError>,
        pub stub_get_pair: fn(&T, addr: i32) -> Result<(i32, i32), MockError>,
        pub stub_get_concatenation: fn(&T, addr: i32) -> Result<(i32, i32), MockError>,
        pub stub_get_range: fn(&T, addr: i32) -> Result<(i32, i32), MockError>,
        pub stub_get_slice: fn(&T, addr: i32) -> Result<(i32, i32), MockError>,
        pub stub_get_list_len: fn(&T, addr: i32) -> Result<i32, MockError>,
        pub stub_get_list_item: fn(&T, list_addr: i32, item_addr: i32) -> Result<i32, MockError>,
        pub stub_get_list_associations_len: fn(&T, addr: i32) -> Result<i32, MockError>,
        pub stub_get_list_association: fn(&T, list_addr: i32, item_addr: i32) -> Result<i32, MockError>,
        pub stub_get_list_item_with_symbol: fn(&T, list_addr: i32, sym: u32) -> Result<Option<i32>, MockError>,
        pub stub_get_list_items_iter: fn(&T, list_addr: i32) -> MockIterator,
        pub stub_get_list_associations_iter: fn(&T, list_addr: i32) -> MockIterator,
        pub stub_get_char_list_len: fn(&T, addr: i32) -> Result<i32, MockError>,
        pub stub_get_char_list_item: fn(&T, addr: i32, item_index: i32) -> Result<char, MockError>,
        pub stub_get_char_list_iter: fn(&T, list_addr: i32) -> MockIterator,
        pub stub_get_byte_list_len: fn(&T, addr: i32) -> Result<i32, MockError>,
        pub stub_get_byte_list_item: fn(&T, addr: i32, item_index: i32) -> Result<u8, MockError>,
        pub stub_get_byte_list_iter: fn(&T, list_addr: i32) -> MockIterator,
        pub stub_get_symbol_list_len: fn(&T, addr: i32) -> Result<i32, MockError>,
        pub stub_get_symbol_list_item: fn(&T, addr: i32, item_index: i32) -> Result<u32, MockError>,
        pub stub_get_symbol_list_iter: fn(&T, list_addr: i32) -> MockIterator,
        pub stub_get_slice_iter: fn(&T, addr: i32) -> MockIterator,
        pub stub_add_unit: fn(&mut T) -> Result<i32, MockError>,
        pub stub_add_true: fn(&mut T) -> Result<i32, MockError>,
        pub stub_add_false: fn(&mut T) -> Result<i32, MockError>,
        pub stub_add_number: fn(&mut T, value: i32) -> Result<i32, MockError>,
        pub stub_add_type: fn(&mut T, value: GarnishDataType) -> Result<i32, MockError>,
        pub stub_add_char: fn(&mut T, value: char) -> Result<i32, MockError>,
        pub stub_add_byte: fn(&mut T, value: u8) -> Result<i32, MockError>,
        pub stub_add_symbol: fn(&mut T, value: u32) -> Result<i32, MockError>,
        pub stub_add_expression: fn(&mut T, value: i32) -> Result<i32, MockError>,
        pub stub_add_external: fn(&mut T, value: i32) -> Result<i32, MockError>,
        pub stub_add_pair: fn(&mut T, value: (i32, i32)) -> Result<i32, MockError>,
        pub stub_add_concatenation: fn(&mut T, left: i32, right: i32) -> Result<i32, MockError>,
        pub stub_add_range: fn(&mut T, start: i32, end: i32) -> Result<i32, MockError>,
        pub stub_add_slice: fn(&mut T, list: i32, range: i32) -> Result<i32, MockError>,
        pub stub_merge_to_symbol_list: fn(&mut T, first: i32, second: i32) -> Result<i32, MockError>,
        pub stub_start_list: fn(&mut T, len: i32) -> Result<(), MockError>,
        pub stub_add_to_list: fn(&mut T, addr: i32, is_associative: bool) -> Result<(), MockError>,
        pub stub_end_list: fn(&mut T) -> Result<i32, MockError>,
        pub stub_start_char_list: fn(&mut T) -> Result<(), MockError>,
        pub stub_add_to_char_list: fn(&mut T, c: char) -> Result<(), MockError>,
        pub stub_end_char_list: fn(&mut T) -> Result<i32, MockError>,
        pub stub_start_byte_list: fn(&mut T) -> Result<(), MockError>,
        pub stub_add_to_byte_list: fn(&mut T, c: u8) -> Result<(), MockError>,
        pub stub_end_byte_list: fn(&mut T) -> Result<i32, MockError>,
        pub stub_get_register_len: fn(&T) -> i32,
        pub stub_push_register: fn(&mut T, addr: i32) -> Result<(), MockError>,
        pub stub_get_register: fn(&T, addr: i32) -> Option<i32>,
        pub stub_pop_register: fn(&mut T) -> Result<Option<i32>, MockError>,
        pub stub_get_register_iter: fn(&T) -> MockIterator,
        pub stub_get_instruction_len: fn(&T) -> i32,
        pub stub_push_instruction: fn(&mut T, instruction: Instruction, data: Option<i32>) -> Result<i32, MockError>,
        pub stub_get_instruction: fn(&T, addr: i32) -> Option<(Instruction, Option<i32>)>,
        pub stub_get_instruction_iter: fn(&T) -> MockIterator,
        pub stub_get_instruction_cursor: fn(&T) -> i32,
        pub stub_set_instruction_cursor: fn(&mut T, addr: i32) -> Result<(), MockError>,
        pub stub_get_jump_table_len: fn(&T) -> i32,
        pub stub_push_jump_point: fn(&mut T, index: i32) -> Result<(), MockError>,
        pub stub_get_jump_point: fn(&T, index: i32) -> Option<i32>,
        pub stub_get_jump_point_mut: fn(&T, index: i32) -> Option<&'static mut i32>,
        pub stub_get_jump_table_iter: fn(&T) -> MockIterator,
        pub stub_push_jump_path: fn(&mut T, index: i32) -> Result<(), MockError>,
        pub stub_pop_jump_path: fn(&mut T) -> Option<i32>,
        pub stub_get_jump_path_iter: fn(&T) -> MockIterator,
        // stub_size_to_number: fn(&T, from: i32) -> i32,
        // stub_number_to_size: fn(&T, from: i32) -> Option<i32>,
        // stub_number_to_char: fn(&T, from: i32) -> Option<char>,
        // stub_number_to_byte: fn(&T, from: i32) -> Option<u8>,
        // stub_char_to_number: fn(&T, from: char) -> Option<i32>,
        // stub_char_to_byte: fn(&T, from: char) -> Option<u8>,
        // stub_byte_to_number: fn(&T, from: char) -> Option<i32>,
        // stub_byte_to_char: fn(&T, from: u8) -> Option<char>,
        pub stub_add_char_list_from: fn(&mut T, from: i32) -> Result<i32, MockError>,
        pub stub_add_byte_list_from: fn(&mut T, from: i32) -> Result<i32, MockError>,
        pub stub_add_symbol_from: fn(&mut T, from: i32) -> Result<i32, MockError>,
        pub stub_add_byte_from: fn(&mut T, from: i32) -> Result<i32, MockError>,
        pub stub_add_number_from: fn(&mut T, from: i32) -> Result<i32, MockError>,
        // stub_parse_number: fn(&T, from: &str) -> Result<i32, MockError>,
        // stub_parse_symbol: fn(&T, from: &str) -> Result<u32, MockError>,
        // stub_parse_char: fn(&T, from: &str) -> Result<char, MockError>,
        // stub_parse_byte: fn(&T, from: &str) -> Result<u8, MockError>,
        // stub_parse_char_list: fn(&T, from: &str) -> Result<Vec<char>, MockError>,
        // stub_parse_byte_list: fn(&T, from: &str) -> Result<Vec<u8>, MockError>,
    }

    impl MockGarnishData<BasicData> {
        pub fn new_basic_data(type_stack: Vec<GarnishDataType>) -> Self {
            let data = BasicData::new(type_stack);
            let mut mock = Self::default_with_data(data);

            mock.stub_get_data_type = |data, i| Ok(data.type_stack.get(i as usize).cloned().unwrap_or(GarnishDataType::Unit));
            mock.stub_pop_register = |data| match data.registers.pop() {
                None => {
                    assert!(false, "Ran out of test register data.");
                    Err(MockError {})
                },
                i => Ok(i)
            };
            mock.stub_get_instruction_cursor = |_| 0;

            mock
        }
    }

    impl<T> MockGarnishData<T> where T: Default {
        pub fn default_with_data(data: T) -> Self {
            let mut m = MockGarnishData::default();
            m.data = data;
            m
        }
        pub fn data(&self) -> &T {
            &self.data
        }

        pub fn data_mut(&mut self) -> &mut T {
            &mut self.data
        }
    }

    impl<T> Default for MockGarnishData<T>
    where
        T: Default,
    {
        fn default() -> Self {
            MockGarnishData {
                data: Default::default(),
                stub_get_data_len: stub_fn_0,
                stub_get_data_iter: stub_fn_0,
                stub_get_value_stack_len: stub_fn_0,
                stub_push_value_stack: stub_fn_1_mut,
                stub_pop_value_stack: stub_fn_0_mut,
                stub_get_value: stub_fn_1,
                stub_get_value_mut: stub_fn_1,
                stub_get_current_value: stub_fn_0,
                stub_get_current_value_mut: stub_fn_0,
                stub_get_value_iter: stub_fn_0,
                stub_get_data_type: stub_fn_1,
                stub_get_number: stub_fn_1,
                stub_get_type: stub_fn_1,
                stub_get_char: stub_fn_1,
                stub_get_byte: stub_fn_1,
                stub_get_symbol: stub_fn_1,
                stub_get_expression: stub_fn_1,
                stub_get_external: stub_fn_1,
                stub_get_pair: stub_fn_1,
                stub_get_concatenation: stub_fn_1,
                stub_get_range: stub_fn_1,
                stub_get_slice: stub_fn_1,
                stub_get_list_len: stub_fn_1,
                stub_get_list_item: stub_fn_2,
                stub_get_list_associations_len: stub_fn_1,
                stub_get_list_association: stub_fn_2,
                stub_get_list_item_with_symbol: stub_fn_2,
                stub_get_list_items_iter: stub_fn_1,
                stub_get_list_associations_iter: stub_fn_1,
                stub_get_char_list_len: stub_fn_1,
                stub_get_char_list_item: stub_fn_2,
                stub_get_char_list_iter: stub_fn_1,
                stub_get_byte_list_len: stub_fn_1,
                stub_get_byte_list_item: stub_fn_2,
                stub_get_byte_list_iter: stub_fn_1,
                stub_get_symbol_list_len: stub_fn_1,
                stub_get_symbol_list_item: stub_fn_2,
                stub_get_symbol_list_iter: stub_fn_1,
                stub_get_slice_iter: stub_fn_1,
                stub_add_unit: stub_fn_0_mut,
                stub_add_true: stub_fn_0_mut,
                stub_add_false: stub_fn_0_mut,
                stub_add_number: stub_fn_1_mut,
                stub_add_type: stub_fn_1_mut,
                stub_add_char: stub_fn_1_mut,
                stub_add_byte: stub_fn_1_mut,
                stub_add_symbol: stub_fn_1_mut,
                stub_add_expression: stub_fn_1_mut,
                stub_add_external: stub_fn_1_mut,
                stub_add_pair: stub_fn_1_mut,
                stub_add_concatenation: stub_fn_2_mut,
                stub_add_range: stub_fn_2_mut,
                stub_add_slice: stub_fn_2_mut,
                stub_merge_to_symbol_list: stub_fn_2_mut,
                stub_start_list: stub_fn_1_mut,
                stub_add_to_list: stub_fn_2_mut,
                stub_end_list: stub_fn_0_mut,
                stub_start_char_list: stub_fn_0_mut,
                stub_add_to_char_list: stub_fn_1_mut,
                stub_end_char_list: stub_fn_0_mut,
                stub_start_byte_list: stub_fn_0_mut,
                stub_add_to_byte_list: stub_fn_1_mut,
                stub_end_byte_list: stub_fn_0_mut,
                stub_get_register_len: stub_fn_0,
                stub_push_register: stub_fn_1_mut,
                stub_get_register: stub_fn_1,
                stub_pop_register: stub_fn_0_mut,
                stub_get_register_iter: stub_fn_0,
                stub_get_instruction_len: stub_fn_0,
                stub_push_instruction: stub_fn_2_mut,
                stub_get_instruction: stub_fn_1,
                stub_get_instruction_iter: stub_fn_0,
                stub_get_instruction_cursor: stub_fn_0,
                stub_set_instruction_cursor: stub_fn_1_mut,
                stub_get_jump_table_len: stub_fn_0,
                stub_push_jump_point: stub_fn_1_mut,
                stub_get_jump_point: stub_fn_1,
                stub_get_jump_point_mut: stub_fn_1,
                stub_get_jump_table_iter: stub_fn_0,
                stub_push_jump_path: stub_fn_1_mut,
                stub_pop_jump_path: stub_fn_0_mut,
                stub_get_jump_path_iter: stub_fn_0,
                // stub_size_to_number: stub_fn_1,
                // stub_number_to_size: stub_fn_1,
                // stub_number_to_char: stub_fn_1,
                // stub_number_to_byte: stub_fn_1,
                // stub_char_to_number: stub_fn_1,
                // stub_char_to_byte: stub_fn_1,
                // stub_byte_to_number: stub_fn_1,
                // stub_byte_to_char: stub_fn_1,
                stub_add_char_list_from: stub_fn_1_mut,
                stub_add_byte_list_from: stub_fn_1_mut,
                stub_add_symbol_from: stub_fn_1_mut,
                stub_add_byte_from: stub_fn_1_mut,
                stub_add_number_from: stub_fn_1_mut,
                // stub_parse_number: stub_parse_fn,
                // stub_parse_symbol: stub_parse_fn,
                // stub_parse_char: stub_parse_fn,
                // stub_parse_byte: stub_parse_fn,
                // stub_parse_char_list: stub_parse_fn,
                // stub_parse_byte_list: stub_parse_fn,
            }
        }
    }

    impl<T> GarnishData for MockGarnishData<T>
    where
        T: Default,
    {
        type Error = MockError;
        type Symbol = u32;
        type Byte = u8;
        type Char = char;
        type Number = i32;
        type Size = i32;
        type SizeIterator = MockIterator;
        type NumberIterator = MockIterator;
        type InstructionIterator = MockIterator;
        type DataIndexIterator = MockIterator;
        type ValueIndexIterator = MockIterator;
        type RegisterIndexIterator = MockIterator;
        type JumpTableIndexIterator = MockIterator;
        type JumpPathIndexIterator = MockIterator;
        type ListIndexIterator = MockIterator;

        fn get_data_len(&self) -> Self::Size {
            (self.stub_get_data_len)(self.data())
        }

        fn get_data_iter(&self) -> Self::DataIndexIterator {
            (self.stub_get_data_iter)(self.data())
        }

        fn get_value_stack_len(&self) -> Self::Size {
            (self.stub_get_value_stack_len)(self.data())
        }

        fn push_value_stack(&mut self, addr: Self::Size) -> Result<(), Self::Error> {
            (self.stub_push_value_stack)(self.data_mut(), addr)
        }

        fn pop_value_stack(&mut self) -> Option<Self::Size> {
            (self.stub_pop_value_stack)(self.data_mut())
        }

        fn get_value(&self, addr: Self::Size) -> Option<Self::Size> {
            (self.stub_get_value)(self.data(), addr)
        }

        fn get_value_mut(&mut self, addr: Self::Size) -> Option<&mut Self::Size> {
            (self.stub_get_value_mut)(self.data_mut(), addr)
        }

        fn get_current_value(&self) -> Option<Self::Size> {
            (self.stub_get_current_value)(self.data())
        }

        fn get_current_value_mut(&mut self) -> Option<&mut Self::Size> {
            (self.stub_get_current_value_mut)(self.data_mut())
        }

        fn get_value_iter(&self) -> Self::ValueIndexIterator {
            (self.stub_get_value_iter)(self.data())
        }

        fn get_data_type(&self, addr: Self::Size) -> Result<GarnishDataType, Self::Error> {
            (self.stub_get_data_type)(self.data(), addr)
        }

        fn get_number(&self, addr: Self::Size) -> Result<Self::Number, Self::Error> {
            (self.stub_get_number)(self.data(), addr)
        }

        fn get_type(&self, addr: Self::Size) -> Result<GarnishDataType, Self::Error> {
            (self.stub_get_type)(self.data(), addr)
        }

        fn get_char(&self, addr: Self::Size) -> Result<Self::Char, Self::Error> {
            (self.stub_get_char)(self.data(), addr)
        }

        fn get_byte(&self, addr: Self::Size) -> Result<Self::Byte, Self::Error> {
            (self.stub_get_byte)(self.data(), addr)
        }

        fn get_symbol(&self, addr: Self::Size) -> Result<Self::Symbol, Self::Error> {
            (self.stub_get_symbol)(self.data(), addr)
        }

        fn get_expression(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
            (self.stub_get_expression)(self.data(), addr)
        }

        fn get_external(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
            (self.stub_get_external)(self.data(), addr)
        }

        fn get_pair(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
            (self.stub_get_pair)(self.data(), addr)
        }

        fn get_concatenation(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
            (self.stub_get_concatenation)(self.data(), addr)
        }

        fn get_range(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
            (self.stub_get_range)(self.data(), addr)
        }

        fn get_slice(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
            (self.stub_get_slice)(self.data(), addr)
        }

        fn get_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
            (self.stub_get_list_len)(self.data(), addr)
        }

        fn get_list_item(&self, list_addr: Self::Size, item_addr: Self::Number) -> Result<Self::Size, Self::Error> {
            (self.stub_get_list_item)(self.data(), list_addr, item_addr)
        }

        fn get_list_associations_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
            (self.stub_get_list_associations_len)(self.data(), addr)
        }

        fn get_list_association(&self, list_addr: Self::Size, item_addr: Self::Number) -> Result<Self::Size, Self::Error> {
            (self.stub_get_list_association)(self.data(), list_addr, item_addr)
        }

        fn get_list_item_with_symbol(&self, list_addr: Self::Size, sym: Self::Symbol) -> Result<Option<Self::Size>, Self::Error> {
            (self.stub_get_list_item_with_symbol)(self.data(), list_addr, sym)
        }

        fn get_list_items_iter(&self, list_addr: Self::Size) -> Self::ListIndexIterator {
            (self.stub_get_list_items_iter)(self.data(), list_addr)
        }

        fn get_list_associations_iter(&self, list_addr: Self::Size) -> Self::ListIndexIterator {
            (self.stub_get_list_associations_iter)(self.data(), list_addr)
        }

        fn get_char_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
            (self.stub_get_char_list_len)(self.data(), addr)
        }

        fn get_char_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Self::Char, Self::Error> {
            (self.stub_get_char_list_item)(self.data(), addr, item_index)
        }

        fn get_char_list_iter(&self, list_addr: Self::Size) -> Self::ListIndexIterator {
            (self.stub_get_char_list_iter)(self.data(), list_addr)
        }

        fn get_byte_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
            (self.stub_get_byte_list_len)(self.data(), addr)
        }

        fn get_byte_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Self::Byte, Self::Error> {
            (self.stub_get_byte_list_item)(self.data(), addr, item_index)
        }

        fn get_byte_list_iter(&self, list_addr: Self::Size) -> Self::ListIndexIterator {
            (self.stub_get_byte_list_iter)(self.data(), list_addr)
        }

        fn get_symbol_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
            (self.stub_get_symbol_list_len)(self.data(), addr)
        }

        fn get_symbol_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Self::Symbol, Self::Error> {
            (self.stub_get_symbol_list_item)(self.data(), addr, item_index)
        }

        fn get_symbol_list_iter(&self, list_addr: Self::Size) -> Self::ListIndexIterator {
            (self.stub_get_symbol_list_iter)(self.data(), list_addr)
        }

        fn get_slice_iter(&self, addr: Self::Size) -> Self::ListIndexIterator {
            (self.stub_get_slice_iter)(self.data(), addr)
        }

        fn add_unit(&mut self) -> Result<Self::Size, Self::Error> {
            (self.stub_add_unit)(self.data_mut())
        }

        fn add_true(&mut self) -> Result<Self::Size, Self::Error> {
            (self.stub_add_true)(self.data_mut())
        }

        fn add_false(&mut self) -> Result<Self::Size, Self::Error> {
            (self.stub_add_false)(self.data_mut())
        }

        fn add_number(&mut self, value: Self::Number) -> Result<Self::Size, Self::Error> {
            (self.stub_add_number)(self.data_mut(), value)
        }

        fn add_type(&mut self, value: GarnishDataType) -> Result<Self::Size, Self::Error> {
            (self.stub_add_type)(self.data_mut(), value)
        }

        fn add_char(&mut self, value: Self::Char) -> Result<Self::Size, Self::Error> {
            (self.stub_add_char)(self.data_mut(), value)
        }

        fn add_byte(&mut self, value: Self::Byte) -> Result<Self::Size, Self::Error> {
            (self.stub_add_byte)(self.data_mut(), value)
        }

        fn add_symbol(&mut self, value: Self::Symbol) -> Result<Self::Size, Self::Error> {
            (self.stub_add_symbol)(self.data_mut(), value)
        }

        fn add_expression(&mut self, value: Self::Size) -> Result<Self::Size, Self::Error> {
            (self.stub_add_expression)(self.data_mut(), value)
        }

        fn add_external(&mut self, value: Self::Size) -> Result<Self::Size, Self::Error> {
            (self.stub_add_external)(self.data_mut(), value)
        }

        fn add_pair(&mut self, value: (Self::Size, Self::Size)) -> Result<Self::Size, Self::Error> {
            (self.stub_add_pair)(self.data_mut(), value)
        }

        fn add_concatenation(&mut self, left: Self::Size, right: Self::Size) -> Result<Self::Size, Self::Error> {
            (self.stub_add_concatenation)(self.data_mut(), left, right)
        }

        fn add_range(&mut self, start: Self::Size, end: Self::Size) -> Result<Self::Size, Self::Error> {
            (self.stub_add_range)(self.data_mut(), start, end)
        }

        fn add_slice(&mut self, list: Self::Size, range: Self::Size) -> Result<Self::Size, Self::Error> {
            (self.stub_add_slice)(self.data_mut(), list, range)
        }

        fn merge_to_symbol_list(&mut self, first: Self::Size, second: Self::Size) -> Result<Self::Size, Self::Error> {
            (self.stub_merge_to_symbol_list)(self.data_mut(), first, second)
        }

        fn start_list(&mut self, len: Self::Size) -> Result<(), Self::Error> {
            (self.stub_start_list)(self.data_mut(), len)
        }

        fn add_to_list(&mut self, addr: Self::Size, is_associative: bool) -> Result<(), Self::Error> {
            (self.stub_add_to_list)(self.data_mut(), addr, is_associative)
        }

        fn end_list(&mut self) -> Result<Self::Size, Self::Error> {
            (self.stub_end_list)(self.data_mut())
        }

        fn start_char_list(&mut self) -> Result<(), Self::Error> {
            (self.stub_start_char_list)(self.data_mut())
        }

        fn add_to_char_list(&mut self, c: Self::Char) -> Result<(), Self::Error> {
            (self.stub_add_to_char_list)(self.data_mut(), c)
        }

        fn end_char_list(&mut self) -> Result<Self::Size, Self::Error> {
            (self.stub_end_char_list)(self.data_mut())
        }

        fn start_byte_list(&mut self) -> Result<(), Self::Error> {
            (self.stub_start_byte_list)(self.data_mut())
        }

        fn add_to_byte_list(&mut self, c: Self::Byte) -> Result<(), Self::Error> {
            (self.stub_add_to_byte_list)(self.data_mut(), c)
        }

        fn end_byte_list(&mut self) -> Result<Self::Size, Self::Error> {
            (self.stub_end_byte_list)(self.data_mut())
        }

        fn get_register_len(&self) -> Self::Size {
            (self.stub_get_register_len)(self.data())
        }

        fn push_register(&mut self, addr: Self::Size) -> Result<(), Self::Error> {
            (self.stub_push_register)(self.data_mut(), addr)
        }

        fn get_register(&self, addr: Self::Size) -> Option<Self::Size> {
            (self.stub_get_register)(self.data(), addr)
        }

        fn pop_register(&mut self) -> Result<Option<Self::Size>, Self::Error> {
            (self.stub_pop_register)(self.data_mut())
        }

        fn get_register_iter(&self) -> Self::RegisterIndexIterator {
            (self.stub_get_register_iter)(self.data())
        }

        fn get_instruction_len(&self) -> Self::Size {
            (self.stub_get_instruction_len)(self.data())
        }

        fn push_instruction(&mut self, instruction: Instruction, data: Option<Self::Size>) -> Result<Self::Size, Self::Error> {
            (self.stub_push_instruction)(self.data_mut(), instruction, data)
        }

        fn get_instruction(&self, addr: Self::Size) -> Option<(Instruction, Option<Self::Size>)> {
            (self.stub_get_instruction)(self.data(), addr)
        }

        fn get_instruction_iter(&self) -> Self::InstructionIterator {
            (self.stub_get_instruction_iter)(self.data())
        }

        fn get_instruction_cursor(&self) -> Self::Size {
            (self.stub_get_instruction_cursor)(self.data())
        }

        fn set_instruction_cursor(&mut self, addr: Self::Size) -> Result<(), Self::Error> {
            (self.stub_set_instruction_cursor)(self.data_mut(), addr)
        }

        fn get_jump_table_len(&self) -> Self::Size {
            (self.stub_get_jump_table_len)(self.data())
        }

        fn push_jump_point(&mut self, index: Self::Size) -> Result<(), Self::Error> {
            (self.stub_push_jump_point)(self.data_mut(), index)
        }

        fn get_jump_point(&self, index: Self::Size) -> Option<Self::Size> {
            (self.stub_get_jump_point)(self.data(), index)
        }

        fn get_jump_point_mut(&mut self, index: Self::Size) -> Option<&mut Self::Size> {
            (self.stub_get_jump_point_mut)(self.data_mut(), index)
        }

        fn get_jump_table_iter(&self) -> Self::JumpTableIndexIterator {
            (self.stub_get_jump_table_iter)(self.data())
        }

        fn push_jump_path(&mut self, index: Self::Size) -> Result<(), Self::Error> {
            (self.stub_push_jump_path)(self.data_mut(), index)
        }

        fn pop_jump_path(&mut self) -> Option<Self::Size> {
            (self.stub_pop_jump_path)(self.data_mut())
        }

        fn get_jump_path_iter(&self) -> Self::JumpPathIndexIterator {
            (self.stub_get_jump_path_iter)(self.data())
        }

        fn size_to_number(from: Self::Size) -> Self::Number {
            from
        }

        fn number_to_size(_from: Self::Number) -> Option<Self::Size> {
            unimplemented!()
        }

        fn number_to_char(_from: Self::Number) -> Option<Self::Char> {
            unimplemented!()
        }

        fn number_to_byte(_from: Self::Number) -> Option<Self::Byte> {
            unimplemented!()
        }

        fn char_to_number(_from: Self::Char) -> Option<Self::Number> {
            unimplemented!()
        }

        fn char_to_byte(_from: Self::Char) -> Option<Self::Byte> {
            unimplemented!()
        }

        fn byte_to_number(_from: Self::Byte) -> Option<Self::Number> {
            unimplemented!()
        }

        fn byte_to_char(_from: Self::Byte) -> Option<Self::Char> {
            unimplemented!()
        }

        fn add_char_list_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
            (self.stub_add_char_list_from)(self.data_mut(), from)
        }

        fn add_byte_list_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
            (self.stub_add_byte_list_from)(self.data_mut(), from)
        }

        fn add_symbol_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
            (self.stub_add_symbol_from)(self.data_mut(), from)
        }

        fn add_byte_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
            (self.stub_add_byte_from)(self.data_mut(), from)
        }

        fn add_number_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
            (self.stub_add_number_from)(self.data_mut(), from)
        }

        fn parse_number(_from: &str) -> Result<Self::Number, Self::Error> {
            unimplemented!()
        }

        fn parse_symbol(_from: &str) -> Result<Self::Symbol, Self::Error> {
            unimplemented!()
        }

        fn parse_char(_from: &str) -> Result<Self::Char, Self::Error> {
            unimplemented!()
        }

        fn parse_byte(_from: &str) -> Result<Self::Byte, Self::Error> {
            unimplemented!()
        }

        fn parse_char_list(_from: &str) -> Result<Vec<Self::Char>, Self::Error> {
            unimplemented!()
        }

        fn parse_byte_list(_from: &str) -> Result<Vec<Self::Byte>, Self::Error> {
            unimplemented!()
        }

        fn make_size_iterator_range(_min: Self::Size, _max: Self::Size) -> Self::SizeIterator {
            unimplemented!()
        }

        fn make_number_iterator_range(_min: Self::Number, _max: Self::Number) -> Self::NumberIterator {
            unimplemented!()
        }
    }
}
