use std::collections::HashMap;
use std::convert::TryInto;

use expr_lang_common::{
    CopyValue, DataType, DataVecWriter, ExpressionValue, ExpressionValueBuilder,
    ExpressionValueConsumer, Instruction, InstructionSet, Result,
};

#[derive(Eq, PartialEq, Debug)]
pub struct InstructionSetBuilder {
    instructions: Vec<u8>,
    expression_map: HashMap<String, usize>,
    expression_table: Vec<usize>,
    symbol_table: HashMap<String, usize>,
    value_cursor: usize,
    data: Vec<u8>,
    data_refs: Vec<usize>,
}

impl InstructionSetBuilder {
    pub fn new() -> Self {
        let mut constant_data: Vec<u8> = vec![];
        // Since unit only has one value, push a single one to start
        // all put units will reference it
        constant_data.push(DataType::Unit.try_into().unwrap());

        let mut symbol_table = HashMap::new();
        symbol_table.insert("".to_string(), 0);

        let instructions = vec![0];

        return InstructionSetBuilder {
            instructions,
            expression_map: HashMap::new(),
            expression_table: vec![],
            symbol_table,
            data: constant_data,
            data_refs: vec![],
            value_cursor: 1,
        };
    }

    pub fn put<T>(&mut self, value: T) -> Result<()>
    where
        T: ExpressionValueBuilder,
    {
        let v: ExpressionValue = value.into();
        let len = v.get_data().len();
        // make sure our data has enough space before copy
        for _i in 0..len {
            self.data.push(0);
        }

        self.copy_value(&v)?;
        // each value only produces a single ref, simple pop from vec
        // ref should exist, potential panic from unwrap is appropriate
        let r = self.data_refs.pop().unwrap();

        DataVecWriter::new(&mut self.instructions)
            .push_instruction(Instruction::Put)
            .push_size(r);

        Ok(())
    }

    pub fn make_pair(&mut self) {
        self.add(Instruction::MakePair)
    }

    pub fn make_link(&mut self) {
        self.add(Instruction::MakeLink)
    }

    pub fn make_inclusive_range(&mut self) {
        self.add(Instruction::MakeInclusiveRange);
    }

    pub fn make_exclusive_range(&mut self) {
        self.add(Instruction::MakeExclusiveRange);
    }

    pub fn make_start_exclusive_range(&mut self) {
        self.add(Instruction::MakeStartExclusiveRange);
    }

    pub fn make_end_exclusive_range(&mut self) {
        self.add(Instruction::MakeEndExclusiveRange);
    }

    pub fn start_list(&mut self) {
        self.add(Instruction::StartList);
    }

    pub fn make_list(&mut self) {
        self.add(Instruction::MakeList);
    }

    pub fn perform_addition(&mut self) {
        self.add(Instruction::PerformAddition);
    }

    pub fn perform_subtraction(&mut self) {
        self.add(Instruction::PerformSubtraction);
    }

    pub fn perform_multiplication(&mut self) {
        self.add(Instruction::PerformMultiplication);
    }

    pub fn perform_division(&mut self) {
        self.add(Instruction::PerformDivision);
    }

    pub fn perform_integer_division(&mut self) {
        self.add(Instruction::PerformIntegerDivision);
    }

    pub fn perform_remainder(&mut self) {
        self.add(Instruction::PerformRemainder);
    }

    pub fn perform_exponential(&mut self) {
        self.add(Instruction::PerformExponential);
    }

    pub fn perform_negation(&mut self) {
        self.add(Instruction::PerformNegation);
    }

    pub fn perform_absolute_value(&mut self) {
        self.add(Instruction::PerformAbsoluteValue);
    }

    pub fn perform_bitwise_and(&mut self) {
        self.add(Instruction::PerformBitwiseAnd);
    }

    pub fn perform_bitwise_or(&mut self) {
        self.add(Instruction::PerformBitwiseOr);
    }

    pub fn perform_bitwise_xor(&mut self) {
        self.add(Instruction::PerformBitwiseXor);
    }

    pub fn perform_bitwise_not(&mut self) {
        self.add(Instruction::PerformBitwiseNot);
    }

    pub fn perform_bitwise_left_shift(&mut self) {
        self.add(Instruction::PerformBitwiseLeftShift);
    }

    pub fn perform_bitwise_right_shift(&mut self) {
        self.add(Instruction::PerformBitwiseRightShift);
    }

