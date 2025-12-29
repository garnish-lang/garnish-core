use crate::{BasicData, BasicDataCustom, BasicGarnishData, DataError, error::DataErrorType};

pub trait CloneDelegate
{
    fn lookup_value(&mut self, value: usize) -> Result<usize, DataError>;
}

pub struct BasicCloneDelegate<'a, T>
where
    T: BasicDataCustom,
{
    data: &'a mut BasicGarnishData<T>,
    lookup_start: usize,
    lookup_end: usize,
}

impl<'a, T> BasicCloneDelegate<'a, T>
where
    T: BasicDataCustom,
{
    pub(crate) fn new(data: &'a mut BasicGarnishData<T>, lookup_start: usize, lookup_end: usize) -> Self {
        Self { data, lookup_start, lookup_end }
    }
}

impl<'a, T> CloneDelegate for BasicCloneDelegate<'a, T>
where
    T: BasicDataCustom,
{
    fn lookup_value(&mut self, value: usize) -> Result<usize, DataError> {
        self.data.lookup_in_data_slice(self.lookup_start, self.lookup_end, value)
    }
}

impl<T> BasicGarnishData<T>
where
    T: BasicDataCustom,
{
    pub(crate) fn push_clone_data(&mut self, from: usize) -> Result<usize, DataError> {
        let index_stack_start = self.create_index_stack(from)?;
        self.clone_index_stack(index_stack_start, 0)
    }

    pub(crate) fn clone_index_stack(&mut self, top_index: usize, offset: usize) -> Result<usize, DataError> {
        let clone_range = top_index..self.data_block().cursor;

        let lookup_end = self.data_block().start + self.data_block().cursor;
        let mut lookup_start = lookup_end;

        for i in clone_range.rev() {
            let index = self.get_from_data_block_ensure_index(i)?.as_clone_item()?;
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
                BasicData::Range(left, right) => {
                    let left = self.lookup_in_data_slice(lookup_start, lookup_end, left)?;
                    let right = self.lookup_in_data_slice(lookup_start, lookup_end, right)?;

                    self.push_to_data_block(BasicData::Range(left, right))?
                }
                BasicData::Slice(left, right) => {
                    let left = self.lookup_in_data_slice(lookup_start, lookup_end, left)?;
                    let right = self.lookup_in_data_slice(lookup_start, lookup_end, right)?;

                    self.push_to_data_block(BasicData::Slice(left, right))?
                }
                BasicData::Partial(left, right) => {
                    let left = self.lookup_in_data_slice(lookup_start, lookup_end, left)?;
                    let right = self.lookup_in_data_slice(lookup_start, lookup_end, right)?;

                    self.push_to_data_block(BasicData::Partial(left, right))?
                }
                BasicData::List(length, association_length) => {
                    let start = index + 1;
                    let end = start + length * 2;

                    let list_index = self.push_to_data_block(BasicData::List(length, association_length))?;

                    for i in start..end {
                        match self.get_from_data_block_ensure_index(i)? {
                            BasicData::ListItem(item) => {
                                let item = self.lookup_in_data_slice(lookup_start, lookup_end, item.clone())?;
                                self.push_to_data_block(BasicData::ListItem(item))?;
                            }
                            BasicData::AssociativeItem(symbol, item) => {
                                let item = self.lookup_in_data_slice(lookup_start, lookup_end, item.clone())?;
                                self.push_to_data_block(BasicData::AssociativeItem(symbol.clone(), item))?;
                            }
                            BasicData::Empty => {
                                self.push_to_data_block(BasicData::Empty)?;
                            }
                            t => Err(DataError::new(
                                "Associative item in list is not a valid item",
                                DataErrorType::NotAssociativeItem(t.get_data_type()),
                            ))?,
                        }
                    }

                    list_index
                }
                BasicData::Concatenation(left, right) => {
                    let left = self.lookup_in_data_slice(lookup_start, lookup_end, left)?;
                    let right = self.lookup_in_data_slice(lookup_start, lookup_end, right)?;

                    self.push_to_data_block(BasicData::Concatenation(left, right))?
                }
                BasicData::Custom(custom) => {
                    let mut delegate = BasicCloneDelegate::new(self, lookup_start, lookup_end);
                    let cloned = T::create_cloned_custom_data(&mut delegate, custom)?;
                    self.push_to_data_block(BasicData::Custom(cloned))?
                }
                BasicData::Empty => self.push_to_data_block(BasicData::Empty)?,
                BasicData::UninitializedList(length, count) => {
                    let start = index + 1;
                    let end = start + length * 2;

                    let list_index = self.push_to_data_block(BasicData::UninitializedList(length, count))?;

                    for i in start..end {
                        match self.get_from_data_block_ensure_index(i)? {
                            BasicData::ListItem(item) => {
                                let item = self.lookup_in_data_slice(lookup_start, lookup_end, item.clone())?;
                                self.push_to_data_block(BasicData::ListItem(item))?;
                            }
                            BasicData::AssociativeItem(symbol, item) => {
                                let item = self.lookup_in_data_slice(lookup_start, lookup_end, item.clone())?;
                                self.push_to_data_block(BasicData::AssociativeItem(symbol.clone(), item))?;
                            }
                            BasicData::Empty => {
                                self.push_to_data_block(BasicData::Empty)?;
                            }
                            t => Err(DataError::new(
                                "Associative item in list is not a valid item",
                                DataErrorType::NotAssociativeItem(t.get_data_type()),
                            ))?,
                        }
                    }

                    list_index
                }
                BasicData::ListItem(_) => {
                    Err(DataError::new("Cannot clone", DataErrorType::CannotClone))?
                }
                BasicData::AssociativeItem(_, _) => {
                    Err(DataError::new("Cannot clone", DataErrorType::CannotClone))?
                }
                BasicData::Value(previous, value) => {
                    let previous = self.lookup_in_data_slice(lookup_start, lookup_end, previous)?;
                    let value = self.lookup_in_data_slice(lookup_start, lookup_end, value)?;
                    self.push_to_data_block(BasicData::Value(previous, value))?
                }
                BasicData::ValueRoot(value) => {
                    let value = self.lookup_in_data_slice(lookup_start, lookup_end, value)?;
                    self.push_to_data_block(BasicData::ValueRoot(value))?
                }
                BasicData::Register(previous, value) => {
                    let previous = self.lookup_in_data_slice(lookup_start, lookup_end, previous)?;
                    let value = self.lookup_in_data_slice(lookup_start, lookup_end, value)?;
                    self.push_to_data_block(BasicData::Register(previous, value))?
                }
                BasicData::RegisterRoot(value) => {
                    let value = self.lookup_in_data_slice(lookup_start, lookup_end, value)?;
                    self.push_to_data_block(BasicData::RegisterRoot(value))?
                }
                BasicData::InstructionWithData(instruction, data) => {
                    let data = self.lookup_in_data_slice(lookup_start, lookup_end, data)?;
                    self.push_to_data_block(BasicData::InstructionWithData(instruction.clone(), data))?
                }
                BasicData::Instruction(instruction) => {
                    self.push_to_data_block(BasicData::Instruction(instruction.clone()))?
                }
                BasicData::JumpPoint(point) => {
                    self.push_to_data_block(BasicData::JumpPoint(point))?
                }
                BasicData::Frame(previous, register) => {
                    let previous = self.lookup_in_data_slice(lookup_start, lookup_end, previous)?;
                    let register = self.lookup_in_data_slice(lookup_start, lookup_end, register)?;
                    self.push_to_data_block(BasicData::Frame(previous, register))?
                }
                BasicData::FrameIndex(previous) => {
                    let previous = self.lookup_in_data_slice(lookup_start, lookup_end, previous)?;
                    self.push_to_data_block(BasicData::FrameIndex(previous))?
                }
                BasicData::FrameRegister(register) => {
                    let register = self.lookup_in_data_slice(lookup_start, lookup_end, register)?;
                    self.push_to_data_block(BasicData::FrameRegister(register))?
                }
                BasicData::FrameRoot => {
                    self.push_to_data_block(BasicData::FrameRoot)?
                }
                BasicData::CloneItem(_) => {
                    Err(DataError::new("Cannot clone", DataErrorType::CannotClone))?
                }
                BasicData::CloneIndexMap(_, _) => {
                    Err(DataError::new("Cannot clone", DataErrorType::CannotClone))?
                }
            };

            *self.get_from_data_block_ensure_index_mut(i)? = BasicData::CloneIndexMap(index, new_index - offset);

            lookup_start -= 1;
        }

        let (_original, new) = self.get_from_data_block_ensure_index(top_index)?.as_clone_index_map()?;

        Ok(new)
    }

    pub(crate) fn lookup_in_data_slice(&self, start: usize, end: usize, lookup_index: usize) -> Result<usize, DataError> {
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
mod clone {
    use garnish_lang_traits::{GarnishData, GarnishDataType, Instruction};

    use crate::{BasicData, BasicDataCustom, BasicGarnishData, DataError, basic::clone::CloneDelegate, basic::ordering::OrderingDelegate, basic_object, error::DataErrorType};

    #[test]
    fn circular_reference_is_error() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let pair_one = data.push_object_to_data_block(basic_object!((Number 100) = (Number 200))).unwrap();
        let pair_two = data.push_object_to_data_block(basic_object!((Number 100) = (Number 200))).unwrap();
        *data.get_from_data_block_ensure_index_mut(pair_two).unwrap() = BasicData::Pair(pair_two, pair_one);
        
        let result = data.push_clone_data(pair_two);

        assert_eq!(result, Err(DataError::new("Clone limit reached", DataErrorType::CloneLimitReached)));
    }

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
    fn empty() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_to_data_block(BasicData::Empty).unwrap();
        let index = data.push_clone_data(index).unwrap();

        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data
            .data_mut()
            .splice(30..33, vec![BasicData::Empty, BasicData::CloneIndexMap(0, 2), BasicData::Empty]);
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
        expected_data
            .data_mut()
            .splice(30..33, vec![BasicData::Char('a'), BasicData::CloneIndexMap(0, 2), BasicData::Char('a')]);
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
        expected_data
            .data_mut()
            .splice(30..33, vec![BasicData::Byte(1), BasicData::CloneIndexMap(0, 2), BasicData::Byte(1)]);
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
            vec![BasicData::Symbol(100), BasicData::CloneIndexMap(0, 2), BasicData::Symbol(100)],
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
            vec![BasicData::Expression(100), BasicData::CloneIndexMap(0, 2), BasicData::Expression(100)],
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
            vec![BasicData::External(100), BasicData::CloneIndexMap(0, 2), BasicData::External(100)],
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

    #[test]
    fn list_with_associations() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data
            .push_object_to_data_block(basic_object!(((SymRaw 100) = (Number 100)), ((SymRaw 200) = (Number 200)), ((SymRaw 250) = (Number 250))))
            .unwrap();

        let index = data.push_clone_data(index).unwrap();

        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().resize(90, BasicData::Empty);
        expected_data.data_mut().splice(
            30..72,
            vec![
                BasicData::Symbol(100),
                BasicData::Number(100.into()),
                BasicData::Pair(0, 1), // 2
                BasicData::Symbol(200),
                BasicData::Number(200.into()),
                BasicData::Pair(3, 4), // 5
                BasicData::Symbol(250),
                BasicData::Number(250.into()),
                BasicData::Pair(6, 7), // 8
                BasicData::List(3, 3), // 9
                BasicData::ListItem(2),
                BasicData::ListItem(5),
                BasicData::ListItem(8),
                BasicData::AssociativeItem(100, 1), // 13
                BasicData::AssociativeItem(200, 4),
                BasicData::AssociativeItem(250, 7),
                BasicData::CloneIndexMap(9, 35), // 16
                BasicData::CloneIndexMap(2, 34),
                BasicData::CloneIndexMap(5, 33),
                BasicData::CloneIndexMap(8, 32),
                BasicData::CloneIndexMap(1, 31),
                BasicData::CloneIndexMap(0, 30),
                BasicData::CloneIndexMap(4, 29),
                BasicData::CloneIndexMap(3, 28),
                BasicData::CloneIndexMap(7, 27),
                BasicData::CloneIndexMap(6, 26),
                BasicData::Symbol(250), // 26
                BasicData::Number(250.into()),
                BasicData::Symbol(200), // 28
                BasicData::Number(200.into()),
                BasicData::Symbol(100), // 30
                BasicData::Number(100.into()),
                BasicData::Pair(26, 27), // 32
                BasicData::Pair(28, 29),
                BasicData::Pair(30, 31),
                BasicData::List(3, 3), // 35
                BasicData::ListItem(34),
                BasicData::ListItem(33),
                BasicData::ListItem(32),
                BasicData::AssociativeItem(100, 31),
                BasicData::AssociativeItem(200, 29),
                BasicData::AssociativeItem(250, 27), // 41
            ],
        );
        expected_data.data_block_mut().cursor = 42;
        expected_data.data_block_mut().size = 50;
        expected_data.custom_data_block_mut().start = 80;

        assert_eq!(index, 35);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn uninitialized_list_with_associations() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        
        let num1 = data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        let pair = data.push_object_to_data_block(basic_object!((SymRaw 200) = (Number 200))).unwrap();
        
        let mut list_index = data.start_list(3).unwrap();
        list_index = data.add_to_list(list_index, num1).unwrap();
        list_index = data.add_to_list(list_index, pair).unwrap();
        
        let index = data.push_clone_data(list_index).unwrap();

        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().resize(70, BasicData::Empty);
        expected_data.data_mut().splice(
            30..57,
            vec![
                BasicData::Number(100.into()), // 0
                BasicData::Symbol(200), // 1
                BasicData::Number(200.into()), // 2
                BasicData::Pair(1, 2), // 3
                BasicData::UninitializedList(3, 2), // 4
                BasicData::ListItem(0), // 5
                BasicData::ListItem(3), // 6
                BasicData::Empty, // 7
                BasicData::Empty, // 8
                BasicData::AssociativeItem(200, 2), // 9
                BasicData::Empty, // 10
                BasicData::CloneIndexMap(4, 20), // 11
                BasicData::CloneIndexMap(0, 19), // 12
                BasicData::CloneIndexMap(3, 18), // 13
                BasicData::CloneIndexMap(2, 17), // 14
                BasicData::CloneIndexMap(1, 16), // 15
                BasicData::Symbol(200), // 16
                BasicData::Number(200.into()), // 17
                BasicData::Pair(16, 17), // 18
                BasicData::Number(100.into()), // 19
                BasicData::UninitializedList(3, 2), // 20
                BasicData::ListItem(19), // 21
                BasicData::ListItem(18), // 22
                BasicData::Empty, // 23
                BasicData::Empty, // 24
                BasicData::AssociativeItem(200, 17), // 25
                BasicData::Empty, // 26
            ],
        );
        expected_data.data_block_mut().cursor = 27;
        expected_data.data_block_mut().size = 30;
        expected_data.custom_data_block_mut().start = 60;

        assert_eq!(index, 20);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn list_item_is_error() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_to_data_block(BasicData::ListItem(0)).unwrap();
        let index = data.push_clone_data(index);

        assert_eq!(index, Err(DataError::new("Cannot clone", DataErrorType::CannotClone)));
    } 

    #[test]
    fn associative_item_is_error() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_to_data_block(BasicData::AssociativeItem(100, 0)).unwrap();
        let index = data.push_clone_data(index);

        assert_eq!(index, Err(DataError::new("Cannot clone", DataErrorType::CannotClone)));
    } 

    #[test]
    fn clone_item_is_error() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_to_data_block(BasicData::CloneItem(0)).unwrap();
        let index = data.push_clone_data(index);

        assert_eq!(index, Err(DataError::new("Cannot clone", DataErrorType::CannotClone)));
    } 

    #[test]
    fn clone_index_map_is_error() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_to_data_block(BasicData::CloneIndexMap(0, 0)).unwrap();
        let index = data.push_clone_data(index);

        assert_eq!(index, Err(DataError::new("Cannot clone", DataErrorType::CannotClone)));
    }

    #[test]
    fn value() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let previous = data.push_to_data_block(BasicData::ValueRoot(index)).unwrap();
        let index = data.push_to_data_block(BasicData::Number(200.into())).unwrap();
        let index = data.push_to_data_block(BasicData::Value(previous, index)).unwrap();

        let index = data.push_clone_data(index).unwrap();

        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().resize(60, BasicData::Empty);
        expected_data.data_mut().splice(
            30..42,
            vec![
                BasicData::Number(100.into()),
                BasicData::ValueRoot(0),
                BasicData::Number(200.into()),
                BasicData::Value(1, 2),
                BasicData::CloneIndexMap(3, 11), // 4
                BasicData::CloneIndexMap(1, 10),
                BasicData::CloneIndexMap(2, 9),
                BasicData::CloneIndexMap(0, 8),
                BasicData::Number(100.into()), // 8
                BasicData::Number(200.into()),
                BasicData::ValueRoot(8),
                BasicData::Value(10, 9),
            ],
        );
        expected_data.data_block_mut().cursor = 12;
        expected_data.data_block_mut().size = 20;
        expected_data.custom_data_block_mut().start = 50;

        assert_eq!(index, 11);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn register() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let previous = data.push_to_data_block(BasicData::RegisterRoot(index)).unwrap();
        let index = data.push_to_data_block(BasicData::Number(200.into())).unwrap();
        let index = data.push_to_data_block(BasicData::Register(previous, index)).unwrap();

        let index = data.push_clone_data(index).unwrap();

        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().resize(60, BasicData::Empty);
        expected_data.data_mut().splice(
            30..42,
            vec![
                BasicData::Number(100.into()),
                BasicData::RegisterRoot(0),
                BasicData::Number(200.into()),
                BasicData::Register(1, 2),
                BasicData::CloneIndexMap(3, 11), // 4
                BasicData::CloneIndexMap(1, 10),
                BasicData::CloneIndexMap(2, 9),
                BasicData::CloneIndexMap(0, 8),
                BasicData::Number(100.into()), // 8
                BasicData::Number(200.into()),
                BasicData::RegisterRoot(8),
                BasicData::Register(10, 9),
            ],
        );
        expected_data.data_block_mut().cursor = 12;
        expected_data.data_block_mut().size = 20;
        expected_data.custom_data_block_mut().start = 50;

        assert_eq!(index, 11);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn instruction() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let instruction_data = data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        let index = data.push_to_data_block(BasicData::InstructionWithData(Instruction::Add, instruction_data)).unwrap();
        let index = data.push_clone_data(index).unwrap();
        
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(
            30..36,
            vec![
                BasicData::Number(100.into()),
                BasicData::InstructionWithData(Instruction::Add, 0),
                BasicData::CloneIndexMap(1, 5),
                BasicData::CloneIndexMap(0, 4),
                BasicData::Number(100.into()),
                BasicData::InstructionWithData(Instruction::Add, 4),
            ],
        );
        expected_data.data_block_mut().cursor = 6;

        assert_eq!(index, 5);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn jump_point() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_to_data_block(BasicData::JumpPoint(100)).unwrap();
        let index = data.push_clone_data(index).unwrap();

        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data
            .data_mut()
            .splice(30..33, vec![BasicData::JumpPoint(100), BasicData::CloneIndexMap(0, 2), BasicData::JumpPoint(100)]);
        expected_data.data_block_mut().cursor = 3;

        assert_eq!(index, 2);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn frame() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        data.push_to_data_block(BasicData::JumpPoint(10)).unwrap();
        let first_frame = data.push_to_data_block(BasicData::FrameRoot).unwrap();
        let value2 = data.push_to_data_block(BasicData::Number(200.into())).unwrap();
        let register = data.push_to_data_block(BasicData::RegisterRoot(value2)).unwrap();
        data.push_to_data_block(BasicData::JumpPoint(100)).unwrap();
        let index = data.push_to_data_block(BasicData::Frame(first_frame, register)).unwrap();
        let index = data.push_clone_data(index).unwrap();

        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().resize(60, BasicData::Empty);
        expected_data
            .data_mut()
            .splice(30..48, vec![
                BasicData::JumpPoint(10),
                BasicData::FrameRoot,
                BasicData::Number(200.into()),
                BasicData::RegisterRoot(2),
                BasicData::JumpPoint(100),
                BasicData::Frame(1, 3),
                BasicData::CloneIndexMap(5, 17), // 6
                BasicData::CloneIndexMap(4, 16),
                BasicData::CloneIndexMap(1, 15),
                BasicData::CloneIndexMap(3, 14),
                BasicData::CloneIndexMap(0, 13),
                BasicData::CloneIndexMap(2, 12),
                BasicData::Number(200.into()), // 12
                BasicData::JumpPoint(10),
                BasicData::RegisterRoot(12),
                BasicData::FrameRoot, 
                BasicData::JumpPoint(100),
                BasicData::Frame(15, 14), // 17
            ]);
        expected_data.data_block_mut().cursor = 18;
        expected_data.data_block_mut().size = 20;
        expected_data.custom_data_block_mut().start = 50;

        assert_eq!(index, 17);
        assert_eq!(data, expected_data);
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
    struct TestCustom {
        value: usize,
        name: &'static str,
    }

    impl BasicDataCustom for TestCustom {
        fn push_clone_items_for_custom_data(delegate: &mut impl OrderingDelegate, value: Self) -> Result<(), DataError> {
            delegate.push_clone_items(value.value)?;
            Ok(())
        }

        fn create_cloned_custom_data(delegate: &mut impl CloneDelegate, value: Self) -> Result<Self, DataError> {
            let index = delegate.lookup_value(value.value)?;
            Ok(Self {
                value: index,
                name: value.name,
            })
        }
    }

    #[test]
    fn custom() {
        let mut data = BasicGarnishData::<TestCustom>::new().unwrap();
        let index = data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let index = data.push_to_data_block(BasicData::Custom(TestCustom { value: index, name: "test" })).unwrap();
        let index = data.push_clone_data(index).unwrap();

        let mut expected_data = BasicGarnishData::<TestCustom>::new().unwrap();
        expected_data.data_mut().splice(
            30..36,
            vec![
                BasicData::Number(100.into()),
                BasicData::Custom(TestCustom { value: 0, name: "test" }), 
                BasicData::CloneIndexMap(1, 5),
                BasicData::CloneIndexMap(0, 4),
                BasicData::Number(100.into()),
                BasicData::Custom(TestCustom { value: 4, name: "test" })
            ],
        );
        expected_data.data_block_mut().cursor = 6;

        assert_eq!(index, 5);
        assert_eq!(data, expected_data);
    }
}

