use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use garnish_common::{
    character_type_to_bytes, float_type_to_bytes, has_end, has_start, has_step,
    number_type_to_bytes, range_type_to_bytes, read_byte_size, read_float, read_integer, read_size,
    size_to_bytes, size_type_to_bytes, skip_byte_size, skip_size, skip_type, skip_type_and_2_sizes,
    skip_type_and_byte_size, three_size_type_to_bytes, two_size_type_to_bytes, two_sizes_to_bytes,
    CopyValue, DataType, ExpressionValue, ExpressionValueConsumer, Result,
};

use crate::runtime::{CallFrame, CallType, ExpressionRuntime};

pub fn insert_if_not_present(s: &str, map: &mut HashMap<String, usize>) -> usize {
    match map.get(s) {
        Some(v) => *v,
        None => {
            let v = map.len() + 1;
            map.insert(s.to_string(), v);
            v
        }
    }
}

impl ExpressionValueConsumer for ExpressionRuntime {
    fn insert_at_value_cursor(&mut self, data: u8) -> Result {
        if self.value_cursor >= self.data.len() {
            self.resize_data()?;
        }

        self.data[self.value_cursor] = data;
        self.value_cursor += 1;

        Ok(())
    }

    fn insert_all_at_value_cursor(&mut self, data: &[u8]) -> Result {
        if self.value_cursor + data.len() >= self.data.len() {
            self.resize_data()?;
        }

        self.data[self.value_cursor..(self.value_cursor + data.len())].clone_from_slice(data);
        self.value_cursor += data.len();

        Ok(())
    }

    fn insert_at_ref_cursor(&mut self, r: usize) {
        self.registers[self.ref_cursor] = r;
        self.ref_cursor += 1;
    }

    fn get_value_cursor(&self) -> usize {
        self.value_cursor
    }

    fn get_data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    fn get_symbol_table_mut(&mut self) -> &mut HashMap<String, usize> {
        &mut self.symbol_table
    }

    fn get_expression_table_mut(&mut self) -> &mut Vec<usize> {
        return &mut self.expression_table;
    }

    fn get_expression_map_mut(&mut self) -> &mut HashMap<String, usize> {
        return &mut self.expression_map;
    }
}

impl ExpressionRuntime {
    pub(crate) fn resize_data(&mut self) -> Result {
        // request double our current size
        let additional = self.data.capacity() * 2;
        self.data.resize(additional, 0);

        // also resize registers to keep ratio 5/1
        let additional = self.registers.capacity() * 2;
        self.registers.resize(additional, 0);
        Ok(())
    }

    pub(crate) fn get_expression_start(&self, index: usize) -> Result<usize> {
        // need to check index before subtraction
        // to avoid potential panic! from subtraction underflow if zero
        if index == 0 {
            return Err(format!(
                "Bad value '{}' for expression index. Out of bounds of expression table.",
                index
            )
            .into());
        }

        // Expressions are stored offset by one
        // so will subtract one from table index to find
        // and one from start location
        match self.expression_table.get(index - 1) {
            None => Err(format!(
                "Bad value '{}' for expression index. Out of bounds of expression table.",
                index
            )
            .into()),
            Some(start) => {
                if *start == 0 {
                    Err(format!("Expression at index {} has no start.", index).into())
                } else {
                    Ok(*start)
                }
            }
        }
    }

    pub(crate) fn read_size_at(&self, index: usize) -> Result<usize> {
        read_size(&self.data[index..])
    }

    pub(crate) fn read_byte_size_at(&self, index: usize) -> Result<u8> {
        read_byte_size(&self.data[index..])
    }

    pub(crate) fn read_integer_at(&self, index: usize) -> Result<i32> {
        read_integer(&self.data[index..])
    }

    pub(crate) fn read_float_at(&self, index: usize) -> Result<f32> {
        read_float(&self.data[index..])
    }

