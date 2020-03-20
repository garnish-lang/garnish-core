use crate::ExpressionRuntime;
use expr_lang_common::{
    has_end, has_start, has_step, skip_byte_sizes, skip_size, skip_sizes, skip_type,
    skip_type_and_byte_size, skip_type_and_size, DataType, Result,
};
use std::cmp::Ordering;
use std::convert::TryFrom;

impl ExpressionRuntime {
    pub(crate) fn perform_equality_comparison(&mut self) -> Result {
        self.insert_comparison_result(|ordering| ordering == Ordering::Equal)
    }

    pub(crate) fn perform_inequality_comparison(&mut self) -> Result {
        match self.ordering_of_values()? {
            Some(ordering) => self.insert_symbol_value_from_condition(ordering != Ordering::Equal),
            None => self.insert_symbol_value_from_condition(true),
        }
    }

    pub(crate) fn perform_less_than_comparison(&mut self) -> Result {
        self.insert_comparison_result(|ordering| {
            println!("{:?}", ordering);
            ordering == Ordering::Less
        })
    }

    pub(crate) fn perform_less_than_or_equal_comparison(&mut self) -> Result {
        self.insert_comparison_result(|ordering| {
            ordering == Ordering::Less || ordering == Ordering::Equal
        })
    }

    pub(crate) fn perform_greater_than_comparison(&mut self) -> Result {
        self.insert_comparison_result(|ordering| ordering == Ordering::Greater)
    }

    pub(crate) fn perform_greater_than_or_equal_comparison(&mut self) -> Result {
        self.insert_comparison_result(|ordering| {
            ordering == Ordering::Greater || ordering == Ordering::Equal
        })
    }

    pub(crate) fn perform_type_comparison(&mut self) -> Result {
        let (left, right) = self.consume_last_two_refs_and_set_value_cursor()?;

        self.insert_symbol_value_from_condition(
            DataType::try_from(self.data[left])? == DataType::try_from(self.data[right])?,
        )
    }

    fn insert_comparison_result<T>(&mut self, f: T) -> Result
    where
        T: Fn(Ordering) -> bool,
    {
        match self.ordering_of_values()? {
            Some(ordering) => self.insert_symbol_value_from_condition(f(ordering)),
            None => self.insert_symbol_value_from_condition(false),
        }
    }

    fn ordering_of_values(&mut self) -> Result<Option<Ordering>> {
        let (left_ref, right_ref) = self.consume_last_two_refs_and_set_value_cursor()?;
        self.ordering_of_values_refs(left_ref, right_ref)
    }