    pub fn perform_logical_and(&mut self) {
        self.add(Instruction::PerformLogicalAND);
    }

    pub fn perform_logical_or(&mut self) {
        self.add(Instruction::PerformLogicalOR);
    }

    pub fn perform_logical_xor(&mut self) {
        self.add(Instruction::PerformLogicalXOR);
    }

    pub fn perform_logical_not(&mut self) {
        self.add(Instruction::PerformLogicalNOT);
    }

    pub fn perform_type_cast(&mut self) {
        self.add(Instruction::PerformTypeCast);
    }

    pub fn perform_equality_comparison(&mut self) {
        self.add(Instruction::PerformEqualityComparison);
    }

    pub fn perform_inequality_comparison(&mut self) {
        self.add(Instruction::PerformInequalityComparison);
    }

    pub fn perform_less_than_comparison(&mut self) {
        self.add(Instruction::PerformLessThanComparison);
    }

    pub fn perform_less_than_or_equal_comparison(&mut self) {
        self.add(Instruction::PerformLessThanOrEqualComparison);
    }

    pub fn perform_greater_than_comparison(&mut self) {
        self.add(Instruction::PerformGreaterThanComparison);
    }

    pub fn perform_greater_than_or_equal_comparison(&mut self) {
        self.add(Instruction::PerformGreaterThanOrEqualComparison);
    }

    pub fn perform_type_comparison(&mut self) {
        self.add(Instruction::PerformTypeComparison);
    }

    pub fn perform_access(&mut self) {
        self.add(Instruction::PerformAccess);
    }

    pub fn apply(&mut self) {
        self.add(Instruction::Apply);
    }

    pub fn partially_apply(&mut self) {
        self.add(Instruction::PartiallyApply);
    }

    pub fn start_expression<T>(&mut self, name: T)
    where
        T: ToString,
    {
        let n = name.to_string();

        let pos = self.instructions.len();
        let index = self.expression_table.len() + 1;

        // make sure expression name is also in symbol table
        self.insert_symbol_if_not_present(&n);

        // if name is already in expression map
        // update position that is stored in expression table
        // else insert new record into expression map and table
        match self.expression_map.get(&name.to_string()) {
            None => {
                // offset pos by 1 so 0 can be an invalid value
                self.expression_table.push(pos);
                self.expression_map.insert(name.to_string(), index);
            }
            Some(table_index) => {
                self.expression_table[(*table_index) - 1] = pos;
            }
        }
    }

    pub fn end_expression(&mut self) {
        self.add(Instruction::EndExpression);
    }

    pub fn execute_expression<T>(&mut self, name: T)
    where
        T: ToString,
    {
        let n = name.to_string();
        // make sure expression name is also in symbol table
        self.insert_symbol_if_not_present(&n);

        let index = self.insert_expression_if_not_present(&n);

        self.add(Instruction::ExecuteExpression);
        DataVecWriter::new(&mut self.instructions).push_size(index);
    }

    pub fn put_input(&mut self) {
        self.add(Instruction::PutInput);
    }

    pub fn push_input(&mut self) {
        self.add(Instruction::PushInput);
    }

    pub fn push_unit_input(&mut self) {
        self.add(Instruction::PushUnitInput);
    }

    pub fn put_result(&mut self) {
        self.add(Instruction::PutResult);
    }

    pub fn output_result(&mut self) {
        self.add(Instruction::OutputResult);
    }

    pub fn resolve(&mut self, name: &String) -> Result<()> {
        self.add(Instruction::Resolve);

        let symbol_index = self.insert_symbol_if_not_present(name);

        DataVecWriter::new(&mut self.instructions).push_size(symbol_index);

        let v: ExpressionValue = ExpressionValue::symbol(symbol_index).into();
        let len = v.get_data().len();

        // make sure our data has enough space before copy
        for _i in 0..len {
            self.data.push(0);
        }

        self.copy_value(&v)?;

        Ok(())
    }

    pub fn invoke(&mut self) {
        self.add(Instruction::Invoke);
    }

    pub fn conditional_execute(&mut self, if_true: Option<String>, if_false: Option<String>) {
        let if_true_index = self.index_of_expression_name(if_true);
        let if_false_index = self.index_of_expression_name(if_false);

        self.add(Instruction::ConditionalExecute);
        DataVecWriter::new(&mut self.instructions)
            .push_size(if_true_index)
            .push_size(if_false_index);
    }