    pub(crate) fn read_char_at(&self, index: usize) -> Result<Option<char>> {
        let length = self.read_byte_size_at(index)?;
        let codes_start = skip_byte_size(index);

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
            _ => return Ok(None),
        };

        Ok(std::char::from_u32(read_integer(&bytes)? as u32))
    }

    pub(crate) fn current_input_ref(&self) -> usize {
        self.input_stack[self.input_stack.len() - 1]
    }

    pub(crate) fn current_instruction_cursor(&self) -> usize {
        return self.call_stack[self.call_stack.len() - 1].cursor;
    }

    pub(crate) fn get_value_ref(&self, index: usize) -> Result<usize> {
        let r = self.registers[index] as usize;
        let t = DataType::try_from(self.data[r])?;

        if t == DataType::Reference {
            self.read_size_at(r + 1)
        } else {
            Ok(self.registers[index] as usize)
        }
    }

    pub(crate) fn last_ref(&self) -> Result<usize> {
        if self.ref_cursor < 1 {
            Err("Not enough references on stack.".into())
        } else {
            Ok(self.get_value_ref(self.ref_cursor - 1)?)
        }
    }

    pub(crate) fn last_two_refs(&self) -> Result<(usize, usize)> {
        if self.ref_cursor < 2 {
            Err("Not enough references on stack.".into())
        } else {
            Ok((
                self.get_value_ref(self.ref_cursor - 2)?,
                self.get_value_ref(self.ref_cursor - 1)?,
            ))
        }
    }

    pub(crate) fn consume_last_ref(&mut self) -> Result<usize> {
        let r = self.last_ref()?;
        self.ref_cursor -= 1;

        Ok(r)
    }

    // retrieve references to top two values on register stack
    // and move the reference cursor back by two
    pub(crate) fn consume_last_two_refs(&mut self) -> Result<(usize, usize)> {
        let refs = self.last_two_refs()?;
        self.ref_cursor -= 2;
        Ok(refs)
    }

    // move value cursor back to where ref stack was pointing to left
    pub(crate) fn set_value_cursor_to_ref_cursor(&mut self) {
        self.value_cursor = self.registers[self.ref_cursor];
    }

    pub(crate) fn consume_last_two_refs_and_set_value_cursor(&mut self) -> Result<(usize, usize)> {
        let refs = self.consume_last_two_refs()?;
        self.set_value_cursor_to_ref_cursor();
        Ok(refs)
    }

    pub(crate) fn consume_constant_reference(&mut self) -> Result<usize> {
        let value = read_size(&self.instructions[(self.current_instruction_cursor() + 1)..])?;
        let last = self.call_stack.len() - 1;
        self.call_stack[last].cursor = skip_size(self.call_stack[last].cursor);

        Ok(value)
    }

    pub(crate) fn write_size_at(&mut self, index: usize, size: usize) -> Result {
        let sizes = size_to_bytes(size);
        if index + sizes.len() >= self.data.len() {
            println!("data size {}", sizes.len());
            self.resize_data()?;
        }

        self.data[index..(index + sizes.len())].clone_from_slice(&sizes);
        Ok(())
    }

    pub(crate) fn write_two_sizes_at(&mut self, index: usize, size1: usize, size2: usize) {
        let sizes = two_sizes_to_bytes(size1, size2);
        self.data[index..(index + sizes.len())].clone_from_slice(&sizes);
    }

    pub(crate) fn insert_unit(&mut self) -> Result {
        self.insert_data_value(&[DataType::Unit.try_into().unwrap()])
    }

    pub(crate) fn insert_integer(&mut self, num: i32) -> Result {
        self.insert_data_value(&number_type_to_bytes(num))
    }

    pub(crate) fn insert_float(&mut self, num: f32) -> Result {
        self.insert_data_value(&float_type_to_bytes(num))
    }

    pub(crate) fn insert_character(&mut self, char: char) -> Result {
        self.insert_data_value(&character_type_to_bytes(&char.to_string()))
    }

    pub(crate) fn insert_reference_value(&mut self, ref_value: usize) -> Result {
        self.insert_size_value(DataType::Reference, ref_value)
    }

    pub(crate) fn insert_symbol_value(&mut self, symbol_value: usize) -> Result {
        self.insert_size_value(DataType::Symbol, symbol_value)
    }

    pub(crate) fn insert_list_value(&mut self, length: usize, key_count: usize) -> Result {
        // ideally use below line need to see if common ref cursor
        // manipulation can be used
        // self.insert_two_size_value(DataType::List, length, key_count);
        self.insert_all_at_value_cursor(&two_size_type_to_bytes(DataType::List, length, key_count))
    }

    pub(crate) fn insert_range_value(
        &mut self,
        flags: u8,
        start: Option<usize>,
        end: Option<usize>,
        step: Option<usize>,
    ) -> Result {
        self.insert_data_value(&range_type_to_bytes(flags, start, end, step))
    }

    pub(crate) fn insert_pair_value(&mut self, left: usize, right: usize) -> Result {
        self.insert_two_size_value(DataType::Pair, left, right)
    }

    pub(crate) fn insert_slice_value(&mut self, left: usize, right: usize) -> Result {
        self.insert_two_size_value(DataType::Slice, left, right)
    }

    pub(crate) fn insert_link_value(&mut self, head: usize, value: usize, next: usize) -> Result {
        self.insert_data_value(&three_size_type_to_bytes(DataType::Link, head, value, next))
    }

    pub(crate) fn edit_link_next_value(&mut self, link_ref: usize, value: usize) -> Result {
        let value_bytes = size_to_bytes(value);
        let link_value_start = skip_type_and_2_sizes(link_ref);
        let link_value_end = skip_size(link_value_start);

        self.data[link_value_start..link_value_end].clone_from_slice(&value_bytes);

        Ok(())
    }

    pub(crate) fn get_range_values_at(
        &self,
        pos: usize,
    ) -> Result<(u8, Option<usize>, Option<usize>, Option<usize>)> {
        let range_flags = self.read_byte_size_at(skip_type(pos))?;

        let mut start_ref = None;
        let mut end_ref = None;
        let mut step_ref = None;

        let mut value_cursor = skip_type_and_byte_size(pos);

        if has_start(range_flags) {
            start_ref = Some(self.read_size_at(value_cursor)?);
            value_cursor = skip_size(value_cursor);
        }

        if has_end(range_flags) {
            end_ref = Some(self.read_size_at(value_cursor)?);
            value_cursor = skip_size(value_cursor);
        }

        if has_step(range_flags) {
            step_ref = Some(self.read_size_at(value_cursor)?);
        }

        Ok((range_flags, start_ref, end_ref, step_ref))
    }

    pub(crate) fn insert_partial_value(&mut self, left: usize, right: usize) -> Result {
        self.insert_two_size_value(DataType::Partial, left, right)
    }

    fn insert_two_size_value(
        &mut self,
        data_type: DataType,
        size_value1: usize,
        size_value2: usize,
    ) -> Result {
        self.insert_data_value(&two_size_type_to_bytes(data_type, size_value1, size_value2))
    }

    fn insert_size_value(&mut self, data_type: DataType, size_value: usize) -> Result {
        self.insert_data_value(&size_type_to_bytes(data_type, size_value))
    }

    pub(crate) fn insert_data_value(&mut self, data: &[u8]) -> Result {
        if self.value_cursor + data.len() >= self.data.len() {
            println!("data size {}", data.len());
            self.resize_data()?;
        }

        self.data[self.value_cursor..(self.value_cursor + data.len())].clone_from_slice(data);
        self.insert_at_ref_cursor(self.value_cursor);
        self.value_cursor += data.len();

        Ok(())
    }

    pub(crate) fn copy_into(&mut self, value: &ExpressionValue) -> Result {
        self.copy_value(value)
    }

    pub(crate) fn push_frame(&mut self, start: usize) {
        self.add_call_frame(start, CallType::Normal)
    }

    pub(crate) fn push_conditional_frame(&mut self, start: usize) {
        self.add_call_frame(start, CallType::Conditional)
    }

    pub(crate) fn push_expression_iteration_frame(&mut self, start: usize) {
        self.add_call_frame(start, CallType::ExpressionIteration)
    }

    pub(crate) fn push_iteration_frame(&mut self, start: usize) {
        self.add_call_frame(start, CallType::Iteration)
    }

    fn add_call_frame(&mut self, start: usize, call_type: CallType) {
        self.call_stack.push(CallFrame {
            result_start: self.result_stack.len(),
            cursor: start - 1,
            call_type: call_type,
        });
    }
}

