use std::convert::{TryFrom, TryInto};

use garnish_lang_common::{
    hash_of_character_list, insert_associative_list_keys, skip_sizes, skip_type,
    skip_type_and_2_sizes, skip_type_and_size, two_sizes_to_bytes, DataType,
    ExpressionValueConsumer, ExpressionValueRef, RangeFlags, Result,
};

use crate::{ExpressionContext, ExpressionRuntime};

impl ExpressionRuntime {
    pub(crate) fn partially_apply(&mut self) -> Result {
        let (left_ref, right_ref) = self.consume_last_two_refs()?;

        match (
            DataType::try_from(self.data[left_ref])?,
            DataType::try_from(self.data[right_ref])?,
        ) {
            (DataType::Expression, _) | (DataType::ExternalMethod, _) => {
                match DataType::try_from(self.data[right_ref])? {
                    DataType::Pair => {
                        // check if pair left is symbol
                        // if so, make a list with pair as only item
                        // and use that as partial value
                        let pair_left_ref = self.read_size_at(skip_type(right_ref))?;
                        match DataType::try_from(self.data[pair_left_ref])? {
                            DataType::Symbol => {
                                // make list
                                self.start_list();
                                self.insert_reference_value(right_ref)?;
                                self.make_list()?;

                                let list_ref = self.consume_last_ref()?;

                                self.insert_partial_value(left_ref, list_ref)
                            }
                            // not an association, make partial value like normal
                            _ => self.insert_partial_value(left_ref, right_ref),
                        }
                    }
                    DataType::List => {
                        let list_ref = self.insert_new_ordered_list(right_ref)?;
                        self.insert_partial_value(left_ref, list_ref)
                    }
                    _ => self.insert_partial_value(left_ref, right_ref),
                }
            }
            (DataType::Partial, _) => {
                let partial_expression_ref = self.read_size_at(skip_type(left_ref))?;
                let partial_value_ref = self.read_size_at(skip_type_and_size(left_ref))?;

                match self.make_applied_list(partial_value_ref, right_ref)? {
                    Some(list_ref) => self.insert_partial_value(partial_expression_ref, list_ref),
                    None => {
                        self.insert_at_ref_cursor(left_ref);
                        Ok(())
                    }
                }
            }
            _ => self.insert_unit(),
        }
    }

    pub(crate) fn apply<T>(&mut self, context: &T) -> Result
    where
        T: ExpressionContext,
    {
        let (left_ref, right_ref) = self.last_two_refs()?;

        match (
            DataType::try_from(self.data[left_ref])?,
            DataType::try_from(self.data[right_ref])?,
        ) {
            (DataType::Range, DataType::Integer) | (DataType::Range, DataType::Float) => {
                self.consume_last_two_refs()?;

                let (flags, start, end, _) = self.get_range_values_at(left_ref)?;

                self.insert_range_value(flags.set_has_step(), start, end, Some(right_ref))
            }
            (DataType::Expression, _) => {
                // consumes right ref
                self.push_input()?;

                // consume left ref
                self.consume_last_ref()?;
                // get name of expression
                let expression_index = self.read_size_at(skip_type(left_ref))?;

                match self.get_expression_start(expression_index) {
                    Err(_) => self.insert_unit(),
                    Ok(start) => {
                        // add next cursor
                        self.push_frame(start);
                        Ok(())
                    }
                }
            }
            (DataType::ExternalMethod, _) => {
                let symbol_index = self.read_size_at(skip_type(left_ref))?;
                match self.symbol_table.iter().find(|(_s, i)| **i == symbol_index) {
                    None => {
                        // TODO: needs test when can created a Runtime with raw data for instruction set
                        self.insert_unit()
                    }
                    Some((name, _i)) => {
                        let value = context.execute(
                            name.clone(),
                            ExpressionValueRef::new_with_start(
                                &self.data,
                                Some(&self.symbol_table),
                                right_ref,
                            )?,
                        );
                        self.copy_into(&value)
                    }
                }
            }
            (DataType::Partial, _) => {
                let partial_expression_ref = self.read_size_at(skip_type(left_ref))?;
                let partial_value_ref = self.read_size_at(skip_type_and_size(left_ref))?;

                let input_ref = match self.make_applied_list(partial_value_ref, right_ref)? {
                    Some(list_ref) => list_ref,
                    None => partial_value_ref,
                };

                self.insert_at_ref_cursor(input_ref);
                self.push_input()?;

                match DataType::try_from(self.data[partial_expression_ref])? {
                    DataType::Expression => {
                        // get name of expression
                        let expression_index =
                            self.read_size_at(skip_type(partial_expression_ref))?;

                        match self.get_expression_start(expression_index) {
                            Err(_) => self.insert_unit(),
                            Ok(start) => {
                                // add next cursor
                                self.push_frame(start);
                                Ok(())
                            }
                        }
                    }
                    DataType::ExternalMethod => {
                        let symbol_index = self.read_size_at(skip_type(partial_expression_ref))?;
                        match self.symbol_table.iter().find(|(_s, i)| **i == symbol_index) {
                            None => {
                                // TODO: needs test when can created a Runtime with raw data for instruction set
                                self.insert_unit()
                            }
                            Some((name, _i)) => {
                                let value = context.execute(
                                    name.clone(),
                                    ExpressionValueRef::new_with_start(
                                        &self.data,
                                        Some(&self.symbol_table),
                                        input_ref,
                                    )?,
                                );
                                self.copy_into(&value)
                            }
                        }
                    }
                    _ => self.insert_unit(),
                }
            }
            _ => self.perform_access(),
        }
    }

