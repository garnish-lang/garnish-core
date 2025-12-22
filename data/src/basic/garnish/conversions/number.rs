use crate::{BasicData, BasicDataCustom, BasicGarnishData, BasicNumber, DataError};

impl<T> BasicGarnishData<T>
where
    T: BasicDataCustom,
{
    pub(crate) fn convert_basic_data_at_to_number(&mut self, from: usize) -> Result<Option<BasicNumber>, DataError> {
        Ok(match self.get_from_data_block_ensure_index(from)? {
            BasicData::Number(number) => Some(number.clone()),
            _ => None,
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
}