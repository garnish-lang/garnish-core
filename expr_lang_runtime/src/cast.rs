use crate::ExpressionRuntime;
use expr_lang_common::{
    characters_to_bytes, has_end, has_start, has_step, is_end_exclusive, is_start_exclusive,
    read_integer, size_to_bytes, skip_byte_sizes, skip_size, skip_sizes, skip_type,
    skip_type_and_byte_size, skip_type_and_size, skip_type_and_sizes, DataType,
    ExpressionValueConsumer, Result,
};
use std::convert::{TryFrom, TryInto};

impl ExpressionRuntime {
    pub(crate) fn perform_type_cast(&mut self) -> Result {
        // always consume right side
        let right_ref = self.consume_last_ref()?;
        self.set_value_cursor_to_ref_cursor();

        // might not need to consume left
        // in case of no-op cast
        let left_ref = self.last_ref()?;

        let left_type = DataType::try_from(self.data[left_ref])?;
        let right_type = DataType::try_from(self.data[right_ref])?;

        // no-op cast
        if left_type == right_type {
            return Ok(());
        }

        // consume left ref
        self.consume_last_ref()?;
        self.set_value_cursor_to_ref_cursor();

        match (left_type, right_type) {
            (_, DataType::CharacterList) => {
                let s = self.value_at_to_string(left_ref)?;
                self.insert_character_list(&s)
            }
            (DataType::Character, DataType::Integer) => {
                let length = self.read_byte_size_at(skip_type(left_ref))?;

                if length == 0 || length > 4 {
                    return self.insert_unit();
                }

                let codes_start = skip_type_and_byte_size(left_ref);

                let bytes: [u8; 4] = match length {
                    1 => [self.data[codes_start], 0, 0, 0],
                    2 => [self.data[codes_start], self.data[codes_start + 1], 0, 0],
                    3 => [
                        self.data[codes_start],
                        self.data[codes_start + 1],
                        self.data[codes_start + 2],
                        0,
                    ],
                    4 => [
                        self.data[codes_start],
                        self.data[codes_start + 1],
                        self.data[codes_start + 2],
                        self.data[codes_start + 3],
                    ],
                    _ => unreachable!(),
                };

                self.insert_integer(read_integer(&bytes)?)
            }
            (DataType::Integer, DataType::Character) => {
                let num = self.read_integer_at(skip_type(left_ref))?;
                match std::char::from_u32(num as u32) {
                    Some(c) => {
                        let pos = self.value_cursor;

                        let bytes = characters_to_bytes(&c.to_string());
                        self.insert_at_value_cursor(DataType::Character.try_into().unwrap())?;
                        self.insert_at_value_cursor(bytes.len() as u8)?;
                        self.insert_data_value(&bytes)?;

                        self.insert_at_ref_cursor(pos);

                        Ok(())
                    }
                    None => self.insert_unit(),
                }
            }
            (DataType::Integer, DataType::Float) => {
                let num = self.read_integer_at(skip_type(left_ref))?;
                self.insert_float(num as f32)
            }
            (DataType::Integer, DataType::Symbol) => {
                let num = self.read_integer_at(skip_type(left_ref))?;
                let value = match self.symbol_table.iter().find(|(_k, v)| **v == num as usize) {
                    None => {
                        return self.insert_unit();
                    }
                    Some((_k, v)) => *v,
                };

                self.insert_symbol_value(value)
            }
            (DataType::Float, DataType::Integer) => {
                let num = self.read_float_at(skip_type(left_ref))?;
                self.insert_integer(num as i32)
            }
            (DataType::Symbol, DataType::Integer) => {
                let symbol_value = self.read_size_at(skip_type(left_ref))?;
                self.insert_integer(symbol_value as i32)
            }
            (DataType::CharacterList, DataType::Integer) => {
                let s = self.read_character_list(left_ref)?;

                match s.parse::<i32>() {
                    Err(_) => self.insert_unit(),
                    Ok(v) => self.insert_integer(v),
                }
            }
            (DataType::CharacterList, DataType::Float) => {
                let s = self.read_character_list(left_ref)?;

                match s.parse::<f32>() {
                    Err(_) => self.insert_unit(),
                    Ok(v) => self.insert_float(v),
                }
            }
            (DataType::CharacterList, DataType::Symbol) => {
                let s = self.read_character_list(left_ref)?;

                let value = match self.symbol_table.get(&s) {
                    None => return self.insert_unit(),
                    Some(v) => *v,
                };

                self.insert_symbol_value(value)
            }
            _ => self.insert_unit(),
        }
    }