    pub(crate) fn ordering_of_values_refs(
        &self,
        left_ref: usize,
        right_ref: usize,
    ) -> Result<Option<Ordering>> {
        Ok(
            match (
                DataType::try_from(self.data[left_ref])?,
                DataType::try_from(self.data[right_ref])?,
            ) {
                (DataType::Unit, DataType::Unit) => Some(Ordering::Equal),
                (DataType::Integer, DataType::Integer) => {
                    let left = self.read_integer_at(skip_type(left_ref))?;
                    let right = self.read_integer_at(skip_type(right_ref))?;

                    left.partial_cmp(&right)
                }
                (DataType::Integer, DataType::Float) => {
                    let left = self.read_integer_at(skip_type(left_ref))?;
                    let right = self.read_float_at(skip_type(right_ref))?;

                    (left as f32).partial_cmp(&right)
                }
                (DataType::Float, DataType::Integer) => {
                    let left = self.read_float_at(skip_type(left_ref))?;
                    let right = self.read_integer_at(skip_type(right_ref))?;

                    left.partial_cmp(&(right as f32))
                }
                (DataType::Float, DataType::Float) => {
                    let left = self.read_float_at(skip_type(left_ref))?;
                    let right = self.read_float_at(skip_type(right_ref))?;

                    left.partial_cmp(&right)
                }
                (DataType::Symbol, DataType::Symbol) => {
                    let left = self.read_size_at(skip_type(left_ref))?;
                    let right = self.read_size_at(skip_type(right_ref))?;

                    if left == right {
                        Some(Ordering::Equal)
                    } else {
                        let left_symbol = self.symbol_table.iter().find(|(_k, v)| **v == left);
                        let right_symbol = self.symbol_table.iter().find(|(_k, v)| **v == right);

                        match left_symbol {
                            Some((lk, _lv)) => match right_symbol {
                                Some((rk, _rv)) => lk.partial_cmp(&rk),
                                None => None,
                            },
                            None => None,
                        }
                    }
                }
                (DataType::Character, DataType::Character) => {
                    Some(self.ordering_of_characters(left_ref, right_ref)?)
                }
                (DataType::Character, DataType::CharacterList) => {
                    Some(self.ordering_of_character_character_list(left_ref, right_ref)?)
                }
                (DataType::CharacterList, DataType::Character) => {
                    // method compares character to character list
                    // match to invert Less <> Greater
                    Some(
                        match self.ordering_of_character_character_list(right_ref, left_ref)? {
                            Ordering::Less => Ordering::Greater,
                            Ordering::Greater => Ordering::Less,
                            Ordering::Equal => Ordering::Equal,
                        },
                    )
                }
                (DataType::CharacterList, DataType::CharacterList) => {
                    let left_length = self.read_size_at(skip_type(left_ref))?;
                    let right_length = self.read_size_at(skip_type(right_ref))?;

                    let left_start = skip_type_and_size(left_ref);
                    let right_start = skip_type_and_size(right_ref);

                    let (min, length_ordering) = if left_length > right_length {
                        (right_length, Ordering::Greater)
                    } else if left_length < right_length {
                        (left_length, Ordering::Less)
                    } else {
                        (left_length, Ordering::Equal)
                    };

                    for i in 0..min {
                        let offset = skip_sizes(0, i);
                        let left_c = self.read_size_at(left_start + offset)?;
                        let right_c = self.read_size_at(right_start + offset)?;

                        let ordering = self.ordering_of_characters(left_c, right_c)?;

                        // make early decision on ordering
                        // based on character ordering
                        if ordering != Ordering::Equal {
                            return Ok(Some(ordering));
                        }
                    }

                    // characters up to min are equal
                    // check length ordering to determine final ordering
                    Some(length_ordering)
                }
                (DataType::Range, DataType::Range) => {
                    let left_flags = self.read_byte_size_at(skip_type(left_ref))?;
                    let right_flags = self.read_byte_size_at(skip_type(right_ref))?;

                    if left_flags != right_flags {
                        return Ok(None);
                    }

                    let offset = skip_type_and_byte_size(0);

                    // capture left_ref and right_ref for usage in ordering_of_range_part method
                    // since they never change
                    let f = |condition, offset| {
                        self.range_part_are_equal(condition, left_ref, right_ref, offset)
                    };

                    match f(has_start(left_flags), offset)? {
                        (true, offset) => match f(has_end(left_flags), offset)? {
                            (true, offset) => match f(has_step(left_flags), offset)? {
                                (true, _) => Some(Ordering::Equal),
                                (false, _) => None,
                            },
                            (false, _) => None,
                        },
                        (false, _) => None,
                    }
                }
                (DataType::Pair, DataType::Pair) => {
                    match self.pair_part_is_equal(skip_type(left_ref), skip_type(right_ref))? {
                        true => match self.pair_part_is_equal(
                            skip_type_and_size(left_ref),
                            skip_type_and_size(right_ref),
                        )? {
                            true => Some(Ordering::Equal),
                            false => None,
                        },
                        false => None,
                    }
                }
                (DataType::List, DataType::List) => {
                    let left_length = self.read_size_at(skip_type(left_ref))?;
                    let right_length = self.read_size_at(skip_type(right_ref))?;

                    if left_length != right_length {
                        return Ok(None);
                    }

                    let left_start = skip_type_and_size(left_ref);
                    let right_start = skip_type_and_size(right_ref);

                    for i in 0..left_length {
                        let item_offset = skip_sizes(0, i);
                        match self.ordering_of_values_refs(
                            self.read_size_at(left_start + item_offset)?,
                            self.read_size_at(right_start + item_offset)?,
                        )? {
                            Some(ordering) => match ordering {
                                Ordering::Less | Ordering::Greater => return Ok(None),
                                Ordering::Equal => continue,
                            },
                            None => return Ok(None),
                        }
                    }

                    Some(Ordering::Equal)
                }
                _ => None,
            },
        )
    }

    fn pair_part_is_equal(&self, left_ref: usize, right_ref: usize) -> Result<bool> {
        let left_part = self.read_size_at(left_ref)?;
        let right_part = self.read_size_at(right_ref)?;

        Ok(match self.ordering_of_values_refs(left_part, right_part)? {
            Some(o) => {
                if o != Ordering::Equal {
                    false
                } else {
                    true
                }
            }
            None => false,
        })
    }

