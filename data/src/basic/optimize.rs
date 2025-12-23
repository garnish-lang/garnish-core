use garnish_lang_traits::GarnishDataType;

use crate::{BasicData, BasicDataCustom, BasicGarnishData, DataError, error::DataErrorType};

#[derive(Debug)]
enum Evaluate {
    Visited,
    New,
}

#[derive(Debug)]
struct EvaluateNode {
    index: usize,
    state: Evaluate,
}

impl EvaluateNode {
    fn new(index: usize) -> Self {
        Self { index, state: Evaluate::New }
    }

    fn visited(self) -> Self{
        Self { index: self.index, state: Evaluate::Visited }
    }
}

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

    pub(crate) fn push_clone_data_from(&mut self, from: usize, offset: usize) -> Result<usize, DataError> {
        let start_index = self.data_block().cursor;
        let mut index_stack = vec![];
        let mut top_node = Some(self.push_to_data_block(BasicData::CloneNodeNew(None, from))?);

        while let Some(clone_node_index) = top_node {
            let (previous, index, state) = match self.get_from_data_block_ensure_index(clone_node_index)?.clone() {
                BasicData::CloneNodeNew(previous, index) => (previous, index, Evaluate::New),
                BasicData::CloneNodeVisited(previous, index) => (previous, index, Evaluate::Visited),
                _ => return Err(DataError::new("Not clone node", DataErrorType::NotACloneNode)),
            };

            match self.get_from_data_block_ensure_index(index)? {
                BasicData::Unit => {
                    index_stack.push(index);
                    top_node = previous;
                }
                BasicData::True => {
                    index_stack.push(index);
                    top_node = previous;
                }
                BasicData::False => {
                    index_stack.push(index);
                    top_node = previous;
                }
                BasicData::Type(_) => {
                    todo!()
                }
                BasicData::Number(_) => {
                    todo!()
                }
                BasicData::Char(_) => {
                    todo!()
                }
                BasicData::Byte(_) => {
                    todo!()
                }
                BasicData::Symbol(_) => {
                    todo!()
                }
                BasicData::SymbolList(_) => {
                    todo!()
                }
                BasicData::Expression(_) => {
                    todo!()
                }
                BasicData::External(_) => {
                    todo!()
                }
                BasicData::CharList(_) => {
                    todo!()
                }
                BasicData::ByteList(_) => {
                    todo!()
                }
                BasicData::Pair(left, right) => match state {
                    Evaluate::New => {
                        let (left, right) = (left.clone(), right.clone());
                        let index = self.push_to_data_block(BasicData::CloneNodeVisited(previous, index))?;
                        let index = self.push_to_data_block(BasicData::CloneNodeNew(Some(index), right))?;
                        let index = self.push_to_data_block(BasicData::CloneNodeNew(Some(index), left))?;
                        top_node = Some(index);
                    }
                    Evaluate::Visited => {
                        index_stack.push(index);
                        top_node = previous;
                    }
                },
                BasicData::Range(_, _) => {
                    todo!()
                }
                BasicData::Slice(_, _) => {
                    todo!()
                }
                BasicData::Partial(_, _) => {
                    todo!()
                }
                BasicData::List(_, _) => {
                    todo!()
                }
                BasicData::Concatenation(_, _) => {
                    todo!()
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
            }
        }

        let addtional_items = self.data_block().cursor - start_index;
        let full_offset = offset + addtional_items;

        let mut value = from;
        for index in index_stack {
            value = match self.get_from_data_block_ensure_index(index)? {
                BasicData::Unit => {
                    self.push_to_data_block(BasicData::Unit)?
                }
                BasicData::True => {
                    self.push_to_data_block(BasicData::True)?
                }
                BasicData::False => {
                    self.push_to_data_block(BasicData::False)?
                }
                BasicData::Type(_) => {
                    todo!()
                }
                BasicData::Number(_) => {
                    todo!()
                }
                BasicData::Char(_) => {
                    todo!()
                }
                BasicData::Byte(_) => {
                    todo!()
                }
                BasicData::Symbol(_) => {
                    todo!()
                }
                BasicData::SymbolList(_) => {
                    todo!()
                }
                BasicData::Expression(_) => {
                    todo!()
                }
                BasicData::External(_) => {
                    todo!()
                }
                BasicData::CharList(_) => {
                    todo!()
                }
                BasicData::ByteList(_) => {
                    todo!()
                }
                BasicData::Pair(_, _) => {
                    let new_index = self.data_block().cursor;
                    self.push_to_data_block(BasicData::Pair(new_index - 2 - full_offset, new_index - 1 - full_offset))?
                }
                BasicData::Range(_, _) => {
                    todo!()
                }
                BasicData::Slice(_, _) => {
                    todo!()
                }
                BasicData::Partial(_, _) => {
                    todo!()
                }
                BasicData::List(_, _) => {
                    todo!()
                }
                BasicData::Concatenation(_, _) => {
                    todo!()
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
            }
        }

        Ok(value - full_offset)
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
    use crate::{BasicData, BasicGarnishData, basic_object};

    #[test]
    fn unit() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(Unit)).unwrap();
        let index = data.push_clone_data_from(index, 0).unwrap();
        
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(30..33, vec![
            BasicData::Unit,
            BasicData::CloneNodeNew(None, 0),
            BasicData::Unit,
        ]);
        expected_data.data_block_mut().cursor = 3;
        
        assert_eq!(index, 1);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn pair() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!((True = False))).unwrap();

        let index = data.push_clone_data_from(index, 0).unwrap();
        
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(30..40, vec![
            BasicData::True,
            BasicData::False,
            BasicData::Pair(0, 1),
            BasicData::CloneNodeNew(None, 2),
            BasicData::CloneNodeVisited(None, 2),
            BasicData::CloneNodeNew(Some(4), 1),
            BasicData::CloneNodeNew(Some(5), 0),
            BasicData::True,
            BasicData::False,
            BasicData::Pair(3, 4),
        ]);
        expected_data.data_block_mut().cursor = 10;

        assert_eq!(index, 5);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn pair_with_offset() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!((True = False))).unwrap();

        let index = data.push_clone_data_from(index, 3).unwrap();
        
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(30..40, vec![
            BasicData::True,
            BasicData::False,
            BasicData::Pair(0, 1),
            BasicData::CloneNodeNew(None, 2),
            BasicData::CloneNodeVisited(None, 2),
            BasicData::CloneNodeNew(Some(4), 1),
            BasicData::CloneNodeNew(Some(5), 0),
            BasicData::True,
            BasicData::False,
            BasicData::Pair(0, 1),
        ]);
        expected_data.data_block_mut().cursor = 10;

        println!("{}", data.dump_data_block());

        assert_eq!(index, 2);
        assert_eq!(data, expected_data);
    }
}
