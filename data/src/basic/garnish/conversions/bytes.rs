use crate::{BasicData, BasicDataCustom, BasicGarnishData, BasicNumber, DataError};

impl<T> BasicGarnishData<T>
where
    T: BasicDataCustom,
{
    pub(crate) fn convert_basic_data_at_to_bytes(&mut self, from: usize) -> Result<Vec<u8>, DataError> {
        Ok(match self.get_from_data_block_ensure_index(from)? {
            BasicData::Unit => vec![],
            BasicData::True => 1u8.to_le_bytes().to_vec(),
            BasicData::False => 0u8.to_le_bytes().to_vec(),
            BasicData::Byte(value) => value.to_le_bytes().to_vec(),
            BasicData::Char(value) => u32::from(value.clone()).to_le_bytes().to_vec(),
            BasicData::Symbol(value) => value.to_le_bytes().to_vec(),
            BasicData::ByteList(length) => {
                let start = from + 1;
                let end = start + length;
                self.data()[start..end]
                    .iter()
                    .map(|c| c.as_byte())
                    .collect::<Result<Vec<u8>, DataError>>()?
            }
            BasicData::CharList(length) => {
                let start = from + 1;
                let end = start + length;
                self.data()[start..end]
                    .iter()
                    .map(|c| c.as_char().map(|c| u32::from(c).to_le_bytes().to_vec()).unwrap_or(vec![]))
                    .flatten()
                    .collect::<Vec<u8>>()
            }
            BasicData::SymbolList(length) => {
                let start = from + 1;
                let end = start + length;
                self.data()[start..end]
                    .iter()
                    .map(|c| match c {
                        BasicData::Symbol(value) => value.to_le_bytes().to_vec(),
                        BasicData::Number(value) => match value {
                            BasicNumber::Integer(value) => value.to_le_bytes().to_vec(),
                            BasicNumber::Float(value) => value.to_le_bytes().to_vec(),
                        },
                        _ => vec![],
                    })
                    .flatten()
                    .collect::<Vec<u8>>()
            }
            BasicData::Number(value) => match value {
                BasicNumber::Integer(value) => value.to_le_bytes().to_vec(),
                BasicNumber::Float(value) => value.to_le_bytes().to_vec(),
            },
            BasicData::List(length, _) => {
                let start = from + 1;
                let end = start + length;
                let mut bytes = vec![];

                for i in start..end {
                    let item = self.get_from_data_block_ensure_index(i)?.as_list_item()?;
                    let b = self.convert_basic_data_at_to_bytes(item)?;
                    bytes.extend(b);
                }

                bytes
            }
            BasicData::Type(_)
            | BasicData::Expression(_)
            | BasicData::External(_)
            | BasicData::Pair(_, _)
            | BasicData::Range(_, _)
            | BasicData::Slice(_, _)
            | BasicData::Partial(_, _)
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
            | BasicData::Frame(_, _)
            | BasicData::CloneNodeNew(_, _)
            | BasicData::CloneNodeVisited(_, _)
            | BasicData::CloneItem(_)
            | BasicData::CloneIndexMap(_, _) => vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{basic::utilities::test_data, basic_object};

    #[test]
    fn convert_unit() {
        let mut data = test_data();
        let index = data.push_object_to_data_block(basic_object!(Unit)).unwrap();

        let result = data.convert_basic_data_at_to_bytes(index).unwrap();

        assert_eq!(result, vec![])
    }

    #[test]
    fn convert_true() {
        let mut data = test_data();
        let index = data.push_object_to_data_block(basic_object!(True)).unwrap();

        let result = data.convert_basic_data_at_to_bytes(index).unwrap();

        assert_eq!(result, vec![1])
    }

    #[test]
    fn convert_false() {
        let mut data = test_data();
        let index = data.push_object_to_data_block(basic_object!(False)).unwrap();

        let result = data.convert_basic_data_at_to_bytes(index).unwrap();

        assert_eq!(result, vec![0])
    }

    #[test]
    fn convert_number() {
        let mut data = test_data();
        let index = data.push_object_to_data_block(basic_object!(Number 12345)).unwrap();

        let result = data.convert_basic_data_at_to_bytes(index).unwrap();

        assert_eq!(result, vec![57, 48, 0, 0])
    }

    #[test]
    fn convert_float() {
        let mut data = test_data();
        let index = data.push_object_to_data_block(basic_object!(Number 12345.6789)).unwrap();

        let result = data.convert_basic_data_at_to_bytes(index).unwrap();

        assert_eq!(result, vec![161, 248, 49, 230, 214, 28, 200, 64])
    }

    #[test]
    fn convert_symbol() {
        let mut data = test_data();
        let index = data.push_object_to_data_block(basic_object!(SymRaw 12345)).unwrap();

        let result = data.convert_basic_data_at_to_bytes(index).unwrap();

        assert_eq!(result, vec![57, 48, 0, 0, 0, 0, 0, 0])
    }

    #[test]
    fn convert_symbol_list() {
        let mut data = test_data();
        let index = data.push_object_to_data_block(basic_object!(SymList(SymRaw 12345, SymRaw 6789))).unwrap();

        let result = data.convert_basic_data_at_to_bytes(index).unwrap();

        assert_eq!(result, vec![57, 48, 0, 0, 0, 0, 0, 0, 133, 26, 0, 0, 0, 0, 0, 0])
    }

    #[test]
    fn convert_char() {
        let mut data = test_data();
        let index = data.push_object_to_data_block(basic_object!(Char 'a')).unwrap();

        let result = data.convert_basic_data_at_to_bytes(index).unwrap();

        assert_eq!(result, vec![97, 0, 0, 0])
    }

    #[test]
    fn convert_byte() {
        let mut data = test_data();
        let index = data.push_object_to_data_block(basic_object!(Byte 100)).unwrap();

        let result = data.convert_basic_data_at_to_bytes(index).unwrap();

        assert_eq!(result, vec![100])
    }

    #[test]
    fn convert_byte_list() {
        let mut data = test_data();
        let index = data.push_object_to_data_block(basic_object!(ByteList 100, 200)).unwrap();

        let result = data.convert_basic_data_at_to_bytes(index).unwrap();

        assert_eq!(result, vec![100, 200])
    }

    #[test]
    fn convert_char_list() {
        let mut data = test_data();
        let index = data.push_object_to_data_block(basic_object!(CharList "abc")).unwrap();

        let result = data.convert_basic_data_at_to_bytes(index).unwrap();

        assert_eq!(result, vec![97, 0, 0, 0, 98, 0, 0, 0, 99, 0, 0, 0])
    }

    #[test]
    fn convert_list() {
        let mut data = test_data();
        let index = data
            .push_object_to_data_block(basic_object!((Number 100), (CharList "Some Text")))
            .unwrap();

        let result = data.convert_basic_data_at_to_bytes(index).unwrap();

        assert_eq!(
            result,
            vec![
                100, 0, 0, 0, 83, 0, 0, 0, 111, 0, 0, 0, 109, 0, 0, 0, 101, 0, 0, 0, 32, 0, 0, 0, 84, 0, 0, 0, 101, 0, 0, 0, 120, 0, 0, 0, 116, 0, 0,
                0
            ]
        )
    }
}