    fn insert_list_item_references(&mut self, list_ref: usize) -> Result {
        let list_length = self.read_size_at(skip_type(list_ref))?;

        // insert all values from right list
        let item_start = skip_type_and_2_sizes(list_ref);
        for i in 0..list_length {
            let item_ref = self.read_size_at(skip_sizes(item_start, i))?;
            self.insert_reference_value(item_ref)?;
        }

        Ok(())
    }

    fn insert_new_ordered_list(&mut self, list_ref: usize) -> Result<usize> {
        // create new list with associations at the end of the item area
        let mut length = self.read_size_at(skip_type(list_ref))?;
        let mut key_count = self.read_size_at(skip_type_and_size(list_ref))?;
        let positional_count = length - key_count;

        let item_area_start = skip_type_and_2_sizes(list_ref);
        let key_area_start = skip_sizes(item_area_start, length);

        let new_list_ref = self.value_cursor;
        self.insert_at_value_cursor(DataType::List.try_into().unwrap())?;
        self.insert_all_at_value_cursor(&two_sizes_to_bytes(length, key_count))?;

        let new_positional_item_area_start = skip_type_and_2_sizes(new_list_ref);
        let new_key_item_area_start = skip_sizes(new_positional_item_area_start, positional_count);
        let mut new_item_cursor = new_positional_item_area_start;
        let mut new_key_item_cursor = new_key_item_area_start;
        let mut key_refs = vec![];

        for i in 0..length {
            let item_ref = self.read_size_at(skip_sizes(item_area_start, i))?;

            // check key area for item_ref
            // insert ref if not present

            let mut is_key = false;
            for j in 0..key_count {
                let key_ref = self.read_size_at(skip_sizes(key_area_start, j))?;
                if item_ref == key_ref {
                    is_key = true;
                    break;
                }
            }

            if is_key {
                // check if key is already in new list
                let item_key_ref = self.read_size_at(skip_type(item_ref))?;
                let item_key_type = DataType::try_from(self.data[item_key_ref])?;
                let item_symbol_value = match item_key_type {
                    DataType::Symbol => self.read_size_at(skip_type(item_key_ref))?,
                    DataType::CharacterList => {
                        hash_of_character_list(item_key_ref, &self.data) as usize
                    }
                    _ => 0,
                };

                let mut exists = None;
                for j in 0..key_refs.len() {
                    let area_pos = skip_sizes(new_key_item_area_start, j);
                    let pair_ref = self.read_size_at(area_pos)?;
                    let key_ref = self.read_size_at(skip_type(pair_ref))?;
                    let key_type = DataType::try_from(self.data[key_ref])?;
                    let symbol_value = match key_type {
                        DataType::Symbol => self.read_size_at(skip_type(key_ref))?,
                        DataType::CharacterList => {
                            hash_of_character_list(key_ref, &self.data) as usize
                        }
                        _ => 0,
                    };

                    if symbol_value == item_symbol_value {
                        exists = Some((area_pos, j));
                    }
                }

                match exists {
                    Some((pos, existing_item_index)) => {
                        // replace existing pair value
                        self.write_size_at(pos, item_ref)?;
                        key_refs[existing_item_index] = item_ref;
                        key_count -= 1;
                        length -= 1;
                    }
                    None => {
                        self.write_size_at(new_key_item_cursor, item_ref)?;
                        new_key_item_cursor = skip_sizes(new_key_item_cursor, 1);
                        key_refs.push(item_ref);
                    }
                }
            } else {
                self.write_size_at(new_item_cursor, item_ref)?;
                new_item_cursor = skip_sizes(new_item_cursor, 1);
            }
        }

        let new_key_area_start = skip_sizes(new_positional_item_area_start, length);
        insert_associative_list_keys(
            &mut self.data,
            new_key_area_start..(new_key_area_start + key_count),
            &key_refs,
        )?;

        // may not have used every key due to duplicates
        // rewrite modified lengths
        self.write_size_at(skip_type(new_list_ref), length)?;
        self.write_size_at(skip_type_and_size(new_list_ref), key_count)?;

        self.value_cursor = skip_sizes(new_key_area_start, key_count);

        Ok(new_list_ref)
    }