    fn range_part_are_equal(
        &self,
        condition: bool,
        left_ref: usize,
        right_ref: usize,
        offset: usize,
    ) -> Result<(bool, usize)> {
        if condition {
            let left_start_ref = self.read_size_at(left_ref + offset)?;
            let right_start_ref = self.read_size_at(right_ref + offset)?;

            match self.ordering_of_values_refs(left_start_ref, right_start_ref)? {
                Some(o) => {
                    if o != Ordering::Equal {
                        return Ok((false, offset));
                    }
                }
                None => return Ok((false, offset)),
            }

            Ok((true, skip_size(offset)))
        } else {
            Ok((true, offset))
        }
    }

    fn ordering_of_character_character_list(
        &self,
        character_ref: usize,
        character_list_ref: usize,
    ) -> Result<Ordering> {
        let character_list_length = self.read_size_at(skip_type(character_list_ref))?;

        // if character list is empty, character will be greater
        if character_list_length == 0 {
            Ok(Ordering::Greater)
        } else if character_list_length > 1 {
            // if character list has more than 1 character
            // character will be less
            Ok(Ordering::Less)
        } else {
            // get first character from character list
            // and compare it to character
            let first_character_ref = self.read_size_at(skip_type_and_size(character_list_ref))?;

            Ok(self.ordering_of_characters(character_ref, first_character_ref)?)
        }
    }

    fn ordering_of_characters(&self, left_ref: usize, right_ref: usize) -> Result<Ordering> {
        let left_length = self.read_byte_size_at(skip_type(left_ref))? as usize;
        let right_length = self.read_byte_size_at(skip_type(right_ref))? as usize;

        // only compare if same length
        if left_length < right_length {
            return Ok(Ordering::Less);
        } else if left_length > right_length {
            return Ok(Ordering::Greater);
        }

        let left_start = skip_type_and_byte_size(left_ref);
        let right_start = skip_type_and_byte_size(right_ref);

        for i in 0..left_length {
            let offset = skip_byte_sizes(0, i);
            let left_c = self.read_byte_size_at(left_start + offset)?;
            let right_c = self.read_byte_size_at(right_start + offset)?;

            let ordering = left_c.cmp(&right_c);

            if ordering != Ordering::Equal {
                return Ok(ordering);
            }
        }

        return Ok(Ordering::Equal);
    }
}

#[cfg(test)]
mod equality_tests {
    use crate::runtime::ExpressionRuntime;
    use expr_lang_common::ExpressionValue;
    use expr_lang_instruction_set_builder::InstructionSetBuilder;

    #[test]
    fn unit_unit_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.perform_equality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn unit_any_not_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_equality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn integer_integer_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_equality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn integer_integer_not_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.perform_equality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn integer_float_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_equality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn integer_float_not_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::float(20.0)).unwrap();
        instructions.perform_equality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn float_integer_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_equality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn float_integer_not_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.perform_equality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn float_float_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.perform_equality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn float_float_not_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.put(ExpressionValue::float(20.0)).unwrap();
        instructions.perform_equality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_character_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("a".to_string()))
            .unwrap();
        instructions.perform_equality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_character_not_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("z".to_string()))
            .unwrap();
        instructions.perform_equality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_list_character_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("a".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("a".to_string()))
            .unwrap();
        instructions.perform_equality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_list_character_not_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("abcd".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("z".to_string()))
            .unwrap();
        instructions.perform_equality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_character_list_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("a".to_string()))
            .unwrap();
        instructions.perform_equality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_character_list_not_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("abcd".to_string()))
            .unwrap();
        instructions.perform_equality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_list_character_list_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("abcd".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("abcd".to_string()))
            .unwrap();
        instructions.perform_equality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_list_character_list_not_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("abc".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("abcd".to_string()))
            .unwrap();
        instructions.perform_equality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn symbol_symbol_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions.perform_equality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn symbol_symbol_not_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("dogs".to_string()))
            .unwrap();
        instructions.perform_equality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn range_range_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::integer_range(Some(10), Some(20)))
            .unwrap();
        instructions
            .put(ExpressionValue::integer_range(Some(10), Some(20)))
            .unwrap();
        instructions.perform_equality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn range_range_not_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::integer_range(Some(10), Some(20)))
            .unwrap();
        instructions
            .put(ExpressionValue::integer_range(Some(15), Some(25)))
            .unwrap();
        instructions.perform_equality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn pair_pair_equal() {
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
                ExpressionValue::symbol("cats"),
                ExpressionValue::integer(10),
            ))
            .unwrap();
        instructions.perform_equality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn pair_pair_not_equal() {
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
                ExpressionValue::symbol("dogs"),
                ExpressionValue::integer(10),
            ))
            .unwrap();
        instructions.perform_equality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn list_list_equal() {
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
            .put(
                ExpressionValue::list()
                    .add(ExpressionValue::integer(10))
                    .add(ExpressionValue::integer(20))
                    .add(ExpressionValue::integer(30)),
            )
            .unwrap();
        instructions.perform_equality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn list_list_not_equal() {
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
            .put(
                ExpressionValue::list()
                    .add(ExpressionValue::integer(15))
                    .add(ExpressionValue::integer(25))
                    .add(ExpressionValue::integer(35)),
            )
            .unwrap();
        instructions.perform_equality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }
}

