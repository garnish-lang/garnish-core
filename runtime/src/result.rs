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
