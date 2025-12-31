use garnish_lang_traits::GarnishDataFactory;
use crate::{BasicDataCustom, BasicDataFactory, BasicGarnishData, DataError, basic::companion::BasicDataCompanion};

impl<T, Companion> BasicGarnishData<T, Companion>
where
    T: BasicDataCustom,
    Companion: BasicDataCompanion<T>,
{
    pub(crate) fn convert_basic_data_at_to_symbol(&mut self, from: usize) -> Result<u64, DataError> {
        let s = self.string_from_basic_data_at(from)?;
        BasicDataFactory::parse_symbol(s.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use garnish_lang_traits::GarnishDataFactory;

    use crate::{BasicDataFactory, basic::utilities::test_data, basic_object};

    #[test]
    fn test_convert_basic_data_at_to_symbol() {
        let mut data = test_data();
        let index = data.push_object_to_data_block(
            basic_object!((SymRaw 1234), (CharList "Some Text"), ((Number 100) = (Number 200)))
        ).unwrap();

        let expected_symbol = BasicDataFactory::parse_symbol("[Symbol 1234] Some Text (100 = 200)").unwrap();
        assert_eq!(data.convert_basic_data_at_to_symbol(index), Ok(expected_symbol));
    }
}