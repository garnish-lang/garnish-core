#[cfg(test)]
mod tests {
    use crate::simple::clone::test_data_impl::TestData;
    use garnish_lang::helpers::{clone_data, clone_data_with_custom_handler, clone_data_with_handlers, clone_data_with_invalid_handler};
    use garnish_lang::simple::{SimpleData, SimpleDataType, SimpleGarnishData, SimpleNumber};
    use garnish_lang::{GarnishData, GarnishDataType};

    #[test]
    fn copy_number() {
        let mut from = SimpleGarnishData::new();
        let addr = from.add_number(SimpleNumber::Integer(40)).unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 6);
        assert_eq!(to.get_data().get(6).unwrap().as_number().unwrap(), SimpleNumber::Integer(40));
    }

    #[test]
    fn copy_unit() {
        let mut from = SimpleGarnishData::new();
        let addr = from.add_unit().unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 0);
        assert!(to.get_data().get(0).unwrap().is_unit());
    }

    #[test]
    fn copy_true() {
        let mut from = SimpleGarnishData::new();
        let addr = from.add_true().unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 2);
        assert!(to.get_data().get(2).unwrap().is_true());
    }

    #[test]
    fn copy_false() {
        let mut from = SimpleGarnishData::new();
        let addr = from.add_false().unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 1);
        assert!(to.get_data().get(1).unwrap().is_false());
    }

    #[test]
    fn copy_type() {
        let mut from = SimpleGarnishData::new();
        let addr = from.add_type(GarnishDataType::Byte).unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 6);
        assert_eq!(to.get_data().get(6).unwrap().as_type().unwrap(), GarnishDataType::Byte);
    }

    #[test]
    fn copy_char() {
        let mut from = SimpleGarnishData::new();
        let addr = from.add_char('a').unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 6);
        assert_eq!(to.get_data().get(6).unwrap().as_char().unwrap(), 'a');
    }

    #[test]
    fn copy_char_list() {
        let mut from = SimpleGarnishData::new();
        let addr = from.parse_add_char_list("\"stuff\"").unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 6);
        assert_eq!(to.get_data().get(6).unwrap().as_char_list().unwrap(), "stuff");
    }

    #[test]
    fn copy_byte() {
        let mut from = SimpleGarnishData::new();
        let addr = from.add_byte(10).unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 6);
        assert_eq!(to.get_data().get(6).unwrap().as_byte().unwrap(), 10);
    }

    #[test]
    fn copy_byte_list() {
        let mut from = SimpleGarnishData::new();
        let addr = from.parse_add_byte_list("''100 150 200''").unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 6);
        assert_eq!(to.get_data().get(6).unwrap().as_byte_list().unwrap(), vec![100, 150, 200]);
    }

    #[test]
    fn copy_range() {
        let mut from = SimpleGarnishData::new();
        let s = from.add_number(SimpleNumber::Integer(1)).unwrap();
        let e = from.add_number(SimpleNumber::Integer(5)).unwrap();
        let addr = from.add_range(s, e).unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 8);
        assert_eq!(to.get_data().get(8).unwrap().as_range().unwrap(), (6, 7));
        assert_eq!(to.get_data().get(6).unwrap().as_number().unwrap(), SimpleNumber::Integer(1));
        assert_eq!(to.get_data().get(7).unwrap().as_number().unwrap(), SimpleNumber::Integer(5));
    }

    #[test]
    fn copy_symbol() {
        let mut from = SimpleGarnishData::new();
        let addr = from.add_symbol(100).unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 6);
        assert_eq!(to.get_data().get(6).unwrap().as_symbol().unwrap(), 100);
    }

    #[test]
    fn copy_symbol_list() {
        let mut from = SimpleGarnishData::new();
        let sym1 = from.add_symbol(100).unwrap();
        let sym2 = from.add_symbol(200).unwrap();
        let sym_list = from.merge_to_symbol_list(sym1, sym2).unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(sym_list, &from, &mut to).unwrap();

        assert_eq!(new_addr, 8);
        assert_eq!(to.get_data().get(8).unwrap().as_symbol_list().unwrap(), vec![100, 200]);
    }

    #[test]
    fn copy_expression() {
        let mut from = SimpleGarnishData::new();
        let addr = from.add_expression(100).unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 6);
        assert_eq!(to.get_data().get(6).unwrap().as_expression().unwrap(), 100);
    }

    #[test]
    fn copy_external() {
        let mut from = SimpleGarnishData::new();
        let addr = from.add_external(100).unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 6);
        assert_eq!(to.get_data().get(6).unwrap().as_external().unwrap(), 100);
    }

    #[derive(Copy, Clone, PartialOrd, PartialEq, Eq, Debug, Hash)]
    struct CustomData {
        num: usize,
    }
    
    impl SimpleDataType for CustomData {}

    #[test]
    fn copy_custom_no_handler() {
        let mut from = SimpleGarnishData::<CustomData>::new_custom();
        let addr = from.add_custom(CustomData { num: 12345 }).unwrap();

        let mut to = SimpleGarnishData::<CustomData>::new_custom();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(addr, &from, &mut to).unwrap();

        assert_eq!(new_addr, 0);
        assert!(to.get_data().get(0).unwrap().is_unit());
    }

    #[test]
    fn copy_custom_handler() {
        let mut from = SimpleGarnishData::<CustomData>::new_custom();
        let addr = from.add_custom(CustomData { num: 12345 }).unwrap();

        let mut to = SimpleGarnishData::<CustomData>::new_custom();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data_with_custom_handler(addr, &from, &mut to, |_addr, _from, to| to.add_number(SimpleNumber::Integer(12345))).unwrap();

        assert_eq!(new_addr, 6);
        assert_eq!(to.get_data().get(6).unwrap().as_number().unwrap(), SimpleNumber::Integer(12345));
    }

    #[test]
    fn copy_invalid_no_handler() {
        let from = TestData::new();
        let mut to = TestData::new();

        clone_data(0, &from, &mut to).unwrap();

        assert!(to.unit_added);
        assert!(!to.number_added);
    }

    #[test]
    fn copy_invalid_handler() {
        let from = TestData::new();
        let mut to = TestData::new();

        clone_data_with_invalid_handler(0, &from, &mut to, |_, _, to| to.add_number(SimpleNumber::Integer(10))).unwrap();

        assert!(!to.unit_added);
        assert!(to.number_added);
    }

    #[test]
    fn copy_with_handlers() {
        let from = TestData::new();
        let mut to = TestData::new();

        clone_data_with_handlers(0, &from, &mut to, |_, _, to| to.add_number(SimpleNumber::Integer(10)), |_, _, to| to.add_char(0)).unwrap();

        assert!(to.char_added);

        clone_data_with_handlers(1, &from, &mut to, |_, _, to| to.add_number(SimpleNumber::Integer(10)), |_, _, to| to.add_char(0)).unwrap();

        assert!(to.number_added);

        assert!(!to.unit_added);
    }

    #[test]
    fn copy_pair() {
        let mut from = SimpleGarnishData::new();
        let d1 = from.add_symbol(100).unwrap();
        let d2 = from.add_number(SimpleNumber::Integer(200)).unwrap();
        let d3 = from.add_pair((d1, d2)).unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(d3, &from, &mut to).unwrap();

        assert_eq!(new_addr, 8);
        assert_eq!(to.get_data().get(8).unwrap().as_pair().unwrap(), (6, 7));
        assert_eq!(to.get_data().get(6).unwrap().as_symbol().unwrap(), 100);
        assert_eq!(to.get_data().get(7).unwrap().as_number().unwrap(), SimpleNumber::Integer(200));
    }

    #[test]
    fn copy_concatenation() {
        let mut from = SimpleGarnishData::new();
        let d1 = from.add_symbol(100).unwrap();
        let d2 = from.add_number(SimpleNumber::Integer(200)).unwrap();
        let d3 = from.add_concatenation(d1, d2).unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(d3, &from, &mut to).unwrap();

        assert_eq!(new_addr, 8);
        assert_eq!(to.get_data().get(8).unwrap().as_concatenation().unwrap(), (6, 7));
        assert_eq!(to.get_data().get(6).unwrap().as_symbol().unwrap(), 100);
        assert_eq!(to.get_data().get(7).unwrap().as_number().unwrap(), SimpleNumber::Integer(200));
    }

    #[test]
    fn copy_list() {
        let mut from = SimpleGarnishData::new();
        from.start_list(3).unwrap();
        from.add_number(SimpleNumber::Integer(100)).and_then(|i| from.add_to_list(i, false)).unwrap();
        from.add_number(SimpleNumber::Integer(200)).and_then(|i| from.add_to_list(i, false)).unwrap();
        from.add_number(SimpleNumber::Integer(300)).and_then(|i| from.add_to_list(i, false)).unwrap();
        let d4 = from.end_list().unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(d4, &from, &mut to).unwrap();

        assert_eq!(new_addr, 9);
        assert_eq!(to.get_data().get(9).unwrap().as_list().unwrap(), (vec![6, 7, 8], vec![]));
        assert_eq!(to.get_data().get(6).unwrap().as_number().unwrap(), SimpleNumber::Integer(100));
        assert_eq!(to.get_data().get(7).unwrap().as_number().unwrap(), SimpleNumber::Integer(200));
        assert_eq!(to.get_data().get(8).unwrap().as_number().unwrap(), SimpleNumber::Integer(300));
    }

    #[test]
    fn copy_nested_list() {
        let mut from = SimpleGarnishData::new();
        from.start_list(3).unwrap();
        from.add_number(SimpleNumber::Integer(100)) // 6
            .and_then(|i| from.add_to_list(i, false))
            .unwrap();
        from.add_number(SimpleNumber::Integer(200)) // 7
            .and_then(|i| from.add_to_list(i, false))
            .unwrap();
        from.add_number(SimpleNumber::Integer(300)) // 8
            .and_then(|i| from.add_to_list(i, false))
            .unwrap();
        let d4 = from.end_list().unwrap();

        from.start_list(3).unwrap(); // 9

        from.add_number(SimpleNumber::Integer(400)) // 10
            .and_then(|i| from.add_to_list(i, false))
            .unwrap();
        from.add_number(SimpleNumber::Integer(500)) // 11
            .and_then(|i| from.add_to_list(i, false))
            .unwrap();
        from.add_number(SimpleNumber::Integer(600)) // 12
            .and_then(|i| from.add_to_list(i, false))
            .unwrap();

        let d5 = from.end_list().unwrap(); // 13

        let d6 = from.get_data_len(); // 14
        from.get_data_mut().push(SimpleData::List(vec![d4, d5], vec![]));

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(d6, &from, &mut to).unwrap();

        assert_eq!(new_addr, 14);
        assert_eq!(to.get_data().get(new_addr).unwrap().as_list().unwrap(), (vec![9, 13], vec![]));
        assert_eq!(to.get_data().get(9).unwrap().as_list().unwrap(), (vec![6, 7, 8], vec![]));
        assert_eq!(to.get_data().get(13).unwrap().as_list().unwrap(), (vec![10, 11, 12], vec![]));
        assert_eq!(to.get_data().get(6).unwrap().as_number().unwrap(), SimpleNumber::Integer(100));
        assert_eq!(to.get_data().get(7).unwrap().as_number().unwrap(), SimpleNumber::Integer(200));
        assert_eq!(to.get_data().get(8).unwrap().as_number().unwrap(), SimpleNumber::Integer(300));
        assert_eq!(to.get_data().get(10).unwrap().as_number().unwrap(), SimpleNumber::Integer(400));
        assert_eq!(to.get_data().get(11).unwrap().as_number().unwrap(), SimpleNumber::Integer(500));
        assert_eq!(to.get_data().get(12).unwrap().as_number().unwrap(), SimpleNumber::Integer(600));
    }

    #[test]
    fn copy_list_with_associations() {
        let mut from = SimpleGarnishData::new();
        from.start_list(3).unwrap();

        let left = from.add_symbol(200).unwrap();
        let right = from.add_number(SimpleNumber::Integer(100)).unwrap();
        from.add_pair((left, right)).and_then(|i| from.add_to_list(i, false)).unwrap();
        let d4 = from.end_list().unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(d4, &from, &mut to).unwrap();

        assert_eq!(new_addr, 9);
        assert_eq!(to.get_data().get(9).unwrap().as_list().unwrap(), (vec![8], vec![8]));
        assert_eq!(to.get_data().get(7).unwrap().as_number().unwrap(), SimpleNumber::Integer(100));
        assert_eq!(to.get_data().get(6).unwrap().as_symbol().unwrap(), 200);
    }

    #[test]
    fn copy_slice() {
        let mut from = SimpleGarnishData::new();
        let d1 = from.add_number(SimpleNumber::Integer(1)).unwrap();
        let d2 = from.add_number(SimpleNumber::Integer(3)).unwrap();
        let d3 = from.add_range(d1, d2).unwrap();
        from.start_list(3).unwrap();
        from.add_number(SimpleNumber::Integer(100)).and_then(|i| from.add_to_list(i, false)).unwrap();
        from.add_number(SimpleNumber::Integer(200)).and_then(|i| from.add_to_list(i, false)).unwrap();
        from.add_number(SimpleNumber::Integer(300)).and_then(|i| from.add_to_list(i, false)).unwrap();
        let d4 = from.end_list().unwrap();
        let d5 = from.add_slice(d4, d3).unwrap();

        let mut to = SimpleGarnishData::new();
        to.add_number(SimpleNumber::Integer(10)).unwrap();
        to.add_number(SimpleNumber::Integer(20)).unwrap();
        to.add_number(SimpleNumber::Integer(30)).unwrap();

        let new_addr = clone_data(d5, &from, &mut to).unwrap();

        assert_eq!(new_addr, 13);
        assert_eq!(to.get_data().get(13).unwrap().as_slice().unwrap(), (9, 12));
        assert_eq!(to.get_data().get(12).unwrap().as_range().unwrap(), (10, 11));
        assert_eq!(to.get_data().get(10).unwrap().as_number().unwrap(), SimpleNumber::Integer(1));
        assert_eq!(to.get_data().get(11).unwrap().as_number().unwrap(), SimpleNumber::Integer(3));
        assert_eq!(to.get_data().get(9).unwrap().as_list().unwrap(), (vec![6, 7, 8], vec![]));
        assert_eq!(to.get_data().get(6).unwrap().as_number().unwrap(), SimpleNumber::Integer(100));
        assert_eq!(to.get_data().get(7).unwrap().as_number().unwrap(), SimpleNumber::Integer(200));
        assert_eq!(to.get_data().get(8).unwrap().as_number().unwrap(), SimpleNumber::Integer(300));
    }
}

