use garnish_lang_traits::GarnishDataType;

use crate::{BasicData, BasicDataCustom, BasicGarnishData, DataError};

impl<T> BasicGarnishData<T>
where
    T: BasicDataCustom,
{
    pub(crate) fn optimize_data_block(&mut self) -> Result<(), DataError> {
        let current_data_end = self.data_block().start + self.data_block().cursor;
        let true_start = self.data_block().start + self.data_retention_count();

        let offset = current_data_end - true_start;

        let mut registers = vec![];
        let mut current = self.current_register();
        while let Some(index) = current {
            let (previous, value) = self.get_from_data_block_ensure_index(index)?.as_register()?;
            registers.push(value);
            current = previous;
        }
        registers.reverse();

        let mut previous = None;
        for value in registers.iter() {
            let new_value = self.get_from_data_block_ensure_index(*value)?.clone();
            let new_value_index = self.push_to_data_block(new_value)?;
            let index = self.push_to_data_block(BasicData::Register(previous, new_value_index - offset))?;
            previous = Some(index - offset);
        }

        println!("{}", self.dump_data_block());

        let new_data_end = self.data_block().start + self.data_block().cursor;
        let from_range = current_data_end..new_data_end;
        let mut current = true_start;
        dbg!(current);

        for i in from_range.clone() {
            self.data_mut()[current] = self.data_mut()[i].clone();
            current += 1;
        }

        self.data_block_mut().cursor = current - self.data_block().start;
        dbg!(current);

        let clear_range = current..new_data_end;
        for i in clear_range.clone() {
            self.data_mut()[i] = BasicData::Empty;
        }

        dbg!(current_data_end, true_start, offset, registers, from_range, clear_range, current);

        Ok(())
    }

    pub(crate) fn push_clone_data(&mut self, from: usize) -> Result<usize, DataError> {
        let index_stack_start = self.create_index_stack(from)?;
        self.clone_index_stack(index_stack_start, 0)
    }

    fn create_index_stack(&mut self, from: usize) -> Result<usize, DataError> {
        let start = self.push_to_data_block(BasicData::CloneItem(from))?;
        let mut current = start;

        while current < self.data_block().cursor {
            let index = self.get_from_data_block_ensure_index(current)?.as_clone_item()?;

            dbg!(current, index);
            
            match self.get_from_data_block_ensure_index(index)? {
                BasicData::Unit => {}
                BasicData::True => {},
                BasicData::False => {},
                BasicData::Pair(left, right) => {
                    let (left, right) = (left.clone(), right.clone());
                    self.push_to_data_block(BasicData::CloneItem(right))?;
                    self.push_to_data_block(BasicData::CloneItem(left))?;
                }
                _ => todo!(),
            }

            current += 1;
        }
        
        Ok(start)
    }

    fn clone_index_stack(&mut self, top_index: usize, _offset: usize) -> Result<usize, DataError> {
        let range = top_index..self.data_block().cursor;

        for i in range.rev() {
            let index = self.get_from_data_block_ensure_index(i)?.as_clone_item()?;

            let new_index = match self.get_from_data_block_ensure_index(index)?.clone() {
                BasicData::Unit => {
                    self.push_to_data_block(BasicData::Unit)?
                }
                BasicData::True => {
                    self.push_to_data_block(BasicData::True)?
                }
                BasicData::False => {
                    self.push_to_data_block(BasicData::False)?
                }
                BasicData::Type(t) => {
                    self.push_to_data_block(BasicData::Type(t))?
                }
                BasicData::Number(n) => {
                    self.push_to_data_block(BasicData::Number(n))?
                }
                BasicData::Char(c) => {
                    self.push_to_data_block(BasicData::Char(c))?
                }
                BasicData::Byte(b) => {
                    self.push_to_data_block(BasicData::Byte(b))?
                }
                BasicData::Symbol(s) => {
                    self.push_to_data_block(BasicData::Symbol(s))?
                }
                BasicData::SymbolList(len) => {
                    self.push_to_data_block(BasicData::SymbolList(len))?
                }
                BasicData::Expression(e) => {
                    self.push_to_data_block(BasicData::Expression(e))?
                }
                BasicData::External(e) => {
                    self.push_to_data_block(BasicData::External(e))?
                }
                BasicData::CharList(len) => {
                    self.push_to_data_block(BasicData::CharList(len))?
                }
                BasicData::ByteList(len) => {
                    self.push_to_data_block(BasicData::ByteList(len))?
                }
                BasicData::Pair(_, _) => {
                    let new_index = self.data_block().cursor;
                    self.push_to_data_block(BasicData::Pair(new_index - 2, new_index - 1))?
                }
                BasicData::Range(_, _) => {
                    let new_index = self.data_block().cursor;
                    self.push_to_data_block(BasicData::Range(new_index - 2, new_index - 1))?
                }
                BasicData::Slice(_, _) => {
                    let new_index = self.data_block().cursor;
                    self.push_to_data_block(BasicData::Slice(new_index - 2, new_index - 1))?
                }
                BasicData::Partial(_, _) => {
                    let new_index = self.data_block().cursor;
                    self.push_to_data_block(BasicData::Partial(new_index - 2, new_index - 1))?
                }
                BasicData::List(_, _) => {
                    todo!()
                }
                BasicData::Concatenation(_, _) => {
                    let new_index = self.data_block().cursor;
                    self.push_to_data_block(BasicData::Concatenation(new_index - 2, new_index - 1))?
                }
                BasicData::Custom(_) => {
                    todo!()
                }
                BasicData::Empty => {
                    todo!()
                }
                BasicData::UninitializedList(_, _) => {
                    todo!()
                }
                BasicData::ListItem(_) => {
                    todo!()
                }
                BasicData::AssociativeItem(_, _) => {
                    todo!()
                }
                BasicData::Value(_, _) => {
                    todo!()
                }
                BasicData::Register(_, _) => {
                    todo!()
                }
                BasicData::Instruction(_, _) => {
                    todo!()
                }
                BasicData::JumpPoint(_) => {
                    todo!()
                }
                BasicData::Frame(_, _) => {
                    todo!()
                }
                BasicData::CloneNodeNew(_, _) => {
                    todo!()
                }
                BasicData::CloneNodeVisited(_, _) => {
                    todo!()
                }
                BasicData::CloneItem(_) => {
                    todo!()
                }
                BasicData::CloneIndexMap(_, _) => {
                    todo!()
                }
            };

            *self.get_from_data_block_ensure_index_mut(i)? = BasicData::CloneIndexMap(index, new_index);
        }

        let (_original, new) = self.get_from_data_block_ensure_index(top_index)?.as_clone_index_map()?;

        Ok(new)
    }
}

