use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Eq, PartialEq)]
pub struct NoSource {

}

impl Display for NoSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("")
    }
}

impl std::error::Error for NoSource {

}

#[derive(Debug, Eq, PartialEq)]
pub struct CompilerError<Source: 'static + std::error::Error = NoSource> {
    message: String,
    line: usize,
    column: usize,
    source: Option<Source>
}

impl<Source: 'static + std::error::Error> Default for CompilerError<Source> {
    fn default() -> Self {
        CompilerError { message: String::new(), line: 0, column: 0, source: None }
    }
}

impl<Source: 'static + std::error::Error> CompilerError<Source> {
    pub fn new(message: &str, line: usize, column: usize) -> Self {
        CompilerError { message: message.to_string(), line, column, source: None }
    }
}

impl<Source: 'static + std::error::Error> Display for CompilerError<Source> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{:?}", self).as_str())
    }
}

impl<Source: 'static + std::error::Error> std::error::Error for CompilerError<Source> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.source {
            None => None,
            Some(s) => Some(s)
        }
    }
}

impl<Source: 'static + std::error::Error> From<Source> for CompilerError<Source> {
    fn from(source: Source) -> Self {
        let mut e = CompilerError::default();
        e.source = Some(source);
        e
    }
}