    fn get_string_at_if(&self, condition: bool, index: usize) -> Result<(usize, String)> {
        Ok(if condition {
            (
                skip_size(index),
                self.value_at_to_string(self.read_size_at(index)?)?,
            )
        } else {
            (index, "()".to_string())
        })
    }

    fn value_at_to_string(&self, index: usize) -> Result<String> {
        Ok(match DataType::try_from(self.data[index])? {
            DataType::Unit => "()".to_string(),
            DataType::Integer => {
                let num = self.read_integer_at(skip_type(index))?;
                num.to_string()
            }
            DataType::Float => {
                let num = self.read_float_at(skip_type(index))?;
                num.to_string()
            }
            DataType::Character => {
                let length = self.read_byte_size_at(skip_type(index))? as usize;
                let codes_start = skip_type_and_byte_size(index);

                String::from_utf8_lossy(&self.data[codes_start..(codes_start + length)]).to_string()
            }
            DataType::CharacterList => self.read_character_list(index)?,
            DataType::Symbol => {
                let symbol_value = self.read_size_at(skip_type(index))?;
                let s = match self.symbol_table.iter().find(|(_k, v)| **v == symbol_value) {
                    None => String::new(),
                    Some((k, _v)) => k.clone(),
                };

                format!(":{}", s)
            }
            DataType::Range => {
                let flags = self.read_byte_size_at(skip_type(index))?;

                let r = skip_type_and_byte_size(index);

                let (r, start_str) = self.get_string_at_if(has_start(flags), r)?;
                let (r, end_str) = self.get_string_at_if(has_end(flags), r)?;

                let step_str = if has_step(flags) {
                    format!(" ~ {}", self.value_at_to_string(self.read_size_at(r)?)?)
                } else {
                    "".to_string()
                };

                let ex_start_str = if is_start_exclusive(flags) { ">" } else { "" };
                let ex_end_str = if is_end_exclusive(flags) { "<" } else { "" };

                format!(
                    "{}{}..{}{}{}",
                    start_str, ex_start_str, ex_end_str, end_str, step_str
                )
            }
            DataType::Pair => {
                let left = self.read_size_at(skip_type(index))?;
                let right = self.read_size_at(skip_type_and_size(index))?;

                let left_str = self.value_at_to_string(left)?;
                let right_str = self.value_at_to_string(right)?;

                format!("{} = {}", left_str, right_str)
            }
            DataType::List => {
                let length = self.read_size_at(skip_type(index))?;
                let items_start = skip_type_and_sizes(index, 2);

                let mut item_strings: Vec<String> = vec![];

                for i in 0..length {
                    let item_start = skip_sizes(items_start, i);
                    let item_ref = self.read_size_at(item_start)?;
                    item_strings.push(self.value_at_to_string(item_ref)?);
                }

                format!("({})", item_strings.join(", "))
            }
            _ => String::new(),
        })
    }

