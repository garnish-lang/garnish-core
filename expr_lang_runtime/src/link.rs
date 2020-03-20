use crate::ExpressionRuntime;
use expr_lang_common::{skip_type, DataType, Result};
use std::convert::TryFrom;

impl ExpressionRuntime {
    pub(crate) fn make_link(&mut self) -> Result {
        let (left_ref, right_ref) = self.consume_last_two_refs()?;

        match (
            DataType::try_from(self.data[left_ref])?,
            DataType::try_from(self.data[right_ref])?,
        ) {
            (DataType::Link, _) => {
                // store to edit Link value's next later
                let r = self.value_cursor;

                // get Link values head value
                let head_value = self.read_size_at(skip_type(left_ref))?;

                self.insert_link_value(head_value, right_ref, 0)?;

                self.edit_link_next_value(left_ref, r)?;
            }
            (_, DataType::Link) => self.insert_unit()?,
            _ => {
                // insert head first, keep reference for second link's head value
                let head_ref = self.value_cursor;
                self.insert_link_value(0, left_ref, 0)?;

                // remove head from ref stack
                self.consume_last_ref()?;

                // keep reference to edit head link's next value
                let link_ref = self.value_cursor;
                self.insert_link_value(head_ref, right_ref, 0)?;

                // edit head link, setting link ref as next value
                self.edit_link_next_value(head_ref, link_ref)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::ExpressionRuntime;
    use expr_lang_common::ExpressionValue;
    use expr_lang_instruction_set_builder::InstructionSetBuilder;

    #[test]
    fn make_link() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(12345)).unwrap();
        instructions.put(ExpressionValue::integer(98765)).unwrap();
        instructions.make_link();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        // assertions of head link
        let head = result.get_link_head().unwrap();
        assert!(head.is_link());
        assert!(head.get_link_head().unwrap().is_unit());
        assert_eq!(head.get_link_value().unwrap().as_integer().unwrap(), 12345);
        assert!(!head.get_link_next().unwrap().is_unit());

        // assertions of resulting link
        assert!(result.is_link());
        assert!(!result.get_link_head().unwrap().is_unit());
        assert_eq!(
            result.get_link_value().unwrap().as_integer().unwrap(),
            98765
        );
        assert!(result.get_link_next().unwrap().is_unit());
    }

    #[test]
    fn make_link_with_link_left() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(12345)).unwrap();
        instructions.put(ExpressionValue::integer(98765)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::integer(111111)).unwrap();

        instructions.make_link();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        // assertions of second link which is previous of result

        let previous_head = result
            .get_link_head()
            .unwrap()
            .get_link_next()
            .unwrap()
            .get_link_head()
            .unwrap()
            .get_link_value()
            .unwrap()
            .as_integer()
            .unwrap();
        let result_head = result
            .get_link_head()
            .unwrap()
            .get_link_value()
            .unwrap()
            .as_integer()
            .unwrap();

        assert_eq!(previous_head, result_head);

        let head = result.get_link_head().unwrap();
        let previous = head.get_link_next().unwrap();

        assert!(!previous.get_link_next().unwrap().is_unit());

        // assertions of resulting link
        assert!(result.is_link());
        assert_eq!(
            result
                .get_link_head()
                .unwrap()
                .get_link_value()
                .unwrap()
                .as_integer()
                .unwrap(),
            previous
                .get_link_head()
                .unwrap()
                .get_link_value()
                .unwrap()
                .as_integer()
                .unwrap()
        );
        assert_eq!(
            result.get_link_value().unwrap().as_integer().unwrap(),
            111111
        );
        assert!(result.get_link_next().unwrap().is_unit());
    }

    #[test]
    fn make_link_with_link_right() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(111111)).unwrap();

        instructions.put(ExpressionValue::integer(12345)).unwrap();
        instructions.put(ExpressionValue::integer(98765)).unwrap();
        instructions.make_link();

        instructions.make_link();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit())
    }
}