#[cfg(test)]
mod tests {
    use garnish_common::{DataType, DataVecWriter, ExpressionValue};
    use garnish_instruction_set_builder::InstructionSetBuilder;

    use crate::runtime::tests::data_slice;
    use crate::runtime::ExpressionRuntime;

    #[test]
    fn copy_into_copies_unit_value() {
        let instructions = InstructionSetBuilder::new();
        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        expression_runtime
            .copy_into(&ExpressionValue::unit().into())
            .unwrap();

        let result = data_slice(&expression_runtime, 1);
        let expected = DataVecWriter::write_with(|w| w.push_data_type(DataType::Unit));

        assert_eq!(result, expected);
        assert_eq!(
            expression_runtime.registers[expression_runtime.ref_cursor - 1],
            1
        );
    }

    #[test]
    fn copy_into_copies_number_value() {
        let instructions = InstructionSetBuilder::new();
        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        expression_runtime
            .copy_into(&ExpressionValue::integer(10).into())
            .unwrap();

        let expected =
            DataVecWriter::write_with(|w| w.push_data_type(DataType::Integer).push_integer(10));
        let result = data_slice(&expression_runtime, expected.len());

        assert_eq!(result, expected);
        assert_eq!(
            expression_runtime.registers[expression_runtime.ref_cursor - 1],
            1
        );
    }