    fn insert_character_list(&mut self, s: &str) -> Result {
        let mut refs = vec![];

        for c in s.chars() {
            let pos = self.value_cursor;

            // numbers will only consist of ascii characters
            // so assume length of 1 for each
            self.insert_at_value_cursor(DataType::Character.try_into().unwrap())?;
            self.insert_at_value_cursor(c.len_utf8() as u8)?;
            self.insert_data_value(&characters_to_bytes(&c.to_string()))?;

            refs.push(pos);
        }

        let pos = self.value_cursor;

        self.insert_at_value_cursor(DataType::CharacterList.try_into().unwrap())?;
        self.insert_data_value(&size_to_bytes(refs.len()))?;

        for r in refs {
            self.insert_data_value(&size_to_bytes(r))?;
        }

        self.insert_at_ref_cursor(pos);

        Ok(())
    }

    fn read_character_list(&self, start: usize) -> Result<String> {
        let length = self.read_size_at(skip_type(start))?;
        let characters_start = skip_type_and_size(start);

        let mut data: Vec<u8> = vec![];

        for i in 0..length {
            let char_ref_start = skip_sizes(characters_start, i);
            let char_start = self.read_size_at(char_ref_start)?;
            let char_length = self.read_byte_size_at(skip_type(char_start))? as usize;
            let code_points_start = skip_type_and_byte_size(char_start);

            for j in 0..char_length {
                let code_point_index = skip_byte_sizes(code_points_start, j);
                let code_point = self.read_byte_size_at(code_point_index)?;

                data.push(code_point);
            }
        }

        Ok(String::from_utf8_lossy(&data).to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::ExpressionRuntime;
    use expr_lang_common::{DataType, ExpressionValue};
    use expr_lang_instruction_set_builder::InstructionSetBuilder;

    #[test]
    fn cast_unit_to_string() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions
            .put(ExpressionValue::character_list("".to_string()))
            .unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_type().unwrap(), DataType::CharacterList);
        assert_eq!(result.as_string().unwrap(), "()".to_string());
    }

