use crate::ExpressionRuntime;
use garnish_common::{skip_type, DataType, Result};
use std::convert::TryFrom;

impl ExpressionRuntime {
    pub(crate) fn perform_logical_and(&mut self) -> Result {
        let (left_ref, right_ref) = self.consume_last_two_refs_and_set_value_cursor()?;

        match (
            DataType::try_from(self.data[left_ref])?,
            DataType::try_from(self.data[right_ref])?,
        ) {
            (DataType::Unit, DataType::Unit) | (DataType::Unit, _) | (_, DataType::Unit) => {
                // for AND, a unit on either side will always be false
                self.insert_symbol_value(self.false_value)
            }
            (DataType::Symbol, DataType::Symbol) => {
                let left_symbol = self.read_size_at(skip_type(left_ref))?;
                let right_symbol = self.read_size_at(skip_type(right_ref))?;
                self.insert_symbol_value_from_condition(
                    left_symbol != self.false_value && right_symbol != self.false_value,
                )
            }
            // one value is not a unit or a symbol so it will be true
            // check other value for false symbol
            (DataType::Symbol, _) => {
                let left_symbol = self.read_size_at(skip_type(left_ref))?;
                self.insert_symbol_value_from_condition(left_symbol == self.true_value)
            }
            (_, DataType::Symbol) => {
                let right_symbol = self.read_size_at(skip_type(right_ref))?;
                self.insert_symbol_value_from_condition(right_symbol == self.true_value)
            }
            _ => {
                // any other combination of types will be true
                self.insert_symbol_value(self.true_value)
            }
        }
    }

    pub(crate) fn perform_logical_or(&mut self) -> Result {
        let (left_ref, right_ref) = self.consume_last_two_refs_and_set_value_cursor()?;

        match (
            DataType::try_from(self.data[left_ref])?,
            DataType::try_from(self.data[right_ref])?,
        ) {
            (DataType::Unit, DataType::Unit) => self.insert_symbol_value(self.false_value),
            (DataType::Unit, _) | (_, DataType::Unit) => self.insert_symbol_value(self.true_value),
            (DataType::Symbol, DataType::Symbol) => {
                let left_symbol = self.read_size_at(skip_type(left_ref))?;
                let right_symbol = self.read_size_at(skip_type(right_ref))?;
                self.insert_symbol_value_from_condition(
                    left_symbol != self.false_value || right_symbol != self.false_value,
                )
            }
            // for OR, all remaining combinations will have at least one truthy value
            // so they will be true
            _ => self.insert_symbol_value(self.true_value),
        }
    }

    pub(crate) fn perform_logical_xor(&mut self) -> Result {
        let (left_ref, right_ref) = self.consume_last_two_refs_and_set_value_cursor()?;

        match (
            DataType::try_from(self.data[left_ref])?,
            DataType::try_from(self.data[right_ref])?,
        ) {
            (DataType::Unit, DataType::Unit) => self.insert_symbol_value(self.false_value),
            (DataType::Unit, _) | (_, DataType::Unit) => self.insert_symbol_value(self.true_value),
            (DataType::Symbol, DataType::Symbol) => {
                let left_symbol = self.read_size_at(skip_type(left_ref))?;
                let right_symbol = self.read_size_at(skip_type(right_ref))?;

                self.insert_symbol_value_from_condition(
                    (left_symbol != self.false_value) ^ (right_symbol != self.false_value),
                )
            }
            (DataType::Symbol, _) => {
                let left_symbol = self.read_size_at(skip_type(left_ref))?;
                self.insert_symbol_value_from_condition(left_symbol == self.false_value)
            }
            (_, DataType::Symbol) => {
                let right_symbol = self.read_size_at(skip_type(right_ref))?;
                self.insert_symbol_value_from_condition(right_symbol == self.false_value)
            }
            // for XOR, all remaining combinations will have two truthy values
            // so they will be false
            _ => self.insert_symbol_value(self.false_value),
        }
    }

    pub(crate) fn perform_logical_not(&mut self) -> Result {
        let last_ref = self.consume_last_ref()?;
        self.set_value_cursor_to_ref_cursor();

        match DataType::try_from(self.data[last_ref])? {
            DataType::Symbol => {
                let symbol_value = self.read_size_at(skip_type(last_ref))?;
                self.insert_symbol_value_from_condition(symbol_value == self.false_value)
            }
            // Unit is falsy so result will be true
            DataType::Unit => self.insert_symbol_value(self.true_value),
            // any other value is truthy so result will be false
            _ => self.insert_symbol_value(self.false_value),
        }
    }