    pub fn result_conditional_execute(&mut self, name: String, if_false: Option<String>) {
        self.add(Instruction::ResultConditionalExecute);

        let expr_index = self.insert_expression_if_not_present(&name);
        self.insert_symbol_if_not_present(&name);

        let if_false_index = self.index_of_expression_name(if_false);

        DataVecWriter::new(&mut self.instructions)
            .push_size(expr_index)
            .push_size(if_false_index);
    }

    pub fn iterate(&mut self) {
        self.add(Instruction::Iterate);
    }

    pub fn iterate_to_single_value(&mut self) {
        self.add(Instruction::IterateToSingleResult);
    }

    pub fn iteration_output(&mut self) {
        self.add(Instruction::IterationOutput);
    }

    pub fn iteration_continue(&mut self) {
        self.add(Instruction::IterationContinue);
    }

    pub fn iteration_skip(&mut self) {
        self.add(Instruction::IterationSkip);
    }

    pub fn iteration_complete(&mut self) {
        self.add(Instruction::IterationComplete);
    }

    pub fn reiterate(&mut self) {
        self.add(Instruction::Reiterate);
    }

    fn add(&mut self, instruction: Instruction) {
        self.instructions.push(instruction.try_into().unwrap());
    }

    fn index_of_expression_name(&mut self, name: Option<String>) -> usize {
        match name {
            None => 0,
            Some(s) => {
                self.insert_symbol_if_not_present(&s);
                self.insert_expression_if_not_present(&s)
            }
        }
    }

    fn insert_symbol_if_not_present(&mut self, name: &String) -> usize {
        match self.symbol_table.get(name) {
            Some(i) => *i,
            None => {
                let value = self.symbol_table.len();
                self.symbol_table.insert(name.clone(), value);

                value
            }
        }
    }

    fn insert_expression_if_not_present(&mut self, name: &String) -> usize {
        match self.expression_map.get(name) {
            // return existing position
            Some(position) => *position,
            None => {
                // insert place holder that to be updated in start expression
                let index = self.expression_table.len() + 1;
                self.expression_table.push(0);
                self.expression_map.insert(name.clone(), index);

                index
            }
        }
    }
}

impl ExpressionValueConsumer for InstructionSetBuilder {
    fn insert_at_value_cursor(&mut self, data: u8) -> Result<()> {
        if self.value_cursor >= self.data.len() {
            return Err("Out of memory".into());
        }

        self.data[self.value_cursor] = data;
        self.value_cursor += 1;

        Ok(())
    }

    fn insert_all_at_value_cursor(&mut self, data: &[u8]) -> Result<()> {
        if self.value_cursor + data.len() >= self.data.len() + 1 {
            return Err("Out of memory".into());
        }

        let range = self.value_cursor..(self.value_cursor + data.len());
        self.data[range].clone_from_slice(data);
        self.value_cursor += data.len();

        Ok(())
    }

    fn insert_at_ref_cursor(&mut self, r: usize) {
        self.data_refs.push(r);
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

impl InstructionSet for InstructionSetBuilder {
    fn get_data(&self) -> &Vec<u8> {
        return &self.data;
    }

    fn get_instructions(&self) -> &Vec<u8> {
        return &self.instructions;
    }

    fn get_expression_table(&self) -> &Vec<usize> {
        return &self.expression_table;
    }

    fn get_expression_map(&self) -> &HashMap<String, usize> {
        return &self.expression_map;
    }

    fn get_symbol_table(&self) -> &HashMap<String, usize> {
        return &self.symbol_table;
    }
}

impl<'a> InstructionSet for &'a InstructionSetBuilder {
    fn get_data(&self) -> &Vec<u8> {
        return &self.data;
    }

    fn get_instructions(&self) -> &Vec<u8> {
        return &self.instructions;
    }

    fn get_expression_table(&self) -> &Vec<usize> {
        return &self.expression_table;
    }

    fn get_expression_map(&self) -> &HashMap<String, usize> {
        return &self.expression_map;
    }

    fn get_symbol_table(&self) -> &HashMap<String, usize> {
        return &self.symbol_table;
    }
}

#[cfg(test)]
mod tests {
    use expr_lang_common::{
        DataType, DataVecWriter, ExpressionValue, Instruction, RANGE_END_EXCLUSIVE, RANGE_HAS_STEP,
        RANGE_OPEN_END, RANGE_OPEN_START,
    };

    use crate::InstructionSetBuilder;

    fn write_data(f: fn(DataVecWriter) -> DataVecWriter) -> Vec<u8> {
        DataVecWriter::write_with(|w| f(w.push_data_type(DataType::Unit)))
    }

    fn write_instructions(f: fn(DataVecWriter) -> DataVecWriter) -> Vec<u8> {
        DataVecWriter::write_with(|w| f(w.push_byte_size(0)))
    }

    #[test]
    fn put_unit() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.put(ExpressionValue::unit()).unwrap();

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w.push_instruction(Instruction::Put).push_size(1))
        );

