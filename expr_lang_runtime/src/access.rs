use std::convert::TryFrom;
use std::ops::Range;

use expr_lang_common::{
    get_value_with_hash, has_end, has_start, has_step, hash_of_character_list, is_end_exclusive,
    is_start_exclusive, skip_size, skip_sizes, skip_type, skip_type_and_2_sizes,
    skip_type_and_byte_size, skip_type_and_bytes_and_sizes, skip_type_and_size,
    skip_type_and_sizes, DataType, Result,
};

use super::ExpressionRuntime;

#[derive(Copy, Clone)]
enum Finality {
    Finite(usize),
    Infinite,
}

impl ExpressionRuntime {
    pub(crate) fn perform_access(&mut self) -> Result {
        // ref_cursor is pos for new insert
        // plus 1 to get last (right)
        // plus 2 to get 2nd last (left)
        let (left_ref, right_ref) = self.consume_last_two_refs()?;

        let right_type = DataType::try_from(self.data[right_ref])?;

        // can only access list or associative list
        match DataType::try_from(self.data[left_ref])? {
            DataType::Pair => match right_type {
                DataType::Symbol => {
                    let symbol_value = self.read_size_at(skip_type(right_ref))?;
                    if symbol_value == self.left_value {
                        let r = self.read_size_at(skip_type(left_ref))?;
                        self.insert_reference_value(r)?;
                    } else if symbol_value == self.right_value {
                        let r = self.read_size_at(skip_type_and_size(left_ref))?;
                        self.insert_reference_value(r)?;
                    } else {
                        self.insert_unit()?;
                    }
                }
                _ => self.insert_unit()?,
            },
            DataType::Partial => match right_type {
                DataType::Symbol => {
                    let symbol_value = self.read_size_at(skip_type(right_ref))?;
                    if symbol_value == self.base_value {
                        let r = self.read_size_at(skip_type(left_ref))?;
                        self.insert_reference_value(r)?;
                    } else if symbol_value == self.value_value {
                        let r = self.read_size_at(skip_type_and_size(left_ref))?;
                        self.insert_reference_value(r)?;
                    } else {
                        self.insert_unit()?;
                    }
                }
                _ => self.insert_unit()?,
            },
            DataType::CharacterList => {
                let length = self.read_size_at(skip_type(left_ref))?;

                match right_type {
                    DataType::Symbol => {
                        let symbol_value = self.read_size_at(skip_type(right_ref))?;
                        if symbol_value == self.length_value {
                            self.insert_integer(length as i32)?
                        }
                    }
                    DataType::Integer => self.insert_item_from_index(
                        right_ref,
                        length,
                        skip_type_and_size(left_ref),
                    )?,
                    DataType::Range => self.insert_slice_if_valid(left_ref, right_ref)?,
                    _ => self.insert_unit()?,
                }
            }
            DataType::Range => {
                let flags = self.read_byte_size_at(skip_type(left_ref))?;

                match right_type {
                    DataType::Symbol => {
                        let symbol_value = self.read_size_at(skip_type(right_ref))?;

                        if symbol_value == self.is_start_exclusive_value {
                            self.insert_symbol_value_from_condition(is_start_exclusive(flags))?;
                        } else if symbol_value == self.is_end_exclusive_value {
                            self.insert_symbol_value_from_condition(is_end_exclusive(flags))?;
                        } else if symbol_value == self.is_start_open_value {
                            self.insert_symbol_value_from_condition(!has_start(flags))?;
                        } else if symbol_value == self.is_end_open_value {
                            self.insert_symbol_value_from_condition(!has_end(flags))?;
                        } else if symbol_value == self.length_value {
                            self.insert_length_of_range(left_ref)?
                        } else if symbol_value == self.start_value {
                            if !has_start(flags) {
                                self.insert_unit()?;
                            } else {
                                let start_ref =
                                    self.read_size_at(skip_type_and_byte_size(left_ref))?;
                                self.insert_reference_value(start_ref)?;
                            }
                        } else if symbol_value == self.end_value {
                            if !has_end(flags) {
                                self.insert_unit()?;
                            } else {
                                let offset = if has_start(flags) {
                                    skip_type_and_bytes_and_sizes(left_ref, 1, 1)
                                } else {
                                    skip_type_and_byte_size(left_ref)
                                };

                                let r = self.read_size_at(offset)?;
                                self.insert_reference_value(r)?;
                            }
                        } else if symbol_value == self.step_value {
                            if !has_step(flags) {
                                self.insert_unit()?;
                            } else {
                                let mut offset = skip_type_and_byte_size(left_ref);

                                if has_start(flags) {
                                    offset = skip_size(offset);
                                }

                                if has_end(flags) {
                                    offset = skip_size(offset);
                                };

                                let r = self.read_size_at(offset)?;
                                self.insert_reference_value(r)?;
                            }
                        }
                    }
                    _ => self.insert_unit()?,
                }
            }
            DataType::List => {
                let length = self.read_size_at(skip_type(left_ref))?;
                let key_count = self.read_size_at(skip_type_and_size(left_ref))?;
                let key_area_start = skip_type_and_sizes(left_ref, 2 + length);
                let key_area_range = key_area_start..skip_sizes(key_area_start, key_count);

                match right_type {
                    DataType::Symbol => {
                        let right_value = self.read_size_at(skip_type(right_ref))?;
                        if right_value == self.length_value {
                            self.insert_integer(length as i32)?;
                        } else if right_value == self.key_count_value {
                            self.insert_integer(key_count as i32)?;
                        } else {
                            self.insert_value_from_key(right_value, key_count, &key_area_range)?
                        }
                    }
                    DataType::CharacterList => {
                        let hash = hash_of_character_list(right_ref, &self.data);
                        self.insert_value_from_key(hash, key_count, &key_area_range)?
                    }
                    DataType::Integer => self.insert_item_from_index(
                        right_ref,
                        length,
                        skip_type_and_2_sizes(left_ref),
                    )?,
                    DataType::Range => self.insert_slice_if_valid(left_ref, right_ref)?,
                    _ => self.insert_unit()?,
                }
            }
            DataType::Link => {
                match right_type {
                    DataType::Symbol => {
                        let mut next_ref = self.read_size_at(skip_type(left_ref))?;
                        let mut list_count = 0;
                        let mut character_list_count = 0;
                        // easier to assume CharacterList at start
                        let mut count_type = DataType::CharacterList;

                        // A Link needs at least 2 items to be created
                        // so this loop will always run at least once
                        while self.read_size_at(skip_type_and_sizes(next_ref, 2))? != 0 {
                            let value_ref = self.read_size_at(skip_type_and_size(next_ref))?;
                            let (list_length, character_list_length, length_type) =
                                self.length_of_link_item(value_ref)?;

                            list_count += match list_length {
                                Some(finality) => match finality {
                                    Finality::Finite(length) => length,
                                    Finality::Infinite => {
                                        return self.insert_symbol_value(self.infinite_value)
                                    }
                                },
                                None => return self.insert_unit(),
                            };

                            character_list_count += match character_list_length {
                                Some(finality) => match finality {
                                    Finality::Finite(length) => length,
                                    Finality::Infinite => {
                                        return self.insert_symbol_value(self.infinite_value)
                                    }
                                },
                                None => return self.insert_unit(),
                            };

                            if length_type != DataType::CharacterList {
                                count_type = DataType::List;
                            }

                            next_ref = self.read_size_at(skip_type_and_sizes(next_ref, 2))?;
                        }

                        let value_ref = self.read_size_at(skip_type_and_size(next_ref))?;
                        let (list_length, character_list_length, length_type) =
                            self.length_of_link_item(value_ref)?;

                        list_count += match list_length {
                            Some(finality) => match finality {
                                Finality::Finite(length) => length,
                                Finality::Infinite => {
                                    return self.insert_symbol_value(self.infinite_value)
                                }
                            },
                            None => return self.insert_unit(),
                        };

                        character_list_count += match character_list_length {
                            Some(finality) => match finality {
                                Finality::Finite(length) => length,
                                Finality::Infinite => {
                                    return self.insert_symbol_value(self.infinite_value)
                                }
                            },
                            None => return self.insert_unit(),
                        };

                        if length_type != DataType::CharacterList {
                            count_type = DataType::List;
                        }

                        match count_type {
                            DataType::CharacterList => {
                                self.insert_integer(character_list_count as i32)?
                            }
                            _ => self.insert_integer(list_count as i32)?,
                        }
                    }
                    DataType::Range => self.insert_slice_if_valid(left_ref, right_ref)?,
                    _ => self.insert_unit()?,
                }
            }
            DataType::Slice => {
                match right_type {
                    DataType::Symbol => {
                        let range_ref = self.read_size_at(skip_type_and_size(left_ref))?;
                        match self.length_of_range_at(range_ref)? {
                            Some(finality) => match finality {
                                Finality::Finite(length) => self.insert_integer(length as i32)?,
                                Finality::Infinite => {
                                    // slices can't be infinite in length
                                    // if range is infinite
                                    // length of slice will use the connected list's length to fill in open parts

                                    // ❗️ assuming right is list for now
                                    let source_ref = self.read_size_at(skip_type(left_ref))?;
                                    let list_length = self.read_size_at(skip_type(source_ref))?;
                                    let flags = self.read_byte_size_at(skip_type(range_ref))?;

                                    let (mut length, sizes) =
                                        match (has_start(flags), has_end(flags)) {
                                            (true, true) => unreachable!(), // this case should resolve to Finite in length_of_range_at
                                            (true, false) => {
                                                let start_ref = self.read_size_at(
                                                    skip_type_and_byte_size(range_ref),
                                                )?;
                                                let start =
                                                    self.read_integer_at(skip_type(start_ref))?;
                                                (list_length as i32 - start, 1)
                                            }
                                            (false, true) => {
                                                let end_ref = self.read_size_at(
                                                    skip_type_and_byte_size(range_ref),
                                                )?;
                                                let end =
                                                    self.read_integer_at(skip_type(end_ref))?;
                                                // just subtracting gives us length as if excluding 1
                                                // so add back
                                                (
                                                    list_length as i32 - (list_length as i32 - end)
                                                        + 1,
                                                    1,
                                                )
                                            }
                                            (false, false) => {
                                                // if no start or end
                                                // we use the length of list
                                                (list_length as i32, 0)
                                            }
                                        };

                                    if has_step(flags) {
                                        // step is 2 sizes away since we already checked for both start and end
                                        let step_ref = self.read_size_at(
                                            skip_type_and_bytes_and_sizes(range_ref, 1, sizes),
                                        )?;
                                        let step = self.read_integer_at(skip_type(step_ref))?;

                                        if step == 0 {
                                            return self.insert_symbol_value(self.infinite_value);
                                        }

                                        // new length will be how every many steps will be made
                                        length = (length as f64 / step as f64).ceil() as i32;
                                    }

                                    self.insert_integer(length)?
                                }
                            },
                            None => self.insert_unit()?,
                        }
                    }
                    DataType::Range => self.insert_slice_if_valid(left_ref, right_ref)?,
                    _ => self.insert_unit()?,
                }
            }
            _ => self.insert_unit()?,
        }

        Ok(())
    }

