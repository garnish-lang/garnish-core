use garnish_lang_traits::GarnishDataFactory;

use crate::{BasicNumber, DataError, NumberIterator, SizeIterator, symbol_value};

pub struct BasicDataFactory;

impl GarnishDataFactory<usize, BasicNumber, char, u8, u64, DataError, SizeIterator, NumberIterator> for BasicDataFactory {
    fn size_to_number(from: usize) -> BasicNumber {
        todo!()
    }

    fn number_to_size(from: BasicNumber) -> Option<usize> {
        todo!()
    }

    fn number_to_char(from: BasicNumber) -> Option<char> {
        todo!()
    }

    fn number_to_byte(from: BasicNumber) -> Option<u8> {
        todo!()
    }

    fn char_to_number(from: char) -> Option<BasicNumber> {
        todo!()
    }

    fn char_to_byte(from: char) -> Option<u8> {
        todo!()
    }

    fn byte_to_number(from: u8) -> Option<BasicNumber> {
        todo!()
    }

    fn byte_to_char(from: u8) -> Option<char> {
        todo!()
    }

    fn parse_number(from: &str) -> Result<BasicNumber, DataError> {
        todo!()
    }

    fn parse_symbol(from: &str) -> Result<u64, DataError> {
        Ok(symbol_value(from))
    }

    fn parse_char(from: &str) -> Result<char, DataError> {
        todo!()
    }

    fn parse_byte(from: &str) -> Result<u8, DataError> {
        todo!()
    }

    fn parse_char_list(from: &str) -> Result<Vec<char>, DataError> {
        todo!()
    }

    fn parse_byte_list(from: &str) -> Result<Vec<u8>, DataError> {
        todo!()
    }

    fn make_size_iterator_range(min: usize, max: usize) -> SizeIterator {
        SizeIterator::new(min, max)
    }

    fn make_number_iterator_range(min: BasicNumber, max: BasicNumber) -> NumberIterator {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use garnish_lang_traits::GarnishDataFactory;

    use crate::basic::garnish::factory::BasicDataFactory;

    #[test]
    fn parse_symbol() {
        let result = BasicDataFactory::parse_symbol("my_symbol").unwrap();
        assert_eq!(result, 8904929874702161741);
    }
}