    pub(crate) fn insert_symbol_value_from_condition(&mut self, condition: bool) -> Result {
        self.insert_symbol_value(if condition {
            self.true_value
        } else {
            self.false_value
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::ExpressionRuntime;
    use garnish_common::ExpressionValue;
    use garnish_instruction_set_builder::InstructionSetBuilder;

    //
    // AND
    //

    #[test]
    fn and_true_symbol_true_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("true")).unwrap();
        instructions.put(ExpressionValue::symbol("true")).unwrap();
        instructions.perform_logical_and();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn and_true_symbol_false_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("true")).unwrap();
        instructions.put(ExpressionValue::symbol("false")).unwrap();
        instructions.perform_logical_and();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "false".to_string());
    }

    #[test]
    fn and_false_symbol_true_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("false")).unwrap();
        instructions.put(ExpressionValue::symbol("true")).unwrap();
        instructions.perform_logical_and();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "false".to_string());
    }

    #[test]
    fn and_false_symbol_false_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("false")).unwrap();
        instructions.put(ExpressionValue::symbol("false")).unwrap();
        instructions.perform_logical_and();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "false".to_string());
    }

    #[test]
    fn and_true_symbol_any_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("true")).unwrap();
        instructions.put(ExpressionValue::symbol("cats")).unwrap();
        instructions.perform_logical_and();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn and_any_symbol_true_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("cats")).unwrap();
        instructions.put(ExpressionValue::symbol("true")).unwrap();
        instructions.perform_logical_and();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn and_false_symbol_any_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("false")).unwrap();
        instructions.put(ExpressionValue::symbol("cats")).unwrap();
        instructions.perform_logical_and();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "false".to_string());
    }

    #[test]
    fn and_any_symbol_false_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("cats")).unwrap();
        instructions.put(ExpressionValue::symbol("false")).unwrap();
        instructions.perform_logical_and();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "false".to_string());
    }

    #[test]
    fn and_any_symbol_any_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("cats")).unwrap();
        instructions.put(ExpressionValue::symbol("dogs")).unwrap();
        instructions.perform_logical_and();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn and_unit_unit() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.perform_logical_and();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "false".to_string());
    }

    #[test]
    fn and_unit_true_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions
            .put(ExpressionValue::symbol("true".to_string()))
            .unwrap();
        instructions.perform_logical_and();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "false".to_string());
    }

    #[test]
    fn and_true_symbol_unit() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::symbol("true".to_string()))
            .unwrap();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.perform_logical_and();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "false".to_string());
    }

    #[test]
    fn and_true_symbol_any_value() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::symbol("true".to_string()))
            .unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_logical_and();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn and_false_symbol_any_value() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::symbol("false".to_string()))
            .unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_logical_and();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "false".to_string());
    }

    #[test]
    fn and_any_value_true_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions
            .put(ExpressionValue::symbol("true".to_string()))
            .unwrap();
        instructions.perform_logical_and();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn and_any_value_false_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions
            .put(ExpressionValue::symbol("false".to_string()))
            .unwrap();
        instructions.perform_logical_and();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "false".to_string());
    }

    #[test]
    fn and_any_value_any_value() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions
            .put(ExpressionValue::character_list("false".to_string()))
            .unwrap();
        instructions.perform_logical_and();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    //
    // OR
    //

    #[test]
    fn or_true_symbol_true_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("true")).unwrap();
        instructions.put(ExpressionValue::symbol("true")).unwrap();
        instructions.perform_logical_or();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn or_true_symbol_false_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("true")).unwrap();
        instructions.put(ExpressionValue::symbol("false")).unwrap();
        instructions.perform_logical_or();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn or_false_symbol_true_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("false")).unwrap();
        instructions.put(ExpressionValue::symbol("true")).unwrap();
        instructions.perform_logical_or();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn or_false_symbol_false_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("false")).unwrap();
        instructions.put(ExpressionValue::symbol("false")).unwrap();
        instructions.perform_logical_or();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "false".to_string());
    }

    #[test]
    fn or_true_symbol_any_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("true")).unwrap();
        instructions.put(ExpressionValue::symbol("cats")).unwrap();
        instructions.perform_logical_or();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn or_any_symbol_true_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("cats")).unwrap();
        instructions.put(ExpressionValue::symbol("true")).unwrap();
        instructions.perform_logical_or();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn or_false_symbol_any_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("false")).unwrap();
        instructions.put(ExpressionValue::symbol("cats")).unwrap();
        instructions.perform_logical_or();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn or_any_symbol_false_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("cats")).unwrap();
        instructions.put(ExpressionValue::symbol("false")).unwrap();
        instructions.perform_logical_or();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn or_any_symbol_any_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("dogs")).unwrap();
        instructions.put(ExpressionValue::symbol("cats")).unwrap();
        instructions.perform_logical_or();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn or_unit_unit() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.perform_logical_or();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "false".to_string());
    }

    #[test]
    fn or_unit_true_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions
            .put(ExpressionValue::symbol("true".to_string()))
            .unwrap();
        instructions.perform_logical_or();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn or_true_symbol_unit() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::symbol("true".to_string()))
            .unwrap();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.perform_logical_or();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn or_true_symbol_any_value() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::symbol("true".to_string()))
            .unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_logical_or();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn or_false_symbol_any_value() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::symbol("false".to_string()))
            .unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_logical_or();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn or_any_value_true_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions
            .put(ExpressionValue::symbol("true".to_string()))
            .unwrap();
        instructions.perform_logical_or();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn or_any_value_false_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions
            .put(ExpressionValue::symbol("false".to_string()))
            .unwrap();
        instructions.perform_logical_or();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn or_any_value_any_value() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions
            .put(ExpressionValue::character_list("false".to_string()))
            .unwrap();
        instructions.perform_logical_or();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    //
    // XOR
    //

    #[test]
    fn xor_true_symbol_true_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("true")).unwrap();
        instructions.put(ExpressionValue::symbol("true")).unwrap();
        instructions.perform_logical_xor();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "false".to_string());
    }

    #[test]
    fn xor_true_symbol_false_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("true")).unwrap();
        instructions.put(ExpressionValue::symbol("false")).unwrap();
        instructions.perform_logical_xor();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn xor_false_symbol_true_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("false")).unwrap();
        instructions.put(ExpressionValue::symbol("true")).unwrap();
        instructions.perform_logical_xor();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn xor_false_symbol_false_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("false")).unwrap();
        instructions.put(ExpressionValue::symbol("false")).unwrap();
        instructions.perform_logical_xor();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "false".to_string());
    }

    #[test]
    fn xor_true_symbol_any_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("true")).unwrap();
        instructions.put(ExpressionValue::symbol("cats")).unwrap();
        instructions.perform_logical_xor();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "false".to_string());
    }

    #[test]
    fn xor_any_symbol_true_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("cats")).unwrap();
        instructions.put(ExpressionValue::symbol("true")).unwrap();
        instructions.perform_logical_xor();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "false".to_string());
    }

    #[test]
    fn xor_false_symbol_any_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("false")).unwrap();
        instructions.put(ExpressionValue::symbol("cats")).unwrap();
        instructions.perform_logical_xor();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn xor_any_symbol_false_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("cats")).unwrap();
        instructions.put(ExpressionValue::symbol("false")).unwrap();
        instructions.perform_logical_xor();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn xor_any_symbol_any_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::symbol("cats")).unwrap();
        instructions.put(ExpressionValue::symbol("dogs")).unwrap();
        instructions.perform_logical_xor();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "false".to_string());
    }

    #[test]
    fn xor_unit_unit() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.perform_logical_xor();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "false".to_string());
    }

    #[test]
    fn xor_unit_true_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions
            .put(ExpressionValue::symbol("true".to_string()))
            .unwrap();
        instructions.perform_logical_xor();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn xor_true_symbol_unit() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::symbol("true".to_string()))
            .unwrap();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.perform_logical_xor();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn xor_true_symbol_any_value() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::symbol("true".to_string()))
            .unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_logical_xor();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "false".to_string());
    }

    #[test]
    fn xor_false_symbol_any_value() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::symbol("false".to_string()))
            .unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_logical_xor();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn xor_any_value_true_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions
            .put(ExpressionValue::symbol("true".to_string()))
            .unwrap();
        instructions.perform_logical_xor();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "false".to_string());
    }

    #[test]
    fn xor_any_value_false_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions
            .put(ExpressionValue::symbol("false".to_string()))
            .unwrap();
        instructions.perform_logical_xor();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn xor_any_value_any_value() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions
            .put(ExpressionValue::character_list("false".to_string()))
            .unwrap();
        instructions.perform_logical_xor();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "false".to_string());
    }

    //
    // NOT
    //

    #[test]
    fn not_true_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::symbol("true".to_string()))
            .unwrap();
        instructions.perform_logical_not();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "false".to_string());
    }

    #[test]
    fn not_false_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::symbol("false".to_string()))
            .unwrap();
        instructions.perform_logical_not();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn not_any_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions.perform_logical_not();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "false".to_string());
    }

    #[test]
    fn not_unit() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.perform_logical_not();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "true".to_string());
    }

    #[test]
    fn not_any_value() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::character_list("cats".to_string()))
            .unwrap();
        instructions.perform_logical_not();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "false".to_string());
    }
}
