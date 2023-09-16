use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Clone)]
pub struct DataError {
    message: String,
}

impl Display for DataError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message.as_str())
    }
}

impl std::error::Error for DataError {}

impl From<String> for DataError {
    fn from(s: String) -> Self {
        DataError { message: s }
    }
}

impl From<DataError> for String {
    fn from(err: DataError) -> Self {
        format!("{}", err)
    }
}
