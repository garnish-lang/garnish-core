use std::convert::TryFrom;

use garnish_common::{empty_range_flags, DataType, RangeFlags, Result};

use crate::ExpressionRuntime;

impl ExpressionRuntime {
    pub(crate) fn make_inclusive_range(&mut self) -> Result {
        self.make_range(empty_range_flags())
    }

    pub(crate) fn make_exclusive_range(&mut self) -> Result {
        self.make_range(
            empty_range_flags()
                .set_is_start_exclusive()
                .set_is_end_exclusive(),
        )
    }

    pub(crate) fn make_start_exclusive_range(&mut self) -> Result {
        self.make_range(empty_range_flags().set_is_start_exclusive())
    }

    pub(crate) fn make_end_exclusive_range(&mut self) -> Result {
        self.make_range(empty_range_flags().set_is_end_exclusive())
    }

    fn make_range(&mut self, flags: u8) -> Result {
        let (left_ref, right_ref) = self.consume_last_two_refs()?;

        match (
            DataType::try_from(self.data[left_ref])?,
            DataType::try_from(self.data[right_ref])?,
        ) {
            (DataType::Unit, DataType::Unit) => self.insert_range_value(
                flags.set_is_start_open().set_is_end_open(),
                None,
                None,
                None,
            ),
            (DataType::Integer, DataType::Integer)
            | (DataType::Character, DataType::Character)
            | (DataType::Float, DataType::Float) => {
                self.insert_range_value(flags, Some(left_ref), Some(right_ref), None)
            }
            (DataType::Unit, DataType::Integer)
            | (DataType::Unit, DataType::Character)
            | (DataType::Unit, DataType::Float) => {
                self.insert_range_value(flags.set_is_start_open(), None, Some(right_ref), None)
            }
            (DataType::Integer, DataType::Unit)
            | (DataType::Character, DataType::Unit)
            | (DataType::Float, DataType::Unit) => {
                self.insert_range_value(flags.set_is_end_open(), Some(left_ref), None, None)
            }
            _ => self.insert_unit(),
        }
    }
}

#[cfg(test)]
mod tests {
    use garnish_common::{empty_range_flags, DataType, ExpressionValue, RangeFlags};
    use garnish_instruction_set_builder::InstructionSetBuilder;

    use crate::ExpressionRuntime;

