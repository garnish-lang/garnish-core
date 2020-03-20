use crate::runtime::ExpressionRuntime;
use expr_lang_common::Result;

impl ExpressionRuntime {
    pub(crate) fn execute_put(&mut self) -> Result {
        let constant_ref = self.consume_constant_reference()?;
        self.insert_reference_value(constant_ref)
    }
}

#[cfg(test)]
mod tests {
    use crate::runtime::ExpressionRuntime;
    use expr_lang_common::ExpressionValue;
    use expr_lang_instruction_set_builder::InstructionSetBuilder;

    #[test]
    fn perform_put_unit() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit());
    }

    #[test]
    fn put_number() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 10);
    }

    #[test]
    fn put_character() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::character("z".into()))
            .unwrap();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_char().unwrap(), 'z');
    }

    #[test]
    fn put_character_list() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::character_list("cats".into()))
            .unwrap();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "cats");
    }

    #[test]
    fn put_expression() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::expression("cats"))
            .unwrap();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), 2);
        assert_eq!(result.as_string().unwrap(), String::from("cats"));
    }

    #[test]
    fn put_external_method() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::external_method("cats"))
            .unwrap();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), 2);
        assert_eq!(result.as_string().unwrap(), String::from("cats"));
    }

    #[test]
    fn put_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("cats")).unwrap();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), 2);
        assert_eq!(result.as_string().unwrap(), String::from("cats"));
    }
}
