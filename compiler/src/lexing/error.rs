#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct LexingError {
    message: String,
}

impl LexingError {
    pub fn new(message: String) -> Self {
        LexingError { message }
    }

    pub fn get_message(&self) -> &String {
        &self.message
    }
}

impl From<String> for LexingError {
    fn from(s: String) -> Self {
        LexingError::new(s)
    }
}
