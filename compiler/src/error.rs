use crate::{LexerToken, SecondaryDefinition};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Eq, PartialEq)]
pub struct NoSource {}

impl Display for NoSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("")
    }
}

impl std::error::Error for NoSource {}

#[derive(Debug, Eq, PartialEq)]
pub struct CompilerError<Source: 'static + std::error::Error = NoSource> {
    message: String,
    line: usize,
    column: usize,
    source: Option<Source>,
}

impl<Source: 'static + std::error::Error> CompilerError<Source> {
    pub fn new<T: ToString>(message: T, line: usize, column: usize) -> Self {
        CompilerError {
            message: message.to_string(),
            line,
            column,
            source: None,
        }
    }

    pub fn new_message(message: String) -> Self {
        CompilerError {
            message,
            line: 0,
            column: 0,
            source: None,
        }
    }

    pub fn append_token_details(mut self, token: &LexerToken) -> Self {
        self.line = token.get_line();
        self.column = token.get_column();
        self
    }

    pub fn get_message(&self) -> &String {
        &self.message
    }
}

impl<Source: 'static + std::error::Error> Default for CompilerError<Source> {
    fn default() -> Self {
        CompilerError {
            message: String::new(),
            line: 0,
            column: 0,
            source: None,
        }
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
            Some(s) => Some(s),
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

impl<Source: 'static + std::error::Error> From<CompilerError<Source>> for String {
    fn from(e: CompilerError<Source>) -> Self {
        match e.source {
            None => format!("{} at line {} col {}", e.message, e.line, e.column),
            Some(s) => format!("{} at line {} col {}", s, e.line, e.column),
        }
    }
}

// Creation utilities

pub(crate) fn composition_error<T, S: std::error::Error + 'static>(
    first: SecondaryDefinition,
    second: SecondaryDefinition,
    token: &LexerToken,
) -> Result<T, CompilerError<S>> {
    Err(CompilerError::new_message(format!("Syntax Error: A {:?} token cannot follow a {:?} token", second, first)).append_token_details(token))
}

pub(crate) fn unmatched_grouping_error<T, S: std::error::Error + 'static>(token: &LexerToken) -> Result<T, CompilerError<S>> {
    Err(CompilerError::new_message(format!("Syntax Error: Unmatched grouping token")).append_token_details(token))
}

pub(crate) fn unclosed_grouping_error<T, S: std::error::Error + 'static>(token: &LexerToken) -> Result<T, CompilerError<S>> {
    Err(CompilerError::new_message(format!("Syntax Error: Unclosed grouping")).append_token_details(token))
}

pub(crate) fn implementation_error<T, S: std::error::Error + 'static>(message: String) -> Result<T, CompilerError<S>> {
    Err(CompilerError::new_message(format!("Implementation Error: {}", message)))
}

pub(crate) fn implementation_error_with_token<T, S: std::error::Error + 'static>(message: String, token: &LexerToken) -> Result<T, CompilerError<S>> {
    append_token_details(Err(CompilerError::new_message(format!("Implementation Error: {}", message))), token)
}

pub(crate) fn append_token_details<T, S: std::error::Error + 'static>(
    result: Result<T, CompilerError<S>>,
    token: &LexerToken,
) -> Result<T, CompilerError<S>> {
    result.map_err(|e| e.append_token_details(token))
}
