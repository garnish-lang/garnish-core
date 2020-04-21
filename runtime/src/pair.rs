use crate::ExpressionRuntime;
use garnish_lang_common::Result;

impl ExpressionRuntime {
    pub(crate) fn make_pair(&mut self) -> Result {
        let (left_value, right_value) = self.consume_last_two_refs()?;
        self.insert_pair_value(left_value, right_value)
    }
}

#[cfg(test)]
mod tests {
    use crate::ExpressionRuntime;
    use garnish_lang_common::ExpressionValue;
    use garnish_lang_instruction_set_builder::InstructionSetBuilder;

    #[test]
    fn make_pair() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.make_pair();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_pair_left().unwrap().as_integer().unwrap(), 10);
        assert_eq!(result.get_pair_right().unwrap().as_integer().unwrap(), 20);
    }

    #[test]
    fn make_pair_with_one_value_has_error_result() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.make_pair();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        assert!(expression_runtime.execute("main").is_err());
    }

    #[test]
    fn make_pair_with_no_values_has_error_result() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.make_pair();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        assert!(expression_runtime.execute("main").is_err());
    }
}
