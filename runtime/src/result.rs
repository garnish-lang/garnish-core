use crate::ExpressionRuntime;
use garnish_lang_common::{ExpressionResult, Result};

impl ExpressionRuntime {
    pub fn get_result(&self, index: usize) -> Result<Option<ExpressionResult>> {
        Ok(if index >= self.result_stack.len() {
            None
        } else {
            let start = self.result_stack[index];
            Some(ExpressionResult::new(
                &self.data[start..],
                Some(&self.symbol_table),
            )?)
        })
    }

    pub(crate) fn put_result(&mut self) -> Result {
        let frame = &self.call_stack[self.call_stack.len() - 1];
        if self.result_stack.len() == frame.result_start {
            self.put_input()?;
        } else {
            self.insert_reference_value(self.result_stack[self.result_stack.len() - 1])?;
        }

        Ok(())
    }

    pub(crate) fn output_result(&mut self) -> Result {
        // actually have a referenced value
        if self.ref_cursor != 0 {
            let result = self.get_value_ref(self.ref_cursor - 1)?;
            self.result_stack.push(result);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::ExpressionRuntime;
    use garnish_lang_common::ExpressionValue;
    use garnish_lang_instruction_set_builder::InstructionSetBuilder;

    #[test]
    fn no_explicit_output_instruction_should_yield_implicit_result() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        expression_runtime.execute("main").unwrap();

        let expected: Vec<usize> = vec![1];
        let result = expression_runtime.result_stack;

        assert_eq!(result, expected);
    }

    #[test]
    fn output_result_yields_two_results() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.output_result();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        expression_runtime.execute("main").unwrap();

        let expected: Vec<usize> = vec![1, 6];
        let result = expression_runtime.result_stack;

        assert_eq!(result, expected);
    }

    #[test]
    fn result_returned_from_execution() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 10);
    }

    #[test]
    fn get_result_by_index() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.output_result();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        expression_runtime.execute("main").unwrap();

        let result = expression_runtime.get_result(0).unwrap().unwrap();

        assert_eq!(result.as_integer().unwrap(), 10);

        let result = expression_runtime.get_result(1).unwrap().unwrap();

        assert_eq!(result.as_integer().unwrap(), 20);
    }

    #[test]
    fn put_result_yields_proper_value() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.perform_addition();

        instructions.output_result();

        instructions.put_result();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.perform_addition();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 60);
    }

    #[test]
    fn put_result_uses_input_value_if_no_result() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("main");

        instructions.put_result();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.perform_addition();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        expression_runtime
            .set_input(&ExpressionValue::integer(10).into())
            .unwrap();

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 40);
    }
}