#[cfg(test)]
#[allow(unused)]
mod test_data_impl {
    use garnish_lang::simple::{DataError, NumberIterator, SimpleNumber, SizeIterator};
    use garnish_lang::{GarnishData, GarnishDataType, Instruction, SymbolListPart};

    pub struct TestData {
        pub unit_added: bool,
        pub number_added: bool,
        pub char_added: bool,
    }

    impl TestData {
        pub fn new() -> Self {
            Self {
                unit_added: false,
                number_added: false,
                char_added: false,
            }
        }
    }

    impl GarnishData for TestData {
        type Error = DataError;
        type Symbol = usize;
        type Byte = usize;
        type Char = usize;
        type Number = SimpleNumber;
        type Size = usize;
        type SizeIterator = SizeIterator;
        type NumberIterator = NumberIterator;
        type InstructionIterator = SizeIterator;
        type DataIndexIterator = SizeIterator;
        type ValueIndexIterator = SizeIterator;
        type RegisterIndexIterator = SizeIterator;
        type JumpTableIndexIterator = SizeIterator;
        type JumpPathIndexIterator = SizeIterator;
        type ListIndexIterator = NumberIterator;
        type ListItemIterator = SizeIterator;
        type ConcatenationItemIterator = SizeIterator;

        fn get_data_len(&self) -> Self::Size {
            unimplemented!()
        }

