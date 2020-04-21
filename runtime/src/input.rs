use std::convert::TryInto;

use garnish_lang_common::{DataType, ExpressionValue, ExpressionValueConsumer, Result};

use crate::runtime::ExpressionRuntime;

impl ExpressionRuntime {
    pub fn set_input(&mut self, value: &ExpressionValue) -> Result {
        let start = self.constant_data_size;
        self.input_stack.push(start);
        self.copy_into(value)?;
        self.input_size = value.get_data().len();

        Ok(())
    }

    pub(crate) fn put_input(&mut self) -> Result {
        if self.input_stack.is_empty() {
            let pos = self.value_cursor;
            self.insert_at_value_cursor(DataType::Unit.try_into().unwrap())?;
            self.insert_at_ref_cursor(pos);
        } else {
            let input_ref = self.current_input_ref();
            self.insert_reference_value(input_ref)?;
        }

        Ok(())
    }

    pub(crate) fn push_input(&mut self) -> Result {
        let input = self.last_ref()?;
        self.input_stack.push(input);
        Ok(())
    }

    pub(crate) fn push_unit_input(&mut self) -> Result {
        self.insert_unit()?;
        self.push_input()?;
        Ok(())
    }

    pub(crate) fn pop_input(&mut self) {
        if !self.input_stack.is_empty() {
            self.input_stack.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use garnish_lang_common::{DataType, DataVecWriter, ExpressionValue};
    use garnish_lang_instruction_set_builder::InstructionSetBuilder;

    use crate::runtime::tests::data_slice;
    use crate::runtime::ExpressionRuntime;

    #[test]
    fn input_gets_set() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.perform_addition();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let expression_value: ExpressionValue = ExpressionValue::integer(30).into();

        expression_runtime.set_input(&expression_value).unwrap();

        let expected =
            DataVecWriter::write_with(|w| w.push_data_type(DataType::Integer).push_integer(30));

        let result = data_slice(&expression_runtime, expected.len());

        assert_eq!(result, expected);
        assert_eq!(
            expression_runtime.input_size,
            expression_value.get_data().len()
        );
    }

    #[test]
    fn input_with_symbol_merges_symbol_tables() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::symbol("cat")).unwrap();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let expression_value = ExpressionValue::symbol("bear");

        expression_runtime
            .set_input(&expression_value.into())
            .unwrap();

        assert_eq!(*expression_runtime.symbol_table.get("bear").unwrap(), 21);
    }

    #[test]
    fn input_with_symbol_updates_symbol_references() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::symbol("cat")).unwrap();
        instructions.put(ExpressionValue::symbol("rabbit")).unwrap();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let expression_value = ExpressionValue::symbol("bear");

        expression_runtime
            .set_input(&expression_value.into())
            .unwrap();

        let expected =
            DataVecWriter::write_with(|w| w.push_data_type(DataType::Symbol).push_size(22));
        let result = data_slice(&expression_runtime, expected.len());

        assert_eq!(result, expected);
    }

    #[test]
    fn input_with_symbol_same_symbol_no_update() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::symbol("bear")).unwrap();
        instructions.put(ExpressionValue::symbol("cats")).unwrap();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let expression_value = ExpressionValue::symbol("cats");

        expression_runtime
            .set_input(&expression_value.into())
            .unwrap();

        let result = data_slice(&expression_runtime, 2);

        assert_eq!(result, vec![DataType::Symbol.try_into().unwrap(), 3]);
        assert_eq!(*expression_runtime.symbol_table.get("cats").unwrap(), 3);
    }

    #[test]
    fn put_input() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put_input();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        expression_runtime
            .set_input(&ExpressionValue::integer(20).into())
            .unwrap();
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 20);
        assert_eq!(
            expression_runtime.registers[expression_runtime.ref_cursor - 1],
            6
        );
    }

    #[test]
    fn put_input_without_input_yields_unit() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put_input();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit());
        assert_eq!(
            expression_runtime.data[expression_runtime.ref_cursor + 1],
            0
        );
    }

    #[test]
    fn expression_call_evaluations_yielding_result() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("add_numbers");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.perform_addition();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions.execute_expression("add_numbers");

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();
        assert_eq!(result.as_integer().unwrap(), 30);
    }

    #[test]
    fn expression_call_with_push_input_evaluates_yielding_result() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("add_numbers");

        instructions.put_input();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.perform_addition();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(5)).unwrap();
        instructions.push_input();
        instructions.execute_expression("add_numbers");

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();
        assert_eq!(result.as_integer().unwrap(), 25);
    }
}