    #[test]
    fn make_inclusive_range() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.make_inclusive_range();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_type().unwrap(), DataType::Range);
        assert_eq!(result.get_range_flags().unwrap(), empty_range_flags());
        assert_eq!(result.get_range_min().unwrap().as_integer().unwrap(), 10);
        assert_eq!(result.get_range_max().unwrap().as_integer().unwrap(), 20);
    }

    #[test]
    fn make_inclusive_character_range() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("z".into()))
            .unwrap();
        instructions.make_inclusive_range();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_type().unwrap(), DataType::Range);
        assert_eq!(result.get_range_flags().unwrap(), empty_range_flags());
        assert_eq!(result.get_range_min().unwrap().as_char().unwrap(), 'a');
        assert_eq!(result.get_range_max().unwrap().as_char().unwrap(), 'z');
    }

    #[test]
    fn make_inclusive_float_range() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::float(2.5)).unwrap();
        instructions.put(ExpressionValue::float(7.5)).unwrap();
        instructions.make_inclusive_range();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_type().unwrap(), DataType::Range);
        assert_eq!(result.get_range_flags().unwrap(), empty_range_flags());
        assert_eq!(result.get_range_min().unwrap().as_float().unwrap(), 2.5);
        assert_eq!(result.get_range_max().unwrap().as_float().unwrap(), 7.5);
    }

    #[test]
    fn make_exclusive_range() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.make_exclusive_range();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_type().unwrap(), DataType::Range);

        assert_eq!(
            result.get_range_flags().unwrap(),
            empty_range_flags()
                .set_is_start_exclusive()
                .set_is_end_exclusive()
        );

        assert_eq!(result.get_range_min().unwrap().as_integer().unwrap(), 10);
        assert_eq!(result.get_range_max().unwrap().as_integer().unwrap(), 20);
    }

    #[test]
    fn make_start_exclusive_range() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.make_start_exclusive_range();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_type().unwrap(), DataType::Range);

        assert_eq!(
            result.get_range_flags().unwrap(),
            empty_range_flags().set_is_start_exclusive()
        );

        assert_eq!(result.get_range_min().unwrap().as_integer().unwrap(), 10);
        assert_eq!(result.get_range_max().unwrap().as_integer().unwrap(), 20);
    }

    #[test]
    fn make_end_exclusive_range() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.make_end_exclusive_range();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_type().unwrap(), DataType::Range);

        assert_eq!(
            result.get_range_flags().unwrap(),
            empty_range_flags().set_is_end_exclusive()
        );

        assert_eq!(result.get_range_min().unwrap().as_integer().unwrap(), 10);
        assert_eq!(result.get_range_max().unwrap().as_integer().unwrap(), 20);
    }

    #[test]
    fn make_full_open_range() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.make_inclusive_range();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_type().unwrap(), DataType::Range);
        assert_eq!(
            result.get_range_flags().unwrap(),
            empty_range_flags().set_is_start_open().set_is_end_open()
        );
        assert!(result.get_range_min().unwrap().is_unit());
        assert!(result.get_range_max().unwrap().is_unit());
    }

    #[test]
    fn make_start_open_range() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.make_inclusive_range();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_type().unwrap(), DataType::Range);
        assert_eq!(
            result.get_range_flags().unwrap(),
            empty_range_flags().set_is_start_open()
        );
        assert!(result.get_range_min().unwrap().is_unit());
        assert_eq!(result.get_range_max().unwrap().as_integer().unwrap(), 20);
    }

    #[test]
    fn make_end_open_range() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.make_inclusive_range();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_type().unwrap(), DataType::Range);
        assert_eq!(
            result.get_range_flags().unwrap(),
            empty_range_flags().set_is_end_open()
        );
        assert_eq!(result.get_range_min().unwrap().as_integer().unwrap(), 10);
        assert!(result.get_range_max().unwrap().is_unit());
    }

    #[test]
    fn make_start_open_character_range() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions
            .put(ExpressionValue::character("z".into()))
            .unwrap();
        instructions.make_inclusive_range();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_type().unwrap(), DataType::Range);
        assert_eq!(
            result.get_range_flags().unwrap(),
            empty_range_flags().set_is_start_open()
        );
        assert!(result.get_range_min().unwrap().is_unit());
        assert_eq!(result.get_range_max().unwrap().as_char().unwrap(), 'z');
    }

    #[test]
    fn make_end_open_character_range() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.make_inclusive_range();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_type().unwrap(), DataType::Range);
        assert_eq!(
            result.get_range_flags().unwrap(),
            empty_range_flags().set_is_end_open()
        );
        assert_eq!(result.get_range_min().unwrap().as_char().unwrap(), 'a');
        assert!(result.get_range_max().unwrap().is_unit());
    }

    #[test]
    fn make_start_open_float_range() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.put(ExpressionValue::float(7.5)).unwrap();
        instructions.make_inclusive_range();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_type().unwrap(), DataType::Range);
        assert_eq!(
            result.get_range_flags().unwrap(),
            empty_range_flags().set_is_start_open()
        );
        assert!(result.get_range_min().unwrap().is_unit());
        assert_eq!(result.get_range_max().unwrap().as_float().unwrap(), 7.5);
    }

    #[test]
    fn make_end_open_float_range() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::float(2.5)).unwrap();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.make_inclusive_range();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_type().unwrap(), DataType::Range);
        assert_eq!(
            result.get_range_flags().unwrap(),
            empty_range_flags().set_is_end_open()
        );
        assert_eq!(result.get_range_min().unwrap().as_float().unwrap(), 2.5);
        assert!(result.get_range_max().unwrap().is_unit());
    }

    #[test]
    fn make_range_with_non_number_yields_unit() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions
            .put(ExpressionValue::character("z".into()))
            .unwrap();
        instructions.make_inclusive_range();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        assert!(expression_runtime.execute("main").unwrap().is_unit());
    }

    #[test]
    fn make_range_with_one_value_has_error_result() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.make_inclusive_range();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        assert!(expression_runtime.execute("main").is_err());
    }

    #[test]
    fn make_range_with_zero_values_has_error_result() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.make_inclusive_range();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        assert!(expression_runtime.execute("main").is_err());
    }
}