#[cfg(test)]
mod inequality_tests {
    use crate::runtime::ExpressionRuntime;
    use expr_lang_common::ExpressionValue;
    use expr_lang_instruction_set_builder::InstructionSetBuilder;

    #[test]
    fn unit_unit_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.perform_inequality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn unit_any_not_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_inequality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn integer_integer_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_inequality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn integer_integer_not_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.perform_inequality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn integer_float_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_inequality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn integer_float_not_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::float(20.0)).unwrap();
        instructions.perform_inequality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn float_integer_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_inequality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn float_integer_not_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.perform_inequality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn float_float_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.perform_inequality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn float_float_not_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.put(ExpressionValue::float(20.0)).unwrap();
        instructions.perform_inequality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_character_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("a".to_string()))
            .unwrap();
        instructions.perform_inequality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_character_not_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("z".to_string()))
            .unwrap();
        instructions.perform_inequality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_list_character_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("a".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("a".to_string()))
            .unwrap();
        instructions.perform_inequality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_list_character_not_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("abcd".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("z".to_string()))
            .unwrap();
        instructions.perform_inequality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_character_list_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("a".to_string()))
            .unwrap();
        instructions.perform_inequality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_character_list_not_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("abcd".to_string()))
            .unwrap();
        instructions.perform_inequality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_list_character_list_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("abcd".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("abcd".to_string()))
            .unwrap();
        instructions.perform_inequality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_list_character_list_not_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("abc".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("abcd".to_string()))
            .unwrap();
        instructions.perform_inequality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn symbol_symbol_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions.perform_inequality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn symbol_symbol_not_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("dogs".to_string()))
            .unwrap();
        instructions.perform_inequality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn range_range_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::integer_range(Some(10), Some(20)))
            .unwrap();
        instructions
            .put(ExpressionValue::integer_range(Some(10), Some(20)))
            .unwrap();
        instructions.perform_inequality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn range_range_not_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::integer_range(Some(10), Some(20)))
            .unwrap();
        instructions
            .put(ExpressionValue::integer_range(Some(15), Some(25)))
            .unwrap();
        instructions.perform_inequality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn pair_pair_equal() {
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
                ExpressionValue::symbol("cats"),
                ExpressionValue::integer(10),
            ))
            .unwrap();
        instructions.perform_inequality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn pair_pair_not_equal() {
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
                ExpressionValue::symbol("dogs"),
                ExpressionValue::integer(10),
            ))
            .unwrap();
        instructions.perform_inequality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn list_list_equal() {
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
            .put(
                ExpressionValue::list()
                    .add(ExpressionValue::integer(10))
                    .add(ExpressionValue::integer(20))
                    .add(ExpressionValue::integer(30)),
            )
            .unwrap();
        instructions.perform_inequality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn list_list_not_equal() {
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
            .put(
                ExpressionValue::list()
                    .add(ExpressionValue::integer(15))
                    .add(ExpressionValue::integer(25))
                    .add(ExpressionValue::integer(35)),
            )
            .unwrap();
        instructions.perform_inequality_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }
}

#[cfg(test)]
mod less_than_tests {
    use crate::ExpressionRuntime;
    use expr_lang_common::ExpressionValue;
    use expr_lang_instruction_set_builder::InstructionSetBuilder;

