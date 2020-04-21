use std::cmp::Ordering;
use std::convert::TryFrom;

use garnish_lang_common::Result;
use garnish_lang_common::{skip_type, DataType};

use crate::ExpressionRuntime;

impl ExpressionRuntime {
    pub(crate) fn conditional_execute(&mut self) -> Result {
        let true_expr_index = self.consume_constant_reference()?;
        let false_expr_index = self.consume_constant_reference()?;

        let conditional_value_ref = self.consume_last_ref()?;

        let conditional_value = match DataType::try_from(self.data[conditional_value_ref])? {
            DataType::Unit => false,
            DataType::Symbol => {
                let symbol_value = self.read_size_at(skip_type(conditional_value_ref))?;
                symbol_value != self.false_value
            }
            _ => true,
        };

        let expr_index = match conditional_value {
            true if true_expr_index != 0 => true_expr_index,
            false if false_expr_index != 0 => false_expr_index,
            _ => return Ok(()), // conditional expression will not be invoked
        };

        let expression_start = self.get_expression_start(expr_index)?;

        self.push_conditional_frame(expression_start);

        Ok(())
    }

    pub(crate) fn result_conditional_execute(&mut self) -> Result {
        let true_expr_index = self.consume_constant_reference()?;
        let false_expr_index = self.consume_constant_reference()?;

        let conditional_value_ref = self.consume_last_ref()?;

        let last_result_ref = if self.result_stack.is_empty() {
            if self.input_stack.is_empty() {
                0 // Unit value
            } else {
                let last_input_index = self.input_stack.len() - 1;
                self.input_stack[last_input_index]
            }
        } else {
            let last_result_index = self.result_stack.len() - 1;
            self.result_stack[last_result_index]
        };

        let conditional_value =
            match self.ordering_of_values_refs(last_result_ref, conditional_value_ref)? {
                Some(ordering) => ordering == Ordering::Equal,
                None => false,
            };

        let expr_index = match conditional_value {
            true if true_expr_index != 0 => true_expr_index,
            false if false_expr_index != 0 => false_expr_index,
            _ => return Ok(()), // conditional expression will not be invoked
        };

        let expression_start = self.get_expression_start(expr_index)?;

        self.push_frame(expression_start);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use garnish_lang_common::ExpressionValue;
    use garnish_lang_instruction_set_builder::InstructionSetBuilder;

    use crate::ExpressionRuntime;

    #[test]
    fn conditional_execution_of_expression_for_true_if_true() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("condition");
        instructions.put(ExpressionValue::integer(200)).unwrap();
        instructions.end_expression();

        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(120)).unwrap();

        instructions.put(ExpressionValue::symbol("true")).unwrap();
        instructions.conditional_execute(Some("condition".to_string()), None);

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 200);
    }

    #[test]
    fn conditional_execution_of_expression_for_false_if_false() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("condition");
        instructions.put(ExpressionValue::integer(200)).unwrap();
        instructions.end_expression();

        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(120)).unwrap();

        instructions.put(ExpressionValue::symbol("false")).unwrap();
        instructions.conditional_execute(None, Some("condition".to_string()));

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 200);
    }

    #[test]
    fn no_conditional_execution_of_expression_for_true_if_false() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("condition");
        instructions.put(ExpressionValue::integer(200)).unwrap();
        instructions.end_expression();

        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(120)).unwrap();

        instructions.put(ExpressionValue::symbol("false")).unwrap();
        instructions.conditional_execute(Some("condition".to_string()), None);

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 120);
    }

    #[test]
    fn no_conditional_execution_of_expression_for_false_if_true() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("condition");
        instructions.put(ExpressionValue::integer(200)).unwrap();
        instructions.end_expression();

        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(120)).unwrap();

        instructions.put(ExpressionValue::symbol("true")).unwrap();
        instructions.conditional_execute(None, Some("condition".to_string()));

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 120);
    }

    #[test]
    fn result_conditional_execution_of_expression_for_true_if_true() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("condition");
        instructions.put(ExpressionValue::integer(200)).unwrap();
        instructions.end_expression();

        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(120)).unwrap();
        instructions.output_result();

        instructions.put(ExpressionValue::integer(120)).unwrap();
        instructions.result_conditional_execute("condition".to_string(), None);

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 200);
    }

    #[test]
    fn result_conditional_execution_of_expression_for_false_if_false() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("default");
        instructions.end_expression();

        instructions.start_expression("condition");
        instructions.put(ExpressionValue::integer(200)).unwrap();
        instructions.end_expression();

        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(120)).unwrap();
        instructions.output_result();

        instructions.put(ExpressionValue::integer(100)).unwrap();
        instructions
            .result_conditional_execute("default".to_string(), Some("condition".to_string()));

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 200);
    }

    #[test]
    fn no_result_conditional_execution_of_expression_for_true_if_false() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("condition");
        instructions.put(ExpressionValue::integer(200)).unwrap();
        instructions.end_expression();

        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(120)).unwrap();
        instructions.output_result();

        instructions.put(ExpressionValue::integer(100)).unwrap();
        instructions.result_conditional_execute("condition".to_string(), None);

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 120);
    }

    #[test]
    fn no_result_conditional_execution_of_expression_for_false_if_true() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("default");
        instructions.end_expression();

        instructions.start_expression("condition");
        instructions.put(ExpressionValue::integer(200)).unwrap();
        instructions.end_expression();

        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(120)).unwrap();
        instructions.output_result();

        instructions.put(ExpressionValue::integer(120)).unwrap();
        instructions
            .result_conditional_execute("default".to_string(), Some("condition".to_string()));

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 120);
    }

    #[test]
    fn result_conditional_execution_of_expression_uses_input_if_no_result() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("condition");
        instructions.put(ExpressionValue::integer(200)).unwrap();
        instructions.end_expression();

        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(120)).unwrap();
        instructions.result_conditional_execute("condition".to_string(), None);

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        expression_runtime
            .set_input(&ExpressionValue::integer(120).into())
            .unwrap();
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 200);
    }

    #[test]
    fn result_conditional_execution_of_expression_uses_unit_if_no_result_or_input() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("condition");
        instructions.put(ExpressionValue::integer(200)).unwrap();
        instructions.end_expression();

        instructions.start_expression("main");

        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.result_conditional_execute("condition".to_string(), None);

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 200);
    }
}