    #[test]
    fn copy_into_copies_character_list_value() {
        let instructions = InstructionSetBuilder::new();
        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        expression_runtime
            .copy_into(&ExpressionValue::character_list("bear".into()).into())
            .unwrap();

        let expected = DataVecWriter::write_with(|w| {
            w.push_data_type(DataType::Character)
                .push_byte_size(1)
                .push_character("b")
                .push_data_type(DataType::Character)
                .push_byte_size(1)
                .push_character("e")
                .push_data_type(DataType::Character)
                .push_byte_size(1)
                .push_character("a")
                .push_data_type(DataType::Character)
                .push_byte_size(1)
                .push_character("r")
                .push_data_type(DataType::CharacterList)
                .push_size(4)
                .push_size(1)
                .push_size(4)
                .push_size(7)
                .push_size(10)
        });
        let result = data_slice(&expression_runtime, expected.len());

        assert_eq!(result, expected);
        assert_eq!(
            expression_runtime.registers[expression_runtime.ref_cursor - 1],
            13
        );
    }

    #[test]
    fn copy_into_copies_multiple_character_list_values() {
        let instructions = InstructionSetBuilder::new();
        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        expression_runtime
            .copy_into(&ExpressionValue::character_list("cat".into()).into())
            .unwrap();
        expression_runtime
            .copy_into(&ExpressionValue::character_list("bear".into()).into())
            .unwrap();

        let expected = DataVecWriter::write_with(|w| {
            w.push_data_type(DataType::Character)
                .push_byte_size(1)
                .push_character("c")
                .push_data_type(DataType::Character)
                .push_byte_size(1)
                .push_character("a")
                .push_data_type(DataType::Character)
                .push_byte_size(1)
                .push_character("t")
                .push_data_type(DataType::CharacterList) // 9
                .push_size(3) // 10
                .push_size(1) // 14
                .push_size(4) // 18
                .push_size(7) // 22
                .push_data_type(DataType::Character) // 26
                .push_byte_size(1)
                .push_character("b")
                .push_data_type(DataType::Character) // 29
                .push_byte_size(1)
                .push_character("e")
                .push_data_type(DataType::Character) // 32
                .push_byte_size(1)
                .push_character("a")
                .push_data_type(DataType::Character) // 35
                .push_byte_size(1)
                .push_character("r")
                .push_data_type(DataType::CharacterList) // 38
                .push_size(4)
                .push_size(27)
                .push_size(30)
                .push_size(33)
                .push_size(36)
        });

        let result = data_slice(&expression_runtime, expected.len());

        assert_eq!(result, expected);
        assert_eq!(
            expression_runtime.registers[expression_runtime.ref_cursor - 1],
            39
        );
        assert_eq!(
            expression_runtime.registers[expression_runtime.ref_cursor - 2],
            10
        );
    }

