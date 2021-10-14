pub type GarnishLangRuntimeResult<T=()> = Result<T, GarnishLangRuntimeError>;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct GarnishLangRuntimeData<T> {
    message: String,
    data: T
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct GarnishLangRuntimeError {
    message: String
}

impl <T> GarnishLangRuntimeData<T> {
    pub fn new(message: String, data: T) -> Self {
        return GarnishLangRuntimeData { message, data }
    }

    pub fn new_from_data(data: T) -> Self {
        return GarnishLangRuntimeData { message: String::default(), data }
    }

    pub fn get_message(&self) -> &String {
        &self.message
    }

    pub fn get_data(&self) -> &T {
        &self.data
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

pub fn result() -> GarnishLangRuntimeData<()> {
    GarnishLangRuntimeData::new("".to_string(), ())
}

pub fn error(message: String) -> GarnishLangRuntimeError {
    GarnishLangRuntimeError::new(message)
}