    fn make_applied_list(&mut self, left_ref: usize, right_ref: usize) -> Result<Option<usize>> {
        match (
            DataType::try_from(self.data[left_ref])?,
            DataType::try_from(self.data[right_ref])?,
        ) {
            (DataType::Unit, _) => {
                // replace unit value with right ref
                // instead of making two values into list
                return Ok(Some(right_ref));
            }
            (_, DataType::Unit) => {
                // does nothing
                return Ok(None);
            }
            (DataType::List, DataType::List) => {
                self.start_list();
                let list_length = self.read_size_at(skip_type(left_ref))?;
                let mut new_value_cursor = 0;
                let new_value_end = self.read_size_at(skip_type(right_ref))?;
                let new_value_item_start = skip_type_and_2_sizes(right_ref);

                // insert all values from right list
                let item_start = skip_type_and_2_sizes(left_ref);
                for i in 0..list_length {
                    let item_ref = self.read_size_at(skip_sizes(item_start, i))?;

                    let insert_ref = if new_value_cursor != new_value_end
                        && DataType::try_from(self.data[item_ref])? == DataType::Unit
                    {
                        // insert value from new values and increment cursor
                        let r =
                            self.read_size_at(skip_sizes(new_value_item_start, new_value_cursor))?;

                        let is_associative = match DataType::try_from(self.data[r])? {
                            DataType::Pair => {
                                let key_ref = self.read_size_at(skip_type(r))?;
                                match DataType::try_from(self.data[key_ref])? {
                                    DataType::Symbol => true,
                                    _ => false,
                                }
                            }
                            _ => false,
                        };

                        if is_associative {
                            item_ref
                        } else {
                            new_value_cursor += 1;
                            r
                        }
                    } else {
                        item_ref
                    };

                    self.insert_reference_value(insert_ref)?;
                }

                // add remaining values
                while new_value_cursor != new_value_end {
                    let r =
                        self.read_size_at(skip_sizes(new_value_item_start, new_value_cursor))?;
                    self.insert_reference_value(r)?;
                    new_value_cursor += 1;
                }

                self.make_list()?;

                let list_ref = self.consume_last_ref()?;

                // create ordered list from right list
                let new_list_ref = self.insert_new_ordered_list(list_ref)?;

                return Ok(Some(new_list_ref));
            }
            (DataType::List, _) => {
                self.start_list();
                let list_length = self.read_size_at(skip_type(left_ref))?;
                let mut value_used = false;

                // check type
                // if associative pair, don't used to skip Unit
                let is_associative = match DataType::try_from(self.data[right_ref])? {
                    DataType::Pair => {
                        let key_ref = self.read_size_at(skip_type(right_ref))?;
                        match DataType::try_from(self.data[key_ref])? {
                            DataType::Symbol => true,
                            _ => false,
                        }
                    }
                    _ => false,
                };

                // insert all values from right list
                let item_start = skip_type_and_2_sizes(left_ref);
                if is_associative {
                    // associative values don't replace unit values
                    // just insert all values sequentially
                    for i in 0..list_length {
                        let item_ref = self.read_size_at(skip_sizes(item_start, i))?;
                        self.insert_reference_value(item_ref)?;
                    }

                    self.insert_reference_value(right_ref)?;
                } else {
                    for i in 0..list_length {
                        let item_ref = self.read_size_at(skip_sizes(item_start, i))?;

                        let insert_ref = if !value_used
                            && !is_associative
                            && DataType::try_from(self.data[item_ref])? == DataType::Unit
                        {
                            value_used = true;
                            right_ref
                        } else {
                            item_ref
                        };

                        self.insert_reference_value(insert_ref)?;
                    }

                    if !value_used {
                        self.insert_reference_value(right_ref)?;
                    }
                }

                self.make_list()?;

                let list_ref = self.consume_last_ref()?;

                // create ordered list from right list
                let new_list_ref = self.insert_new_ordered_list(list_ref)?;

                return Ok(Some(new_list_ref));
            }
            (_, DataType::List) => {
                self.start_list();
                self.insert_reference_value(left_ref)?;
                self.insert_list_item_references(right_ref)?;
            }
            _ => {
                self.start_list();
                self.insert_reference_value(left_ref)?;
                self.insert_reference_value(right_ref)?;
            }
        }

        self.make_list()?;

        let list_ref = self.consume_last_ref()?;

        return Ok(Some(list_ref));
    }
}

#[cfg(test)]
mod apply {
    use garnish_lang_common::{has_step, ExpressionValue, ExpressionValueRef};
    use garnish_lang_instruction_set_builder::InstructionSetBuilder;

    use crate::{ExpressionContext, ExpressionRuntime};

