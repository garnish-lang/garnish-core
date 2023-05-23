use crate::data::SimpleNumber;
use crate::DataError;
use std::iter;
use std::str::FromStr;

pub fn parse_char_list(input: &str) -> Result<String, DataError> {
    let mut new = String::new();

    if input.len() == 0 {
        return Ok(new);
    }

    let mut start_quote_count = 0;
    for c in input.chars() {
        if c == '"' {
            start_quote_count += 1;
        } else {
            break;
        }
    }

    let l = input.len();
    if start_quote_count == input.len() {
        return Ok(new);
    }

    let v = input.chars().collect::<Vec<char>>();
    let real_len = input.len() - start_quote_count * 2;

    let mut check_escape = false;
    let mut in_unicode = false;
    let mut unicode_characters = String::new();

    for c in input.chars().skip(start_quote_count).take(real_len) {
        if in_unicode {
            if c == '}' {
                match parse_number_internal(unicode_characters.as_str(), 16)? {
                    SimpleNumber::Float(_) => Err(DataError::from(format!(
                        "Float numbers are not allowed in Unicode escape. {:?}",
                        unicode_characters
                    )))?,
                    SimpleNumber::Integer(v) => match char::from_u32(v as u32) {
                        None => Err(DataError::from(format!(
                            "Invalid unicode value {:?}. Max is {:?}",
                            unicode_characters,
                            char::MAX.to_digit(16)
                        )))?,
                        Some(v) => {
                            new.push(v);
                            unicode_characters = String::new();
                        }
                    },
                }
                in_unicode = false;
            } else {
                if c != '{' {
                    unicode_characters.push(c);
                }
            }

            continue;
        }

        if check_escape {
            match c {
                'n' => new.push('\n'),
                't' => new.push('\t'),
                'r' => new.push('\r'),
                '0' => new.push('\0'),
                '\\' => new.push('\\'),
                '"' => new.push('"'),
                'u' => in_unicode = true,
                _ => return Err(DataError::from(format!("Invalid escape character '{}'", c))),
            }

            check_escape = false;
            continue;
        }

        match c {
            '\\' => check_escape = true,
            '\n' | '\t' if start_quote_count <= 1 => (), // skip
            _ => new.push(c),
        }
    }

    Ok(new)
}

pub fn parse_byte_list(input: &str) -> Result<Vec<u8>, DataError> {
    let mut bytes = vec![];

    let mut start_quote_count = 0;
    for c in input.chars() {
        if c == '\'' {
            start_quote_count += 1;
        } else {
            break;
        }
    }

    let real_len = input.len() - start_quote_count * 2;

    if start_quote_count >= 2 {
        parse_byte_list_numbers(&input[start_quote_count..(input.len() - start_quote_count)])
    } else {
        let mut check_escape = false;
        for c in input.chars().skip(start_quote_count).take(real_len) {
            if check_escape {
                match c {
                    'n' => bytes.push('\n' as u8),
                    't' => bytes.push('\t' as u8),
                    'r' => bytes.push('\r' as u8),
                    '0' => bytes.push('\0' as u8),
                    '\\' => bytes.push('\\' as u8),
                    '\'' => bytes.push('\'' as u8),
                    _ => return Err(DataError::from(format!("Invalid escape character '{}'", c))),
                }

                check_escape = false;
                continue;
            }

            if c == '\\' {
                check_escape = true
            } else {
                bytes.push(c as u8);
            }
        }

        Ok(bytes)
    }
}

pub fn parse_byte_list_numbers(input: &str) -> Result<Vec<u8>, DataError> {
    let mut current_number = String::new();
    let mut numbers = vec![];

    for c in input.chars().chain(iter::once(' ')) {
        if c.is_numeric() || c == '_' {
            current_number.push(c);
        } else if c == ' ' && current_number.len() > 0 {
            match parse_simple_number(current_number.as_str())? {
                SimpleNumber::Float(_) => Err(DataError::from(format!(
                    "Float numbers are not allowed in ByteLists. {:?}",
                    current_number
                )))?,
                SimpleNumber::Integer(v) => {
                    if v < 0 || v > u8::MAX as i32 {
                        Err(DataError::from(format!("Number to large for byte value {:?}", current_number)))?;
                    }

                    numbers.push(v as u8);
                    current_number = String::new();
                }
            }
        } else {
            Err(DataError::from(format!("Invalid character in byte number {:?}", c)))?;
        }
    }

    Ok(numbers)
}

pub fn parse_simple_number(input: &str) -> Result<SimpleNumber, DataError> {
    parse_number_internal(input, 10)
}

