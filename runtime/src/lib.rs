pub struct GarnishLangRuntime {

}

impl GarnishLangRuntime {
    pub fn new() -> Self {
        return GarnishLangRuntime {}
    }
}

#[cfg(test)]
mod tests {
    use crate::GarnishLangRuntime;

    #[test]
    fn create_runtime() {
        GarnishLangRuntime::new();
    }
}