    fn length_of_link_item(
        &self,
        item_ref: usize,
    ) -> Result<(Option<Finality>, Option<Finality>, DataType)> {
        Ok(match DataType::try_from(self.data[item_ref])? {
            DataType::List => {
                // all list items are included in link's length
                let length = self.read_size_at(skip_type(item_ref))?;
                (
                    Some(Finality::Finite(length)),
                    Some(Finality::Finite(length)),
                    DataType::List,
                )
            }
            DataType::CharacterList => {
                // all list items are included in link's length
                let length = self.read_size_at(skip_type(item_ref))?;
                (
                    Some(Finality::Finite(1)),
                    Some(Finality::Finite(length)),
                    DataType::CharacterList,
                )
            }
            DataType::Slice => {
                // all slice items are included in link's length
                let range_ref = self.read_size_at(skip_type_and_size(item_ref))?;
                let length = self.length_of_range_at(range_ref)?;

                // check source for character like type
                let mut source_ref = self.read_size_at(skip_type(item_ref))?;

                // drill down slice objects to find type of slice
                while DataType::try_from(self.data[source_ref])? == DataType::Slice {
                    source_ref = self.read_size_at(skip_type(source_ref))?;
                }

                let length_type = match DataType::try_from(self.data[source_ref])? {
                    DataType::CharacterList | DataType::Character => DataType::CharacterList,
                    _ => DataType::List,
                };

                (length, length, length_type)
            }
            DataType::Character => (
                Some(Finality::Finite(1)),
                Some(Finality::Finite(1)),
                DataType::CharacterList,
            ),
            _ => (
                Some(Finality::Finite(1)),
                Some(Finality::Finite(1)),
                DataType::List,
            ),
        })
    }

