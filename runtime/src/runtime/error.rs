use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use crate::Instruction;

#[derive(Debug, Eq, PartialEq)]
pub struct RuntimeError<Source: 'static + std::error::Error> {
    message: String,
    source: Option<Source>,
}

impl<Source: 'static + std::error::Error> RuntimeError<Source> {
    pub fn new(message: &str) -> Self {
        RuntimeError {
            message: message.to_string(),
            source: None,
        }
    }

    pub fn new_message(message: String) -> Self {
        RuntimeError {
            message,
            source: None,
        }
    }
}

impl<Source: 'static + std::error::Error> Default for RuntimeError<Source> {
    fn default() -> Self {
        RuntimeError {
            message: String::new(),
            source: None,
        }
    }
}

impl<Source: 'static + std::error::Error> Display for RuntimeError<Source> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{:?}", self).as_str())
    }
}

impl<Source: 'static + std::error::Error> std::error::Error for RuntimeError<Source> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.source {
            None => None,
            Some(s) => Some(s),
        }
    }
}

impl<Source: 'static + std::error::Error> From<Source> for RuntimeError<Source> {
    fn from(source: Source) -> Self {
        let mut e = RuntimeError::default();
        e.source = Some(source);
        e
    }
}

// Creation utilites

pub(crate) fn instruction_error<T, E: std::error::Error + 'static, I: Debug>(instruction: Instruction, index: I) -> Result<T, RuntimeError<E>> {
    Err(RuntimeError::new_message(format!("Expected instruction {:?} at index {:?} to have data. Found None.", instruction, index)))
}

pub(crate) fn state_error<T, E: std::error::Error>(message: String) -> Result<T, RuntimeError<E>> {
    Err(RuntimeError::new(message.as_str()))
}