use crate::{BasicData, BasicDataCustom, BasicGarnishData, DataError, error::DataErrorType};

pub trait OrderingDelegate
{
    fn push_clone_items(&mut self, value: usize) -> Result<(), DataError>;
}

pub struct BasicOrderingDelegate<'a, T>
where
    T: BasicDataCustom,
{
    data: &'a mut BasicGarnishData<T>,
}

impl<'a, T> BasicOrderingDelegate<'a, T>
where
    T: BasicDataCustom,
{
    pub(crate) fn new(data: &'a mut BasicGarnishData<T>) -> Self {
        Self { data }
    }
}

impl<'a, T> OrderingDelegate for BasicOrderingDelegate<'a, T>
where
    T: BasicDataCustom,
{
    fn push_clone_items(&mut self, value: usize) -> Result<(), DataError> {
        self.data.push_to_data_block(BasicData::CloneItem(value))?;
        Ok(())
    }
}

impl<T> BasicGarnishData<T>
where
    T: BasicDataCustom,
{
    pub(crate) fn create_index_stack(&mut self, from: usize) -> Result<usize, DataError> {
        let start = self.push_to_data_block(BasicData::CloneItem(from))?;
        let mut current = start;

        let max_iterations = (self.data_block().size / 2).pow(2);
        let mut iterations = 0;

        while current < self.data_block().cursor {
            let index = self.get_from_data_block_ensure_index(current)?.as_clone_item()?;

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
                BasicData::Custom(custom) => {
                    let custom = custom.clone();
                    let mut delegate = BasicOrderingDelegate::new(self);
                    T::push_clone_items_for_custom_data(&mut delegate, custom)?;
                }
                BasicData::Empty => {}
                BasicData::UninitializedList(_len, count) => {
                    let start = index + 1;
                    let end = start + count;
                    for i in start..end {
                        let list_item = self.get_from_data_block_ensure_index(i)?;
                        match list_item {
                            BasicData::ListItem(item) => {
                                self.push_to_data_block(BasicData::CloneItem(item.clone()))?;
                            }
                            BasicData::Empty => {}
                            t => {
                                Err(DataError::new(
                                    "Uninitialized list contains non-list item",
                                    DataErrorType::UninitializedListContainsNonListItem(t.get_data_type()),
                                ))?;
                            }
                        }
                    }
                }
                BasicData::ListItem(_) => {}
                BasicData::AssociativeItem(_, _) => {}
                BasicData::Value(previous, value) => {
                    let (previous, value) = (previous.clone(), value.clone());
                    self.push_to_data_block(BasicData::CloneItem(previous))?;
                    self.push_to_data_block(BasicData::CloneItem(value))?;
                }
                BasicData::ValueRoot(value) => {
                    let value = value.clone();
                    self.push_to_data_block(BasicData::CloneItem(value))?;
                }
                BasicData::Register(previous, value) => {
                    let (previous, value) = (previous.clone(), value.clone());
                    self.push_to_data_block(BasicData::CloneItem(previous))?;
                    self.push_to_data_block(BasicData::CloneItem(value))?;
                }
                BasicData::RegisterRoot(value) => {
                    let value = value.clone();
                    self.push_to_data_block(BasicData::CloneItem(value))?;
                }
                BasicData::Instruction(_instruction, data) => {
                    self.push_to_data_block(BasicData::CloneItem(data.clone()))?;
                }
                BasicData::InstructionRoot(_instruction) => {}
                BasicData::JumpPoint(_point) => {}
                BasicData::Frame(previous, register) => {
                    let (previous, register) = (previous.clone(), register.clone());
                    self.get_from_data_block_ensure_index(index - 1)?;
                    self.push_to_data_block(BasicData::CloneItem(index - 1))?;
                    self.push_to_data_block(BasicData::CloneItem(previous))?;
                    self.push_to_data_block(BasicData::CloneItem(register))?;
                }
                BasicData::FrameIndex(previous) => {
                    let previous = previous.clone();
                    self.get_from_data_block_ensure_index(index - 1)?;
                    self.push_to_data_block(BasicData::CloneItem(index - 1))?;
                    self.push_to_data_block(BasicData::CloneItem(previous))?;
                }
                BasicData::FrameRegister(register) => {
                    let register = register.clone();
                    self.get_from_data_block_ensure_index(index - 1)?;
                    self.push_to_data_block(BasicData::CloneItem(index - 1))?;
                    self.push_to_data_block(BasicData::CloneItem(register))?;
                }
                BasicData::FrameRoot => {
                    self.get_from_data_block_ensure_index(index - 1)?;
                    self.push_to_data_block(BasicData::CloneItem(index - 1))?;
                }
                BasicData::CloneItem(_) => {}
                BasicData::CloneIndexMap(_, _) => {}
            }

            current += 1;

            iterations += 1;
            if iterations > max_iterations {
                Err(DataError::new("Clone limit reached", DataErrorType::CloneLimitReached))?
            }
        }

        Ok(start)
    }
}

