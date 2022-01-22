use crate::DataError;

pub fn parse_char_list(input: &str) -> Result<String, DataError> {
    let mut new = String::new();

    let mut start_quote_count = 0;
    for c in input.chars() {
        if c == '"' {
            start_quote_count += 1;
        } else {
            break;
        }
    }

    let real_len = input.len() - start_quote_count * 2;

    let mut check_escape = false;
    for c in input.chars().skip(start_quote_count).take(real_len) {
        if check_escape {
            match c {
                'n' => new.push('\n'),
                't' => new.push('\t'),
                'r' => new.push('\r'),
                '0' => new.push('\0'),
                '\\' => new.push('\\'),
                '"' => new.push('"'),
                _ => return Err(DataError::from(format!("Invalid escape character '{}'", c))),
            }

            check_escape = false;
            continue;
        }

        if c == '\\' {
            check_escape = true
        } else {
            new.push(c);
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

#[cfg(test)]
mod char_list {
    use crate::simple::data::parsing::parse_char_list;

    #[test]
    fn skip_starting_and_ending_quotes() {
        let input = "\"\"\"Some String\"\"\"";
        assert_eq!(parse_char_list(input).unwrap(), "Some String".to_string())
    }

    #[test]
    fn convert_newlines() {
        let input = "Some\\nString";
        assert_eq!(parse_char_list(input).unwrap(), "Some\nString".to_string())
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
}

#[cfg(test)]
mod byte_list {
    use crate::simple::data::parsing::parse_byte_list;

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
}