    #[test]
    fn integer_integer_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn integer_integer_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn integer_integer_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn integer_float_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::float(20.0)).unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn integer_float_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn integer_float_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn float_float_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.put(ExpressionValue::float(20.0)).unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn float_float_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(20.0)).unwrap();
        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn float_float_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn float_integer_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn float_integer_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(20.0)).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn float_integer_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_character_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("z".into()))
            .unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_character_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("z".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_character_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_list_character_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("z".into()))
            .unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_list_character_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("abc".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_list_character_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_character_list_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("abc".into()))
            .unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_character_list_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("z".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("a".into()))
            .unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_character_list_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("a".into()))
            .unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_character_unicode_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("🐼".into()))
            .unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_unicode_character_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("🐼".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_list_character_list_same_length_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("cats".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("dogs".into()))
            .unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_list_character_list_same_length_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("dogs".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("cats".into()))
            .unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_list_character_list_same_length_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("cats".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("cats".into()))
            .unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_list_longer_character_list_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("cat".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("cats".into()))
            .unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn longer_character_list_character_list_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("cats".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("cat".into()))
            .unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn symbol_symbol_same_length_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("dogs".to_string()))
            .unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn symbol_symbol_same_length_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::symbol("dogs".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn symbol_symbol_same_length_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn symbol_longer_symbol_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::symbol("cat".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn longer_symbol_symbol_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("cat".to_string()))
            .unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn range_compare_is_false() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::integer_range(Some(10), Some(20)))
            .unwrap();
        instructions
            .put(ExpressionValue::integer_range(Some(10), Some(20)))
            .unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn pair_compare_is_false() {
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
                ExpressionValue::symbol("cats"),
                ExpressionValue::integer(10),
            ))
            .unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn list_compare_is_false() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::list().add(ExpressionValue::integer(10)))
            .unwrap();
        instructions
            .put(ExpressionValue::list().add(ExpressionValue::integer(10)))
            .unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn unit_compare_is_false() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.perform_less_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }
}

#[cfg(test)]
mod less_than_or_equal_tests {
    use crate::ExpressionRuntime;
    use expr_lang_common::ExpressionValue;
    use expr_lang_instruction_set_builder::InstructionSetBuilder;

    #[test]
    fn integer_integer_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn integer_integer_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn integer_integer_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn integer_float_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::float(20.0)).unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn integer_float_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn integer_float_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn float_float_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.put(ExpressionValue::float(20.0)).unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn float_float_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(20.0)).unwrap();
        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn float_float_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn float_integer_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn float_integer_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(20.0)).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn float_integer_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_character_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("z".into()))
            .unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_character_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("z".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_character_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_list_character_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("z".into()))
            .unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_list_character_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("abc".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_list_character_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_character_list_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("abc".into()))
            .unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_character_list_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("z".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("a".into()))
            .unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_character_list_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("a".into()))
            .unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_character_unicode_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("🐼".into()))
            .unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_unicode_character_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("🐼".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_list_character_list_same_length_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("cats".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("dogs".into()))
            .unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_list_character_list_same_length_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("dogs".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("cats".into()))
            .unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_list_character_list_same_length_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("cats".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("cats".into()))
            .unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_list_longer_character_list_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("cat".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("cats".into()))
            .unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn longer_character_list_character_list_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("cats".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("cat".into()))
            .unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn symbol_symbol_same_length_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("dogs".to_string()))
            .unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn symbol_symbol_same_length_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::symbol("dogs".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn symbol_symbol_same_length_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn symbol_longer_symbol_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::symbol("cat".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn longer_symbol_symbol_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("cat".to_string()))
            .unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn range_compare() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::integer_range(Some(10), Some(20)))
            .unwrap();
        instructions
            .put(ExpressionValue::integer_range(Some(10), Some(20)))
            .unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn pair_compare() {
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
                ExpressionValue::symbol("cats"),
                ExpressionValue::integer(10),
            ))
            .unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn list_compare() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::list().add(ExpressionValue::integer(10)))
            .unwrap();
        instructions
            .put(ExpressionValue::list().add(ExpressionValue::integer(10)))
            .unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn unit_compare() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.perform_less_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }
}

#[cfg(test)]
mod greater_than_tests {
    use crate::ExpressionRuntime;
    use expr_lang_common::ExpressionValue;
    use expr_lang_instruction_set_builder::InstructionSetBuilder;