    fn insert_length_of_range(&mut self, range_ref: usize) -> Result {
        match self.length_of_range_at(range_ref)? {
            Some(finality) => match finality {
                Finality::Finite(length) => self.insert_integer(length as i32),
                Finality::Infinite => self.insert_symbol_value(self.infinite_value),
            },
            None => self.insert_unit(),
        }
    }

    fn length_of_range_at(&self, range_ref: usize) -> Result<Option<Finality>> {
        let flags = self.read_byte_size_at(skip_type(range_ref))?;

        Ok(match (has_start(flags), has_end(flags)) {
            (true, true) => {
                let start_ref = self.read_size_at(skip_type_and_byte_size(range_ref))?;
                let end_ref = self.read_size_at(skip_type_and_bytes_and_sizes(range_ref, 1, 1))?;

                match (
                    DataType::try_from(self.data[start_ref])?,
                    DataType::try_from(self.data[end_ref])?,
                ) {
                    (DataType::Integer, DataType::Integer) => {
                        let left = self.read_integer_at(skip_type(start_ref))?;
                        let right = self.read_integer_at(skip_type(end_ref))?;

                        let mut length = match (is_start_exclusive(flags), is_end_exclusive(flags))
                        {
                            (true, true) => {
                                // subtraction gives us range as if it excluded one already
                                // subtract an additional one if excluding both
                                let length = (right - left).abs();
                                if length > 0 {
                                    length - 1
                                } else {
                                    length
                                }
                            }
                            (true, false) | (false, true) => (right - left).abs(),
                            // subtraction gives us range as if it excluded one already
                            // add back one if not excluding any
                            (false, false) => (right - left).abs() + 1,
                        };

                        if has_step(flags) {
                            // step is 2 sizes away since we already checked for both start and end
                            let step_ref =
                                self.read_size_at(skip_type_and_bytes_and_sizes(range_ref, 1, 2))?;
                            let step = self.read_integer_at(skip_type(step_ref))?;

                            if step == 0 {
                                return Ok(Some(Finality::Infinite));
                            }

                            // new length will be how every many steps will be made
                            length = (length as f64 / step as f64).ceil() as i32;
                        }

                        Some(Finality::Finite(length as usize))
                    }
                    _ => None,
                }
            }
            _ => Some(Finality::Infinite),
        })
    }

