use std::fmt;
use crate::value::ExpressionValueRef;
use crate::DataType;

impl fmt::Display for ExpressionValueRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.get_type() {
            Err(e) => e.get_message().clone(),
            Ok(t) => match t {
                DataType::Unit => String::from("()"),
                DataType::Integer => format!("{}", self.as_integer().unwrap()),
                DataType::Float => format!("{}", self.as_float().unwrap()),
                DataType::Character => format!("'{}'", self.as_string().unwrap()),
                DataType::CharacterList => format!("\"{}\"", self.as_string().unwrap()),
                DataType::Symbol => format!(":{}", self.as_string().unwrap()),
                _ => String::new()
            }
        };
        write!(f, "{}", s)
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
}