        assert_eq!(
            instructions.data,
            write_data(|w| w.push_data_type(DataType::Unit))
        );
    }

    #[test]
    fn put_integer() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.put(ExpressionValue::integer(1000)).unwrap();

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w.push_instruction(Instruction::Put).push_size(1))
        );

        assert_eq!(
            instructions.data,
            write_data(|w| w.push_data_type(DataType::Integer).push_integer(1000))
        );
    }

    #[test]
    fn put_float() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.put(ExpressionValue::float(3.14)).unwrap();

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w.push_instruction(Instruction::Put).push_size(1))
        );

        assert_eq!(
            instructions.data,
            write_data(|w| w.push_data_type(DataType::Float).push_float(3.14))
        );
    }

    #[test]
    fn put_character() {
        let mut instructions = InstructionSetBuilder::new();
        instructions
            .put(ExpressionValue::character("🐼".into()))
            .unwrap();

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w.push_instruction(Instruction::Put).push_size(1))
        );

        assert_eq!(
            instructions.data,
            write_data(|w| w
                .push_data_type(DataType::Character)
                .push_byte_size(4)
                .push_character("🐼"))
        );
    }

    #[test]
    fn put_character_list() {
        let mut instructions = InstructionSetBuilder::new();
        instructions
            .put(ExpressionValue::character_list("z🐼ö̲".into()))
            .unwrap();

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w.push_instruction(Instruction::Put).push_size(17))
        );

        assert_eq!(
            instructions.data,
            write_data(|w| w
                .push_data_type(DataType::Character) // 0
                .push_byte_size(1) // 1
                .push_character("z") // 2
                .push_data_type(DataType::Character) // 3
                .push_byte_size(4) // 4
                .push_character("🐼") // 5
                .push_data_type(DataType::Character) // 9
                .push_byte_size(5) // 10
                .push_character("ö̲") // 11
                .push_data_type(DataType::CharacterList) // 16
                .push_size(3)
                .push_size(1)
                .push_size(4)
                .push_size(10))
        );
    }

    #[test]
    fn put_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.put(ExpressionValue::symbol("pandas")).unwrap();

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w.push_instruction(Instruction::Put).push_size(1))
        );
        assert_eq!(
            instructions.data,
            write_data(|w| w.push_data_type(DataType::Symbol).push_size(1))
        );
        assert_eq!(*instructions.symbol_table.get("pandas").unwrap(), 1);
    }

    #[test]
    fn put_empty_symbol() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.put(ExpressionValue::symbol("")).unwrap();

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w.push_instruction(Instruction::Put).push_size(1))
        );
        assert_eq!(
            instructions.data,
            write_data(|w| w.push_data_type(DataType::Symbol).push_size(0))
        );
        assert_eq!(*instructions.symbol_table.get("").unwrap(), 0);
    }

    #[test]
    fn put_range() {
        let mut instructions = InstructionSetBuilder::new();
        instructions
            .put(ExpressionValue::integer_range(Some(10), Some(20)).exclude_end())
            .unwrap();

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w.push_instruction(Instruction::Put).push_size(11))
        );
        assert_eq!(
            instructions.data,
            write_data(|w| w
                .push_data_type(DataType::Integer)
                .push_integer(10)
                .push_data_type(DataType::Integer)
                .push_integer(20)
                .push_data_type(DataType::Range)
                .push_byte_size(RANGE_END_EXCLUSIVE)
                .push_size(1)
                .push_size(6))
        );
    }

    #[test]
    fn put_range_with_step() {
        let mut instructions = InstructionSetBuilder::new();
        instructions
            .put(
                ExpressionValue::integer_range(Some(10), Some(20))
                    .exclude_end()
                    .with_step(ExpressionValue::integer(2)),
            )
            .unwrap();

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w.push_instruction(Instruction::Put).push_size(16))
        );
        assert_eq!(
            instructions.data,
            write_data(|w| w
                .push_data_type(DataType::Integer)
                .push_integer(10)
                .push_data_type(DataType::Integer)
                .push_integer(20)
                .push_data_type(DataType::Integer)
                .push_integer(2)
                .push_data_type(DataType::Range)
                .push_byte_size(RANGE_END_EXCLUSIVE | RANGE_HAS_STEP)
                .push_size(1)
                .push_size(6)
                .push_size(11))
        );
    }

    #[test]
    fn put_range_no_start() {
        let mut instructions = InstructionSetBuilder::new();
        instructions
            .put(ExpressionValue::integer_range(None, Some(20)).exclude_end())
            .unwrap();

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w.push_instruction(Instruction::Put).push_size(6))
        );
        assert_eq!(
            instructions.data,
            write_data(|w| w
                .push_data_type(DataType::Integer)
                .push_integer(20)
                .push_data_type(DataType::Range)
                .push_byte_size(RANGE_END_EXCLUSIVE | RANGE_OPEN_START)
                .push_size(1))
        );
    }

    #[test]
    fn put_range_no_end() {
        let mut instructions = InstructionSetBuilder::new();
        instructions
            .put(ExpressionValue::integer_range(Some(10), None).exclude_end())
            .unwrap();

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w.push_instruction(Instruction::Put).push_size(6))
        );
        assert_eq!(
            instructions.data,
            write_data(|w| w
                .push_data_type(DataType::Integer)
                .push_integer(10)
                .push_data_type(DataType::Range)
                .push_byte_size(RANGE_END_EXCLUSIVE | RANGE_OPEN_END)
                .push_size(1))
        );
    }

    #[test]
    fn put_nonexistent_expression() {
        let mut instructions = InstructionSetBuilder::new();
        instructions
            .put(ExpressionValue::expression("my_expression"))
            .unwrap();

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w.push_instruction(Instruction::Put).push_size(1))
        );
        assert_eq!(
            instructions.data,
            write_data(|w| w.push_data_type(DataType::Expression).push_size(1))
        );
        assert_eq!(*instructions.symbol_table.get("my_expression").unwrap(), 1);
        assert_eq!(
            *instructions.expression_map.get("my_expression").unwrap(),
            1
        );
        assert_eq!(*instructions.expression_table.get(0).unwrap(), 0);
    }

    #[test]
    fn reference_nonexistent_expression_then_start_expression() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.execute_expression("my_expression");

        instructions.end_expression();

        instructions.start_expression("my_expression");

        instructions.put(ExpressionValue::integer(10)).unwrap();

        instructions.end_expression();

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w
                .push_instruction(Instruction::ExecuteExpression)
                .push_size(2)
                .push_instruction(Instruction::EndExpression)
                .push_instruction(Instruction::Put)
                .push_size(1)
                .push_instruction(Instruction::EndExpression))
        );
        assert_eq!(
            instructions.data,
            write_data(|w| w.push_data_type(DataType::Integer).push_size(10))
        );
        assert_eq!(*instructions.symbol_table.get("my_expression").unwrap(), 2);
        assert_eq!(
            *instructions.expression_map.get("my_expression").unwrap(),
            2
        );
        assert_eq!(*instructions.expression_table.get(1).unwrap(), 7);
    }

    #[test]
    fn put_nonexistent_expression_then_start_expression() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::expression("my_expression"))
            .unwrap();

        instructions.end_expression();

        instructions.start_expression("my_expression");

        instructions.put(ExpressionValue::integer(10)).unwrap();

        instructions.end_expression();

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w
                .push_instruction(Instruction::Put)
                .push_size(1)
                .push_instruction(Instruction::EndExpression)
                .push_instruction(Instruction::Put)
                .push_size(6)
                .push_instruction(Instruction::EndExpression))
        );
        assert_eq!(
            instructions.data,
            write_data(|w| w
                .push_data_type(DataType::Expression)
                .push_size(2)
                .push_data_type(DataType::Integer)
                .push_integer(10))
        );
        assert_eq!(*instructions.symbol_table.get("my_expression").unwrap(), 2);
        assert_eq!(
            *instructions.expression_map.get("my_expression").unwrap(),
            2
        );
        assert_eq!(*instructions.expression_table.get(1).unwrap(), 7);
    }

    #[test]
    fn put_external_method() {
        let mut instructions = InstructionSetBuilder::new();
        instructions
            .put(ExpressionValue::external_method("external_method"))
            .unwrap();

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w.push_instruction(Instruction::Put).push_size(1))
        );
        assert_eq!(
            instructions.data,
            write_data(|w| w.push_data_type(DataType::ExternalMethod).push_size(1))
        );
        assert_eq!(
            *instructions.symbol_table.get("external_method").unwrap(),
            1
        );
    }

    #[test]
    fn put_same_symbol_twice_only_inserts_one() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.put(ExpressionValue::symbol("pandas")).unwrap();
        instructions.put(ExpressionValue::symbol("pandas")).unwrap();

        assert_eq!(instructions.symbol_table.len(), 2);
        assert_eq!(*instructions.symbol_table.get("pandas").unwrap(), 1);
    }

    #[test]
    fn put_multiple_numbers() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w
                .push_instruction(Instruction::Put)
                .push_size(1)
                .push_instruction(Instruction::Put)
                .push_size(6)
                .push_instruction(Instruction::Put)
                .push_size(11))
        );

        assert_eq!(
            instructions.data,
            write_data(|w| w
                .push_data_type(DataType::Integer)
                .push_size(10)
                .push_data_type(DataType::Integer)
                .push_size(20)
                .push_data_type(DataType::Integer)
                .push_size(30))
        );
    }

    #[test]
    fn put_pair() {
        let mut instructions = InstructionSetBuilder::new();
        instructions
            .put(ExpressionValue::pair(
                ExpressionValue::symbol("bear"),
                ExpressionValue::integer(20),
            ))
            .unwrap();

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w.push_instruction(Instruction::Put).push_size(11))
        );
        assert_eq!(
            instructions.data,
            write_data(|w| w
                .push_data_type(DataType::Symbol)
                .push_size(1)
                .push_data_type(DataType::Integer)
                .push_integer(20)
                .push_data_type(DataType::Pair)
                .push_size(1)
                .push_size(6))
        );
        assert_eq!(*instructions.symbol_table.get("bear").unwrap(), 1);
    }

    #[test]
    fn put_associative_list() {
        let mut instructions = InstructionSetBuilder::new();
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

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w.push_instruction(Instruction::Put).push_size(58))
        );
        assert_eq!(
            instructions.data,
            write_data(|w| w
                .push_data_type(DataType::Symbol) // 0
                .push_size(1)
                .push_data_type(DataType::Integer) // 5
                .push_integer(10)
                .push_data_type(DataType::Pair) // 10
                .push_size(1) // 11
                .push_size(6) // 15
                .push_data_type(DataType::Symbol) // 19
                .push_size(2)
                .push_data_type(DataType::Integer) // 24
                .push_integer(20)
                .push_data_type(DataType::Pair) // 29
                .push_size(20)
                .push_size(25)
                .push_data_type(DataType::Symbol) // 38
                .push_size(3)
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
                .push_size(30))
        );
    }

    #[test]
    fn put_multiple_referential_values() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.put(ExpressionValue::integer(200)).unwrap();
        instructions
            .put(
                ExpressionValue::list()
                    .add(ExpressionValue::integer(10))
                    .add(ExpressionValue::integer(20))
                    .add(ExpressionValue::integer(30)),
            )
            .unwrap();
        instructions
            .put(ExpressionValue::pair(
                ExpressionValue::symbol("cat"),
                ExpressionValue::integer(40),
            ))
            .unwrap();

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w
                .push_instruction(Instruction::Put)
                .push_size(1)
                .push_instruction(Instruction::Put)
                .push_size(21)
                .push_instruction(Instruction::Put)
                .push_size(52))
        );
        assert_eq!(
            instructions.data,
            write_data(|w| w
                .push_data_type(DataType::Integer) // 0
                .push_integer(200)
                .push_data_type(DataType::Integer) // 5
                .push_integer(10)
                .push_data_type(DataType::Integer) // 10
                .push_integer(20)
                .push_data_type(DataType::Integer) // 15
                .push_integer(30)
                .push_data_type(DataType::List) // 20
                .push_size(3) // 21
                .push_size(0) // 25
                .push_size(6) // 29
                .push_size(11) // 33
                .push_size(16) // 37
                .push_data_type(DataType::Symbol) // 41
                .push_size(1) // 42
                .push_data_type(DataType::Integer) // 46
                .push_integer(40) // 47
                .push_data_type(DataType::Pair) // 51
                .push_size(42)
                .push_size(47))
        );
    }

    #[test]
    fn start_expression() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        assert_eq!(instructions.instructions, vec![0]);
        assert_eq!(*instructions.expression_map.get("main").unwrap(), 1);
        assert_eq!(*instructions.expression_table.get(0).unwrap(), 1);
    }

    #[test]
    fn execute_expression() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.execute_expression("main");

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w
                .push_instruction(Instruction::ExecuteExpression)
                .push_size(1))
        );
        assert_eq!(*instructions.expression_map.get("main").unwrap(), 1);
        assert_eq!(*instructions.expression_table.get(0).unwrap(), 1);
    }

    #[test]
    fn execute_non_existent_expression() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.execute_expression("main");

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w
                .push_instruction(Instruction::ExecuteExpression)
                .push_size(1))
        );
        assert_eq!(*instructions.expression_map.get("main").unwrap(), 1);
        assert_eq!(*instructions.expression_table.get(0).unwrap(), 0);
    }

    #[test]
    fn put_input() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.put_input();

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w.push_instruction(Instruction::PutInput))
        );
    }

    #[test]
    fn resolve() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.resolve(&"value".to_string()).unwrap();

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w.push_instruction(Instruction::Resolve).push_size(1))
        );
    }

    #[test]
    fn conditional_execute() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("one");
        instructions.end_expression();
        instructions.start_expression("two");
        instructions.end_expression();

        instructions.conditional_execute(Some("one".to_string()), Some("two".to_string()));

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w
                .push_instruction(Instruction::EndExpression)
                .push_instruction(Instruction::EndExpression)
                .push_instruction(Instruction::ConditionalExecute)
                .push_size(1)
                .push_size(2))
        );
    }

    #[test]
    fn conditional_execute_before_declaration() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.conditional_execute(Some("one".to_string()), Some("two".to_string()));

        instructions.start_expression("one");
        instructions.end_expression();
        instructions.start_expression("two");
        instructions.end_expression();

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w
                .push_instruction(Instruction::ConditionalExecute)
                .push_size(1)
                .push_size(2)
                .push_instruction(Instruction::EndExpression)
                .push_instruction(Instruction::EndExpression))
        );

        assert_eq!(*instructions.symbol_table.get("one").unwrap(), 1);
        assert_eq!(*instructions.symbol_table.get("two").unwrap(), 2);

        assert_eq!(*instructions.expression_map.get("one").unwrap(), 1);
        assert_eq!(*instructions.expression_map.get("two").unwrap(), 2);

        assert_eq!(*instructions.expression_table.get(0).unwrap(), 10);
        assert_eq!(*instructions.expression_table.get(1).unwrap(), 11);
    }

    #[test]
    fn conditional_execute_no_true() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("one");
        instructions.end_expression();
        instructions.start_expression("two");
        instructions.end_expression();

        instructions.conditional_execute(None, Some("two".to_string()));

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w
                .push_instruction(Instruction::EndExpression)
                .push_instruction(Instruction::EndExpression)
                .push_instruction(Instruction::ConditionalExecute)
                .push_size(0)
                .push_size(2))
        );
    }

    #[test]
    fn conditional_execute_no_false() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("one");
        instructions.end_expression();
        instructions.start_expression("two");
        instructions.end_expression();

        instructions.conditional_execute(Some("one".to_string()), None);

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w
                .push_instruction(Instruction::EndExpression)
                .push_instruction(Instruction::EndExpression)
                .push_instruction(Instruction::ConditionalExecute)
                .push_size(1)
                .push_size(0))
        );
    }

    #[test]
    fn result_conditional_execute() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("one");
        instructions.end_expression();

        instructions.result_conditional_execute("one".to_string(), None);

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w
                .push_instruction(Instruction::EndExpression)
                .push_instruction(Instruction::ResultConditionalExecute)
                .push_size(1)
                .push_size(0))
        );
    }

    #[test]
    fn result_conditional_execute_with_false() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("one");
        instructions.end_expression();
        instructions.start_expression("two");
        instructions.end_expression();

        instructions.result_conditional_execute("one".to_string(), Some("two".to_string()));

        assert_eq!(
            instructions.instructions,
            write_instructions(|w| w
                .push_instruction(Instruction::EndExpression)
                .push_instruction(Instruction::EndExpression)
                .push_instruction(Instruction::ResultConditionalExecute)
                .push_size(1)
                .push_size(2))
        );
    }
}