        fn get_data_iter(&self) -> Self::DataIndexIterator {
            unimplemented!()
        }

        fn get_value_stack_len(&self) -> Self::Size {
            unimplemented!()
        }

        fn push_value_stack(&mut self, addr: Self::Size) -> Result<(), Self::Error> {
            unimplemented!()
        }

        fn pop_value_stack(&mut self) -> Option<Self::Size> {
            unimplemented!()
        }

        fn get_value(&self, addr: Self::Size) -> Option<Self::Size> {
            unimplemented!()
        }

        fn get_value_mut(&mut self, addr: Self::Size) -> Option<&mut Self::Size> {
            unimplemented!()
        }

        fn get_current_value(&self) -> Option<Self::Size> {
            unimplemented!()
        }

        fn get_current_value_mut(&mut self) -> Option<&mut Self::Size> {
            unimplemented!()
        }

        fn get_value_iter(&self) -> Self::ValueIndexIterator {
            unimplemented!()
        }

        fn get_data_type(&self, addr: Self::Size) -> Result<GarnishDataType, Self::Error> {
            if addr == 1 { Ok(GarnishDataType::Custom) } else { Ok(GarnishDataType::Invalid) }
        }

        fn get_number(&self, addr: Self::Size) -> Result<Self::Number, Self::Error> {
            unimplemented!()
        }

