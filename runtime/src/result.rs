pub type GarnishLangRuntimeResult<T=()> = Result<T, GarnishLangRuntimeError>;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct GarnishLangRuntimeData {
    message: String,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct GarnishLangRuntimeError {
    message: String
}

impl GarnishLangRuntimeData {
    pub fn new(message: String) -> Self {
        return GarnishLangRuntimeData { message }
    }

    pub fn get_message(&self) -> &String {
        &self.message
    }
}

impl GarnishLangRuntimeError {
    pub fn new(message: String) -> Self {
        return GarnishLangRuntimeError { message }
    }

    pub fn get_message(&self) -> &String {
        &self.message
    }
}

pub fn result() -> GarnishLangRuntimeData {
    GarnishLangRuntimeData::new("".to_string())
}

pub fn error(message: String) -> GarnishLangRuntimeError {
    GarnishLangRuntimeError::new(message)
}
