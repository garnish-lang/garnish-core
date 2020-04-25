use std::fmt;
use unicode_segmentation::UnicodeSegmentation;
use crate::value::{ExpressionValueRef, ExpressionValue, is_start_exclusive, is_end_exclusive};
use crate::DataType;

impl fmt::Display for ExpressionValueRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format_value(self))
    }
}

impl fmt::Display for ExpressionValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.reference() {
            Err(_) => write!(f, ""),
            Ok(v) => v.fmt(f)
        }
    }
}

fn format_value(v: &ExpressionValueRef) -> String {
    match v.get_type() {
        Err(e) => e.get_message().clone(),
        Ok(t) => match t {
            DataType::Unit => String::from("()"),
            DataType::Integer => format!("{}", v.as_integer().unwrap()),
            DataType::Float => format!("{}", v.as_float().unwrap()),
            DataType::Character => format!("'{}'", v.as_string().unwrap()),
            DataType::CharacterList => format!("\"{}\"", v.as_string().unwrap()),
            DataType::Symbol => format!(":{}", v.as_string().unwrap()),
            DataType::Expression
            | DataType::ExternalMethod => format!("{}", v.as_string().unwrap()),
            DataType::Pair => format!("{} = {}",
                format_value(&v.get_pair_left().unwrap()),
                format_value(&v.get_pair_right().unwrap())),
            DataType::Partial => format!("{} ~~ {}",
                format_value(&v.get_partial_base().unwrap()),
                format_value(&v.get_partial_value().unwrap())),
            DataType::Link => format!("{} -> {}",
                format_value(&v.get_link_value().unwrap()),
                format_value(&v.get_link_next().unwrap())),
            DataType::Range => {
                let flags = v.get_range_flags().unwrap();
                let op = match (is_start_exclusive(flags), is_end_exclusive(flags)) {
                    (false, false) => "..",
                    (true, false) => ">..",
                    (false, true) => "..<",
                    (true, true) => ">..<"
                };

                format!("{}{}{}",
                    format_value(&v.get_range_start().unwrap()),
                    op,
                    format_value(&v.get_range_end().unwrap()))
            }
            DataType::List => {
                let mut items: Vec<String> = vec![];

                for i in 0..v.list_len().unwrap() {
                    items.push(format_value(&v.get_list_item(i).unwrap()));
                }
                
                items.join(", ")
            }
            DataType::Slice => {
                let range = v.get_slice_range().unwrap();
                let start = range.get_range_start().unwrap().as_integer().unwrap() as usize;
                let end = range.get_range_end().unwrap().as_integer().unwrap() as usize + 1;

                let source = v.get_slice_source().unwrap();

                match source.get_type().unwrap() {
                    DataType::CharacterList => {
                        let s = source.as_string().unwrap();
                        s.graphemes(true).skip(start).take(end - start).collect::<String>()
                    }
                    DataType::List => {
                        let mut items: Vec<String> = vec![];

                        for i in start..end {
                            items.push(format_value(&source.get_list_item(i).unwrap()));
                        }

                        items.join(", ")
                    }
                    t => format!("Cannot slice type of {}", t)
                }
            }
            _ => String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::value::{ExpressionValue, ExpressionValueRef};
    use std::fmt::Display;

    #[test]
    fn unit() {
        let value: ExpressionValue = ExpressionValue::unit().into();
        assert_eq!(format!("{}", value.reference().unwrap()), "()");
    }

    #[test]
    fn integer() {
        let value: ExpressionValue = ExpressionValue::integer(10).into();
        assert_eq!(format!("{}", value.reference().unwrap()), "10");
    }

    #[test]
    fn float() {
        let value: ExpressionValue = ExpressionValue::float(3.14).into();
        assert_eq!(format!("{}", value.reference().unwrap()), "3.14");
    }

    #[test]
    fn character() {
        let value: ExpressionValue = ExpressionValue::character('a'.to_string()).into();
        assert_eq!(format!("{}", value.reference().unwrap()), "'a'");
    }

    #[test]
    fn character_list() {
        let value: ExpressionValue = ExpressionValue::character_list("pandas".into()).into();
        assert_eq!(format!("{}", value.reference().unwrap()), "\"pandas\"");
    }

    #[test]
    fn symbol() {
        let value: ExpressionValue = ExpressionValue::symbol("my_symbol").into();
        assert_eq!(format!("{}", value.reference().unwrap()), ":my_symbol");
    }

    #[test]
    fn expression() {
        let value: ExpressionValue = ExpressionValue::expression("my_symbol").into();
        assert_eq!(format!("{}", value.reference().unwrap()), "my_symbol");
    }

    #[test]
    fn external_method() {
        let value: ExpressionValue = ExpressionValue::external_method("my_symbol").into();
        assert_eq!(format!("{}", value.reference().unwrap()), "my_symbol");
    }

    #[test]
    fn partial() {
        let value: ExpressionValue = ExpressionValue::partial_expression("my_symbol", ExpressionValue::integer(10)).into();
        assert_eq!(format!("{}", value.reference().unwrap()), "my_symbol ~~ 10");
    }

    #[test]
    fn pair() {
        let value: ExpressionValue = ExpressionValue::pair(
            ExpressionValue::symbol("my_symbol"),
            ExpressionValue::integer(100)
        ).into();

        assert_eq!(format!("{}", value.reference().unwrap()), ":my_symbol = 100");
    }

    #[test]
    fn range() {
        let value: ExpressionValue = ExpressionValue::integer_range(Some(10), Some(20)).into();

        assert_eq!(format!("{}", value.reference().unwrap()), "10..20");
    }

    #[test]
    fn start_exclusive_range() {
        let value: ExpressionValue = ExpressionValue::integer_range(Some(10), Some(20)).exclude_start().into();

        assert_eq!(format!("{}", value.reference().unwrap()), "10>..20");
    }

    #[test]
    fn end_exclusive_range() {
        let value: ExpressionValue = ExpressionValue::integer_range(Some(10), Some(20)).exclude_end().into();

        assert_eq!(format!("{}", value.reference().unwrap()), "10..<20");
    }

    #[test]
    fn exclusive_range() {
        let value: ExpressionValue = ExpressionValue::integer_range(Some(10), Some(20)).exclude_start().exclude_end().into();

        assert_eq!(format!("{}", value.reference().unwrap()), "10>..<20");
    }

    #[test]
    fn list_slice() {
        let value: ExpressionValue = ExpressionValue::list_slice(
            ExpressionValue::list()
                .add(ExpressionValue::integer(10))
                .add(ExpressionValue::integer(20))
                .add(ExpressionValue::integer(30))
                .add(ExpressionValue::integer(40))
                .add(ExpressionValue::integer(50))
                .add(ExpressionValue::integer(60)),
            ExpressionValue::integer_range(Some(1), Some(3))
        ).into();

        assert_eq!(format!("{}", value.reference().unwrap()), "20, 30, 40");
    }

    #[test]
    fn character_list_slice() {
        let value: ExpressionValue = ExpressionValue::character_list_slice(
            ExpressionValue::character_list("panda bear".into()),
            ExpressionValue::integer_range(Some(2), Some(8))
        ).into();

        assert_eq!(format!("{}", value.reference().unwrap()), "nda bea");
    }

    #[test]
    fn link() {
        let value: ExpressionValue = ExpressionValue::link(
            ExpressionValue::integer(10),
            ExpressionValue::integer(20)
        ).into();

        assert_eq!(format!("{}", value.reference().unwrap()), "10 -> 20");
    }

    #[test]
    fn list() {
        let value: ExpressionValue = ExpressionValue::list()
            .add(ExpressionValue::integer(10))
            .add(ExpressionValue::integer(20))
            .add(ExpressionValue::integer(30))
        .into();

        assert_eq!(format!("{}", value.reference().unwrap()), "10, 20, 30");
    }
}