    #[test]
    fn integer_integer_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn integer_integer_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn integer_integer_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn integer_float_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::float(20.0)).unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn integer_float_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn integer_float_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn float_float_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.put(ExpressionValue::float(20.0)).unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn float_float_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(20.0)).unwrap();
        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn float_float_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn float_integer_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn float_integer_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(20.0)).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn float_integer_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_character_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("z".into()))
            .unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_character_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("z".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_character_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_list_character_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("z".into()))
            .unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_list_character_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("abc".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_list_character_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_character_list_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("abc".into()))
            .unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_character_list_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("z".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("a".into()))
            .unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_character_list_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("a".into()))
            .unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_character_unicode_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("🐼".into()))
            .unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_unicode_character_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("🐼".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_list_character_list_same_length_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("cats".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("dogs".into()))
            .unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_list_character_list_same_length_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("dogs".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("cats".into()))
            .unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_list_character_list_same_length_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("cats".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("cats".into()))
            .unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_list_longer_character_list_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("cat".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("cats".into()))
            .unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn longer_character_list_character_list_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("cats".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("cat".into()))
            .unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn symbol_symbol_same_length_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("dogs".to_string()))
            .unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn symbol_symbol_same_length_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::symbol("dogs".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn symbol_symbol_same_length_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn symbol_longer_symbol_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::symbol("cat".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn longer_symbol_symbol_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("cat".to_string()))
            .unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn range_compare_is_false() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::integer_range(Some(10), Some(20)))
            .unwrap();
        instructions
            .put(ExpressionValue::integer_range(Some(10), Some(20)))
            .unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn pair_compare_is_false() {
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
                ExpressionValue::symbol("cats"),
                ExpressionValue::integer(10),
            ))
            .unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn list_compare_is_false() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::list().add(ExpressionValue::integer(10)))
            .unwrap();
        instructions
            .put(ExpressionValue::list().add(ExpressionValue::integer(10)))
            .unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn unit_compare_is_false() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.perform_greater_than_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }
}

#[cfg(test)]
mod greater_than_or_equal_tests {
    use crate::ExpressionRuntime;
    use expr_lang_common::ExpressionValue;
    use expr_lang_instruction_set_builder::InstructionSetBuilder;

    #[test]
    fn integer_integer_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn integer_integer_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn integer_integer_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn integer_float_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::float(20.0)).unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn integer_float_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn integer_float_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn float_float_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.put(ExpressionValue::float(20.0)).unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn float_float_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(20.0)).unwrap();
        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn float_float_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn float_integer_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn float_integer_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(20.0)).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn float_integer_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::float(10.0)).unwrap();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_character_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("z".into()))
            .unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_character_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("z".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_character_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_list_character_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("z".into()))
            .unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_list_character_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("abc".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_list_character_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_character_list_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("abc".into()))
            .unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_character_list_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("z".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("a".into()))
            .unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_character_list_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("a".into()))
            .unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_character_unicode_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("🐼".into()))
            .unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_unicode_character_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character("🐼".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character("a".into()))
            .unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_list_character_list_same_length_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("cats".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("dogs".into()))
            .unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn character_list_character_list_same_length_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("dogs".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("cats".into()))
            .unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_list_character_list_same_length_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("cats".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("cats".into()))
            .unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn character_list_longer_character_list_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("cat".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("cats".into()))
            .unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn longer_character_list_character_list_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("cats".into()))
            .unwrap();
        instructions
            .put(ExpressionValue::character_list("cat".into()))
            .unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn symbol_symbol_same_length_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("dogs".to_string()))
            .unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn symbol_symbol_same_length_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::symbol("dogs".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn symbol_symbol_same_length_equal_to_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn symbol_longer_symbol_less_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::symbol("cat".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }

    #[test]
    fn longer_symbol_symbol_greater_than_other() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::symbol("cats".to_string()))
            .unwrap();
        instructions
            .put(ExpressionValue::symbol("cat".to_string()))
            .unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn range_compare() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::integer_range(Some(10), Some(20)))
            .unwrap();
        instructions
            .put(ExpressionValue::integer_range(Some(10), Some(20)))
            .unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn pair_compare() {
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
                ExpressionValue::symbol("cats"),
                ExpressionValue::integer(10),
            ))
            .unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn list_compare() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::list().add(ExpressionValue::integer(10)))
            .unwrap();
        instructions
            .put(ExpressionValue::list().add(ExpressionValue::integer(10)))
            .unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn unit_compare() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.perform_greater_than_or_equal_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }
}

#[cfg(test)]
mod type_comparison_tests {
    use crate::ExpressionRuntime;
    use expr_lang_common::ExpressionValue;
    use expr_lang_instruction_set_builder::InstructionSetBuilder;

    #[test]
    fn same_types_are_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.perform_type_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.true_value);
    }

    #[test]
    fn different_types_are_not_equal() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::float(20.0)).unwrap();
        instructions.perform_type_comparison();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_symbol().unwrap(), expression_runtime.false_value);
    }
}
