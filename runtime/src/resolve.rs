use crate::context::ExpressionContext;
use std::convert::{TryFrom, TryInto};
use crate::ExpressionRuntime;
use garnish_lang_common::{Result, DataType, skip_type_and_sizes, get_value_with_hash, skip_type_and_size, skip_sizes, skip_type};

impl ExpressionRuntime {
    pub(crate) fn resolve<T>(&mut self, context: &T) -> Result
    where
        T: ExpressionContext,
    {
        let symbol_index = self.consume_constant_reference()?;
        match self.symbol_table.iter().find(|(_k, v)| **v == symbol_index) {
            None => (),
            Some((name, v)) if self.input_stack.len() > 0 => {
                let input_ref = self.current_input_ref();
                println!("has input {}", self.data[input_ref]);
                if DataType::try_from(self.data[input_ref])? == DataType::List {
                    let length = self.read_size_at(skip_type(input_ref))?;
                    let key_count = self.read_size_at(skip_type_and_size(input_ref))?;
                    let key_area_start = skip_type_and_sizes(input_ref, 2 + length);
                    let key_area_range = key_area_start..skip_sizes(key_area_start, key_count);

                    println!("is list");
                    match get_value_with_hash(*v, &self.data, key_count, &key_area_range)? {
                        Some(item) => {
                            println!("found");
                            let pair_value_ref = self.read_size_at(skip_type_and_size(item))?;
                            self.insert_reference_value(pair_value_ref);
                        }
                        // input list doesn't have key
                        // check expression table for a match
                        None => match self.expression_map.get(name) {
                            Some(expression_index) => self.insert_expression_value(*expression_index)?,
                            None => {
                                // final check is the context on this runtime
                                let value = context.resolve(name.clone());
                                self.copy_into(&value)?;
                            }
                        }
                    }
                } else {
                    // input isn't a list
                    // check expression table for a match
                    match self.expression_map.get(name) {
                        Some(expression_index) => self.insert_expression_value(*expression_index)?,
                        None => {
                            // final check is the context on this runtime
                            let value = context.resolve(name.clone());
                            self.copy_into(&value)?;
                        }
                    }
                }
            }
            Some((name, _)) => {
                // no input to check
                // check expression table for a match
                match self.expression_map.get(name) {
                    Some(expression_index) => self.insert_expression_value(*expression_index)?,
                    None => {
                        // final check is the context on this runtime
                        let value = context.resolve(name.clone());
                        self.copy_into(&value)?;
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::context::ExpressionContext;
    use crate::runtime::ExpressionRuntime;
    use garnish_lang_common::{ExpressionValue, ExpressionValueRef};
    use garnish_lang_instruction_set_builder::InstructionSetBuilder;

    #[test]
    fn resolve_value_with_default_context() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("main");
        instructions.resolve(&"value".to_string()).unwrap();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        assert!(expression_runtime.execute("main").unwrap().is_unit());
    }

    #[test]
    fn resolve_value_with_input() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("main");
        instructions.resolve(&"value".to_string()).unwrap();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        expression_runtime.set_input(&ExpressionValue::list()
            .add(ExpressionValue::pair(
                ExpressionValue::symbol("value"),
                ExpressionValue::integer(100)
            )).into());

        let result = expression_runtime.execute("main").unwrap();
        
        println!("{}", result.get_type().unwrap());

        assert_eq!(result.as_integer().unwrap(), 100);
    }

    #[test]
    fn resolve_value_with_other_expression() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("main");
        instructions.resolve(&"value".to_string()).unwrap();
        instructions.end_expression();

        instructions.start_expression("value");
        instructions.put(ExpressionValue::integer(100));
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        assert_eq!(expression_runtime.execute("main").unwrap().as_string().unwrap(), String::from("value"));
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