    fn insert_slice_if_valid(&mut self, left_ref: usize, range_ref: usize) -> Result {
        let range_flags = self.read_byte_size_at(skip_type(range_ref))?;

        let mut range_value = skip_type_and_byte_size(range_ref);

        let start_is_valid_type = if has_start(range_flags) {
            let start_ref = self.read_size_at(range_value)?;
            range_value = skip_size(range_value);

            DataType::try_from(self.data[start_ref])? == DataType::Integer
        } else {
            true
        };

        let end_is_valid_type = if has_end(range_flags) {
            let end_ref = self.read_size_at(range_value)?;
            //                            range_value = skip_size(range_value)

            DataType::try_from(self.data[end_ref])? == DataType::Integer
        } else {
            true
        };

        if start_is_valid_type && end_is_valid_type {
            self.insert_slice_value(left_ref, range_ref)
        } else {
            self.insert_unit()
        }
    }

    fn insert_item_from_index(
        &mut self,
        index_ref: usize,
        length: usize,
        item_start: usize,
    ) -> Result {
        let index = self.read_integer_at(skip_type(index_ref))?;
        match if index < 0 {
            // reverse indexing
            if -index > length as i32 {
                None
            } else {
                Some(length as i32 + index)
            }
        } else {
            // regular indexing
            if index >= length as i32 {
                None
            } else {
                Some(index)
            }
        } {
            Some(index) => {
                let item_ref = skip_sizes(item_start, index as usize);
                let item_value = self.read_size_at(item_ref)?;
                self.insert_reference_value(item_value)?
            }
            None => self.insert_unit()?,
        }

        Ok(())
    }

    fn insert_value_from_key(
        &mut self,
        key_value_target: usize,
        key_count: usize,
        key_area_range: &Range<usize>,
    ) -> Result {
        match get_value_with_hash(key_value_target, &self.data, key_count, key_area_range)? {
            Some(item) => {
                let pair_value_ref = self.read_size_at(skip_type_and_size(item))?;
                self.insert_reference_value(pair_value_ref)
            }
            None => self.insert_unit(),
        }
    }
}

#[cfg(test)]
mod tests {
    use expr_lang_common::{DataType, ExpressionValue};
    use expr_lang_instruction_set_builder::InstructionSetBuilder;

    use crate::runtime::ExpressionRuntime;

    #[test]
    fn associative_list_with_integer() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.make_list();

        instructions.put(ExpressionValue::integer(2)).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 30);
    }
    #[test]
    fn associative_list_with_negative_integer() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.make_list();

        instructions.put(ExpressionValue::integer(-3)).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 10);
    }

    #[test]
    fn associative_list_with_integer_out_of_bounds() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.make_list();

        instructions.put(ExpressionValue::integer(4)).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit());
    }

    #[test]
    fn associative_list_with_negative_integer_out_of_bounds() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.make_list();

        instructions.put(ExpressionValue::integer(-4)).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit());
    }

    #[test]
    fn associative_list_with_symbol() {
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

        instructions.put(ExpressionValue::symbol("cat")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 20);
    }

    #[test]
    fn associative_list_with_symbol_on_nonexistent_item() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.start_list();

        instructions.put(ExpressionValue::symbol("bear")).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.make_pair();

        instructions.make_list();

        instructions.put(ExpressionValue::symbol("cat")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit());
    }

    #[test]
    fn associative_list_with_string() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.start_list();

        instructions
            .put(ExpressionValue::character_list("bear".to_string()))
            .unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.make_pair();

        instructions
            .put(ExpressionValue::character_list("cat".to_string()))
            .unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.make_pair();

        instructions
            .put(ExpressionValue::character_list("dog".to_string()))
            .unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.make_pair();

        instructions.make_list();

        instructions
            .put(ExpressionValue::character_list("cat".to_string()))
            .unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 20);
    }

    #[test]
    fn associative_list_with_string_with_nonexistent_items() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.start_list();

        instructions
            .put(ExpressionValue::character_list("bear".to_string()))
            .unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.make_pair();

        instructions.make_list();

        instructions
            .put(ExpressionValue::character_list("cat".to_string()))
            .unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit());
    }

    #[test]
    fn make_slice_of_list_with_integer_range() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.make_list();

        instructions
            .put(ExpressionValue::integer_range(Some(0), Some(1)))
            .unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.get_slice_source().unwrap().is_list());
        assert!(result.get_slice_range().unwrap().is_range());
    }

    #[test]
    fn make_slice_of_list_with_non_integer_range() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.make_list();

        instructions
            .put(ExpressionValue::float_range(Some(0.0), Some(1.0)))
            .unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit());
    }

    #[test]
    fn character_list_with_integer() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list(
                "The quick brown fox.".to_string(),
            ))
            .unwrap();
        instructions.put(ExpressionValue::integer(4)).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_char().unwrap(), 'q');
    }

    #[test]
    fn character_list_with_negative_integer() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list(
                "The quick brown fox.".to_string(),
            ))
            .unwrap();
        instructions.put(ExpressionValue::integer(-20)).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_char().unwrap(), 'T');
    }

    #[test]
    fn character_list_with_integer_out_of_bounds() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list(
                "The quick brown fox.".to_string(),
            ))
            .unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit());
    }

    #[test]
    fn character_list_with_negative_integer_out_of_bounds() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list(
                "The quick brown fox.".to_string(),
            ))
            .unwrap();
        instructions.put(ExpressionValue::integer(-21)).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit());
    }

    #[test]
    fn character_list_integer_range() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list(
                "The quick brown fox.".to_string(),
            ))
            .unwrap();
        instructions
            .put(ExpressionValue::integer_range(Some(1), Some(10)))
            .unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result.get_slice_source().unwrap().get_type().unwrap(),
            DataType::CharacterList
        );
        assert!(result.get_slice_range().unwrap().is_range());
    }

    #[test]
    fn character_list_non_integer_range() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list(
                "The quick brown fox.".to_string(),
            ))
            .unwrap();
        instructions
            .put(ExpressionValue::float_range(Some(1.0), Some(10.0)))
            .unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit())
    }

    #[test]
    fn character_list_length() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list(
                "The quick brown fox.".to_string(),
            ))
            .unwrap();
        instructions.put(ExpressionValue::symbol("length")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 20)
    }

    #[test]
    fn pair_left() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::pair(
                ExpressionValue::symbol("dog"),
                ExpressionValue::integer(30),
            ))
            .unwrap();
        instructions.put(ExpressionValue::symbol("left")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "dog".to_string())
    }

    #[test]
    fn pair_right() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::pair(
                ExpressionValue::symbol("dog"),
                ExpressionValue::integer(30),
            ))
            .unwrap();
        instructions.put(ExpressionValue::symbol("right")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 30)
    }

    #[test]
    fn partial_base() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::partial_expression(
                "my_expression",
                ExpressionValue::integer(30),
            ))
            .unwrap();
        instructions.put(ExpressionValue::symbol("base")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_string().unwrap(), "my_expression")
    }

    #[test]
    fn partial_value() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::partial_expression(
                "my_expression",
                ExpressionValue::integer(100),
            ))
            .unwrap();
        instructions.put(ExpressionValue::symbol("value")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 100)
    }

    #[test]
    fn list_length() {
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
        instructions.put(ExpressionValue::symbol("length")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 3)
    }

    #[test]
    fn list_key_count() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(
                ExpressionValue::list()
                    .add(ExpressionValue::pair(
                        ExpressionValue::symbol("bear"),
                        ExpressionValue::integer(10),
                    ))
                    .add(ExpressionValue::pair(
                        ExpressionValue::symbol("cat"),
                        ExpressionValue::integer(20),
                    ))
                    .add(ExpressionValue::pair(
                        ExpressionValue::symbol("dog"),
                        ExpressionValue::integer(30),
                    )),
            )
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("key_count"))
            .unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 3)
    }
}