#[cfg(test)]
mod optimize {
    use crate::{BasicData, basic_object};

    use super::*;

    #[test]
    fn default_with_data_all_data_removed() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        data.push_object_to_data_block(basic_object!(Unit, (Number 100), (CharList "hello"), ((ByteList 1, 2, 3) = (Symbol "my_symbol"))))
            .unwrap();
        data.optimize_data_block().unwrap();
        assert_eq!(data.data_size(), 0);
    }

    #[test]
    fn data_within_retention_count_is_kept() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        data.push_object_to_data_block(basic_object!(Unit, (Number 100), (CharList "hello"), ((ByteList 1, 2, 3) = (Symbol "my_symbol"))))
            .unwrap();
        data.retain_all_current_data();
        data.push_object_to_data_block(basic_object!((Number 1234), (CharList "world"))).unwrap();
        data.optimize_data_block().unwrap();
        
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().resize(80, BasicData::Empty);
        expected_data.data_mut().splice(30..49, vec![
            BasicData::Unit,
            BasicData::Number(100.into()),
            BasicData::CharList(5),
            BasicData::Char('h'),
            BasicData::Char('e'),
            BasicData::Char('l'),
            BasicData::Char('l'),
            BasicData::Char('o'),
            BasicData::ByteList(3),
            BasicData::Byte(1),
            BasicData::Byte(2),
            BasicData::Byte(3),
            BasicData::Symbol(8904929874702161741),
            BasicData::Pair(8, 12),
            BasicData::List(4, 0),
            BasicData::ListItem(0),
            BasicData::ListItem(1),
            BasicData::ListItem(2),
            BasicData::ListItem(13),
        ]);

        expected_data.set_data_retention_count(23);
        expected_data.data_block_mut().cursor = 23;
        expected_data.data_block_mut().size = 40;
        expected_data.data_block_mut().start = 30;
        expected_data.custom_data_block_mut().start = 70;
        expected_data.custom_data_block_mut().size = 10;

        assert_eq!(data, expected_data);
    }

    // #[test]
    // fn data_referenced_by_registers_is_kept() {
    //     let mut data = BasicGarnishData::<()>::new().unwrap();
    //     let index = data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
    //     let register_index = data.push_to_data_block(BasicData::Register(None, index)).unwrap();
    //     data.push_object_to_data_block(basic_object!((Number 1234), (CharList "world"))).unwrap();
    //     let index = data.push_object_to_data_block(basic_object!((Number 1234) = (Char 'a'))).unwrap();
    //     let register_index = data.push_to_data_block(BasicData::Register(Some(register_index), index)).unwrap();
    //     data.set_current_register(Some(register_index));

    //     println!("{}", data.dump_data_block());

    //     data.optimize_data_block().unwrap();
    
    //     let mut expected_data = BasicGarnishData::<()>::new().unwrap();
    //     expected_data.data_mut().resize(70, BasicData::Empty);
    //     expected_data.data_mut().splice(30..36, vec![
    //         BasicData::Number(100.into()),
    //         BasicData::Register(None, 0),
    //         BasicData::Number(1234.into()),
    //         BasicData::Char('a'),
    //         BasicData::Pair(2, 3),
    //         BasicData::Register(Some(1), 4),
    //     ]);

    //     expected_data.data_block_mut().cursor = 6;
    //     expected_data.set_current_register(Some(5));

    //     println!("{}", data.dump_data_block());

    //     assert_eq!(data, expected_data);
    // }
}