fn parse_number_internal(input: &str, default_radix: u32) -> Result<SimpleNumber, DataError> {
    let (radix, input) = match input.find('_') {
        None => (default_radix, input),
        Some(i) => {
            let part = &input[0..i];
            if part.starts_with("0") {
                let trimmed = part.trim_matches('0');
                match u32::from_str(trimmed) {
                    Err(_) => Err(DataError::from(format!("Could not parse radix from {:?}", part)))?,
                    Ok(v) => {
                        if v < 2 || v > 36 {
                            // limit of Rust from_str_radix function below
                            Err(DataError::from(format!("Radix must be with in range [2, 36]. Found {:?}", v)))?
                        } else {
                            // + 1 to skip the underscore
                            (v, &input[i + 1..])
                        }
                    }
                }
            } else {
                (default_radix, input)
            }
        }
    };

    match i32::from_str_radix(input, radix) {
        Ok(v) => Ok(v.into()),
        Err(_) => {
            if radix == 10 {
                match f64::from_str(input) {
                    Ok(v) => Ok(v.into()),
                    Err(_) => Err(DataError::from(format!("Could not create SimpleNumber from string {:?}", input))),
                }
            } else {
                Err(DataError::from(format!("Decimal values only support a radix of 10. Found {:?}", radix)))
            }
        }
    }
}

#[cfg(test)]
mod numbers {
    use crate::data::parse_simple_number;
    use crate::data::SimpleNumber::*;

    #[test]
    fn just_numbers_integer() {
        let input = "123456";
        assert_eq!(parse_simple_number(input).unwrap(), Integer(123456));
    }

    #[test]
    fn negative_integer() {
        let input = "-123456";
        assert_eq!(parse_simple_number(input).unwrap(), Integer(-123456));
    }

    #[test]
    fn min_integer() {
        let input = i32::MIN.to_string();
        assert_eq!(parse_simple_number(input.as_str()).unwrap(), Integer(i32::MIN));
    }

    #[test]
    fn max_integer() {
        let input = i32::MAX.to_string();
        assert_eq!(parse_simple_number(input.as_str()).unwrap(), Integer(i32::MAX));
    }

    #[test]
    fn just_numbers_integer_err() {
        let input = "123456?";
        assert!(parse_simple_number(input).is_err());
    }

    #[test]
    fn just_numbers_float() {
        let input = "123456.789";
        assert_eq!(parse_simple_number(input).unwrap(), Float(123456.789));
    }

    #[test]
    fn negative_float() {
        let input = "-123456.789";
        assert_eq!(parse_simple_number(input).unwrap(), Float(-123456.789));
    }

    #[test]
    fn just_numbers_float_err() {
        let input = "123456.789?";
        assert!(parse_simple_number(input).is_err());
    }

    #[test]
    fn just_numbers_base_2() {
        let input = "02_1010101";
        assert_eq!(parse_simple_number(input).unwrap(), Integer(0b1010101));
    }

    #[test]
    fn just_numbers_base_36() {
        let input = "036_C7R";
        assert_eq!(parse_simple_number(input).unwrap(), Integer(15831));
    }

    #[test]
    fn just_numbers_base_1_is_err() {
        let input = "01_1010101";
        assert!(parse_simple_number(input).is_err());
    }

    #[test]
    fn just_numbers_base_37_is_err() {
        let input = "037_1010101";
        assert!(parse_simple_number(input).is_err());
    }

    #[test]
    fn radix_valid_float_is_err() {
        let input = "02_10101.0101";
        assert!(parse_simple_number(input).is_err());
    }

    #[test]
    fn radix_invalid_float_is_err() {
        let input = "016_A6.789";
        assert!(parse_simple_number(input).is_err());
    }
}

#[cfg(test)]
mod char_list {
    use crate::data::parse_char_list;

    #[test]
    fn true_empty() {
        let input = "";
        assert_eq!(parse_char_list(input).unwrap(), "".to_string())
    }

    #[test]
    fn empty() {
        let input = "\"\"";
        assert_eq!(parse_char_list(input).unwrap(), "".to_string())
    }

    #[test]
    fn empty_multi_quote() {
        let input = "\"\"\"\"\"\"";
        assert_eq!(parse_char_list(input).unwrap(), "".to_string())
    }

    #[test]
    fn skip_starting_and_ending_quotes() {
        let input = "\"\"\"Some String\"\"\"";
        assert_eq!(parse_char_list(input).unwrap(), "Some String".to_string())
    }

    #[test]
    fn newlines_and_tabs_are_removed_in_single_double_quote() {
        let input = "\"Some\n\t\t\tString\"";
        assert_eq!(parse_char_list(input).unwrap(), "SomeString".to_string())
    }