#[cfg(test)]
mod range_tests {
    use expr_lang_common::ExpressionValue;
    use expr_lang_instruction_set_builder::InstructionSetBuilder;

    use crate::ExpressionRuntime;

    #[test]
    fn range_inclusive_length() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::integer_range(Some(15), Some(23)))
            .unwrap();
        instructions.put(ExpressionValue::symbol("length")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 9)
    }

    #[test]
    fn range_length_no_start() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::integer_range(None, Some(23)))
            .unwrap();
        instructions.put(ExpressionValue::symbol("length")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result.as_symbol().unwrap(),
            expression_runtime.infinite_value
        )
    }

    #[test]
    fn range_length_no_end() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::integer_range(Some(15), None))
            .unwrap();
        instructions.put(ExpressionValue::symbol("length")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result.as_symbol().unwrap(),
            expression_runtime.infinite_value
        )
    }

    #[test]
    fn range_length_no_start_or_end() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::integer_range(None, None))
            .unwrap();
        instructions.put(ExpressionValue::symbol("length")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result.as_symbol().unwrap(),
            expression_runtime.infinite_value
        )
    }

    #[test]
    fn range_inclusive_length_step() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(
                ExpressionValue::integer_range(Some(0), Some(10))
                    .with_step(ExpressionValue::integer(2)),
            )
            .unwrap();
        instructions.put(ExpressionValue::symbol("length")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        // 0, 2, 4, 6, 8, 10
        assert_eq!(result.as_integer().unwrap(), 6)
    }

    #[test]
    fn range_exclusive_start_length_step() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(
                ExpressionValue::integer_range(Some(0), Some(10))
                    .exclude_start()
                    .with_step(ExpressionValue::integer(2)),
            )
            .unwrap();
        instructions.put(ExpressionValue::symbol("length")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        // 2, 4, 6, 8, 10
        assert_eq!(result.as_integer().unwrap(), 5)
    }

    #[test]
    fn range_exclusive_end_length_step() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(
                ExpressionValue::integer_range(Some(0), Some(10))
                    .exclude_end()
                    .with_step(ExpressionValue::integer(2)),
            )
            .unwrap();
        instructions.put(ExpressionValue::symbol("length")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        // 0, 2, 4, 6, 8
        assert_eq!(result.as_integer().unwrap(), 5)
    }

    #[test]
    fn range_zero_step() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(
                ExpressionValue::integer_range(Some(0), Some(10))
                    .exclude_end()
                    .with_step(ExpressionValue::integer(0)),
            )
            .unwrap();
        instructions.put(ExpressionValue::symbol("length")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result.as_symbol().unwrap(),
            expression_runtime.infinite_value
        )
    }

    #[test]
    fn range_exclusive_length_step() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(
                ExpressionValue::integer_range(Some(0), Some(10))
                    .exclude_start()
                    .exclude_end()
                    .with_step(ExpressionValue::integer(2)),
            )
            .unwrap();
        instructions.put(ExpressionValue::symbol("length")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        // 1, 3, 5, 7, 9
        assert_eq!(result.as_integer().unwrap(), 5)
    }

    #[test]
    fn range_exclusive_end_length() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::integer_range(Some(1), Some(3)).exclude_end())
            .unwrap();

        instructions.put(ExpressionValue::symbol("length")).unwrap();
        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 2);
    }

    #[test]
    fn range_exclusive_start_length() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::integer_range(Some(1), Some(3)).exclude_start())
            .unwrap();

        instructions.put(ExpressionValue::symbol("length")).unwrap();
        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 2);
    }

    #[test]
    fn range_exclusive_length() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(
                ExpressionValue::integer_range(Some(1), Some(3))
                    .exclude_start()
                    .exclude_end(),
            )
            .unwrap();

        instructions.put(ExpressionValue::symbol("length")).unwrap();
        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 1);
    }

    #[test]
    fn range_exclusive_length_zero_length() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(
                ExpressionValue::integer_range(Some(1), Some(1))
                    .exclude_start()
                    .exclude_end(),
            )
            .unwrap();

        instructions.put(ExpressionValue::symbol("length")).unwrap();
        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 0);
    }

    #[test]
    fn range_start() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::integer_range(Some(15), Some(23)))
            .unwrap();
        instructions.put(ExpressionValue::symbol("start")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 15)
    }

    #[test]
    fn range_start_no_start() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::integer_range(None, Some(23)))
            .unwrap();
        instructions.put(ExpressionValue::symbol("start")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit());
    }

    #[test]
    fn range_end() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::integer_range(Some(15), Some(23)))
            .unwrap();
        instructions.put(ExpressionValue::symbol("end")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 23)
    }

    #[test]
    fn range_end_no_end() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::integer_range(Some(15), None))
            .unwrap();
        instructions.put(ExpressionValue::symbol("end")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit())
    }

    #[test]
    fn range_step() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(
                ExpressionValue::integer_range(Some(15), Some(23))
                    .with_step(ExpressionValue::integer(2)),
            )
            .unwrap();
        instructions.put(ExpressionValue::symbol("step")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 2)
    }

    #[test]
    fn range_step_no_step() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::integer_range(Some(15), Some(23)))
            .unwrap();
        instructions.put(ExpressionValue::symbol("step")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit())
    }

    #[test]
    fn range_is_start_exclusive() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(
                ExpressionValue::integer_range(Some(15), Some(23))
                    .with_step(ExpressionValue::integer(2)),
            )
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("is_start_exclusive"))
            .unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value)
    }

    #[test]
    fn range_is_end_exclusive() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(
                ExpressionValue::integer_range(Some(15), Some(23))
                    .with_step(ExpressionValue::integer(2)),
            )
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("is_end_exclusive"))
            .unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value)
    }

    #[test]
    fn range_is_start_open() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(
                ExpressionValue::integer_range(Some(15), Some(23))
                    .with_step(ExpressionValue::integer(2)),
            )
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("is_start_open"))
            .unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value)
    }

    #[test]
    fn range_is_end_open() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(
                ExpressionValue::integer_range(Some(15), Some(23))
                    .with_step(ExpressionValue::integer(2)),
            )
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("is_end_open"))
            .unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value)
    }
}