        fn get_type(&self, addr: Self::Size) -> Result<GarnishDataType, Self::Error> {
            unimplemented!()
        }

        fn get_char(&self, addr: Self::Size) -> Result<Self::Char, Self::Error> {
            unimplemented!()
        }

        fn get_byte(&self, addr: Self::Size) -> Result<Self::Byte, Self::Error> {
            unimplemented!()
        }

        fn get_symbol(&self, addr: Self::Size) -> Result<Self::Symbol, Self::Error> {
            unimplemented!()
        }

        fn get_expression(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn get_external(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn get_pair(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
            unimplemented!()
        }

        fn get_concatenation(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
            unimplemented!()
        }

        fn get_range(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
            unimplemented!()
        }

        fn get_slice(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
            unimplemented!()
        }

        fn get_partial(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
            unimplemented!()
        }

        fn get_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn get_list_item(&self, list_addr: Self::Size, item_addr: Self::Number) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn get_list_associations_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn get_list_association(&self, list_addr: Self::Size, item_addr: Self::Number) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn get_list_item_with_symbol(&self, list_addr: Self::Size, sym: Self::Symbol) -> Result<Option<Self::Size>, Self::Error> {
            unimplemented!()
        }

        fn get_list_items_iter(&self, list_addr: Self::Size) -> Self::ListIndexIterator {
            unimplemented!()
        }

        fn get_list_associations_iter(&self, list_addr: Self::Size) -> Self::ListIndexIterator {
            unimplemented!()
        }

        fn get_char_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn get_char_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Self::Char, Self::Error> {
            unimplemented!()
        }

        fn get_char_list_iter(&self, list_addr: Self::Size) -> Self::ListIndexIterator {
            unimplemented!()
        }

        fn get_byte_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn get_byte_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Self::Byte, Self::Error> {
            unimplemented!()
        }

        fn get_byte_list_iter(&self, list_addr: Self::Size) -> Self::ListIndexIterator {
            unimplemented!()
        }

        fn get_symbol_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn get_symbol_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<SymbolListPart<Self::Symbol, Self::Number>, Self::Error> {
            unimplemented!()
        }

        fn get_symbol_list_iter(&self, list_addr: Self::Size) -> Self::ListIndexIterator {
            unimplemented!()
        }

        fn get_list_item_iter(&self, list_addr: Self::Size) -> Self::ListItemIterator {
            unimplemented!()
        }

        fn get_concatenation_iter(&self, addr: Self::Size) -> Self::ConcatenationItemIterator {
            unimplemented!()
        }

        fn get_slice_iter(&self, addr: Self::Size) -> Self::ListIndexIterator {
            unimplemented!()
        }

        fn get_list_slice_item_iter(&self, list_addr: Self::Size) -> Self::ListItemIterator {
            unimplemented!()
        }

        fn get_concatenation_slice_iter(&self, addr: Self::Size) -> Self::ConcatenationItemIterator {
            unimplemented!()
        }

        fn add_unit(&mut self) -> Result<Self::Size, Self::Error> {
            self.unit_added = true;
            Ok(0)
        }

        fn add_true(&mut self) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn add_false(&mut self) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn add_number(&mut self, value: Self::Number) -> Result<Self::Size, Self::Error> {
            self.number_added = true;
            Ok(0)
        }

        fn add_type(&mut self, value: GarnishDataType) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn add_char(&mut self, value: Self::Char) -> Result<Self::Size, Self::Error> {
            self.char_added = true;
            Ok(0)
        }

        fn add_byte(&mut self, value: Self::Byte) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn add_symbol(&mut self, value: Self::Symbol) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn add_expression(&mut self, value: Self::Size) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn add_external(&mut self, value: Self::Size) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn add_pair(&mut self, value: (Self::Size, Self::Size)) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn add_concatenation(&mut self, left: Self::Size, right: Self::Size) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn add_range(&mut self, start: Self::Size, end: Self::Size) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn add_slice(&mut self, list: Self::Size, range: Self::Size) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn add_partial(&mut self, list: Self::Size, range: Self::Size) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn merge_to_symbol_list(&mut self, first: Self::Size, second: Self::Size) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn start_list(&mut self, len: Self::Size) -> Result<(), Self::Error> {
            unimplemented!()
        }

        fn add_to_list(&mut self, addr: Self::Size, is_associative: bool) -> Result<(), Self::Error> {
            unimplemented!()
        }

        fn end_list(&mut self) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn start_char_list(&mut self) -> Result<(), Self::Error> {
            unimplemented!()
        }

        fn add_to_char_list(&mut self, c: Self::Char) -> Result<(), Self::Error> {
            unimplemented!()
        }

        fn end_char_list(&mut self) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn start_byte_list(&mut self) -> Result<(), Self::Error> {
            unimplemented!()
        }

        fn add_to_byte_list(&mut self, c: Self::Byte) -> Result<(), Self::Error> {
            unimplemented!()
        }

        fn end_byte_list(&mut self) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn get_register_len(&self) -> Self::Size {
            unimplemented!()
        }

        fn push_register(&mut self, addr: Self::Size) -> Result<(), Self::Error> {
            unimplemented!()
        }

        fn get_register(&self, addr: Self::Size) -> Option<Self::Size> {
            unimplemented!()
        }

        fn pop_register(&mut self) -> Result<Option<Self::Size>, Self::Error> {
            unimplemented!()
        }

        fn get_register_iter(&self) -> Self::RegisterIndexIterator {
            unimplemented!()
        }

        fn get_instruction_len(&self) -> Self::Size {
            unimplemented!()
        }

        fn push_instruction(&mut self, instruction: Instruction, data: Option<Self::Size>) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn get_instruction(&self, addr: Self::Size) -> Option<(Instruction, Option<Self::Size>)> {
            unimplemented!()
        }

        fn get_instruction_iter(&self) -> Self::InstructionIterator {
            unimplemented!()
        }

        fn get_instruction_cursor(&self) -> Self::Size {
            unimplemented!()
        }

        fn set_instruction_cursor(&mut self, addr: Self::Size) -> Result<(), Self::Error> {
            unimplemented!()
        }

        fn get_jump_table_len(&self) -> Self::Size {
            unimplemented!()
        }

        fn push_jump_point(&mut self, index: Self::Size) -> Result<(), Self::Error> {
            unimplemented!()
        }

        fn get_jump_point(&self, index: Self::Size) -> Option<Self::Size> {
            unimplemented!()
        }

        fn get_jump_point_mut(&mut self, index: Self::Size) -> Option<&mut Self::Size> {
            unimplemented!()
        }

        fn get_jump_table_iter(&self) -> Self::JumpTableIndexIterator {
            unimplemented!()
        }

        fn push_jump_path(&mut self, index: Self::Size) -> Result<(), Self::Error> {
            unimplemented!()
        }

        fn pop_jump_path(&mut self) -> Option<Self::Size> {
            unimplemented!()
        }

        fn get_jump_path_iter(&self) -> Self::JumpPathIndexIterator {
            unimplemented!()
        }

        fn size_to_number(from: Self::Size) -> Self::Number {
            unimplemented!()
        }

        fn number_to_size(from: Self::Number) -> Option<Self::Size> {
            unimplemented!()
        }

        fn number_to_char(from: Self::Number) -> Option<Self::Char> {
            unimplemented!()
        }

        fn number_to_byte(from: Self::Number) -> Option<Self::Byte> {
            unimplemented!()
        }

        fn char_to_number(from: Self::Char) -> Option<Self::Number> {
            unimplemented!()
        }

        fn char_to_byte(from: Self::Char) -> Option<Self::Byte> {
            unimplemented!()
        }

        fn byte_to_number(from: Self::Byte) -> Option<Self::Number> {
            unimplemented!()
        }

        fn byte_to_char(from: Self::Byte) -> Option<Self::Char> {
            unimplemented!()
        }

        fn add_char_list_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn add_byte_list_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn add_symbol_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn add_byte_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn add_number_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error> {
            unimplemented!()
        }

        fn parse_number(from: &str) -> Result<Self::Number, Self::Error> {
            unimplemented!()
        }

        fn parse_symbol(from: &str) -> Result<Self::Symbol, Self::Error> {
            unimplemented!()
        }

        fn parse_char(from: &str) -> Result<Self::Char, Self::Error> {
            unimplemented!()
        }

        fn parse_byte(from: &str) -> Result<Self::Byte, Self::Error> {
            unimplemented!()
        }

        fn parse_char_list(from: &str) -> Result<Vec<Self::Char>, Self::Error> {
            unimplemented!()
        }

        fn parse_byte_list(from: &str) -> Result<Vec<Self::Byte>, Self::Error> {
            unimplemented!()
        }

        fn make_size_iterator_range(min: Self::Size, max: Self::Size) -> Self::SizeIterator {
            unimplemented!()
        }

        fn make_number_iterator_range(min: Self::Number, max: Self::Number) -> Self::NumberIterator {
            unimplemented!()
        }
    }
}
