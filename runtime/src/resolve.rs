use crate::context::ExpressionContext;
use crate::ExpressionRuntime;
use garnish_common::Result;

impl ExpressionRuntime {
    pub(crate) fn resolve<T>(&mut self, context: &T) -> Result
    where
        T: ExpressionContext,
    {
        let symbol_index = self.consume_constant_reference()?;
        match self.symbol_table.iter().find(|(_k, v)| **v == symbol_index) {
            None => (),
            Some((name, _v)) => {
                let value = context.resolve(name.clone());
                self.copy_into(&value)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::context::ExpressionContext;
    use crate::runtime::ExpressionRuntime;
    use garnish_common::{ExpressionValue, ExpressionValueRef};
    use garnish_instruction_set_builder::InstructionSetBuilder;

    #[test]
    fn resolve_value_with_default_context() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("main");
        instructions.resolve(&"value".to_string()).unwrap();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        assert!(expression_runtime.execute("main").unwrap().is_unit());
    }

    struct TestContext {}

    impl ExpressionContext for TestContext {
        fn resolve(&self, name: String) -> ExpressionValue {
            match name.as_str() {
                "value" => ExpressionValue::integer(100).into(),
                _ => ExpressionValue::unit().into(),
            }
        }

        fn execute(&self, _name: String, _input: ExpressionValueRef) -> ExpressionValue {
            ExpressionValue::unit().into()
        }
    }

    #[test]
    fn resolve_value_with_custom_context() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("main");
        instructions.resolve(&"value".to_string()).unwrap();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        match expression_runtime.execute_with_context("main".to_string(), &TestContext {}) {
            Err(e) => panic!("Failed to execute {}", e.to_string()),
            Ok(result) => assert_eq!(result.as_integer().unwrap(), 100),
        }
    }
}