#[cfg(test)]
mod slice_tests {
    use expr_lang_common::ExpressionValue;
    use expr_lang_instruction_set_builder::InstructionSetBuilder;

    use crate::ExpressionRuntime;

    #[test]
    fn slice_inclusive_length() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.put(ExpressionValue::integer(40)).unwrap();
        instructions.put(ExpressionValue::integer(50)).unwrap();
        instructions.make_list();

        instructions
            .put(ExpressionValue::integer_range(Some(1), Some(3)))
            .unwrap();

        instructions.perform_access();

        instructions.put(ExpressionValue::symbol("length")).unwrap();
        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 3);
    }

    #[test]
    fn slice_length_no_start() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.put(ExpressionValue::integer(40)).unwrap();
        instructions.put(ExpressionValue::integer(50)).unwrap();
        instructions.make_list();

        instructions
            .put(ExpressionValue::integer_range(None, Some(3)))
            .unwrap();

        instructions.perform_access();

        instructions.put(ExpressionValue::symbol("length")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 4)
    }

    #[test]
    fn slice_length_no_end() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.put(ExpressionValue::integer(40)).unwrap();
        instructions.put(ExpressionValue::integer(50)).unwrap();
        instructions.make_list();

        instructions
            .put(ExpressionValue::integer_range(Some(1), None))
            .unwrap();

        instructions.perform_access();

        instructions.put(ExpressionValue::symbol("length")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 4)
    }

    #[test]
    fn slice_length_no_start_or_end() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.put(ExpressionValue::integer(40)).unwrap();
        instructions.put(ExpressionValue::integer(50)).unwrap();
        instructions.make_list();

        instructions
            .put(ExpressionValue::integer_range(None, None))
            .unwrap();

        instructions.perform_access();

        instructions.put(ExpressionValue::symbol("length")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 5)
    }

    #[test]
    fn slice_inclusive_length_step() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.put(ExpressionValue::integer(40)).unwrap();
        instructions.put(ExpressionValue::integer(50)).unwrap();
        instructions.put(ExpressionValue::integer(60)).unwrap();
        instructions.put(ExpressionValue::integer(70)).unwrap();
        instructions.put(ExpressionValue::integer(80)).unwrap();
        instructions.put(ExpressionValue::integer(90)).unwrap();
        instructions.put(ExpressionValue::integer(100)).unwrap();
        instructions.make_list();

        instructions
            .put(
                ExpressionValue::integer_range(Some(0), Some(10))
                    .with_step(ExpressionValue::integer(2)),
            )
            .unwrap();

        instructions.perform_access();

        instructions.put(ExpressionValue::symbol("length")).unwrap();
        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 6);
    }

    #[test]
    fn slice_length_no_start_step() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.put(ExpressionValue::integer(40)).unwrap();
        instructions.put(ExpressionValue::integer(50)).unwrap();
        instructions.put(ExpressionValue::integer(60)).unwrap();
        instructions.put(ExpressionValue::integer(70)).unwrap();
        instructions.put(ExpressionValue::integer(80)).unwrap();
        instructions.put(ExpressionValue::integer(90)).unwrap();
        instructions.put(ExpressionValue::integer(100)).unwrap();
        instructions.make_list();

        instructions
            .put(
                ExpressionValue::integer_range(None, Some(10))
                    .with_step(ExpressionValue::integer(2)),
            )
            .unwrap();

        instructions.perform_access();

        instructions.put(ExpressionValue::symbol("length")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        //
        assert_eq!(result.as_integer().unwrap(), 6)
    }

    #[test]
    fn slice_length_no_end_step() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.put(ExpressionValue::integer(40)).unwrap();
        instructions.put(ExpressionValue::integer(50)).unwrap();
        instructions.put(ExpressionValue::integer(60)).unwrap();
        instructions.put(ExpressionValue::integer(70)).unwrap();
        instructions.put(ExpressionValue::integer(80)).unwrap();
        instructions.put(ExpressionValue::integer(90)).unwrap();
        instructions.put(ExpressionValue::integer(100)).unwrap();
        instructions.make_list();

        instructions
            .put(
                ExpressionValue::integer_range(Some(0), None)
                    .with_step(ExpressionValue::integer(2)),
            )
            .unwrap();

        instructions.perform_access();

        instructions.put(ExpressionValue::symbol("length")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 5)
    }

    #[test]
    fn slice_length_no_start_or_end_step() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.put(ExpressionValue::integer(40)).unwrap();
        instructions.put(ExpressionValue::integer(50)).unwrap();
        instructions.put(ExpressionValue::integer(60)).unwrap();
        instructions.put(ExpressionValue::integer(70)).unwrap();
        instructions.put(ExpressionValue::integer(80)).unwrap();
        instructions.put(ExpressionValue::integer(90)).unwrap();
        instructions.put(ExpressionValue::integer(100)).unwrap();
        instructions.make_list();

        instructions
            .put(ExpressionValue::integer_range(None, None).with_step(ExpressionValue::integer(2)))
            .unwrap();

        instructions.perform_access();

        instructions.put(ExpressionValue::symbol("length")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 5)
    }

    #[test]
    fn slice_of_slice() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.put(ExpressionValue::integer(40)).unwrap();
        instructions.put(ExpressionValue::integer(50)).unwrap();
        instructions.put(ExpressionValue::integer(60)).unwrap();
        instructions.put(ExpressionValue::integer(70)).unwrap();
        instructions.put(ExpressionValue::integer(80)).unwrap();
        instructions.put(ExpressionValue::integer(90)).unwrap();
        instructions.put(ExpressionValue::integer(100)).unwrap();
        instructions.make_list();

        instructions
            .put(ExpressionValue::integer_range(Some(0), Some(8)))
            .unwrap();

        instructions.perform_access();

        instructions
            .put(ExpressionValue::integer_range(Some(1), Some(4)))
            .unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result
                .get_slice_source()
                .unwrap()
                .get_slice_range()
                .unwrap()
                .get_range_end()
                .unwrap()
                .as_integer()
                .unwrap(),
            8
        );
        assert_eq!(
            result
                .get_slice_range()
                .unwrap()
                .get_range_end()
                .unwrap()
                .as_integer()
                .unwrap(),
            4
        );
    }
}

