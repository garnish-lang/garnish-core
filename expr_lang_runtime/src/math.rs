use crate::runtime::ExpressionRuntime;
use expr_lang_common::{skip_type, DataType, Result};
use std::convert::TryFrom;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

impl ExpressionRuntime {
    pub(crate) fn perform_addition(&mut self) -> Result {
        self.perform_binary_operation(i32::add, f32::add)
    }

    pub(crate) fn perform_subtraction(&mut self) -> Result {
        self.perform_binary_operation(i32::sub, f32::sub)
    }

    pub(crate) fn perform_multiplication(&mut self) -> Result {
        self.perform_binary_operation(i32::mul, f32::mul)
    }

    pub(crate) fn perform_division(&mut self) -> Result {
        self.perform_binary_operation(i32::div, f32::div)
    }

    pub(crate) fn perform_integer_division(&mut self) -> Result {
        self.perform_division()?;

        let last_ref = self.last_ref()?;

        match DataType::try_from(self.data[last_ref])? {
            DataType::Integer => (),
            DataType::Float => {
                let val = self.read_float_at(skip_type(last_ref))?;

                // consume reference
                self.ref_cursor -= 1;
                self.set_value_cursor_to_ref_cursor();

                self.insert_integer(val as i32)?;
            }
            _ => unreachable!("Non Integer or Float type value on stack after division."),
        }

        Ok(())
    }

    pub(crate) fn perform_remainder(&mut self) -> Result {
        self.perform_binary_operation(i32::rem, f32::rem)
    }

    pub(crate) fn perform_exponential(&mut self) -> Result {
        self.perform_binary_operation(|l, r| l.pow(r as u32), f32::powf)
    }

    pub(crate) fn perform_negation(&mut self) -> Result {
        self.perform_unary_operation(i32::neg, f32::neg)
    }

    pub(crate) fn perform_absolute_value(&mut self) -> Result {
        self.perform_unary_operation(i32::abs, f32::abs)
    }

    fn perform_unary_operation<I, F>(&mut self, integer_op: I, float_op: F) -> Result
    where
        I: Fn(i32) -> i32,
        F: Fn(f32) -> f32,
    {
        let last_ref = self.consume_last_ref()?;
        self.set_value_cursor_to_ref_cursor();

        match DataType::try_from(self.data[last_ref])? {
            DataType::Integer => {
                let val = self.read_integer_at(skip_type(last_ref))?;
                self.insert_integer(integer_op(val))?;
            }
            DataType::Float => {
                let val = self.read_float_at(skip_type(last_ref))?;
                self.insert_float(float_op(val))?;
            }
            _ => self.insert_unit()?,
        }

        Ok(())
    }

    fn perform_binary_operation<I, F>(&mut self, integer_op: I, float_op: F) -> Result
    where
        I: Fn(i32, i32) -> i32,
        F: Fn(f32, f32) -> f32,
    {
        let (left_ref, right_ref) = self.consume_last_two_refs_and_set_value_cursor()?;

        match (
            DataType::try_from(self.data[left_ref])?,
            DataType::try_from(self.data[right_ref])?,
        ) {
            (DataType::Integer, DataType::Integer) => {
                let left = self.read_integer_at(skip_type(left_ref))?;
                let right = self.read_integer_at(skip_type(right_ref))?;
                self.insert_integer(integer_op(left, right))?;
            }
            (DataType::Float, DataType::Float) => {
                let left = self.read_float_at(skip_type(left_ref))?;
                let right = self.read_float_at(skip_type(right_ref))?;
                self.insert_float(float_op(left, right))?;
            }
            (DataType::Integer, DataType::Float) => {
                let left = self.read_integer_at(skip_type(left_ref))? as f32;
                let right = self.read_float_at(skip_type(right_ref))?;
                self.insert_float(float_op(left, right))?;
            }
            (DataType::Float, DataType::Integer) => {
                let left = self.read_float_at(skip_type(left_ref))?;
                let right = self.read_integer_at(skip_type(right_ref))? as f32;
                self.insert_float(float_op(left, right))?;
            }
            _ => self.insert_unit()?,
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::runtime::ExpressionRuntime;
    use expr_lang_common::ExpressionValue;
    use expr_lang_instruction_set_builder::InstructionSetBuilder;

    #[test]
    fn addition_yields_proper_result() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.perform_addition();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 30);
    }

