fn parse_char_list(input: &str) -> String {
    let mut new = String::new();

    let mut check_escape = false;
    for c in input.chars() {
        if check_escape {
            match c {
                'n' => new.push('\n'),
                't' => new.push('\t'),
                'r' => new.push('\r'),
                '0' => new.push('\0'),
                '\\' => new.push('\\'),
                '"' => new.push('"'),
                _ => (),
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

    new
}

#[cfg(test)]
mod tests {
    use crate::simple::data::parsing::parse_char_list;

    #[test]
    fn convert_newlines() {
        let input = "Some\\nString";
        assert_eq!(parse_char_list(input), "Some\nString".to_string())
    }

    #[test]
    fn convert_tabs() {
        let input = "Some\\tString";
        assert_eq!(parse_char_list(input), "Some\tString".to_string())
    }

    #[test]
    fn convert_carriage_return() {
        let input = "Some\\rString";
        assert_eq!(parse_char_list(input), "Some\rString".to_string())
    }

    #[test]
    fn convert_null() {
        let input = "Some\\0String";
        assert_eq!(parse_char_list(input), "Some\0String".to_string())
    }

    #[test]
    fn convert_backslash() {
        let input = "Some\\\\String";
        assert_eq!(parse_char_list(input), "Some\\String".to_string())
    }

    #[test]
    fn convert_quote() {
        let input = "Some\\\"String";
        assert_eq!(parse_char_list(input), "Some\"String".to_string())
    }
}
