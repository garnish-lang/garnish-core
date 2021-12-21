#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct ParsingError {
    message: String,
}

impl ParsingError {
    pub fn new(message: String) -> Self {
        ParsingError { message }
    }

    pub fn get_message(&self) -> &String {
        &self.message
    }
}

impl From<String> for ParsingError {
    fn from(s: String) -> Self {
        ParsingError::new(s)
    }
}