    #[test]
    fn copy_into_copies_symbol_value() {
        let instructions = InstructionSetBuilder::new();
        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        expression_runtime
            .copy_into(&ExpressionValue::symbol("bear").into())
            .unwrap();

        let expected =
            DataVecWriter::write_with(|w| w.push_data_type(DataType::Symbol).push_size(19));
        let result = data_slice(&expression_runtime, expected.len());

        assert_eq!(result, expected);
        assert_eq!(*expression_runtime.symbol_table.get("bear").unwrap(), 19);
        assert_eq!(
            expression_runtime.registers[expression_runtime.ref_cursor - 1],
            1
        );
    }

    #[test]
    fn copy_into_copies_pair_value() {
        let instructions = InstructionSetBuilder::new();
        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        expression_runtime
            .copy_into(
                &ExpressionValue::pair(
                    ExpressionValue::symbol("bear"),
                    ExpressionValue::integer(10),
                )
                .into(),
            )
            .unwrap();

        let expected = DataVecWriter::write_with(|w| {
            w.push_data_type(DataType::Symbol)
                .push_size(19)
                .push_data_type(DataType::Integer)
                .push_integer(10)
                .push_data_type(DataType::Pair)
                .push_size(1)
                .push_size(6)
        });
        let result = data_slice(&expression_runtime, expected.len());

        assert_eq!(result, expected);
        assert_eq!(*expression_runtime.symbol_table.get("bear").unwrap(), 19);
        assert_eq!(
            expression_runtime.registers[expression_runtime.ref_cursor - 1],
            11
        );
    }

    #[test]
    fn updates_pair_references() {
        let instructions = InstructionSetBuilder::new();
        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        expression_runtime
            .copy_into(&ExpressionValue::integer(5).into())
            .unwrap();

        expression_runtime
            .copy_into(
                &ExpressionValue::pair(
                    ExpressionValue::symbol("bear"),
                    ExpressionValue::integer(10),
                )
                .into(),
            )
            .unwrap();

        let expected = DataVecWriter::write_with(|w| {
            w.push_data_type(DataType::Integer)
                .push_integer(5)
                .push_data_type(DataType::Symbol)
                .push_size(19)
                .push_data_type(DataType::Integer)
                .push_integer(10)
                .push_data_type(DataType::Pair)
                .push_size(6)
                .push_size(11)
        });
        let result = data_slice(&expression_runtime, expected.len());

        assert_eq!(result, expected);
        assert_eq!(*expression_runtime.symbol_table.get("bear").unwrap(), 19);
        assert_eq!(
            expression_runtime.registers[expression_runtime.ref_cursor - 1],
            16
        );
    }

