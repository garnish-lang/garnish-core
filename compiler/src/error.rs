use crate::{LexerToken, SecondaryDefinition};
use garnish_lang_runtime::ExpressionDataType;
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
    pub fn new(message: &str, line: usize, column: usize) -> Self {
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

// Creation utilities

pub(crate) fn composition_error<T, S: std::error::Error + 'static>(
    first: SecondaryDefinition,
    second: SecondaryDefinition,
    token: &LexerToken,
) -> Result<T, CompilerError<S>> {
    Err(CompilerError::new_message(format!("A {:?} token cannot follow a {:?} token", second, first)).append_token_details(token))
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

pub(crate) fn data_parse_error<T, S: std::error::Error + 'static>(
    token: LexerToken,
    desired_type: ExpressionDataType,
) -> Result<T, CompilerError<S>> {
    Err(CompilerError::new(
        format!("Could not create {:?} data from string {:?}.", desired_type, token.get_text()).as_str(),
        token.get_line(),
        token.get_column(),
    ))
}