    #[test]
    fn cast_unit_to_unit() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit());
    }

    #[test]
    fn cast_integer_to_integer() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(12345)).unwrap();
        instructions.put(ExpressionValue::integer(0)).unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 12345);
    }

    #[test]
    fn cast_integer_to_string() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(12345)).unwrap();
        instructions
            .put(ExpressionValue::character_list("".to_string()))
            .unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), 12345.to_string());
    }

    #[test]
    fn cast_integer_to_float() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(12345)).unwrap();
        instructions.put(ExpressionValue::float(0.0)).unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_float().unwrap(), 12345.0);
    }

    #[test]
    fn cast_integer_to_character() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::integer('z' as u32 as i32))
            .unwrap();
        instructions
            .put(ExpressionValue::character("".to_string()))
            .unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_type().unwrap(), DataType::Character);
        assert_eq!(result.as_char().unwrap(), 'z');
    }

    #[test]
    fn cast_integer_to_character_invalid_code_point() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::integer(std::i32::MAX))
            .unwrap();
        instructions
            .put(ExpressionValue::character("".to_string()))
            .unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit());
    }

    #[test]
    fn cast_integer_to_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(1)).unwrap();
        instructions
            .put(ExpressionValue::symbol("".to_string()))
            .unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_type().unwrap(), DataType::Symbol);
        assert_eq!(result.as_string().unwrap(), "main".to_string());
    }

    #[test]
    fn cast_integer_to_symbol_that_does_not_exist() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(1000000)).unwrap();
        instructions
            .put(ExpressionValue::symbol("".to_string()))
            .unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit());
    }

    #[test]
    fn cast_float_to_string() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::float(3.14)).unwrap();
        instructions
            .put(ExpressionValue::character_list("".to_string()))
            .unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), 3.14.to_string());
    }

    #[test]
    fn cast_float_to_integer() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::float(12345.5)).unwrap();
        instructions.put(ExpressionValue::integer(12345)).unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 12345);
    }

    #[test]
    fn cast_float_to_float() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.put(ExpressionValue::float(12345.5)).unwrap();
        instructions.put(ExpressionValue::float(0.0)).unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_float().unwrap(), 12345.5);
    }

    #[test]
    fn cast_symbol_to_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("".to_string()))
            .unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_type().unwrap(), DataType::Symbol);
        assert_eq!(result.as_string().unwrap(), "cats".to_string());
    }

    #[test]
    fn cast_symbol_to_integer() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_type().unwrap(), DataType::Integer);
        assert_eq!(result.as_integer().unwrap(), 2);
    }

    #[test]
    fn cast_symbol_to_string() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("".to_string()))
            .unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_type().unwrap(), DataType::CharacterList);
        assert_eq!(result.as_string().unwrap(), ":cats");
    }

    #[test]
    fn cast_symbol_to_string_missing_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("".to_string()))
            .unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        expression_runtime.symbol_table.remove("cats");

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), ":".to_string());
    }

    #[test]
    fn cast_character_to_character() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::character("c".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("".to_string()))
            .unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_type().unwrap(), DataType::Character);
        assert_eq!(result.as_char().unwrap(), 'c');
    }

    #[test]
    fn cast_character_to_character_unicode() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::character("🐼".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("".to_string()))
            .unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_type().unwrap(), DataType::Character);
        assert_eq!(result.as_char().unwrap(), '🐼');
    }

    #[test]
    fn cast_character_to_integer() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::character("c".to_string()))
            .unwrap();
        instructions.put(ExpressionValue::integer(0)).unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_type().unwrap(), DataType::Integer);
        assert_eq!(result.as_integer().unwrap(), 'c' as u32 as i32);
    }

    #[test]
    fn cast_character_to_integer_invalid_char() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::character("ö̲".to_string()))
            .unwrap();
        instructions.put(ExpressionValue::integer(0)).unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit());
    }

    #[test]
    fn cast_character_to_integer_empty_char() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::character("".to_string()))
            .unwrap();
        instructions.put(ExpressionValue::integer(0)).unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit());
    }

    #[test]
    fn cast_character_to_character_list() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::character("c".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("".to_string()))
            .unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_type().unwrap(), DataType::CharacterList);
        assert_eq!(result.as_string().unwrap(), "c".to_string());
    }

    #[test]
    fn cast_character_list_to_character_list() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::character_list("main".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("".to_string()))
            .unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_type().unwrap(), DataType::CharacterList);
        assert_eq!(result.as_string().unwrap(), "main".to_string());
    }

    #[test]
    fn cast_character_list_to_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::character_list("main".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("".to_string()))
            .unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_type().unwrap(), DataType::Symbol);
        assert_eq!(result.as_string().unwrap(), "main".to_string());
    }

    #[test]
    fn cast_character_list_to_symbol_than_does_not_exist() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::character_list("cats".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("".to_string()))
            .unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit());
    }

    #[test]
    fn cast_character_list_to_integer() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::character_list("10".to_string()))
            .unwrap();
        instructions.put(ExpressionValue::integer(0)).unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_type().unwrap(), DataType::Integer);
        assert_eq!(result.as_integer().unwrap(), 10);
    }

    #[test]
    fn cast_character_list_to_integer_when_not_a_number() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::character_list("cats".to_string()))
            .unwrap();
        instructions.put(ExpressionValue::integer(0)).unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit());
    }

    #[test]
    fn cast_character_list_to_float() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::character_list("3.14".to_string()))
            .unwrap();
        instructions.put(ExpressionValue::float(0.0)).unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.get_type().unwrap(), DataType::Float);
        assert_eq!(result.as_float().unwrap(), "3.14".parse().unwrap());
    }

    #[test]
    fn cast_character_list_to_float_when_not_a_number() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::character_list("cats".to_string()))
            .unwrap();
        instructions.put(ExpressionValue::float(0.0)).unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit());
    }

    #[test]
    fn cast_pair_to_pair() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::pair(
                ExpressionValue::symbol("cats"),
                ExpressionValue::integer(10),
            ))
            .unwrap();
        instructions
            .put(ExpressionValue::pair(
                ExpressionValue::unit(),
                ExpressionValue::unit(),
            ))
            .unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_pair());
        assert_eq!(
            result.get_pair_left().unwrap().as_string().unwrap(),
            "cats".to_string()
        );
        assert_eq!(result.get_pair_right().unwrap().as_integer().unwrap(), 10);
    }

    #[test]
    fn cast_pair_to_character_list() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::pair(
                ExpressionValue::symbol("cats"),
                ExpressionValue::integer(10),
            ))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("".to_string()))
            .unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), ":cats = 10");
    }

    #[test]
    fn cast_range_to_range() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::integer_range(Some(10), Some(20)))
            .unwrap();
        instructions
            .put(ExpressionValue::integer_range(None, None))
            .unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_range());
        assert_eq!(result.get_range_min().unwrap().as_integer().unwrap(), 10);
        assert_eq!(result.get_range_max().unwrap().as_integer().unwrap(), 20);
    }

    #[test]
    fn cast_integer_range_to_character_list() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::integer_range(Some(10), Some(20)))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("".to_string()))
            .unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "10..20")
    }

    #[test]
    fn cast_open_range_to_character_list() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::integer_range(None, None))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("".to_string()))
            .unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "()..()")
    }

    #[test]
    fn cast_exclusive_range_to_character_list() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(
                ExpressionValue::integer_range(Some(10), Some(20))
                    .exclude_end()
                    .exclude_start(),
            )
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("".to_string()))
            .unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "10>..<20")
    }

    #[test]
    fn cast_range_with_step_to_character_list() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(
                ExpressionValue::integer_range(Some(10), Some(20))
                    .with_step(ExpressionValue::integer(2)),
            )
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("".to_string()))
            .unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "10..20 ~ 2")
    }

    #[test]
    fn cast_float_range_to_character_list() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::float_range(Some(1.5), Some(10.5)))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("".to_string()))
            .unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "1.5..10.5")
    }

    #[test]
    fn cast_character_range_to_character_list() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::char_range(Some('a'), Some('z')))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("".to_string()))
            .unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "a..z")
    }

    #[test]
    fn cast_list_to_list() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(ExpressionValue::list().add(ExpressionValue::integer(10)))
            .unwrap();
        instructions.put(ExpressionValue::list()).unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 10)
    }

    #[test]
    fn cast_list_to_character_list() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(
                ExpressionValue::list()
                    .add(ExpressionValue::integer(10))
                    .add(ExpressionValue::integer(20))
                    .add(ExpressionValue::integer(30)),
            )
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("".to_string()))
            .unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "(10, 20, 30)".to_string());
    }

    #[test]
    fn cast_list_to_character_list_all_types_nested() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions
            .put(
                ExpressionValue::list()
                    .add(ExpressionValue::unit())
                    .add(ExpressionValue::character_list("cats".to_string()))
                    .add(ExpressionValue::character("🐼".to_string()))
                    .add(ExpressionValue::integer(20))
                    .add(ExpressionValue::float(3.14))
                    .add(ExpressionValue::symbol("my_symbol"))
                    .add(
                        ExpressionValue::integer_range(Some(10), Some(20))
                            .exclude_end()
                            .with_step(ExpressionValue::integer(2)),
                    )
                    .add(ExpressionValue::pair(
                        ExpressionValue::symbol("dog"),
                        ExpressionValue::character("D".to_string()),
                    ))
                    .add(
                        ExpressionValue::list()
                            .add(ExpressionValue::integer(10))
                            .add(ExpressionValue::integer(20))
                            .add(ExpressionValue::integer(30)),
                    ),
            )
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("".to_string()))
            .unwrap();
        instructions.perform_type_cast();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result.as_string().unwrap(),
            "((), cats, 🐼, 20, 3.14, :my_symbol, 10..<20 ~ 2, :dog = D, (10, 20, 30))".to_string()
        );
    }
}
