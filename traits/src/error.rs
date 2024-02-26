//! Error implementation for use with Garnish runtimes.

use std::error::Error;
use std::fmt::{Display, Formatter};

/// List of possible error types a [`RuntimeError`] can be categorized as.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ErrorType {
    Unknown,
    /// Code used to determine if an operation should be deferred to [`crate::GarnishContext`].
    UnsupportedOpTypes,
}

/// Error implementation for [`crate::GarnishRuntime`] instruction methods.
#[derive(Debug, Eq, PartialEq)]
pub struct RuntimeError<Source: 'static + std::error::Error> {
    code: ErrorType,
    message: String,
    source: Option<Source>,
}

impl<Source: 'static + std::error::Error> RuntimeError<Source> {
    pub fn new(message: &str) -> Self {
        RuntimeError {
            code: ErrorType::Unknown,
            message: message.to_string(),
            source: None,
        }
    }

    pub fn new_message(message: String) -> Self {
        RuntimeError {
            code: ErrorType::Unknown,
            message,
            source: None,
        }
    }

    pub fn unsupported_types() -> Self {
        RuntimeError {
            code: ErrorType::UnsupportedOpTypes,
            message: String::new(),
            source: None,
        }
    }

    pub fn get_message(&self) -> &String { &self.message }

    pub fn get_type(&self) -> ErrorType {
        self.code
    }
}

impl<Source: 'static + std::error::Error> Default for RuntimeError<Source> {
    fn default() -> Self {
        RuntimeError {
            code: ErrorType::Unknown,
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

impl<Source: 'static + std::error::Error> From<RuntimeError<Source>> for String {
    fn from(err: RuntimeError<Source>) -> Self {
        format!("{}", err)
    }
}
