use crate::value::{ExpressionValue, ExpressionValueRef};
use crate::{skip_type, DataType, Result, INTEGER_LENGTH};
use std::collections::HashMap;
use std::convert::TryFrom;

impl ExpressionValueRef<'_> {
    pub fn value(&self) -> Result<ExpressionValue> {
        let mut data: Vec<u8> = vec![];

        let mut cursor = self.value_start;
        loop {
            match DataType::try_from(self.data[cursor])? {
                DataType::Integer => {
                    for i in cursor..(skip_type(cursor) + INTEGER_LENGTH) {
                        data.push(self.data[i]);
                    }

                    break;
                }
                _ => unimplemented!(),
            }
        }

        let mut symbol_table = HashMap::new();
        symbol_table.insert("".to_string(), 0);

        Ok(ExpressionValue {
            data,
            start: 0,
            error: None,
            symbol_table,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::value::{ExpressionValue, ExpressionValueBuilder, ExpressionValueRef};

    #[test]
    fn integer() {
        let value = ExpressionValue::integer(10).build();
        assert_eq!(value.reference().unwrap().value().unwrap(), value);
    }
}