    #[test]
    fn copies_list() {
        let instructions = InstructionSetBuilder::new();
        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        expression_runtime
            .copy_into(
                &ExpressionValue::list()
                    .add(ExpressionValue::integer(10))
                    .add(ExpressionValue::integer(20))
                    .add(ExpressionValue::integer(30))
                    .into(),
            )
            .unwrap();

        let expected = DataVecWriter::write_with(|w| {
            w.push_data_type(DataType::Integer)
                .push_integer(10)
                .push_data_type(DataType::Integer)
                .push_size(20)
                .push_data_type(DataType::Integer)
                .push_integer(30)
                .push_data_type(DataType::List)
                .push_size(3)
                .push_size(0)
                .push_size(1)
                .push_size(6)
                .push_size(11)
        });

        let result = data_slice(&expression_runtime, expected.len());

        assert_eq!(result, expected);
        assert_eq!(
            expression_runtime.registers[expression_runtime.ref_cursor - 1],
            16
        );
    }

    #[test]
    fn copies_list_updates_references() {
        let instructions = InstructionSetBuilder::new();
        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        expression_runtime
            .copy_into(&ExpressionValue::integer(5).into())
            .unwrap();

        expression_runtime
            .copy_into(
                &ExpressionValue::list()
                    .add(ExpressionValue::integer(10))
                    .add(ExpressionValue::integer(20))
                    .add(ExpressionValue::integer(30))
                    .into(),
            )
            .unwrap();

        let expected = DataVecWriter::write_with(|w| {
            w.push_data_type(DataType::Integer)
                .push_integer(5)
                .push_data_type(DataType::Integer)
                .push_integer(10)
                .push_data_type(DataType::Integer)
                .push_size(20)
                .push_data_type(DataType::Integer)
                .push_integer(30)
                .push_data_type(DataType::List)
                .push_size(3)
                .push_size(0)
                .push_size(6)
                .push_size(11)
                .push_size(16)
        });
        let result = data_slice(&expression_runtime, expected.len());

        assert_eq!(result, expected);
        assert_eq!(
            expression_runtime.registers[expression_runtime.ref_cursor - 1],
            21
        );
    }

    #[test]
    fn copy_into_copies_partial_value() {
        let instructions = InstructionSetBuilder::new();
        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        expression_runtime
            .copy_into(
                &ExpressionValue::partial_expression("my_expression", ExpressionValue::integer(10))
                    .into(),
            )
            .unwrap();

        let expected = DataVecWriter::write_with(|w| {
            w.push_data_type(DataType::Expression)
                .push_size(1)
                .push_data_type(DataType::Integer)
                .push_integer(10)
                .push_data_type(DataType::Partial)
                .push_size(1)
                .push_size(6)
        });
        let result = data_slice(&expression_runtime, expected.len());

        assert_eq!(result, expected);
        assert_eq!(
            expression_runtime.registers[expression_runtime.ref_cursor - 1],
            11
        );
    }

    #[test]
    fn updates_partial_references() {
        let instructions = InstructionSetBuilder::new();
        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        expression_runtime
            .copy_into(&ExpressionValue::integer(5).into())
            .unwrap();

        expression_runtime
            .copy_into(
                &ExpressionValue::partial_expression("my_expression", ExpressionValue::integer(10))
                    .into(),
            )
            .unwrap();

        let expected = DataVecWriter::write_with(|w| {
            w.push_data_type(DataType::Integer)
                .push_integer(5)
                .push_data_type(DataType::Expression)
                .push_size(1)
                .push_data_type(DataType::Integer)
                .push_integer(10)
                .push_data_type(DataType::Partial)
                .push_size(6)
                .push_size(11)
        });
        let result = data_slice(&expression_runtime, expected.len());

        assert_eq!(result, expected);
        assert_eq!(
            expression_runtime.registers[expression_runtime.ref_cursor - 1],
            16
        );
    }

