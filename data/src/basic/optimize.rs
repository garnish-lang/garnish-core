use garnish_lang_traits::GarnishDataType;

use crate::{BasicData, BasicDataCustom, BasicGarnishData, DataError, error::DataErrorType};

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
            println!("{}", self.dump_data_block());

            match self.get_from_data_block_ensure_index(index)? {
                BasicData::Unit => {}
                BasicData::True => {}
                BasicData::False => {}
                BasicData::Type(_) => {}
                BasicData::Number(_) => {}
                BasicData::Char(_) => {}
                BasicData::Byte(_) => {}
                BasicData::Symbol(_) => {}
                BasicData::Expression(_) => {}
                BasicData::External(_) => {}
                BasicData::SymbolList(_) => {}
                BasicData::CharList(_) => {}
                BasicData::ByteList(_) => {}
                BasicData::Pair(left, right) => {
                    let (left, right) = (left.clone(), right.clone());
                    self.push_to_data_block(BasicData::CloneItem(right))?;
                    self.push_to_data_block(BasicData::CloneItem(left))?;
                }
                BasicData::Range(left, right) => {
                    let (left, right) = (left.clone(), right.clone());
                    self.push_to_data_block(BasicData::CloneItem(right))?;
                    self.push_to_data_block(BasicData::CloneItem(left))?;
                }
                BasicData::Slice(left, right) => {
                    let (left, right) = (left.clone(), right.clone());
                    self.push_to_data_block(BasicData::CloneItem(right))?;
                    self.push_to_data_block(BasicData::CloneItem(left))?;
                }
                BasicData::Partial(left, right) => {
                    let (left, right) = (left.clone(), right.clone());
                    self.push_to_data_block(BasicData::CloneItem(right))?;
                    self.push_to_data_block(BasicData::CloneItem(left))?;
                }
                BasicData::Concatenation(left, right) => {
                    let (left, right) = (left.clone(), right.clone());
                    self.push_to_data_block(BasicData::CloneItem(right))?;
                    self.push_to_data_block(BasicData::CloneItem(left))?;
                }
                BasicData::List(length, _association_length) => {
                    let start = index + 1;
                    let end = start + length;
                    for i in start..end {
                        let item = self.get_from_data_block_ensure_index(i)?.as_list_item()?;
                        self.push_to_data_block(BasicData::CloneItem(item))?;
                    }
                }
                _ => todo!(),
            }

            current += 1;
        }

        Ok(start)
    }

    fn clone_index_stack(&mut self, top_index: usize, _offset: usize) -> Result<usize, DataError> {
        let clone_range = top_index..self.data_block().cursor;

        let lookup_end = self.data_block().start + self.data_block().cursor;
        let mut lookup_start = lookup_end;

        println!("===== pre clone ======");
        println!("{}", self.dump_data_block());

        for i in clone_range.rev() {
            let index = self.get_from_data_block_ensure_index(i)?.as_clone_item()?;
            let new_index = self.data_block().cursor;

            let new_index = match self.get_from_data_block_ensure_index(index)?.clone() {
                BasicData::Unit => self.push_to_data_block(BasicData::Unit)?,
                BasicData::True => self.push_to_data_block(BasicData::True)?,
                BasicData::False => self.push_to_data_block(BasicData::False)?,
                BasicData::Type(t) => self.push_to_data_block(BasicData::Type(t))?,
                BasicData::Number(n) => self.push_to_data_block(BasicData::Number(n))?,
                BasicData::Char(c) => self.push_to_data_block(BasicData::Char(c))?,
                BasicData::Byte(b) => self.push_to_data_block(BasicData::Byte(b))?,
                BasicData::Symbol(s) => self.push_to_data_block(BasicData::Symbol(s))?,
                BasicData::SymbolList(len) => {
                    let start = index + 1;
                    let end = start + len;
                    let list_index = self.push_to_data_block(BasicData::SymbolList(len))?;
                    for i in start..end {
                        let item = self.get_from_data_block_ensure_index(i)?.clone();
                        self.push_to_data_block(item)?;
                    }
                    list_index
                }
                BasicData::Expression(e) => self.push_to_data_block(BasicData::Expression(e))?,
                BasicData::External(e) => self.push_to_data_block(BasicData::External(e))?,
                BasicData::CharList(len) => {
                    let start = index + 1;
                    let end = start + len;
                    let list_index = self.push_to_data_block(BasicData::CharList(len))?;
                    for i in start..end {
                        let item = self.get_from_data_block_ensure_index(i)?.clone();
                        self.push_to_data_block(item)?;
                    }
                    list_index
                }
                BasicData::ByteList(len) => {
                    let start = index + 1;
                    let end = start + len;
                    let list_index = self.push_to_data_block(BasicData::ByteList(len))?;
                    for i in start..end {
                        let item = self.get_from_data_block_ensure_index(i)?.clone();
                        self.push_to_data_block(item)?;
                    }
                    list_index
                }
                BasicData::Pair(left, right) => {
                    let left = self.lookup_in_data_slice(lookup_start, lookup_end, left)?;
                    let right = self.lookup_in_data_slice(lookup_start, lookup_end, right)?;

                    self.push_to_data_block(BasicData::Pair(left, right))?
                }
                BasicData::Range(_, _) => self.push_to_data_block(BasicData::Range(new_index - 2, new_index - 1))?,
                BasicData::Slice(_, _) => self.push_to_data_block(BasicData::Slice(new_index - 2, new_index - 1))?,
                BasicData::Partial(_, _) => self.push_to_data_block(BasicData::Partial(new_index - 2, new_index - 1))?,
                BasicData::List(length, association_length) => {
                    let start = index + 1;
                    let end = start + length;
                    let list_index = self.push_to_data_block(BasicData::List(length, association_length))?;
                    for i in start..end {
                        let item = self.get_from_data_block_ensure_index(i)?.as_list_item()?;
                        let item = self.lookup_in_data_slice(lookup_start, lookup_end, item)?;
                        self.push_to_data_block(BasicData::ListItem(item))?;
                    }

                    let association_end = end + association_length;
                    for _ in end..association_end {
                        self.push_to_data_block(BasicData::Empty)?;
                    }
                    
                    let full_end = index + 1 + length * 2;
                    for _ in association_end..full_end {
                        self.push_to_data_block(BasicData::Empty)?;
                    }

                    list_index
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

            lookup_start -= 1;
        }

        let (_original, new) = self.get_from_data_block_ensure_index(top_index)?.as_clone_index_map()?;

        Ok(new)
    }

    fn lookup_in_data_slice(&self, start: usize, end: usize, lookup_index: usize) -> Result<usize, DataError> {
        let lookup_slice = &self.data()[start..end];

        let existing_left_index = lookup_slice.iter().find_map(|item| match item {
            BasicData::CloneIndexMap(original, new) => {
                if *original == lookup_index {
                    Some(new.clone())
                } else {
                    None
                }
            }
            _ => None,
        });

        let index = match existing_left_index {
            Some(value) => value,
            None => Err(DataError::new(
                "No mapped index found during clone",
                DataErrorType::NoMappedIndexFoundDuringClone(lookup_index),
            ))?,
        };

        Ok(index)
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
        expected_data.data_mut().splice(
            30..49,
            vec![
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
            ],
        );

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
        expected_data
            .data_mut()
            .splice(30..33, vec![BasicData::Unit, BasicData::CloneIndexMap(0, 2), BasicData::Unit]);
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
        expected_data
            .data_mut()
            .splice(30..33, vec![BasicData::True, BasicData::CloneIndexMap(0, 2), BasicData::True]);
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
        expected_data
            .data_mut()
            .splice(30..33, vec![BasicData::False, BasicData::CloneIndexMap(0, 2), BasicData::False]);
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
        expected_data.data_mut().splice(
            30..33,
            vec![
                BasicData::Type(GarnishDataType::Number),
                BasicData::CloneIndexMap(0, 2),
                BasicData::Type(GarnishDataType::Number),
            ],
        );
        expected_data.data_block_mut().cursor = 3;

        assert_eq!(index, 2);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn char() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(Char 'a')).unwrap();
        let index = data.push_clone_data(index).unwrap();

        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(
            30..33,
            vec![
                BasicData::Char('a'),
                BasicData::CloneIndexMap(0, 2),
                BasicData::Char('a'),
            ],
        );
        expected_data.data_block_mut().cursor = 3;

        assert_eq!(index, 2);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn byte() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(Byte 1)).unwrap();
        let index = data.push_clone_data(index).unwrap();

        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(
            30..33,
            vec![
                BasicData::Byte(1),
                BasicData::CloneIndexMap(0, 2),
                BasicData::Byte(1),
            ],
        );
        expected_data.data_block_mut().cursor = 3;

        assert_eq!(index, 2);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn symbol() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(SymRaw 100)).unwrap();
        let index = data.push_clone_data(index).unwrap();

        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(
            30..33,
            vec![
                BasicData::Symbol(100),
                BasicData::CloneIndexMap(0, 2),
                BasicData::Symbol(100),
            ],
        );
        expected_data.data_block_mut().cursor = 3;

        assert_eq!(index, 2);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn number() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        let index = data.push_clone_data(index).unwrap();

        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(
            30..33,
            vec![
                BasicData::Number(100.into()),
                BasicData::CloneIndexMap(0, 2),
                BasicData::Number(100.into()),
            ],
        );
        expected_data.data_block_mut().cursor = 3;

        assert_eq!(index, 2);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn expression() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(Expression 100)).unwrap();
        let index = data.push_clone_data(index).unwrap();

        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(
            30..33,
            vec![
                BasicData::Expression(100),
                BasicData::CloneIndexMap(0, 2),
                BasicData::Expression(100),
            ],
        );
        expected_data.data_block_mut().cursor = 3;

        assert_eq!(index, 2);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn external() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(External 100)).unwrap();
        let index = data.push_clone_data(index).unwrap();

        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(
            30..33,
            vec![
                BasicData::External(100),
                BasicData::CloneIndexMap(0, 2),
                BasicData::External(100),
            ],
        );
        expected_data.data_block_mut().cursor = 3;

        assert_eq!(index, 2);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn symbol_list() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(SymList(SymRaw 100, Number 200))).unwrap();
        let index = data.push_clone_data(index).unwrap();

        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(
            30..37,
            vec![
                BasicData::SymbolList(2),
                BasicData::Symbol(100),
                BasicData::Number(200.into()),
                BasicData::CloneIndexMap(0, 4),
                BasicData::SymbolList(2),
                BasicData::Symbol(100),
                BasicData::Number(200.into()),
            ],
        );
        expected_data.data_block_mut().cursor = 7;

        assert_eq!(index, 4);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn char_list() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(CharList "hello")).unwrap();
        let index = data.push_clone_data(index).unwrap();

        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().resize(60, BasicData::Empty);
        expected_data.data_mut().splice(
            30..43,
            vec![
                BasicData::CharList(5),
                BasicData::Char('h'),
                BasicData::Char('e'),
                BasicData::Char('l'),
                BasicData::Char('l'),
                BasicData::Char('o'),
                BasicData::CloneIndexMap(0, 7),
                BasicData::CharList(5),
                BasicData::Char('h'),
                BasicData::Char('e'),
                BasicData::Char('l'),
                BasicData::Char('l'),
                BasicData::Char('o'),
            ],
        );
        expected_data.data_block_mut().cursor = 13;
        expected_data.data_block_mut().size = 20;
        expected_data.custom_data_block_mut().start = 50;

        assert_eq!(index, 7);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn byte_list() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(ByteList 100, 200, 250)).unwrap();
        let index = data.push_clone_data(index).unwrap();

        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(
            30..39,
            vec![
                BasicData::ByteList(3),
                BasicData::Byte(100),
                BasicData::Byte(200),
                BasicData::Byte(250),
                BasicData::CloneIndexMap(0, 5),
                BasicData::ByteList(3),
                BasicData::Byte(100),
                BasicData::Byte(200),
                BasicData::Byte(250),
            ],
        );
        expected_data.data_block_mut().cursor = 9;

        assert_eq!(index, 5);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn pair() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!((True = False))).unwrap();

        let index = data.push_clone_data(index).unwrap();

        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(
            30..39,
            vec![
                BasicData::True,
                BasicData::False,
                BasicData::Pair(0, 1),
                BasicData::CloneIndexMap(2, 8),
                BasicData::CloneIndexMap(1, 7),
                BasicData::CloneIndexMap(0, 6),
                BasicData::True,
                BasicData::False,
                BasicData::Pair(6, 7),
            ],
        );
        expected_data.data_block_mut().cursor = 9;

        assert_eq!(index, 8);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn nested_pairs() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!((True = (False = Unit)))).unwrap();

        let index = data.push_clone_data(index).unwrap();

        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().resize(60, BasicData::Empty);
        expected_data.data_mut().splice(
            30..45,
            vec![
                BasicData::True,
                BasicData::False,
                BasicData::Unit,
                BasicData::Pair(1, 2),
                BasicData::Pair(0, 3),
                BasicData::CloneIndexMap(4, 14),
                BasicData::CloneIndexMap(3, 13),
                BasicData::CloneIndexMap(0, 12),
                BasicData::CloneIndexMap(2, 11),
                BasicData::CloneIndexMap(1, 10),
                BasicData::False,
                BasicData::Unit,
                BasicData::True,
                BasicData::Pair(10, 11),
                BasicData::Pair(12, 13),
            ],
        );
        expected_data.data_block_mut().cursor = 15;
        expected_data.data_block_mut().size = 20;
        expected_data.custom_data_block_mut().start = 50;
        expected_data.custom_data_block_mut().size = 10;

        assert_eq!(index, 14);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn range() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(((Number 100)..(Number 200)))).unwrap();

        let index = data.push_clone_data(index).unwrap();

        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(
            30..39,
            vec![
                BasicData::Number(100.into()),
                BasicData::Number(200.into()),
                BasicData::Range(0, 1),
                BasicData::CloneIndexMap(2, 8),
                BasicData::CloneIndexMap(1, 7),
                BasicData::CloneIndexMap(0, 6),
                BasicData::Number(100.into()),
                BasicData::Number(200.into()),
                BasicData::Range(6, 7),
            ],
        );
        expected_data.data_block_mut().cursor = 9;

        assert_eq!(index, 8);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn slice() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(((Number 100) - (Number 200)))).unwrap();

        let index = data.push_clone_data(index).unwrap();

        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(
            30..39,
            vec![
                BasicData::Number(100.into()),
                BasicData::Number(200.into()),
                BasicData::Slice(0, 1),
                BasicData::CloneIndexMap(2, 8),
                BasicData::CloneIndexMap(1, 7),
                BasicData::CloneIndexMap(0, 6),
                BasicData::Number(100.into()),
                BasicData::Number(200.into()),
                BasicData::Slice(6, 7),
            ],
        );
        expected_data.data_block_mut().cursor = 9;

        assert_eq!(index, 8);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn partial() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(((Number 100) ~ (Number 200)))).unwrap();

        let index = data.push_clone_data(index).unwrap();

        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(
            30..39,
            vec![
                BasicData::Number(100.into()),
                BasicData::Number(200.into()),
                BasicData::Partial(0, 1),
                BasicData::CloneIndexMap(2, 8),
                BasicData::CloneIndexMap(1, 7),
                BasicData::CloneIndexMap(0, 6),
                BasicData::Number(100.into()),
                BasicData::Number(200.into()),
                BasicData::Partial(6, 7),
            ],
        );
        expected_data.data_block_mut().cursor = 9;

        assert_eq!(index, 8);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn concatenation() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(((Number 100) <> (Number 200)))).unwrap();

        let index = data.push_clone_data(index).unwrap();

        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(
            30..39,
            vec![
                BasicData::Number(100.into()),
                BasicData::Number(200.into()),
                BasicData::Concatenation(0, 1),
                BasicData::CloneIndexMap(2, 8),
                BasicData::CloneIndexMap(1, 7),
                BasicData::CloneIndexMap(0, 6),
                BasicData::Number(100.into()),
                BasicData::Number(200.into()),
                BasicData::Concatenation(6, 7),
            ],
        );
        expected_data.data_block_mut().cursor = 9;

        assert_eq!(index, 8);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn list() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data
            .push_object_to_data_block(basic_object!((Number 100), (Number 200), (Number 250)))
            .unwrap();

        let index = data.push_clone_data(index).unwrap();

        println!("===== after clone ======");
        println!("{}", data.dump_data_block());

        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().resize(70, BasicData::Empty);
        expected_data.data_mut().splice(
            30..54,
            vec![
                BasicData::Number(100.into()),
                BasicData::Number(200.into()),
                BasicData::Number(250.into()),
                BasicData::List(3, 0),
                BasicData::ListItem(0),
                BasicData::ListItem(1),
                BasicData::ListItem(2),
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
                BasicData::CloneIndexMap(3, 17),
                BasicData::CloneIndexMap(0, 16),
                BasicData::CloneIndexMap(1, 15),
                BasicData::CloneIndexMap(2, 14),
                BasicData::Number(250.into()), // 14
                BasicData::Number(200.into()),
                BasicData::Number(100.into()),
                BasicData::List(3, 0),
                BasicData::ListItem(16),
                BasicData::ListItem(15),
                BasicData::ListItem(14),
                BasicData::Empty,
                BasicData::Empty,
                BasicData::Empty,
            ],
        );
        expected_data.data_block_mut().cursor = 24;
        expected_data.data_block_mut().size = 30;
        expected_data.custom_data_block_mut().start = 60;

        assert_eq!(index, 17);
        assert_eq!(data, expected_data);
    }
}
