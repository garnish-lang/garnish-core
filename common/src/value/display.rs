use std::fmt;
use crate::value::{ExpressionValueRef, ExpressionValue};
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
            DataType::Pair => format!("{} = {}",
                format_value(&v.get_pair_left().unwrap()),
                format_value(&v.get_pair_right().unwrap())),
            DataType::Range => format!("{}..{}",
                format_value(&v.get_range_start().unwrap()),
                format_value(&v.get_range_end().unwrap())),
            DataType::List => {
                let mut items: Vec<String> = vec![];

                for i in 0..v.list_len().unwrap() {
                    items.push(format_value(&v.get_list_item(i).unwrap()));
                }
                
                items.join(", ")
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
    fn list() {
        let value: ExpressionValue = ExpressionValue::list()
            .add(ExpressionValue::integer(10))
            .add(ExpressionValue::integer(20))
            .add(ExpressionValue::integer(30))
        .into();

        assert_eq!(format!("{}", value.reference().unwrap()), "10, 20, 30");
    }
}