    #[test]
    fn newlines_and_tabs_are_retained_in_multi_double_quote() {
        let input = "\"\"Some\n\t\t\tString\"\"";
        assert_eq!(parse_char_list(input).unwrap(), "Some\n\t\t\tString".to_string())
    }

    #[test]
    fn convert_newlines() {
        let input = "Some\\nString";
        assert_eq!(parse_char_list(input).unwrap(), "Some\nString".to_string())
    }

    #[test]
    fn convert_unicode() {
        let input = "Some\\u{25A1}String";
        assert_eq!(parse_char_list(input).unwrap(), "Some\u{25A1}String".to_string())
    }

    #[test]
    fn convert_multiple_newlines() {
        let input = "Some\\n\\nString";
        assert_eq!(parse_char_list(input).unwrap(), "Some\n\nString".to_string())
    }

    #[test]
    fn convert_tabs() {
        let input = "Some\\tString";
        assert_eq!(parse_char_list(input).unwrap(), "Some\tString".to_string())
    }

    #[test]
    fn convert_carriage_return() {
        let input = "Some\\rString";
        assert_eq!(parse_char_list(input).unwrap(), "Some\rString".to_string())
    }

    #[test]
    fn convert_null() {
        let input = "Some\\0String";
        assert_eq!(parse_char_list(input).unwrap(), "Some\0String".to_string())
    }

    #[test]
    fn convert_backslash() {
        let input = "Some\\\\String";
        assert_eq!(parse_char_list(input).unwrap(), "Some\\String".to_string())
    }

    #[test]
    fn convert_quote() {
        let input = "Some\\\"String";
        assert_eq!(parse_char_list(input).unwrap(), "Some\"String".to_string())
    }

    #[test]
    fn invalid_escape_sequence() {
        let input = "Some\\yString";
        assert!(parse_char_list(input).is_err())
    }

    #[test]
    fn invalid_unicode() {
        let input = "Some\\u{FFFFFF}String";
        assert!(parse_char_list(input).is_err())
    }
}

#[cfg(test)]
mod byte_list {
    use crate::data::parse_byte_list;

    #[test]
    fn skip_starting_and_ending_quotes() {
        let input = "'a'";
        assert_eq!(parse_byte_list(input).unwrap(), vec!['a' as u8])
    }

    #[test]
    fn convert_newlines() {
        let input = "'\\n'";
        assert_eq!(parse_byte_list(input).unwrap(), vec!['\n' as u8])
    }

    #[test]
    fn convert_tabs() {
        let input = "'\\t'";
        assert_eq!(parse_byte_list(input).unwrap(), vec!['\t' as u8])
    }

    #[test]
    fn convert_carriage_return() {
        let input = "'\\r'";
        assert_eq!(parse_byte_list(input).unwrap(), vec!['\r' as u8])
    }

    #[test]
    fn convert_null() {
        let input = "'\\0'";
        assert_eq!(parse_byte_list(input).unwrap(), vec!['\0' as u8])
    }

    #[test]
    fn convert_backslash() {
        let input = "'\\\\'";
        assert_eq!(parse_byte_list(input).unwrap(), vec!['\\' as u8])
    }

    #[test]
    fn convert_quote() {
        let input = "'\\''";
        assert_eq!(parse_byte_list(input).unwrap(), vec!['\'' as u8])
    }

    #[test]
    fn invalid_escape_sequence() {
        let input = "'\\y'";
        assert!(parse_byte_list(input).is_err())
    }

    #[test]
    fn double_quote_is_series_off_byte_numbers() {
        let input = "''100 150 200 250''";
        assert_eq!(parse_byte_list(input).unwrap(), vec![100, 150, 200, 250])
    }

    #[test]
    fn double_quote_is_series_off_byte_numbers_radix_two() {
        let input = "''02_1111 02_0101 02_1001''";
        assert_eq!(parse_byte_list(input).unwrap(), vec![0b1111, 0b0101, 0b1001])
    }

    #[test]
    fn double_quote_is_series_off_byte_numbers_invalid_number() {
        let input = "''abc 150 200 250''";
        assert!(parse_byte_list(input).is_err())
    }

    #[test]
    fn double_quote_is_series_off_byte_numbers_number_to_large() {
        let input = "''100 300 150''";
        assert!(parse_byte_list(input).is_err())
    }

    #[test]
    fn double_quote_is_series_off_byte_numbers_number_negative() {
        let input = "''100 -150 200''";
        assert!(parse_byte_list(input).is_err())
    }
}
