use std::result::Result;

pub type GarnishLangRuntimeResult<T = ()> = Result<T, GarnishLangRuntimeError>;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum GarnishLangRuntimeState {
    Running,
    End,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct GarnishLangRuntimeData {
    state: GarnishLangRuntimeState,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct GarnishLangRuntimeError {
    message: String,
}

impl GarnishLangRuntimeData {
    pub fn new(state: GarnishLangRuntimeState) -> Self {
        return GarnishLangRuntimeData { state };
    }

    pub fn get_state(&self) -> GarnishLangRuntimeState {
        self.state
    }
}

impl GarnishLangRuntimeError {
    pub fn new(message: String) -> Self {
        return GarnishLangRuntimeError { message };
    }

    pub fn get_message(&self) -> &String {
        &self.message
    }
}

pub fn error(message: String) -> GarnishLangRuntimeError {
    GarnishLangRuntimeError::new(message)
}

pub trait DropResult<T, F> {
    fn drop(self) -> Result<(), F>;
}

impl<T, F> DropResult<T, F> for Result<T, F> {
    fn drop(self) -> Result<(), F> {
        self.and(Ok(()))
    }
}

pub trait RuntimeResult<T> {
    fn as_runtime_result(self) -> GarnishLangRuntimeResult<T>;
}

impl<T, F> RuntimeResult<T> for Result<T, F>
where
    F: ToString,
{
    fn as_runtime_result(self) -> GarnishLangRuntimeResult<T> {
        match self {
            Err(e) => Err(error(e.to_string())),
            Ok(v) => Ok(v),
        }
    }
}

impl<T> RuntimeResult<T> for Option<T> {
    fn as_runtime_result(self) -> GarnishLangRuntimeResult<T> {
        match self {
            None => Err(error(format!("No value found"))),
            Some(v) => Ok(v),
        }
    }
}
