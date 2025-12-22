use crate::{BasicData, BasicDataCustom, BasicGarnishData, BasicNumber, DataError};

impl<T> BasicGarnishData<T>
where
    T: BasicDataCustom,
{
    pub(crate) fn convert_basic_data_at_to_number(&mut self, from: usize) -> Result<Option<BasicNumber>, DataError> {
        Ok(match self.get_from_data_block_ensure_index(from)? {
            BasicData::Number(number) => Some(number.clone()),
            BasicData::Char(value) => {
                match value.to_digit(10) {
                    Some(value) => Some(BasicNumber::Integer(value as i32)),
                    None => None,
                }
            },
            BasicData::Byte(value) => Some(BasicNumber::Integer(value.clone() as i32)),
            BasicData::CharList(value) => {
                let start = from + 1;
                let end = start + value;
                let mut chars = String::new();
                for i in start..end {
                    let char = self.get_from_data_block_ensure_index(i)?.as_char()?;
                    chars.push(char);
                }
                match chars.parse::<i32>() {
                    Ok(value) => Some(BasicNumber::Integer(value)),
                    Err(_) => None,
                }
            },
            BasicData::ByteList(value) => {
                let start = from + 1;
                let end = start + value;
                let mut bytes = vec![];
                for i in start..end {
                    let byte = self.get_from_data_block_ensure_index(i)?.as_byte()?;
                    bytes.push(byte);
                }
                if bytes.len() > 4 {
                    return Ok(None);
                }
                let mut conversion_bytes = [0; 4];
                for (i, byte) in bytes.iter().enumerate() {
                    conversion_bytes[i] = *byte;
                }
                let num = i32::from_le_bytes(conversion_bytes);
                Some(BasicNumber::Integer(num))
            }
            BasicData::Unit
            | BasicData::True
            | BasicData::False
            | BasicData::Type(_)
            | BasicData::Symbol(_)
            | BasicData::SymbolList(_)
            | BasicData::Expression(_)
            | BasicData::External(_)
            | BasicData::Pair(_, _)
            | BasicData::Range(_, _)
            | BasicData::Slice(_, _)
            | BasicData::Partial(_, _)
            | BasicData::List(_, _)
            | BasicData::Concatenation(_, _)
            | BasicData::Custom(_)
            | BasicData::Empty
            | BasicData::UninitializedList(_, _)
            | BasicData::ListItem(_)
            | BasicData::AssociativeItem(_, _)
            | BasicData::Value(_, _)
            | BasicData::Register(_, _)
            | BasicData::Instruction(_, _)
            | BasicData::JumpPoint(_)
            | BasicData::Frame(_, _) => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{BasicNumber, basic::utilities::test_data, basic_object};

    #[test]
    fn convert_number() {
        let mut data = test_data();
        let index = data.push_object_to_data_block(basic_object!(Number 12345)).unwrap();
        let result = data.convert_basic_data_at_to_number(index).unwrap();
        assert_eq!(result, Some(BasicNumber::Integer(12345)));
    }

    #[test]
    fn convert_char() {
        let mut data = test_data();
        let index = data.push_object_to_data_block(basic_object!(Char '0')).unwrap();
        let result = data.convert_basic_data_at_to_number(index).unwrap();
        assert_eq!(result, Some(BasicNumber::Integer(0)));
    }

    #[test]
    fn convert_char_with_non_numeric_char() {
        let mut data = test_data();
        let index = data.push_object_to_data_block(basic_object!(Char 'a')).unwrap();
        let result = data.convert_basic_data_at_to_number(index).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn convert_char_list() {
        let mut data = test_data();
        let index = data.push_object_to_data_block(basic_object!(CharList "123456789")).unwrap();
        let result = data.convert_basic_data_at_to_number(index).unwrap();
        assert_eq!(result, Some(BasicNumber::Integer(123456789)));
    }

    #[test]
    fn convert_char_list_with_non_numeric_char_list() {
        let mut data = test_data();
        let index = data.push_object_to_data_block(basic_object!(CharList "abcde")).unwrap();
        let result = data.convert_basic_data_at_to_number(index).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn convert_byte() {
        let mut data = test_data();
        let index = data.push_object_to_data_block(basic_object!(Byte 100)).unwrap();
        let result = data.convert_basic_data_at_to_number(index).unwrap();
        assert_eq!(result, Some(BasicNumber::Integer(100)));
    }

    #[test]
    fn convert_byte_list() {
        let mut data = test_data();
        let index = data.push_object_to_data_block(basic_object!(ByteList 100, 200)).unwrap();
        let result = data.convert_basic_data_at_to_number(index).unwrap();
        assert_eq!(result, Some(BasicNumber::Integer(51300)));
    }

    #[test]
    fn convert_byte_list_with_too_many_bytes() {
        let mut data = test_data();
        let index = data.push_object_to_data_block(basic_object!(ByteList 100, 200, 250, 100, 100)).unwrap();
        let result = data.convert_basic_data_at_to_number(index).unwrap();
        assert_eq!(result, None);
    }
}