    #[test]
    fn copies_associative_list() {
        let instructions = InstructionSetBuilder::new();
        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        expression_runtime
            .copy_into(
                &ExpressionValue::list()
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
                    ))
                    .into(),
            )
            .unwrap();

        let expected = DataVecWriter::write_with(|w| {
            w.push_data_type(DataType::Symbol) // 0
                .push_size(19)
                .push_data_type(DataType::Integer) // 5
                .push_integer(10)
                .push_data_type(DataType::Pair) // 10
                .push_size(1)
                .push_size(6)
                .push_data_type(DataType::Symbol) // 19
                .push_size(20)
                .push_data_type(DataType::Integer) // 24
                .push_size(20)
                .push_data_type(DataType::Pair) // 29
                .push_size(20)
                .push_size(25)
                .push_data_type(DataType::Symbol) // 38
                .push_size(21)
                .push_data_type(DataType::Integer) // 43
                .push_integer(30)
                .push_data_type(DataType::Pair) // 48
                .push_size(39)
                .push_size(44)
                .push_data_type(DataType::List) // 57
                .push_size(3)
                .push_size(3)
                .push_size(11)
                .push_size(30)
                .push_size(49)
                .push_size(49)
                .push_size(11)
                .push_size(30)
        });

        let result = data_slice(&expression_runtime, expected.len());

        assert_eq!(result, expected);
        assert_eq!(
            expression_runtime.registers[expression_runtime.ref_cursor - 1],
            58
        );
    }

    #[test]
    fn copies_associative_list_updates_references() {
        let instructions = InstructionSetBuilder::new();
        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        expression_runtime
            .copy_into(&ExpressionValue::integer(5).into())
            .unwrap();

        expression_runtime
            .copy_into(
                &ExpressionValue::list()
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
                    ))
                    .into(),
            )
            .unwrap();

        let expected = DataVecWriter::write_with(|w| {
            w.push_data_type(DataType::Integer)
                .push_integer(5)
                .push_data_type(DataType::Symbol) // 5
                .push_size(19)
                .push_data_type(DataType::Integer) // 10
                .push_integer(10)
                .push_data_type(DataType::Pair) // 15
                .push_size(6)
                .push_size(11)
                .push_data_type(DataType::Symbol) // 24
                .push_size(20)
                .push_data_type(DataType::Integer) // 29
                .push_size(20)
                .push_data_type(DataType::Pair) // 34
                .push_size(25)
                .push_size(30)
                .push_data_type(DataType::Symbol) // 43
                .push_size(21)
                .push_data_type(DataType::Integer) // 48
                .push_integer(30)
                .push_data_type(DataType::Pair) // 53
                .push_size(44)
                .push_size(49)
                .push_data_type(DataType::List) // 62
                .push_size(3)
                .push_size(3)
                .push_size(16)
                .push_size(35)
                .push_size(54)
                .push_size(54)
                .push_size(16)
                .push_size(35)
        });

        let result = data_slice(&expression_runtime, expected.len());

        assert_eq!(result, expected);
        assert_eq!(
            expression_runtime.registers[expression_runtime.ref_cursor - 1],
            63
        );
    }
}

#[cfg(test)]
mod memory_tests {
    use garnish_common::{DataType, DataVecWriter, ExpressionValue};
    use garnish_instruction_set_builder::InstructionSetBuilder;

    use crate::runtime::tests::data_slice;
    use crate::runtime::ExpressionRuntime;

    #[test]
    fn memory_resizes() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("main");
        instructions.put(ExpressionValue::integer(0)).unwrap();
        instructions.put(ExpressionValue::integer(100000)).unwrap();
        instructions.make_inclusive_range();
        instructions.put(ExpressionValue::expression("iterate")).unwrap();
        instructions.iterate();
        instructions.end_expression();
        
        instructions.start_expression("iterate");
        instructions.put_input();
        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main");

        assert!(result.is_ok());
    }
}

