use std::result;

pub type Result<T = ()> = result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new(message: &'static str) -> Self {
        Error::new_from_string(message.to_string())
    }

    pub fn new_from_string(message: String) -> Self {
        Error { message }
    }

    pub fn get_message(&self) -> &String {
        &self.message
    }
}

impl ToString for Error {
    fn to_string(&self) -> String {
        format!("Error: {}", self.message)
    }
}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        Error::new(s)
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::new_from_string(s)
    }
}