    #[test]
    fn addition_yields_unit_if_not_number() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions
            .put(ExpressionValue::character("z".into()))
            .unwrap();
        instructions.perform_addition();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit());
    }

    #[test]
    fn addition_with_one_number_has_error_result() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_addition();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        assert!(expression_runtime.execute("main").is_err());
    }

    #[test]
    fn addition_with_zero_numbers_has_error_result() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.perform_addition();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        assert!(expression_runtime.execute("main").is_err());
    }

    #[test]
    fn addition_with_integers() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.perform_addition();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 30);
    }

    #[test]
    fn addition_with_floats() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::float(1.2)).unwrap();
        instructions.put(ExpressionValue::float(2.4)).unwrap();
        instructions.perform_addition();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_float().unwrap(), 1.2 + 2.4);
    }

    #[test]
    fn addition_with_integer_float() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(4)).unwrap();
        instructions.put(ExpressionValue::float(2.4)).unwrap();
        instructions.perform_addition();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_float().unwrap(), 4f32 + 2.4);
    }

    #[test]
    fn addition_with_float_integer() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::float(1.2)).unwrap();
        instructions.put(ExpressionValue::integer(3)).unwrap();
        instructions.perform_addition();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_float().unwrap(), 1.2 + 3f32);
    }

    #[test]
    fn subtraction_with_integers() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_subtraction();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 10);
    }

    #[test]
    fn subtraction_with_floats() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::float(1.5)).unwrap();
        instructions.put(ExpressionValue::float(3.2)).unwrap();
        instructions.perform_subtraction();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_float().unwrap(), 1.5 - 3.2);
    }

    #[test]
    fn subtraction_with_integer_float() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(4)).unwrap();
        instructions.put(ExpressionValue::float(2.4)).unwrap();
        instructions.perform_subtraction();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_float().unwrap(), 4f32 - 2.4);
    }

    #[test]
    fn subtraction_with_float_integer() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::float(1.2)).unwrap();
        instructions.put(ExpressionValue::integer(3)).unwrap();
        instructions.perform_subtraction();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_float().unwrap(), 1.2 - 3f32);
    }

    #[test]
    fn multiplication_with_integers() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_multiplication();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 200);
    }

    #[test]
    fn multiplication_with_floats() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::float(1.5)).unwrap();
        instructions.put(ExpressionValue::float(3.2)).unwrap();
        instructions.perform_multiplication();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_float().unwrap(), 1.5 * 3.2);
    }

    #[test]
    fn multiplication_with_integer_float() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(4)).unwrap();
        instructions.put(ExpressionValue::float(2.4)).unwrap();
        instructions.perform_multiplication();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_float().unwrap(), 4f32 * 2.4);
    }

    #[test]
    fn multiplication_with_float_integer() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::float(1.2)).unwrap();
        instructions.put(ExpressionValue::integer(3)).unwrap();
        instructions.perform_multiplication();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_float().unwrap(), 1.2 * 3f32);
    }

    #[test]
    fn division_with_integers() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_division();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 20 / 10);
    }

    #[test]
    fn division_with_floats() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::float(1.5)).unwrap();
        instructions.put(ExpressionValue::float(3.2)).unwrap();
        instructions.perform_division();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_float().unwrap(), 1.5 / 3.2);
    }

    #[test]
    fn division_with_integer_float() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(4)).unwrap();
        instructions.put(ExpressionValue::float(2.4)).unwrap();
        instructions.perform_division();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_float().unwrap(), 4f32 / 2.4);
    }

    #[test]
    fn division_with_float_integer() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::float(1.2)).unwrap();
        instructions.put(ExpressionValue::integer(3)).unwrap();
        instructions.perform_division();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_float().unwrap(), 1.2 / 3f32);
    }

    #[test]
    fn integer_division_with_integers() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_integer_division();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 2);
    }

    #[test]
    fn integer_division_with_floats() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::float(10.5)).unwrap();
        instructions.put(ExpressionValue::float(3.2)).unwrap();
        instructions.perform_integer_division();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 3);
    }

    #[test]
    fn integer_division_with_integer_float() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(4)).unwrap();
        instructions.put(ExpressionValue::float(2.4)).unwrap();
        instructions.perform_integer_division();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 1);
    }

    #[test]
    fn integer_division_with_float_integer() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::float(1.2)).unwrap();
        instructions.put(ExpressionValue::integer(3)).unwrap();
        instructions.perform_integer_division();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 0);
    }

    #[test]
    fn remainder_with_integers() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(22)).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_remainder();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 2);
    }

    #[test]
    fn remainder_with_floats() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::float(10.5)).unwrap();
        instructions.put(ExpressionValue::float(3.2)).unwrap();
        instructions.perform_remainder();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_float().unwrap(), 10.5 % 3.2);
    }

    #[test]
    fn remainder_with_integer_float() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(4)).unwrap();
        instructions.put(ExpressionValue::float(2.4)).unwrap();
        instructions.perform_remainder();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_float().unwrap(), 4f32 % 2.4);
    }

    #[test]
    fn remainder_with_float_integer() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::float(1.2)).unwrap();
        instructions.put(ExpressionValue::integer(3)).unwrap();
        instructions.perform_remainder();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_float().unwrap(), 1.2 % 3f32);
    }

    #[test]
    fn exponential_with_integers() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(22)).unwrap();
        instructions.put(ExpressionValue::integer(3)).unwrap();
        instructions.perform_exponential();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 22i32.pow(3));
    }

    #[test]
    fn exponential_with_floats() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::float(10.5)).unwrap();
        instructions.put(ExpressionValue::float(3.2)).unwrap();
        instructions.perform_exponential();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_float().unwrap(), 10.5f32.powf(3.2));
    }

    #[test]
    fn exponential_with_integer_float() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(4)).unwrap();
        instructions.put(ExpressionValue::float(2.4)).unwrap();
        instructions.perform_exponential();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_float().unwrap(), 4f32.powf(2.4));
    }

    #[test]
    fn exponential_with_float_integer() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::float(1.2)).unwrap();
        instructions.put(ExpressionValue::integer(3)).unwrap();
        instructions.perform_exponential();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_float().unwrap(), 1.2f32.powf(3f32));
    }

    #[test]
    fn negation_with_integer() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(22)).unwrap();
        instructions.perform_negation();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), -22);
    }

    #[test]
    fn negation_with_float() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::float(10.5)).unwrap();
        instructions.perform_negation();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_float().unwrap(), -10.5);
    }

    #[test]
    fn absolute_value_with_integer() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(-22)).unwrap();
        instructions.perform_absolute_value();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 22);
    }

    #[test]
    fn absolute_value_with_float() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::float(-10.5)).unwrap();
        instructions.perform_absolute_value();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_float().unwrap(), 10.5);
    }
}
