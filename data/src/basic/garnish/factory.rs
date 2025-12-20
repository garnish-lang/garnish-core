use garnish_lang_traits::{GarnishDataFactory, GarnishDataType};

use crate::{BasicNumber, DataError, NumberIterator, SizeIterator, error::DataErrorType, parse_byte_list, parse_char_list, parse_simple_number, symbol_value};

pub struct BasicDataFactory;

impl GarnishDataFactory<usize, BasicNumber, char, u8, u64, DataError, SizeIterator, NumberIterator> for BasicDataFactory {
    fn size_to_number(from: usize) -> BasicNumber {
        from.into()
    }

    fn number_to_size(from: BasicNumber) -> Option<usize> {
        Some(from.into())
    }

    fn number_to_char(from: BasicNumber) -> Option<char> {
        match from {
            BasicNumber::Integer(v) => match (v as u8).try_into() {
                Ok(c) => Some(c),
                Err(_) => None,
            },
            BasicNumber::Float(_) => None,
        }
    }

    fn number_to_byte(from: BasicNumber) -> Option<u8> {
        match from {
            BasicNumber::Integer(v) => match v.try_into() {
                Ok(b) => Some(b),
                Err(_) => None,
            },
            BasicNumber::Float(_) => None,
        }
    }

    fn char_to_number(from: char) -> Option<BasicNumber> {
        Some((from as i32).into())
    }

    fn char_to_byte(from: char) -> Option<u8> {
        Some(from as u8)
    }

    fn byte_to_number(from: u8) -> Option<BasicNumber> {
        Some((from as i32).into())
    }

    fn byte_to_char(from: u8) -> Option<char> {
        Some(from.into())
    }

    fn parse_number(from: &str) -> Result<BasicNumber, DataError> {
        parse_simple_number(from)
    }

    fn parse_symbol(from: &str) -> Result<u64, DataError> {
        Ok(symbol_value(from.trim_matches(':')))
    }

    fn parse_char(from: &str) -> Result<char, DataError> {
        let l = parse_char_list(from)?;
        if l.len() == 1 {
            Ok(l.chars().nth(0).unwrap())
        } else {
            Err(DataError::new("Could not parse to type", DataErrorType::CouldNotParse(from.to_string(), GarnishDataType::Char)))
        }
    }

    fn parse_byte(from: &str) -> Result<u8, DataError> {
        let l = parse_byte_list(from)?;
        if l.len() == 1 {
            Ok(l[0])
        } else {
            Err(DataError::new("Could not parse to type", DataErrorType::CouldNotParse(from.to_string(), GarnishDataType::Byte)))
        }
    }

    fn parse_char_list(from: &str) -> Result<Vec<char>, DataError> {
        Ok(parse_char_list(from)?.chars().collect())
    }

    fn parse_byte_list(from: &str) -> Result<Vec<u8>, DataError> {
        parse_byte_list(from)
    }

    fn make_size_iterator_range(min: usize, max: usize) -> SizeIterator {
        SizeIterator::new(min, max)
    }

    fn make_number_iterator_range(min: BasicNumber, max: BasicNumber) -> NumberIterator {
        NumberIterator::new(min, max)
    }
}

#[cfg(test)]
mod tests {
    use garnish_lang_traits::{GarnishDataFactory, GarnishDataType};

    use crate::{DataError, NumberIterator, SizeIterator, basic::garnish::factory::BasicDataFactory, error::DataErrorType};

    #[test]
    fn size_to_number() {
        let result = BasicDataFactory::size_to_number(10);
        assert_eq!(result, 10.into());
    }

    #[test]
    fn number_to_size() {
        let result = BasicDataFactory::number_to_size(10.into());
        assert_eq!(result, Some(10));
    }

    #[test]
    fn number_to_char_success() {
        let result = BasicDataFactory::number_to_char(97.into());
        assert_eq!(result, Some('a'));
    }
    
    #[test]
    fn number_to_char_failure() {
        let result = BasicDataFactory::number_to_char(10.5.into());
        assert_eq!(result, None);
    }

    #[test]
    fn number_to_byte_success() {
        let result = BasicDataFactory::number_to_byte(100.into());
        assert_eq!(result, Some(100));
    }

    #[test]
    fn number_to_byte_failure() {
        let result = BasicDataFactory::number_to_byte(10000.into());
        assert_eq!(result, None);
    }

    #[test]
    fn parse_number() {
        let result = BasicDataFactory::parse_number("100").unwrap();
        assert_eq!(result, 100.into());
    }

    #[test]
    fn parse_number_failure() {
        let result = BasicDataFactory::parse_number("abcd");
        assert_eq!(result, Err(DataError::new("invalid float literal", DataErrorType::FailedToParseFloat("abcd".to_string()))));
    }

    #[test]
    fn parse_char_success() {
        let result = BasicDataFactory::parse_char("a").unwrap();
        assert_eq!(result, 'a');
    }

    #[test]
    fn parse_char_failure() {
        let result = BasicDataFactory::parse_char("abcd");
        assert_eq!(result, Err(DataError::new("Could not parse to type", DataErrorType::CouldNotParse("abcd".to_string(), GarnishDataType::Char))));
    }

    #[test]
    fn parse_char_list_success() {
        let result = BasicDataFactory::parse_char_list("abc").unwrap();
        assert_eq!(result, vec!['a', 'b', 'c']);
    }

    #[test]
    fn parse_byte_list_success() {
        let result = BasicDataFactory::parse_byte_list("''100 150 200''").unwrap();
        assert_eq!(result, vec![100, 150, 200]);
    }

    #[test]
    fn parse_byte_list_failure() {
        let result = BasicDataFactory::parse_byte_list("''1000 2000 3000''");
        assert_eq!(result, Err(DataError::new("Number to large for byte value", DataErrorType::NumberToLargeForByteValue("1000".to_string()))));
    }

    #[test]
    fn parse_symbol() {
        let result = BasicDataFactory::parse_symbol("my_symbol").unwrap();
        assert_eq!(result, 8904929874702161741);
    }

    #[test]
    fn parse_symbol_trims_semi_coloon() {
        let result = BasicDataFactory::parse_symbol(":my_symbol").unwrap();
        assert_eq!(result, 8904929874702161741);
    }

    #[test]
    fn make_size_iterator_range() {
        let mut result = BasicDataFactory::make_size_iterator_range(10, 20);
        assert_eq!(result.next(), Some(10));
        assert_eq!(result.next(), Some(11));
        assert_eq!(result.next(), Some(12));
        assert_eq!(result.next(), Some(13));
        assert_eq!(result.next(), Some(14));
        assert_eq!(result.next(), Some(15));
        assert_eq!(result.next(), Some(16));
        assert_eq!(result.next(), Some(17));
        assert_eq!(result.next(), Some(18));
        assert_eq!(result.next(), Some(19));
        assert_eq!(result.next(), None);
    }

    #[test]
    fn make_number_iterator_range() {
        let mut result = BasicDataFactory::make_number_iterator_range(10.into(), 20.into());
        assert_eq!(result.next(), Some(10.into()));
        assert_eq!(result.next(), Some(11.into()));
        assert_eq!(result.next(), Some(12.into()));
        assert_eq!(result.next(), Some(13.into()));
        assert_eq!(result.next(), Some(14.into()));
        assert_eq!(result.next(), Some(15.into()));
        assert_eq!(result.next(), Some(16.into()));
        assert_eq!(result.next(), Some(17.into()));
        assert_eq!(result.next(), Some(18.into()));
        assert_eq!(result.next(), Some(19.into()));
        assert_eq!(result.next(), None);
    }
}