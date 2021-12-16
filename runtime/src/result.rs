use std::result::Result;

pub type GarnishLangRuntimeResult<N, T = ()> = Result<T, GarnishLangRuntimeError<N>>;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum GarnishLangRuntimeState {
    Running,
    End,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct GarnishLangRuntimeInfo {
    state: GarnishLangRuntimeState,
}

impl GarnishLangRuntimeInfo {
    pub fn new(state: GarnishLangRuntimeState) -> Self {
        return GarnishLangRuntimeInfo { state };
    }

    pub fn get_state(&self) -> GarnishLangRuntimeState {
        self.state
    }
}

pub type GarnishLangRuntimeError<T> = NestedError<T>;

pub fn error<E>(message: String) -> GarnishLangRuntimeError<E> {
    GarnishLangRuntimeError::new(message)
}

pub trait NestInto<T, E> {
    fn nest_into(self) -> NestableResult<T, E>;
}

pub type NestableResult<T, E> = Result<T, NestedError<E>>;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct NestedError<T> {
    message: String,
    caused_by: Option<T>,
}

impl<T> NestedError<T> {
    pub fn new(message: String) -> Self {
        return GarnishLangRuntimeError { message, caused_by: None };
    }

    pub fn get_message(&self) -> &String {
        &self.message
    }
}

impl<T, E> NestInto<T, E> for Result<T, E> {
    fn nest_into(self) -> NestableResult<T, E> {
        match self {
            Err(e) => Err(NestedError {
                message: String::new(),
                caused_by: Some(e),
            }),
            Ok(v) => Ok(v),
        }
    }
}