#[cfg(test)]
mod clone {
    use garnish_lang_traits::GarnishDataType;

    use crate::{BasicData, BasicGarnishData, basic_object};

    #[test]
    fn unit() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(Unit)).unwrap();
        let index = data.push_clone_data(index).unwrap();
        
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(30..33, vec![
            BasicData::Unit,
            BasicData::CloneIndexMap(0, 2),
            BasicData::Unit,
        ]);
        expected_data.data_block_mut().cursor = 3;
        
        assert_eq!(index, 2);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn true_value() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(True)).unwrap();
        let index = data.push_clone_data(index).unwrap();
        
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(30..33, vec![
            BasicData::True,
            BasicData::CloneIndexMap(0, 2),
            BasicData::True,
        ]);
        expected_data.data_block_mut().cursor = 3;
        
        assert_eq!(index, 2);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn false_value() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(False)).unwrap();
        let index = data.push_clone_data(index).unwrap();
        
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(30..33, vec![
            BasicData::False,
            BasicData::CloneIndexMap(0, 2),
            BasicData::False,
        ]);
        expected_data.data_block_mut().cursor = 3;
        
        assert_eq!(index, 2);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn type_value() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(Type Number)).unwrap();
        let index = data.push_clone_data(index).unwrap();
        
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(30..34, vec![
            BasicData::Type(GarnishDataType::Number),
            BasicData::CloneNodeNew(None, 0),
            BasicData::Value(None, 0),
            BasicData::Type(GarnishDataType::Number),
        ]);
        expected_data.data_block_mut().cursor = 4;
        
        assert_eq!(index, 1);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn char() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(Char 'a')).unwrap();
        let index = data.push_clone_data(index).unwrap();
        
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(30..34, vec![
            BasicData::Char('a'),
            BasicData::CloneNodeNew(None, 0),
            BasicData::Value(None, 0),
            BasicData::Char('a'),
        ]);
        expected_data.data_block_mut().cursor = 4;
        
        assert_eq!(index, 1);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn byte() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(Byte 1)).unwrap();
        let index = data.push_clone_data(index).unwrap();
        
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(30..34, vec![
            BasicData::Byte(1),
            BasicData::CloneNodeNew(None, 0),
            BasicData::Value(None, 0),
            BasicData::Byte(1),
        ]);
        expected_data.data_block_mut().cursor = 4;
        
        assert_eq!(index, 1);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn symbol() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(SymRaw 100)).unwrap();
        let index = data.push_clone_data(index).unwrap();
        
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(30..34, vec![
            BasicData::Symbol(100),
            BasicData::CloneNodeNew(None, 0),
            BasicData::Value(None, 0),
            BasicData::Symbol(100),
        ]);
        expected_data.data_block_mut().cursor = 4;
        
        assert_eq!(index, 1);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn number() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        let index = data.push_clone_data(index).unwrap();
        
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(30..34, vec![
            BasicData::Number(100.into()),
            BasicData::CloneNodeNew(None, 0),
            BasicData::Value(None, 0),
            BasicData::Number(100.into()),
        ]);
        expected_data.data_block_mut().cursor = 4;
        
        assert_eq!(index, 1);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn expression() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(Expression 100)).unwrap();
        let index = data.push_clone_data(index).unwrap();
        
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(30..34, vec![
            BasicData::Expression(100),
            BasicData::CloneNodeNew(None, 0),
            BasicData::Value(None, 0),
            BasicData::Expression(100),
        ]);
        expected_data.data_block_mut().cursor = 4;
        
        assert_eq!(index, 1);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn external() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(External 100)).unwrap();
        let index = data.push_clone_data(index).unwrap();
        
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(30..34, vec![
            BasicData::External(100),
            BasicData::CloneNodeNew(None, 0),
            BasicData::Value(None, 0),
            BasicData::External(100),
        ]);
        expected_data.data_block_mut().cursor = 4;
        
        assert_eq!(index, 1);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn symbol_list() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(SymList(SymRaw 100, Number 200))).unwrap();
        let index = data.push_clone_data(index).unwrap();
        
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(30..40, vec![
            BasicData::SymbolList(2),
            BasicData::Symbol(100),
            BasicData::Number(200.into()),
            BasicData::CloneNodeNew(None, 0),
            BasicData::Value(None, 2),
            BasicData::Value(Some(4), 1),
            BasicData::Value(Some(5), 0),
            BasicData::SymbolList(2),
            BasicData::Symbol(100),
            BasicData::Number(200.into()),
        ]);
        expected_data.data_block_mut().cursor = 10;
        
        assert_eq!(index, 0);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn char_list() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(CharList "hello")).unwrap();
        let index = data.push_clone_data(index).unwrap();
        
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().resize(60, BasicData::Empty);
        expected_data.data_mut().splice(30..49, vec![
            BasicData::CharList(5),
            BasicData::Char('h'),
            BasicData::Char('e'),
            BasicData::Char('l'),
            BasicData::Char('l'),
            BasicData::Char('o'),
            BasicData::CloneNodeNew(None, 0),
            BasicData::Value(None, 5),
            BasicData::Value(Some(7), 4),
            BasicData::Value(Some(8), 3),
            BasicData::Value(Some(9), 2),
            BasicData::Value(Some(10), 1),
            BasicData::Value(Some(11), 0),
            BasicData::CharList(5),
            BasicData::Char('h'),
            BasicData::Char('e'),
            BasicData::Char('l'),
            BasicData::Char('l'),
            BasicData::Char('o'),
        ]);
        expected_data.data_block_mut().cursor = 19;
        expected_data.data_block_mut().size = 20;
        expected_data.custom_data_block_mut().start = 50;
        
        assert_eq!(index, 6);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn byte_list() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(ByteList 100, 200, 250)).unwrap();
        let index = data.push_clone_data(index).unwrap();
        
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().resize(60, BasicData::Empty);
        expected_data.data_mut().splice(30..43, vec![
            BasicData::ByteList(3),
            BasicData::Byte(100),
            BasicData::Byte(200),
            BasicData::Byte(250),
            BasicData::CloneNodeNew(None, 0),
            BasicData::Value(None, 3),
            BasicData::Value(Some(5), 2),
            BasicData::Value(Some(6), 1),
            BasicData::Value(Some(7), 0),
            BasicData::ByteList(3),
            BasicData::Byte(100),
            BasicData::Byte(200),
            BasicData::Byte(250),
        ]);
        expected_data.data_block_mut().cursor = 13;
        expected_data.data_block_mut().size = 20;
        expected_data.custom_data_block_mut().start = 50;
        expected_data.custom_data_block_mut().size = 10;
        
        assert_eq!(index, 4);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn pair() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!((True = False))).unwrap();

        let index = data.push_clone_data(index).unwrap();
        
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(30..39, vec![
            BasicData::True,
            BasicData::False,
            BasicData::Pair(0, 1),
            BasicData::CloneIndexMap(2, 8),
            BasicData::CloneIndexMap(1, 7),
            BasicData::CloneIndexMap(0, 6),
            BasicData::True,
            BasicData::False,
            BasicData::Pair(6, 7),
        ]);
        expected_data.data_block_mut().cursor = 9;

        assert_eq!(index, 8);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn pair_with_offset() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!((True = False))).unwrap();

        let index = data.push_clone_data(index).unwrap();
        
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().resize(60, BasicData::Empty);
        expected_data.data_mut().splice(30..43, vec![
            BasicData::True,
            BasicData::False,
            BasicData::Pair(0, 1),
            BasicData::CloneNodeNew(None, 2),
            BasicData::CloneNodeNew(None, 0),
            BasicData::CloneNodeNew(Some(4), 1),
            BasicData::CloneNodeVisited(Some(5), 2),
            BasicData::Value(None, 2),
            BasicData::Value(Some(7), 1),
            BasicData::Value(Some(8), 0),
            BasicData::True,
            BasicData::False,
            BasicData::Pair(0, 1),
        ]);
        expected_data.data_block_mut().cursor = 13;
        expected_data.data_block_mut().size = 20;
        expected_data.custom_data_block_mut().start = 50;
        expected_data.custom_data_block_mut().size = 10;

        assert_eq!(index, 2);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn range() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(((Number 100)..(Number 200)))).unwrap();

        let index = data.push_clone_data(index).unwrap();
        
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().resize(60, BasicData::Empty);
        expected_data.data_mut().splice(30..43, vec![
            BasicData::Number(100.into()),
            BasicData::Number(200.into()),
            BasicData::Range(0, 1),
            BasicData::CloneNodeNew(None, 2),
            BasicData::CloneNodeNew(None, 0),
            BasicData::CloneNodeNew(Some(4), 1),
            BasicData::CloneNodeVisited(Some(5), 2),
            BasicData::Value(None, 2),
            BasicData::Value(Some(7), 1),
            BasicData::Value(Some(8), 0),
            BasicData::Number(100.into()),
            BasicData::Number(200.into()),
            BasicData::Range(3, 4),
        ]);
        expected_data.data_block_mut().cursor = 13;
        expected_data.data_block_mut().size = 20;
        expected_data.custom_data_block_mut().start = 50;
        expected_data.custom_data_block_mut().size = 10;

        assert_eq!(index, 5);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn slice() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(((Number 100) - (Number 200)))).unwrap();

        let index = data.push_clone_data(index).unwrap();
        
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().resize(60, BasicData::Empty);
        expected_data.data_mut().splice(30..43, vec![
            BasicData::Number(100.into()),
            BasicData::Number(200.into()),
            BasicData::Slice(0, 1),
            BasicData::CloneNodeNew(None, 2),
            BasicData::CloneNodeNew(None, 0),
            BasicData::CloneNodeNew(Some(4), 1),
            BasicData::CloneNodeVisited(Some(5), 2),
            BasicData::Value(None, 2),
            BasicData::Value(Some(7), 1),
            BasicData::Value(Some(8), 0),
            BasicData::Number(100.into()),
            BasicData::Number(200.into()),
            BasicData::Slice(3, 4),
        ]);
        expected_data.data_block_mut().cursor = 13;
        expected_data.data_block_mut().size = 20;
        expected_data.custom_data_block_mut().start = 50;
        expected_data.custom_data_block_mut().size = 10;

        assert_eq!(index, 5);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn partial() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(((Number 100) ~ (Number 200)))).unwrap();

        let index = data.push_clone_data(index).unwrap();
        
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().resize(60, BasicData::Empty);
        expected_data.data_mut().splice(30..43, vec![
            BasicData::Number(100.into()),
            BasicData::Number(200.into()),
            BasicData::Partial(0, 1),
            BasicData::CloneNodeNew(None, 2),
            BasicData::CloneNodeNew(None, 0),
            BasicData::CloneNodeNew(Some(4), 1),
            BasicData::CloneNodeVisited(Some(5), 2),
            BasicData::Value(None, 2),
            BasicData::Value(Some(7), 1),
            BasicData::Value(Some(8), 0),
            BasicData::Number(100.into()),
            BasicData::Number(200.into()),
            BasicData::Partial(3, 4),
        ]);
        expected_data.data_block_mut().cursor = 13;
        expected_data.data_block_mut().size = 20;
        expected_data.custom_data_block_mut().start = 50;
        expected_data.custom_data_block_mut().size = 10;

        assert_eq!(index, 5);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn concatenation() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(((Number 100) <> (Number 200)))).unwrap();

        let index = data.push_clone_data(index).unwrap();
        
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().resize(60, BasicData::Empty);
        expected_data.data_mut().splice(30..43, vec![
            BasicData::Number(100.into()),
            BasicData::Number(200.into()),
            BasicData::Concatenation(0, 1),
            BasicData::CloneNodeNew(None, 2),
            BasicData::CloneNodeNew(None, 0),
            BasicData::CloneNodeNew(Some(4), 1),
            BasicData::CloneNodeVisited(Some(5), 2),
            BasicData::Value(None, 2),
            BasicData::Value(Some(7), 1),
            BasicData::Value(Some(8), 0),
            BasicData::Number(100.into()),
            BasicData::Number(200.into()),
            BasicData::Concatenation(3, 4),
        ]);
        expected_data.data_block_mut().cursor = 13;
        expected_data.data_block_mut().size = 20;
        expected_data.custom_data_block_mut().start = 50;
        expected_data.custom_data_block_mut().size = 10;

        assert_eq!(index, 5);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn list() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!((Number 100), (Number 200), (Number 250))).unwrap();

        let index = data.push_clone_data(index).unwrap();
        
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().resize(60, BasicData::Empty);
        expected_data.data_mut().splice(30..43, vec![
            BasicData::Number(100.into()),
            BasicData::Number(200.into()),
            BasicData::Number(250.into()),
            BasicData::List(0, 3),
            BasicData::ListItem(0),
            BasicData::ListItem(1),
            BasicData::ListItem(2),
            BasicData::Empty,
            BasicData::Empty,
            BasicData::Empty,
            BasicData::CloneNodeNew(None, 2),
            BasicData::CloneNodeNew(None, 0),
            BasicData::CloneNodeNew(Some(4), 1),
            BasicData::CloneNodeVisited(Some(5), 2),
            BasicData::Value(None, 2),
            BasicData::Value(Some(7), 1),
            BasicData::Value(Some(8), 0),
            BasicData::Number(100.into()),
            BasicData::Number(200.into()),
            BasicData::Concatenation(3, 4),
        ]);
        expected_data.data_block_mut().cursor = 13;
        expected_data.data_block_mut().size = 20;
        expected_data.custom_data_block_mut().start = 50;
        expected_data.custom_data_block_mut().size = 10;

        assert_eq!(index, 5);
        assert_eq!(data, expected_data);
    }
}
