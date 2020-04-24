use std::fmt;
use crate::value::ExpressionValueRef;
use crate::DataType;

impl fmt::Display for ExpressionValueRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.get_type() {
            Err(e) => e.get_message().clone(),
            Ok(t) => match t {
                DataType::Integer => format!("{}", self.as_integer().unwrap()),
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
    fn integer() {
        let value: ExpressionValue = ExpressionValue::integer(10).into();
        assert_eq!(format!("{}", value.reference().unwrap()), "10");
    }
}