#[cfg(test)]
mod link_tests {
    use expr_lang_common::ExpressionValue;
    use expr_lang_instruction_set_builder::InstructionSetBuilder;

    use crate::ExpressionRuntime;

    #[test]
    fn length_non_list_items() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::integer(40)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::integer(50)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::integer(60)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::integer(70)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::integer(80)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::integer(90)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::integer(100)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::symbol("length")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 10)
    }

    #[test]
    fn length_with_lists() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.put(ExpressionValue::integer(40)).unwrap();
        instructions.put(ExpressionValue::integer(50)).unwrap();
        instructions.make_list();

        instructions.start_list();
        instructions.put(ExpressionValue::integer(60)).unwrap();
        instructions.put(ExpressionValue::integer(70)).unwrap();
        instructions.put(ExpressionValue::integer(80)).unwrap();
        instructions.put(ExpressionValue::integer(90)).unwrap();
        instructions.put(ExpressionValue::integer(100)).unwrap();
        instructions.make_list();

        instructions.make_link();

        instructions.put(ExpressionValue::integer(110)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::integer(120)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::symbol("length")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 12)
    }

    #[test]
    fn length_with_slices() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.put(ExpressionValue::integer(40)).unwrap();
        instructions.put(ExpressionValue::integer(50)).unwrap();
        instructions.make_list();

        instructions
            .put(ExpressionValue::integer_range(Some(0), Some(2)))
            .unwrap();
        instructions.perform_access();

        instructions.start_list();
        instructions.put(ExpressionValue::integer(60)).unwrap();
        instructions.put(ExpressionValue::integer(70)).unwrap();
        instructions.put(ExpressionValue::integer(80)).unwrap();
        instructions.put(ExpressionValue::integer(90)).unwrap();
        instructions.put(ExpressionValue::integer(100)).unwrap();
        instructions.make_list();

        instructions
            .put(ExpressionValue::integer_range(Some(0), Some(2)))
            .unwrap();
        instructions.perform_access();

        instructions.make_link();

        instructions.put(ExpressionValue::integer(110)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::integer(120)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::symbol("length")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 8)
    }

    #[test]
    fn length_with_character_list_and_characters() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("bear".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("cats".to_string()))
            .unwrap();

        instructions.make_link();

        instructions
            .put(ExpressionValue::character("d".to_string()))
            .unwrap();
        instructions.make_link();

        instructions
            .put(ExpressionValue::character("o".to_string()))
            .unwrap();
        instructions.make_link();

        instructions
            .put(ExpressionValue::character("g".to_string()))
            .unwrap();
        instructions.make_link();

        instructions
            .put(ExpressionValue::character("s".to_string()))
            .unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::symbol("length")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 12)
    }

    #[test]
    fn length_with_character_list_and_other_items() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("bear".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("cats".to_string()))
            .unwrap();

        instructions.make_link();

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::integer(40)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::symbol("length")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 6)
    }

    #[test]
    fn length_with_slice_of_character_list_and_characters() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("bear".to_string()))
            .unwrap();

        instructions
            .put(ExpressionValue::integer_range(Some(0), Some(2)))
            .unwrap();
        instructions.perform_access();

        instructions
            .put(ExpressionValue::character_list("cats".to_string()))
            .unwrap();

        instructions
            .put(ExpressionValue::integer_range(Some(0), Some(2)))
            .unwrap();
        instructions.perform_access();

        instructions.make_link();

        instructions
            .put(ExpressionValue::character("d".to_string()))
            .unwrap();
        instructions.make_link();

        instructions
            .put(ExpressionValue::character("o".to_string()))
            .unwrap();
        instructions.make_link();

        instructions
            .put(ExpressionValue::character("g".to_string()))
            .unwrap();
        instructions.make_link();

        instructions
            .put(ExpressionValue::character("s".to_string()))
            .unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::symbol("length")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 10)
    }

    #[test]
    fn length_with_slice_of_slice_of_character_list_and_characters() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("bear".to_string()))
            .unwrap();

        instructions
            .put(ExpressionValue::integer_range(Some(0), Some(2)))
            .unwrap();
        instructions.perform_access();

        instructions
            .put(ExpressionValue::integer_range(Some(0), Some(1)))
            .unwrap();
        instructions.perform_access();

        instructions
            .put(ExpressionValue::character_list("cats".to_string()))
            .unwrap();

        instructions.make_link();

        instructions
            .put(ExpressionValue::character("d".to_string()))
            .unwrap();
        instructions.make_link();

        instructions
            .put(ExpressionValue::character("o".to_string()))
            .unwrap();
        instructions.make_link();

        instructions
            .put(ExpressionValue::character("g".to_string()))
            .unwrap();
        instructions.make_link();

        instructions
            .put(ExpressionValue::character("s".to_string()))
            .unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::symbol("length")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 10)
    }

    #[test]
    fn length_with_slice_of_character_list_and_other_items() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("bear".to_string()))
            .unwrap();

        instructions
            .put(ExpressionValue::integer_range(Some(0), Some(1)))
            .unwrap();
        instructions.perform_access();

        instructions
            .put(ExpressionValue::character_list("cats".to_string()))
            .unwrap();

        instructions
            .put(ExpressionValue::integer_range(Some(0), Some(1)))
            .unwrap();
        instructions.perform_access();

        instructions.make_link();

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::integer(40)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::symbol("length")).unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 8)
    }

    #[test]
    fn slice_of_link() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::integer(40)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::integer(50)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::integer(60)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::integer(70)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::integer(80)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::integer(90)).unwrap();
        instructions.make_link();

        instructions.put(ExpressionValue::integer(100)).unwrap();
        instructions.make_link();

        instructions
            .put(ExpressionValue::integer_range(Some(0), Some(4)))
            .unwrap();

        instructions.perform_access();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);
        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(
            result
                .get_slice_source()
                .unwrap()
                .get_link_value()
                .unwrap()
                .as_integer()
                .unwrap(),
            100
        );
        assert_eq!(
            result
                .get_slice_range()
                .unwrap()
                .get_range_end()
                .unwrap()
                .as_integer()
                .unwrap(),
            4
        );
    }
}
