use crate::{BasicData, BasicDataCustom, BasicGarnishData, DataError};

trait ConversionDelegate<T>
where
    T: BasicDataCustom,
{
    type Output;
    fn init(&mut self) -> Result<(), DataError>;
    fn push_char(&mut self, c: char) -> Result<(), DataError>;
    fn get_data_at(&self, index: usize) -> Result<&BasicData<T>, DataError>;
    fn end(self) -> Result<Self::Output, DataError>;
}

struct BasicDataDelegate<'a, T>
where
    T: BasicDataCustom,
{
    list_index: usize,
    length: usize,
    data: &'a mut BasicGarnishData<T>,
}

impl<'a, T> BasicDataDelegate<'a, T>
where
    T: BasicDataCustom,
{
    fn new(data: &'a mut BasicGarnishData<T>) -> Self {
        Self {
            list_index: 0,
            length: 0,
            data,
        }
    }
}

impl<'a, T> ConversionDelegate<T> for BasicDataDelegate<'a, T>
where
    T: BasicDataCustom,
{
    type Output = usize;

    fn init(&mut self) -> Result<(), DataError> {
        self.list_index = self.data.push_to_data_block(BasicData::CharList(0))?;
        Ok(())
    }

    fn push_char(&mut self, c: char) -> Result<(), DataError> {
        self.data.push_to_data_block(BasicData::Char(c))?;
        self.length += 1;
        Ok(())
    }

    fn get_data_at(&self, index: usize) -> Result<&BasicData<T>, DataError> {
        self.data.get_from_data_block_ensure_index(index)
    }

    fn end(self) -> Result<Self::Output, DataError> {
        let list_length = self.data.get_from_data_block_ensure_index_mut(self.list_index)?.as_char_list_mut()?;
        *list_length = self.length;
        Ok(self.list_index)
    }
}

struct StringDelegate<'a, T> where T: BasicDataCustom {
    string: String,
    data: &'a BasicGarnishData<T>,
}

impl<'a, T> StringDelegate<'a, T> where T: BasicDataCustom {
    fn new(data: &'a BasicGarnishData<T>) -> Self {
        Self { string: String::new(), data }
    }
}

impl<'a, T> ConversionDelegate<T> for StringDelegate<'a, T> where T: BasicDataCustom {
    type Output = String;

    fn init(&mut self) -> Result<(), DataError> {
        Ok(())
    }

    fn push_char(&mut self, c: char) -> Result<(), DataError> {
        self.string.push(c);
        Ok(())
    }

    fn get_data_at(&self, index: usize) -> Result<&BasicData<T>, DataError> {
        self.data.get_from_data_block_ensure_index(index)
    }

    fn end(self) -> Result<Self::Output, DataError> {
        Ok(self.string)
    }
}

impl<T> BasicGarnishData<T>
where
    T: BasicDataCustom,
{
    pub(crate) fn convert_basic_data_at_to_char_list(&mut self, from: usize) -> Result<usize, DataError> {
        let mut delegate = BasicDataDelegate::new(self);
        delegate.init()?;
        convert_with_delegate(&mut delegate, from)?;
        delegate.end()
    }

    pub(crate) fn string_from_basic_data_at(&self, from: usize) -> Result<String, DataError> {
        let mut delegate = StringDelegate::new(self);
        delegate.init()?;
        convert_with_delegate(&mut delegate, from)?;
        delegate.end()
    }
}

fn convert_with_delegate<T>(delegate: &mut impl ConversionDelegate<T>, from: usize) -> Result<(), DataError>
where
    T: BasicDataCustom,
{
    Ok(match delegate.get_data_at(from)? {
        BasicData::Unit => todo!(),
        BasicData::True => todo!(),
        BasicData::False => todo!(),
        BasicData::Type(garnish_data_type) => todo!(),
        BasicData::Number(simple_number) => todo!(),
        BasicData::Char(_) => todo!(),
        BasicData::Byte(_) => todo!(),
        BasicData::Symbol(_) => todo!(),
        BasicData::SymbolList(_) => todo!(),
        BasicData::Expression(_) => todo!(),
        BasicData::External(_) => todo!(),
        BasicData::ByteList(_) => todo!(),
        BasicData::CharList(length) => {
            let start = from + 1;
            let length = length.clone();
            for i in start..start + length {
                let c = delegate.get_data_at(i)?.as_char()?;
                delegate.push_char(c)?;
            }
        }
        BasicData::Pair(_, _) => todo!(),
        BasicData::Range(_, _) => todo!(),
        BasicData::Slice(_, _) => todo!(),
        BasicData::Partial(_, _) => todo!(),
        BasicData::List(_, _) => todo!(),
        BasicData::Concatenation(_, _) => todo!(),
        BasicData::Custom(_) => todo!(),
        BasicData::Empty => todo!(),
        BasicData::UninitializedList(_, _) => todo!(),
        BasicData::ListItem(_) => todo!(),
        BasicData::AssociativeItem(_, _) => todo!(),
        BasicData::Value(_, _) => todo!(),
        BasicData::Register(_, _) => todo!(),
        BasicData::Instruction(instruction, _) => todo!(),
        BasicData::JumpPoint(_) => todo!(),
        BasicData::Frame(_, _) => todo!(),
    })
}

#[cfg(test)]
mod convert_to_char_list {
    use crate::{
        BasicData,
        basic::{object::BasicObject, utilities::test_data},
    };

    #[test]
    fn convert_char_list_clones_original() {
        let mut data = test_data();
        data.push_object_to_data_block(BasicObject::CharList("abc".to_string())).unwrap();
        let char_list = data.convert_basic_data_at_to_char_list(0).unwrap();

        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::CharList(3);
        expected_data.data[1] = BasicData::Char('a');
        expected_data.data[2] = BasicData::Char('b');
        expected_data.data[3] = BasicData::Char('c');
        expected_data.data[4] = BasicData::CharList(3);
        expected_data.data[5] = BasicData::Char('a');
        expected_data.data[6] = BasicData::Char('b');
        expected_data.data[7] = BasicData::Char('c');

        expected_data.data_block.cursor = 8;
        assert_eq!(char_list, 4);
        assert_eq!(data, expected_data);

        let string = data.string_from_basic_data_at(0).unwrap();
        assert_eq!(string, "abc");
    }
}
