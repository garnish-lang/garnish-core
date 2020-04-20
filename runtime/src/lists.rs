use std::convert::TryFrom;

use garnish_common::{
    insert_associative_list_keys, size_to_bytes, skip_sizes, skip_type, DataType,
    ExpressionValueConsumer, Result,
};

use crate::runtime::ExpressionRuntime;

impl ExpressionRuntime {
    pub(crate) fn start_list(&mut self) {
        self.list_stack.push(self.ref_cursor);
    }

    pub(crate) fn make_list(&mut self) -> Result {
        let pos = self.value_cursor;

        self.insert_list_value(0, 0)?;

        // Find key count first
        // aka. number of Pair values in range
        let mut key_refs: Vec<usize> = vec![];
        let mut size = 0;

        let list_start_ref = match self.list_stack.pop() {
            None => Err("Make list called before start list.")?,
            Some(x) => x,
        };

        // we are defiantly making a list
        // if ref cursor is 0, list will be empty
        if self.ref_cursor == 0 {
            self.value_cursor = skip_sizes(self.value_cursor, 3);
        } else {
            let mut item_ref = list_start_ref;
            while item_ref < self.ref_cursor {
                let value_ref = self.get_value_ref(item_ref)?;
                let ref_value_start = self.value_cursor;
                self.insert_all_at_value_cursor(&size_to_bytes(value_ref))?;

                size += 1;

                if DataType::try_from(self.data[value_ref])? == DataType::Pair {
                    let key_ref = self.read_size_at(value_ref + 1)?;
                    let data_type = DataType::try_from(self.data[key_ref])?;
                    if data_type == DataType::Symbol || data_type == DataType::CharacterList {
                        key_refs.push(self.read_size_at(ref_value_start)?);
                    }
                }

                item_ref += 1;
            }

            // set ref cursor past values that have been inserted
            self.ref_cursor = list_start_ref;

            // Update count values
            // Write instead of 'insert'
            // because value cursor is already past these locations
            self.write_two_sizes_at(skip_type(pos), size, key_refs.len());

            let key_range_start = self.value_cursor;
            let key_area_range = key_range_start..skip_sizes(key_range_start, key_refs.len());

            insert_associative_list_keys(&mut self.data, key_area_range, &key_refs)?;

            // manually advance value cursor past key area
            self.value_cursor = skip_sizes(self.value_cursor, key_refs.len());
        }

        // Need to wait until all list values have been consumed
        // before inserting reference to list itself
        self.insert_at_ref_cursor(pos);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use garnish_common::ExpressionValue;
    use garnish_instruction_set_builder::InstructionSetBuilder;

    use crate::runtime::ExpressionRuntime;

    #[test]
    fn make_list_no_start_marker() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.make_list();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main");

        assert!(result.is_err());
    }

    #[test]
    fn make_associative_list_no_pairs() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.make_list();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 10);
        assert_eq!(result.get_list_item(1).unwrap().as_integer().unwrap(), 20);
        assert_eq!(result.get_list_item(2).unwrap().as_integer().unwrap(), 30);
    }

    #[test]
    fn make_associative_list_with_symbol_pairs() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.start_list();

        instructions.put(ExpressionValue::symbol("bear")).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.make_pair();

        instructions.put(ExpressionValue::symbol("cat")).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.make_pair();

        instructions.put(ExpressionValue::symbol("dog")).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.make_pair();

        instructions.make_list();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result
                .get_list_item(0)
                .unwrap()
                .get_pair_left()
                .unwrap()
                .as_symbol()
                .unwrap(),
            2
        );
        assert_eq!(
            result
                .get_list_item(0)
                .unwrap()
                .get_pair_right()
                .unwrap()
                .as_integer()
                .unwrap(),
            10
        );

        assert_eq!(
            result
                .get_list_item(1)
                .unwrap()
                .get_pair_left()
                .unwrap()
                .as_symbol()
                .unwrap(),
            3
        );
        assert_eq!(
            result
                .get_list_item(1)
                .unwrap()
                .get_pair_right()
                .unwrap()
                .as_integer()
                .unwrap(),
            20
        );

        assert_eq!(
            result
                .get_list_item(2)
                .unwrap()
                .get_pair_left()
                .unwrap()
                .as_symbol()
                .unwrap(),
            4
        );
        assert_eq!(
            result
                .get_list_item(2)
                .unwrap()
                .get_pair_right()
                .unwrap()
                .as_integer()
                .unwrap(),
            30
        );
    }

    #[test]
    fn make_associative_list_with_string_pairs() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.start_list();

        instructions
            .put(ExpressionValue::character_list("bear".into()))
            .unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.make_pair();

        instructions
            .put(ExpressionValue::character_list("cat".into()))
            .unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.make_pair();

        instructions
            .put(ExpressionValue::character_list("dog".into()))
            .unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.make_pair();

        instructions.make_list();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result
                .get_list_item(0)
                .unwrap()
                .get_pair_left()
                .unwrap()
                .as_string()
                .unwrap(),
            String::from("bear")
        );
        assert_eq!(
            result
                .get_list_item(0)
                .unwrap()
                .get_pair_right()
                .unwrap()
                .as_integer()
                .unwrap(),
            10
        );

        assert_eq!(
            result
                .get_list_item(1)
                .unwrap()
                .get_pair_left()
                .unwrap()
                .as_string()
                .unwrap(),
            String::from("cat")
        );
        assert_eq!(
            result
                .get_list_item(1)
                .unwrap()
                .get_pair_right()
                .unwrap()
                .as_integer()
                .unwrap(),
            20
        );

        assert_eq!(
            result
                .get_list_item(2)
                .unwrap()
                .get_pair_left()
                .unwrap()
                .as_string()
                .unwrap(),
            String::from("dog")
        );
        assert_eq!(
            result
                .get_list_item(2)
                .unwrap()
                .get_pair_right()
                .unwrap()
                .as_integer()
                .unwrap(),
            30
        );
    }

    #[test]
    fn make_empty_associative_list() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.start_list();
        instructions.make_list();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.list_len().unwrap(), 0);
    }
}
