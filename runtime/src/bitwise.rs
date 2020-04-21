use crate::runtime::ExpressionRuntime;
use garnish_lang_common::{skip_type, DataType, Result};
use std::convert::TryFrom;
use std::ops::{BitAnd, BitOr, BitXor, Shl, Shr};

impl ExpressionRuntime {
    pub(crate) fn perform_bitwise_and(&mut self) -> Result {
        self.perform_bitwise_operation(i32::bitand)
    }

    pub(crate) fn perform_bitwise_or(&mut self) -> Result {
        self.perform_bitwise_operation(i32::bitor)
    }

    pub(crate) fn perform_bitwise_xor(&mut self) -> Result {
        self.perform_bitwise_operation(i32::bitxor)
    }

    pub(crate) fn perform_bitwise_not(&mut self) -> Result {
        let last_ref = self.consume_last_ref()?;
        self.set_value_cursor_to_ref_cursor();

        match DataType::try_from(self.data[last_ref])? {
            DataType::Integer => {
                let val = self.read_integer_at(skip_type(last_ref))?;
                self.insert_integer(!val)?;
            }
            _ => self.insert_unit()?,
        }

        Ok(())
    }

    pub(crate) fn perform_bitwise_left_shift(&mut self) -> Result {
        self.perform_bitwise_operation(i32::shl)
    }

    pub(crate) fn perform_bitwise_right_shift(&mut self) -> Result {
        self.perform_bitwise_operation(i32::shr)
    }

    fn perform_bitwise_operation<T>(&mut self, op: T) -> Result
    where
        T: Fn(i32, i32) -> i32,
    {
        let (left_ref, right_ref) = self.consume_last_two_refs_and_set_value_cursor()?;

        match (
            DataType::try_from(self.data[left_ref])?,
            DataType::try_from(self.data[right_ref])?,
        ) {
            (DataType::Integer, DataType::Integer) => {
                let left = self.read_integer_at(skip_type(left_ref))?;
                let right = self.read_integer_at(skip_type(right_ref))?;
                self.insert_integer(op(left, right))?;
            }
            _ => self.insert_unit()?,
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
    fn bitwise_and() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(12345)).unwrap();
        instructions.put(ExpressionValue::integer(98765)).unwrap();
        instructions.perform_bitwise_and();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 12345 & 98765);
    }

    #[test]
    fn bitwise_or() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(12345)).unwrap();
        instructions.put(ExpressionValue::integer(98765)).unwrap();
        instructions.perform_bitwise_or();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 12345 | 98765);
    }

    #[test]
    fn bitwise_xor() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(12345)).unwrap();
        instructions.put(ExpressionValue::integer(98765)).unwrap();
        instructions.perform_bitwise_xor();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 12345 ^ 98765);
    }

    #[test]
    fn bitwise_not() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(12345)).unwrap();
        instructions.perform_bitwise_not();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), !12345);
    }

    #[test]
    fn bitwise_left_shift() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(12345)).unwrap();
        instructions.put(ExpressionValue::integer(3)).unwrap();
        instructions.perform_bitwise_left_shift();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 12345 << 3);
    }

    #[test]
    fn bitwise_right_shift() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(12345)).unwrap();
        instructions.put(ExpressionValue::integer(3)).unwrap();
        instructions.perform_bitwise_right_shift();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 12345 >> 3);
    }

    #[test]
    fn bitwise_with_non_integer_results_in_unit() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(12345)).unwrap();
        instructions.put(ExpressionValue::float(2.3)).unwrap();
        instructions.perform_bitwise_and();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit());
    }
}
