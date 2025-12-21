use crate::{BasicData, BasicDataCustom, BasicGarnishData, DataError};

impl<T> BasicGarnishData<T>
where
    T: BasicDataCustom,
{
    pub(crate) fn convert_basic_data_at_to_char_list(&mut self, from: usize) -> Result<usize, DataError> {
        match self.get_from_data_block_ensure_index(from)? {
            BasicData::CharList(_) => Ok(from),
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
        }
    }
}

#[cfg(test)]
mod convert_to_char_list {
    use crate::{
        BasicData,
        basic::{object::BasicObject, utilities::test_data},
    };

    #[test]
    fn convert_char_list_returns_original() {
        let mut data = test_data();
        data.push_object_to_data_block(BasicObject::CharList("abc".to_string())).unwrap();
        let char_list = data.convert_basic_data_at_to_char_list(0).unwrap();

        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::CharList(3);
        expected_data.data[1] = BasicData::Char('a');
        expected_data.data[2] = BasicData::Char('b');
        expected_data.data[3] = BasicData::Char('c');
        expected_data.data_block.cursor = 4;
        assert_eq!(char_list, 0);
        assert_eq!(data, expected_data);
    }
}
