use crate::context::ExpressionContext;
use crate::ExpressionRuntime;
use expr_lang_common::{skip_type, DataType, ExpressionValueRef, Result};
use std::convert::TryFrom;

impl ExpressionRuntime {
    pub(crate) fn invoke<T>(&mut self, context: &T) -> Result
    where
        T: ExpressionContext,
    {
        let target_ref = self.last_ref()?;

        let target_type = DataType::try_from(self.data[target_ref])?;
        match target_type {
            DataType::Expression => {
                // get name of expression
                let expression_index = self.read_size_at(skip_type(target_ref))?;

                match self.get_expression_start(expression_index) {
                    Err(_) => {
                        self.insert_unit()?;
                        return Ok(());
                    }
                    Ok(start) => {
                        // add next cursor
                        self.push_frame(start);

                        return Ok(());
                    }
                }
            }
            DataType::ExternalMethod => {
                let value_ref = if self.input_stack.len() > 0 {
                    let input_ref = self.current_input_ref();
                    ExpressionValueRef::new(&self.data[input_ref..], Some(&self.symbol_table))?
                } else {
                    ExpressionValueRef::unit()
                };

                let symbol_index = self.data[target_ref + 1];
                match self
                    .symbol_table
                    .iter()
                    .find(|(_s, i)| **i as u8 == symbol_index)
                {
                    None => {
                        // TODO: needs test when can created a Runtime with raw data for instruction set
                        self.insert_unit()?;
                    }
                    Some((name, _i)) => {
                        let value = context.execute(name.clone(), value_ref);
                        self.copy_into(&value)?;
                    }
                }
            }
            _ => {
                self.insert_unit()?;
            }
        }

        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use crate::context::ExpressionContext;
    use crate::runtime::ExpressionRuntime;
    use expr_lang_common::{ExpressionValue, ExpressionValueRef};
    use expr_lang_instruction_set_builder::InstructionSetBuilder;

    #[test]
    fn invoke_expression() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("add_numbers");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.perform_addition();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::expression("add_numbers"))
            .unwrap();
        instructions.invoke();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 30);
    }

    #[test]
    fn invoke_expression_with_input() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("add_numbers");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put_input();
        instructions.perform_addition();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(15)).unwrap();
        instructions.push_input();

        instructions
            .put(ExpressionValue::expression("add_numbers"))
            .unwrap();
        instructions.invoke();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 25);
    }

    #[test]
    fn invoke_expression_that_doesnt_exist() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("add_numbers");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.perform_addition();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::expression("subtract_numbers"))
            .unwrap();
        instructions.invoke();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit())
    }

    #[test]
    fn invoke_with_number() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.invoke();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit())
    }

    struct SquareRootContext {}

    impl ExpressionContext for SquareRootContext {
        fn resolve(&self, _name: String) -> ExpressionValue {
            unimplemented!()
        }

        fn execute(&self, name: String, input: ExpressionValueRef) -> ExpressionValue {
            match name.as_str() {
                "sqrt" => {
                    ExpressionValue::integer((input.as_integer().unwrap() as f32).sqrt() as i32)
                        .into()
                }
                "constant_value" => ExpressionValue::integer(150).into(),
                _ => panic!(),
            }
        }
    }

    #[test]
    fn invoke_external_method() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(144)).unwrap();
        instructions.push_input();

        instructions
            .put(ExpressionValue::external_method("sqrt"))
            .unwrap();
        instructions.invoke();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime
            .execute_with_context("main".to_string(), &SquareRootContext {})
            .unwrap();
        assert_eq!(result.as_integer().unwrap(), 12)
    }

    #[test]
    fn invoke_external_method_no_input() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::external_method("constant_value"))
            .unwrap();
        instructions.invoke();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime
            .execute_with_context("main".to_string(), &SquareRootContext {})
            .unwrap();

        assert_eq!(result.as_integer().unwrap(), 150);
    }
}