    #[test]
    fn non_invokable_value_defer_to_access() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("bears".to_string()))
            .unwrap();
        instructions.put(ExpressionValue::symbol("length")).unwrap();
        instructions.apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 5);
    }

    #[test]
    fn expression() {
        let mut instructions = InstructionSetBuilder::new();
        insert_add_expression(&mut instructions);

        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::expression("add"))
            .unwrap();
        instructions
            .put(
                ExpressionValue::list()
                    .add(ExpressionValue::integer(10))
                    .add(ExpressionValue::integer(20)),
            )
            .unwrap();

        instructions.apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 30);
    }

    struct MathContext {}

    impl ExpressionContext for MathContext {
        fn resolve(&self, _name: String) -> ExpressionValue {
            unimplemented!()
        }

        fn execute(&self, name: String, input: ExpressionValueRef) -> ExpressionValue {
            match name.as_str() {
                "sqrt" => {
                    ExpressionValue::integer((input.as_integer().unwrap() as f32).sqrt() as i32)
                        .into()
                }
                "log" => {
                    let left = input.get_list_item(0).unwrap().as_float().unwrap();
                    let right = input.get_list_item(1).unwrap().as_float().unwrap();

                    let result = left.log(right);

                    ExpressionValue::float(result).into()
                }
                "constant_value" => ExpressionValue::integer(150).into(),
                _ => panic!(),
            }
        }
    }

    #[test]
    fn external_method() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::external_method("sqrt"))
            .unwrap();
        instructions.put(ExpressionValue::integer(144)).unwrap();

        instructions.apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime
            .execute_with_context("main".to_string(), &MathContext {})
            .unwrap();

        assert_eq!(result.as_integer().unwrap(), 12);
    }

    #[test]
    fn partial_expression_single_value() {
        let mut instructions = InstructionSetBuilder::new();
        insert_add_expression(&mut instructions);

        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::expression("add"))
            .unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.partially_apply();

        instructions.put(ExpressionValue::integer(20)).unwrap();

        instructions.apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 30);
    }

    #[test]
    fn partial_expression_list_value() {
        let mut instructions = InstructionSetBuilder::new();
        insert_add_expression(&mut instructions);

        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::expression("add"))
            .unwrap();
        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.make_list();
        instructions.partially_apply();

        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 30);
    }

    #[test]
    fn partial_external_method_single_value() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::external_method("log"))
            .unwrap();
        instructions.put(ExpressionValue::float(100.0)).unwrap();
        instructions.partially_apply();

        instructions.put(ExpressionValue::float(10.0)).unwrap();

        instructions.apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime
            .execute_with_context("main".to_string(), &MathContext {})
            .unwrap();

        assert_eq!(result.as_float().unwrap(), 100.0f32.log(10.0));
    }

    #[test]
    fn partial_external_method_list_value() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::external_method("log"))
            .unwrap();
        instructions.start_list();
        instructions.put(ExpressionValue::float(100.0)).unwrap();
        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.make_list();
        instructions.partially_apply();

        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime
            .execute_with_context("main".to_string(), &MathContext {})
            .unwrap();

        assert_eq!(result.as_float().unwrap(), 100.0f32.log(10.0));
    }

    fn insert_add_expression(instructions: &mut InstructionSetBuilder) {
        instructions.start_expression("add");

        instructions.put_input();
        instructions.put(ExpressionValue::integer(0)).unwrap();
        instructions.perform_access();

        instructions.put_input();
        instructions.put(ExpressionValue::integer(1)).unwrap();
        instructions.perform_access();

        instructions.perform_addition();

        instructions.end_expression();
    }

    #[test]
    fn add_step_to_integer_range() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::integer_range(Some(10), Some(20)))
            .unwrap();
        instructions.put(ExpressionValue::integer(2)).unwrap();

        instructions.apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main".to_string()).unwrap();

        assert!(has_step(result.get_range_flags().unwrap()));
        assert_eq!(result.get_range_min().unwrap().as_integer().unwrap(), 10);
        assert_eq!(result.get_range_max().unwrap().as_integer().unwrap(), 20);
        assert_eq!(result.get_range_step().unwrap().as_integer().unwrap(), 2);
    }

    #[test]
    fn add_step_to_float_range() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::float_range(Some(10.0), Some(20.0)))
            .unwrap();
        instructions.put(ExpressionValue::float(2.0)).unwrap();

        instructions.apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main".to_string()).unwrap();

        assert!(has_step(result.get_range_flags().unwrap()));
        assert_eq!(result.get_range_min().unwrap().as_float().unwrap(), 10.0);
        assert_eq!(result.get_range_max().unwrap().as_float().unwrap(), 20.0);
        assert_eq!(result.get_range_step().unwrap().as_float().unwrap(), 2.0);
    }

    #[test]
    fn add_step_to_character_range() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::char_range(Some('a'), Some('z')))
            .unwrap();
        instructions.put(ExpressionValue::integer(2)).unwrap();

        instructions.apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main".to_string()).unwrap();

        assert!(has_step(result.get_range_flags().unwrap()));
        assert_eq!(result.get_range_min().unwrap().as_char().unwrap(), 'a');
        assert_eq!(result.get_range_max().unwrap().as_char().unwrap(), 'z');
        assert_eq!(result.get_range_step().unwrap().as_integer().unwrap(), 2);
    }
}

#[cfg(test)]
mod partially_apply {
    use garnish_lang_common::ExpressionValue;
    use garnish_lang_instruction_set_builder::InstructionSetBuilder;

