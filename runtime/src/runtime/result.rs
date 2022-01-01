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