    use crate::ExpressionRuntime;

    #[test]
    fn expression_integer() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::expression("my_expression"))
            .unwrap();
        instructions.put(ExpressionValue::integer(120)).unwrap();
        instructions.partially_apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result.get_partial_base().unwrap().as_string().unwrap(),
            "my_expression".to_string()
        );
        assert_eq!(
            result.get_partial_value().unwrap().as_integer().unwrap(),
            120
        );
    }

    #[test]
    fn external_method_integer() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::external_method("my_external_method"))
            .unwrap();
        instructions.put(ExpressionValue::integer(120)).unwrap();
        instructions.partially_apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result.get_partial_base().unwrap().as_string().unwrap(),
            "my_external_method".to_string()
        );
        assert_eq!(
            result.get_partial_value().unwrap().as_integer().unwrap(),
            120
        );
    }

    #[test]
    fn non_expression_external_method() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(100)).unwrap();
        instructions.put(ExpressionValue::integer(120)).unwrap();
        instructions.partially_apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit())
    }

    #[test]
    fn extend_partial_application_with_single_value() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::expression("my_expression"))
            .unwrap();
        instructions.put(ExpressionValue::integer(120)).unwrap();
        instructions.partially_apply();

        instructions.put(ExpressionValue::integer(150)).unwrap();
        instructions.partially_apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result.get_partial_base().unwrap().as_string().unwrap(),
            "my_expression".to_string()
        );

        let list_value = result.get_partial_value().unwrap();

        assert_eq!(
            list_value.get_list_item(0).unwrap().as_integer().unwrap(),
            120
        );
        assert_eq!(
            list_value.get_list_item(1).unwrap().as_integer().unwrap(),
            150
        );
    }

    #[test]
    fn extend_partial_application_with_list_value() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::expression("my_expression"))
            .unwrap();
        instructions.put(ExpressionValue::integer(120)).unwrap();
        instructions.partially_apply();

        instructions.start_list();
        instructions.put(ExpressionValue::integer(150)).unwrap();
        instructions.put(ExpressionValue::integer(180)).unwrap();
        instructions.put(ExpressionValue::integer(210)).unwrap();
        instructions.make_list();
        instructions.partially_apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result.get_partial_base().unwrap().as_string().unwrap(),
            "my_expression".to_string()
        );

        let list_value = result.get_partial_value().unwrap();

        assert_eq!(
            list_value.get_list_item(0).unwrap().as_integer().unwrap(),
            120
        );
        assert_eq!(
            list_value.get_list_item(1).unwrap().as_integer().unwrap(),
            150
        );
        assert_eq!(
            list_value.get_list_item(2).unwrap().as_integer().unwrap(),
            180
        );
        assert_eq!(
            list_value.get_list_item(3).unwrap().as_integer().unwrap(),
            210
        );
    }

    #[test]
    fn extend_partial_application_of_list_with_single_value() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::expression("my_expression"))
            .unwrap();

        instructions.start_list();
        instructions.put(ExpressionValue::integer(120)).unwrap();
        instructions.put(ExpressionValue::integer(150)).unwrap();
        instructions.put(ExpressionValue::integer(180)).unwrap();
        instructions.make_list();

        instructions.partially_apply();

        instructions.put(ExpressionValue::integer(210)).unwrap();
        instructions.partially_apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result.get_partial_base().unwrap().as_string().unwrap(),
            "my_expression".to_string()
        );

        let list_value = result.get_partial_value().unwrap();

        assert_eq!(
            list_value.get_list_item(0).unwrap().as_integer().unwrap(),
            120
        );
        assert_eq!(
            list_value.get_list_item(1).unwrap().as_integer().unwrap(),
            150
        );
        assert_eq!(
            list_value.get_list_item(2).unwrap().as_integer().unwrap(),
            180
        );
        assert_eq!(
            list_value.get_list_item(3).unwrap().as_integer().unwrap(),
            210
        );
    }

    #[test]
    fn extend_partial_application_of_list_with_list_value() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::expression("my_expression"))
            .unwrap();

        instructions.start_list();
        instructions.put(ExpressionValue::integer(120)).unwrap();
        instructions.put(ExpressionValue::integer(150)).unwrap();
        instructions.put(ExpressionValue::integer(180)).unwrap();
        instructions.make_list();

        instructions.partially_apply();

        instructions.start_list();
        instructions.put(ExpressionValue::integer(210)).unwrap();
        instructions.put(ExpressionValue::integer(240)).unwrap();
        instructions.put(ExpressionValue::integer(270)).unwrap();
        instructions.make_list();
        instructions.partially_apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result.get_partial_base().unwrap().as_string().unwrap(),
            "my_expression".to_string()
        );

        let list_value = result.get_partial_value().unwrap();

        assert_eq!(
            list_value.get_list_item(0).unwrap().as_integer().unwrap(),
            120
        );
        assert_eq!(
            list_value.get_list_item(1).unwrap().as_integer().unwrap(),
            150
        );
        assert_eq!(
            list_value.get_list_item(2).unwrap().as_integer().unwrap(),
            180
        );
        assert_eq!(
            list_value.get_list_item(3).unwrap().as_integer().unwrap(),
            210
        );
        assert_eq!(
            list_value.get_list_item(4).unwrap().as_integer().unwrap(),
            240
        );
        assert_eq!(
            list_value.get_list_item(5).unwrap().as_integer().unwrap(),
            270
        );
    }

    #[test]
    fn unit_application_replaced_by_value() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::expression("my_expression"))
            .unwrap();

        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.partially_apply();

        instructions.put(ExpressionValue::integer(120)).unwrap();
        instructions.partially_apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result.get_partial_base().unwrap().as_string().unwrap(),
            "my_expression".to_string()
        );

        assert_eq!(
            result.get_partial_value().unwrap().as_integer().unwrap(),
            120
        );
    }

    #[test]
    fn unit_application_does_nothing_if_second() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::expression("my_expression"))
            .unwrap();

        instructions.put(ExpressionValue::integer(120)).unwrap();
        instructions.partially_apply();

        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.partially_apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result.get_partial_base().unwrap().as_string().unwrap(),
            "my_expression".to_string()
        );

        assert_eq!(
            result.get_partial_value().unwrap().as_integer().unwrap(),
            120
        );
    }

    #[test]
    fn filling_unit_in_list_with_single_value() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::expression("my_expression"))
            .unwrap();

        instructions.start_list();
        instructions.put(ExpressionValue::integer(120)).unwrap();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.put(ExpressionValue::integer(180)).unwrap();
        instructions.make_list();

        instructions.partially_apply();

        instructions.put(ExpressionValue::integer(150)).unwrap();
        instructions.partially_apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result.get_partial_base().unwrap().as_string().unwrap(),
            "my_expression".to_string()
        );

        let list_value = result.get_partial_value().unwrap();

        assert_eq!(
            list_value.get_list_item(0).unwrap().as_integer().unwrap(),
            120
        );
        assert_eq!(
            list_value.get_list_item(1).unwrap().as_integer().unwrap(),
            150
        );
        assert_eq!(
            list_value.get_list_item(2).unwrap().as_integer().unwrap(),
            180
        );
    }

    #[test]
    fn filling_one_unit_in_list_with_single_value() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::expression("my_expression"))
            .unwrap();

        instructions.start_list();
        instructions.put(ExpressionValue::integer(120)).unwrap();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.put(ExpressionValue::integer(180)).unwrap();
        instructions.make_list();

        instructions.partially_apply();

        instructions.put(ExpressionValue::integer(150)).unwrap();
        instructions.partially_apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result.get_partial_base().unwrap().as_string().unwrap(),
            "my_expression".to_string()
        );

        let list_value = result.get_partial_value().unwrap();

        assert_eq!(
            list_value.get_list_item(0).unwrap().as_integer().unwrap(),
            120
        );
        assert_eq!(
            list_value.get_list_item(1).unwrap().as_integer().unwrap(),
            150
        );
        assert!(list_value.get_list_item(2).unwrap().is_unit());
        assert_eq!(
            list_value.get_list_item(3).unwrap().as_integer().unwrap(),
            180
        );
    }

    #[test]
    fn filling_unit_in_list_with_list_value() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::expression("my_expression"))
            .unwrap();

        instructions.start_list();
        instructions.put(ExpressionValue::integer(120)).unwrap();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.put(ExpressionValue::integer(180)).unwrap();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.put(ExpressionValue::integer(240)).unwrap();
        instructions.make_list();

        instructions.partially_apply();

        instructions.start_list();
        instructions.put(ExpressionValue::integer(150)).unwrap();
        instructions.put(ExpressionValue::integer(210)).unwrap();
        instructions.put(ExpressionValue::integer(270)).unwrap();
        instructions.make_list();
        instructions.partially_apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result.get_partial_base().unwrap().as_string().unwrap(),
            "my_expression".to_string()
        );

        let list_value = result.get_partial_value().unwrap();

        assert_eq!(
            list_value.get_list_item(0).unwrap().as_integer().unwrap(),
            120
        );
        assert_eq!(
            list_value.get_list_item(1).unwrap().as_integer().unwrap(),
            150
        );
        assert_eq!(
            list_value.get_list_item(2).unwrap().as_integer().unwrap(),
            180
        );
        assert_eq!(
            list_value.get_list_item(3).unwrap().as_integer().unwrap(),
            210
        );
        assert_eq!(
            list_value.get_list_item(4).unwrap().as_integer().unwrap(),
            240
        );
        assert_eq!(
            list_value.get_list_item(5).unwrap().as_integer().unwrap(),
            270
        );
    }

    #[test]
    fn applying_second_list_with_unit_values_at_start() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::expression("my_expression"))
            .unwrap();

        instructions.start_list();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.put(ExpressionValue::integer(240)).unwrap();
        instructions.make_list();

        instructions.partially_apply();

        instructions.start_list();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.put(ExpressionValue::integer(210)).unwrap();
        instructions.make_list();
        instructions.partially_apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result.get_partial_base().unwrap().as_string().unwrap(),
            "my_expression".to_string()
        );

        let list_value = result.get_partial_value().unwrap();

        assert!(list_value.get_list_item(0).unwrap().is_unit());
        assert_eq!(
            list_value.get_list_item(1).unwrap().as_integer().unwrap(),
            210
        );
        assert_eq!(
            list_value.get_list_item(2).unwrap().as_integer().unwrap(),
            240
        );
    }

    #[test]
    fn associative_pair_application() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::expression("my_expression"))
            .unwrap();

        instructions
            .put(ExpressionValue::pair(
                ExpressionValue::symbol("bear"),
                ExpressionValue::integer(120),
            ))
            .unwrap();
        instructions.partially_apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result.get_partial_base().unwrap().as_string().unwrap(),
            "my_expression".to_string()
        );

        let value = result.get_partial_value().unwrap();
        let first_item = value.get_list_item(0).unwrap();

        assert_eq!(
            first_item.get_pair_left().unwrap().as_string().unwrap(),
            "bear".to_string()
        );

        assert_eq!(
            first_item.get_pair_right().unwrap().as_integer().unwrap(),
            120
        );
    }

    #[test]
    fn list_with_associations_is_reordered_during_partial_application() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::expression("my_expression"))
            .unwrap();

        instructions
            .put(
                ExpressionValue::list()
                    .add(ExpressionValue::pair(
                        ExpressionValue::symbol("bear"),
                        ExpressionValue::integer(120),
                    ))
                    .add(ExpressionValue::pair(
                        ExpressionValue::symbol("cat"),
                        ExpressionValue::integer(150),
                    ))
                    .add(ExpressionValue::integer(180))
                    .add(ExpressionValue::integer(210)),
            )
            .unwrap();
        instructions.partially_apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result.get_partial_base().unwrap().as_string().unwrap(),
            "my_expression".to_string()
        );

        let list = result.get_partial_value().unwrap();
        let third_item = list.get_list_item(2).unwrap();
        let fourth_item = list.get_list_item(3).unwrap();

        assert_eq!(list.get_list_item(0).unwrap().as_integer().unwrap(), 180);
        assert_eq!(list.get_list_item(1).unwrap().as_integer().unwrap(), 210);

        assert_eq!(
            third_item.get_pair_left().unwrap().as_string().unwrap(),
            "bear".to_string()
        );

        assert_eq!(
            third_item.get_pair_right().unwrap().as_integer().unwrap(),
            120
        );
        assert_eq!(
            fourth_item.get_pair_left().unwrap().as_string().unwrap(),
            "cat".to_string()
        );

        assert_eq!(
            fourth_item.get_pair_right().unwrap().as_integer().unwrap(),
            150
        );
    }

    #[test]
    fn non_associative_application_after_associative_pair_application() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::expression("my_expression"))
            .unwrap();

        instructions
            .put(ExpressionValue::pair(
                ExpressionValue::symbol("bear"),
                ExpressionValue::integer(120),
            ))
            .unwrap();
        instructions.partially_apply();

        instructions.put(ExpressionValue::integer(150)).unwrap();
        instructions.partially_apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result.get_partial_base().unwrap().as_string().unwrap(),
            "my_expression".to_string()
        );

        let list = result.get_partial_value().unwrap();

        assert_eq!(list.get_list_item(0).unwrap().as_integer().unwrap(), 150);

        let second_item = list.get_list_item(1).unwrap();

        assert_eq!(
            second_item.get_pair_left().unwrap().as_string().unwrap(),
            "bear".to_string()
        );

        assert_eq!(
            second_item.get_pair_right().unwrap().as_integer().unwrap(),
            120
        );
    }

    #[test]
    fn list_application_after_associative_pair_application() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::expression("my_expression"))
            .unwrap();

        instructions
            .put(ExpressionValue::pair(
                ExpressionValue::symbol("bear"),
                ExpressionValue::integer(120),
            ))
            .unwrap();
        instructions.partially_apply();

        instructions
            .put(
                ExpressionValue::list()
                    .add(ExpressionValue::pair(
                        ExpressionValue::symbol("cat"),
                        ExpressionValue::integer(150),
                    ))
                    .add(ExpressionValue::pair(
                        ExpressionValue::symbol("dog"),
                        ExpressionValue::integer(180),
                    ))
                    .add(ExpressionValue::integer(210))
                    .add(ExpressionValue::integer(240)),
            )
            .unwrap();
        instructions.partially_apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result.get_partial_base().unwrap().as_string().unwrap(),
            "my_expression".to_string()
        );

        let list = result.get_partial_value().unwrap();

        assert_eq!(list.get_list_item(0).unwrap().as_integer().unwrap(), 210);
        assert_eq!(list.get_list_item(1).unwrap().as_integer().unwrap(), 240);

        let third_item = list.get_list_item(2).unwrap();
        let fourth_item = list.get_list_item(3).unwrap();
        let fifth_item = list.get_list_item(4).unwrap();

        assert_eq!(
            third_item.get_pair_left().unwrap().as_string().unwrap(),
            "bear".to_string()
        );

        assert_eq!(
            third_item.get_pair_right().unwrap().as_integer().unwrap(),
            120
        );

        assert_eq!(
            fourth_item.get_pair_left().unwrap().as_string().unwrap(),
            "cat".to_string()
        );

        assert_eq!(
            fourth_item.get_pair_right().unwrap().as_integer().unwrap(),
            150
        );

        assert_eq!(
            fifth_item.get_pair_left().unwrap().as_string().unwrap(),
            "dog".to_string()
        );

        assert_eq!(
            fifth_item.get_pair_right().unwrap().as_integer().unwrap(),
            180
        );
    }

    #[test]
    fn applying_duplicate_associations() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::expression("my_expression"))
            .unwrap();

        instructions
            .put(ExpressionValue::pair(
                ExpressionValue::symbol("bear"),
                ExpressionValue::integer(120),
            ))
            .unwrap();
        instructions.partially_apply();

        instructions
            .put(ExpressionValue::pair(
                ExpressionValue::symbol("bear"),
                ExpressionValue::integer(150),
            ))
            .unwrap();
        instructions.partially_apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result.get_partial_base().unwrap().as_string().unwrap(),
            "my_expression".to_string()
        );

        let list = result.get_partial_value().unwrap();

        assert_eq!(list.list_len().unwrap(), 1);

        let item = list.get_list_item(0).unwrap();

        assert_eq!(
            item.get_pair_left().unwrap().as_string().unwrap(),
            "bear".to_string()
        );

        assert_eq!(item.get_pair_right().unwrap().as_integer().unwrap(), 150);
    }

    #[test]
    fn applying_association_will_not_replace_unit() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::expression("my_expression"))
            .unwrap();

        instructions.start_list();
        instructions.put(ExpressionValue::integer(120)).unwrap();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.put(ExpressionValue::integer(150)).unwrap();
        instructions.make_list();

        instructions.partially_apply();

        instructions
            .put(ExpressionValue::pair(
                ExpressionValue::symbol("bear"),
                ExpressionValue::integer(180),
            ))
            .unwrap();
        instructions.partially_apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result.get_partial_base().unwrap().as_string().unwrap(),
            "my_expression".to_string()
        );

        let list = result.get_partial_value().unwrap();

        assert_eq!(list.list_len().unwrap(), 4);

        assert_eq!(list.get_list_item(0).unwrap().as_integer().unwrap(), 120);
        assert!(list.get_list_item(1).unwrap().is_unit());
        assert_eq!(list.get_list_item(2).unwrap().as_integer().unwrap(), 150);

        let third_item = list.get_list_item(3).unwrap();

        assert_eq!(
            third_item.get_pair_left().unwrap().as_string().unwrap(),
            "bear".to_string()
        );

        assert_eq!(
            third_item.get_pair_right().unwrap().as_integer().unwrap(),
            180
        );
    }

    #[test]
    fn applying_list_with_associations_will_not_replace_unit() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::expression("my_expression"))
            .unwrap();

        instructions.start_list();
        instructions.put(ExpressionValue::integer(120)).unwrap();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.put(ExpressionValue::integer(150)).unwrap();
        instructions.make_list();

        instructions.partially_apply();

        instructions.start_list();
        instructions
            .put(ExpressionValue::pair(
                ExpressionValue::symbol("bear"),
                ExpressionValue::integer(180),
            ))
            .unwrap();
        instructions
            .put(ExpressionValue::pair(
                ExpressionValue::symbol("cat"),
                ExpressionValue::integer(210),
            ))
            .unwrap();
        instructions.make_list();
        instructions.partially_apply();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result.get_partial_base().unwrap().as_string().unwrap(),
            "my_expression".to_string()
        );

        let list = result.get_partial_value().unwrap();

        assert_eq!(list.list_len().unwrap(), 6);

        assert_eq!(list.get_list_item(0).unwrap().as_integer().unwrap(), 120);
        assert!(list.get_list_item(1).unwrap().is_unit());
        assert!(list.get_list_item(2).unwrap().is_unit());
        assert_eq!(list.get_list_item(3).unwrap().as_integer().unwrap(), 150);

        let fourth_item = list.get_list_item(4).unwrap();
        let fifth_item = list.get_list_item(5).unwrap();

        assert_eq!(
            fourth_item.get_pair_left().unwrap().as_string().unwrap(),
            "bear".to_string()
        );

        assert_eq!(
            fourth_item.get_pair_right().unwrap().as_integer().unwrap(),
            180
        );

        assert_eq!(
            fifth_item.get_pair_left().unwrap().as_string().unwrap(),
            "cat".to_string()
        );

        assert_eq!(
            fifth_item.get_pair_right().unwrap().as_integer().unwrap(),
            210
        );
    }
}
