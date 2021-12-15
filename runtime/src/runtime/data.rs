use crate::{error, ExpressionData, ExpressionDataType, GarnishLangRuntimeResult, Instruction, InstructionData, RuntimeResult};

pub trait GarnishLangRuntimeDataPool {
    fn new() -> Self;

    fn set_end_of_constant(&mut self, addr: usize) -> GarnishLangRuntimeResult;
    fn get_end_of_constant_data(&self) -> usize;

    fn remove_non_constant_data(&mut self) -> GarnishLangRuntimeResult;

    fn get_data_len(&self) -> usize;

    fn set_result(&mut self, result: Option<usize>) -> GarnishLangRuntimeResult;
    fn get_result(&self) -> Option<usize>;

    fn push_input(&mut self, addr: usize) -> GarnishLangRuntimeResult;
    fn pop_input(&mut self) -> GarnishLangRuntimeResult<usize>;
    fn get_input(&self, index: usize) -> GarnishLangRuntimeResult<usize>;
    fn get_input_count(&self) -> usize;
    fn get_current_input(&self) -> GarnishLangRuntimeResult<usize>;

    fn get_data_type(&self, index: usize) -> GarnishLangRuntimeResult<ExpressionDataType>;
    fn get_integer(&self, index: usize) -> GarnishLangRuntimeResult<i64>;
    fn get_reference(&self, index: usize) -> GarnishLangRuntimeResult<usize>;
    fn get_symbol(&self, index: usize) -> GarnishLangRuntimeResult<u64>;
    fn get_expression(&self, index: usize) -> GarnishLangRuntimeResult<usize>;
    fn get_external(&self, index: usize) -> GarnishLangRuntimeResult<usize>;
    fn get_pair(&self, index: usize) -> GarnishLangRuntimeResult<(usize, usize)>;
    fn get_list_len(&self, index: usize) -> GarnishLangRuntimeResult<usize>;
    fn get_list_item(&self, list_index: usize, item_index: usize) -> GarnishLangRuntimeResult<usize>;
    fn get_list_associations_len(&self, index: usize) -> GarnishLangRuntimeResult<usize>;
    fn get_list_association(&self, list_index: usize, item_index: usize) -> GarnishLangRuntimeResult<usize>;

    fn add_integer(&mut self, value: i64) -> GarnishLangRuntimeResult<usize>;
    fn add_reference(&mut self, value: usize) -> GarnishLangRuntimeResult<usize>;
    fn add_symbol(&mut self, value: u64) -> GarnishLangRuntimeResult<usize>;
    fn add_expression(&mut self, value: usize) -> GarnishLangRuntimeResult<usize>;
    fn add_external(&mut self, value: usize) -> GarnishLangRuntimeResult<usize>;
    fn add_pair(&mut self, value: (usize, usize)) -> GarnishLangRuntimeResult<usize>;
    fn add_list(&mut self, value: Vec<usize>, associations: Vec<usize>) -> GarnishLangRuntimeResult<usize>;
    fn add_unit(&mut self) -> GarnishLangRuntimeResult<usize>;
    fn add_true(&mut self) -> GarnishLangRuntimeResult<usize>;
    fn add_false(&mut self) -> GarnishLangRuntimeResult<usize>;

    fn push_register(&mut self, addr: usize) -> GarnishLangRuntimeResult;
    fn pop_register(&mut self) -> GarnishLangRuntimeResult<usize>;

    fn push_instruction(&mut self, instruction: InstructionData) -> GarnishLangRuntimeResult;
    fn get_instruction(&self, index: usize) -> GarnishLangRuntimeResult<&InstructionData>;
    fn set_instruction_cursor(&mut self, index: usize) -> GarnishLangRuntimeResult;
    fn advance_instruction_cursor(&mut self) -> GarnishLangRuntimeResult;
    fn get_instruction_cursor(&self) -> GarnishLangRuntimeResult<usize>;
    fn get_instruction_len(&self) -> usize;

    fn push_jump_point(&mut self, index: usize) -> GarnishLangRuntimeResult;
    fn get_jump_point(&self, index: usize) -> GarnishLangRuntimeResult<usize>;

    fn push_jump_path(&mut self, index: usize) -> GarnishLangRuntimeResult;
    fn pop_jump_path(&mut self) -> GarnishLangRuntimeResult<usize>;
    fn get_jump_path(&self, index: usize) -> GarnishLangRuntimeResult<usize>;

    // maybe temporary
    fn add_data(&mut self, data: ExpressionData) -> GarnishLangRuntimeResult<usize>;
}

pub struct SimpleRuntimeData {
    register: Vec<usize>,
    data: Vec<ExpressionData>,
    end_of_constant_data: usize,
    current_result: Option<usize>,
    inputs: Vec<usize>,
    instructions: Vec<InstructionData>,
    instruction_cursor: usize,
    expression_table: Vec<usize>,
    jump_path: Vec<usize>,
}

impl SimpleRuntimeData {
    pub fn get(&self, index: usize) -> GarnishLangRuntimeResult<&ExpressionData> {
        match self.data.get(index) {
            None => Err(error(format!("No data at addr {:?}", index))),
            Some(d) => Ok(d),
        }
    }

    pub fn get_register(&self) -> &Vec<usize> {
        &self.register
    }

    pub fn get_jump_path_vec(&self) -> &Vec<usize> {
        &self.jump_path
    }

    pub fn get_jump_points(&self) -> &Vec<usize> {
        &self.expression_table
    }

    pub fn get_instructions(&self) -> &Vec<InstructionData> {
        &self.instructions
    }

    pub fn get_data(&self) -> &Vec<ExpressionData> {
        &self.data
    }
}

impl GarnishLangRuntimeDataPool for SimpleRuntimeData {
    fn new() -> Self {
        SimpleRuntimeData {
            register: vec![],
            data: vec![ExpressionData::unit()],
            end_of_constant_data: 0,
            current_result: None,
            inputs: vec![],
            instruction_cursor: 0,
            instructions: vec![InstructionData::new(Instruction::EndExecution, None)],
            expression_table: vec![],
            jump_path: vec![],
        }
    }

    fn get_data_type(&self, index: usize) -> GarnishLangRuntimeResult<ExpressionDataType> {
        Ok(self.get(index)?.get_type())
    }

    fn get_integer(&self, index: usize) -> GarnishLangRuntimeResult<i64> {
        self.get(index)?.as_integer().as_runtime_result()
    }

    fn get_reference(&self, index: usize) -> GarnishLangRuntimeResult<usize> {
        self.get(index)?.as_reference().as_runtime_result()
    }

    fn get_symbol(&self, index: usize) -> GarnishLangRuntimeResult<u64> {
        self.get(index)?.as_symbol_value().as_runtime_result()
    }

    fn get_expression(&self, index: usize) -> GarnishLangRuntimeResult<usize> {
        self.get(index)?.as_expression().as_runtime_result()
    }

    fn get_external(&self, index: usize) -> GarnishLangRuntimeResult<usize> {
        self.get(index)?.as_external().as_runtime_result()
    }

    fn get_pair(&self, index: usize) -> GarnishLangRuntimeResult<(usize, usize)> {
        self.get(index)?.as_pair().as_runtime_result()
    }

    fn get_list_len(&self, index: usize) -> GarnishLangRuntimeResult<usize> {
        Ok(self.get(index)?.as_list().as_runtime_result()?.0.len())
    }

    fn get_list_item(&self, list_index: usize, item_index: usize) -> GarnishLangRuntimeResult<usize> {
        match self.get(list_index)?.as_list().as_runtime_result()?.0.get(item_index) {
            None => Err(error(format!("No list item at index {:?} for list at addr {:?}", item_index, list_index))),
            Some(v) => Ok(*v),
        }
    }

    fn get_list_associations_len(&self, index: usize) -> GarnishLangRuntimeResult<usize> {
        Ok(self.get(index)?.as_list().as_runtime_result()?.1.len())
    }

    fn get_list_association(&self, list_index: usize, item_index: usize) -> GarnishLangRuntimeResult<usize> {
        match self.get(list_index)?.as_list().as_runtime_result()?.1.get(item_index) {
            None => Err(error(format!("No list item at index {:?} for list at addr {:?}", item_index, list_index))),
            Some(v) => Ok(*v),
        }
    }

    fn add_integer(&mut self, value: i64) -> GarnishLangRuntimeResult<usize> {
        self.data.push(ExpressionData::integer(value));
        Ok(self.data.len() - 1)
    }

    fn add_reference(&mut self, value: usize) -> GarnishLangRuntimeResult<usize> {
        self.data.push(ExpressionData::reference(value));
        Ok(self.data.len() - 1)
    }

    fn add_symbol(&mut self, value: u64) -> GarnishLangRuntimeResult<usize> {
        self.data.push(ExpressionData::symbol(&"".to_string(), value));
        Ok(self.data.len() - 1)
    }

    fn add_expression(&mut self, value: usize) -> GarnishLangRuntimeResult<usize> {
        self.data.push(ExpressionData::expression(value));
        Ok(self.data.len() - 1)
    }

    fn add_external(&mut self, value: usize) -> GarnishLangRuntimeResult<usize> {
        self.data.push(ExpressionData::external(value));
        Ok(self.data.len() - 1)
    }

    fn add_pair(&mut self, value: (usize, usize)) -> GarnishLangRuntimeResult<usize> {
        self.data.push(ExpressionData::pair(value.0, value.1));
        Ok(self.data.len() - 1)
    }

    fn add_list(&mut self, value: Vec<usize>, associations: Vec<usize>) -> GarnishLangRuntimeResult<usize> {
        self.data.push(ExpressionData::list(value, associations));
        Ok(self.data.len() - 1)
    }

    fn add_data(&mut self, data: ExpressionData) -> GarnishLangRuntimeResult<usize> {
        self.data.push(data);
        Ok(self.data.len() - 1)
    }

    fn add_unit(&mut self) -> GarnishLangRuntimeResult<usize> {
        self.data.push(ExpressionData::unit());
        Ok(self.data.len() - 1)
    }

    fn add_true(&mut self) -> GarnishLangRuntimeResult<usize> {
        self.data.push(ExpressionData::boolean_true());
        Ok(self.data.len() - 1)
    }

    fn add_false(&mut self) -> GarnishLangRuntimeResult<usize> {
        self.data.push(ExpressionData::boolean_false());
        Ok(self.data.len() - 1)
    }

    fn push_register(&mut self, addr: usize) -> GarnishLangRuntimeResult {
        self.register.push(addr);
        Ok(())
    }

    fn pop_register(&mut self) -> GarnishLangRuntimeResult<usize> {
        self.register.pop().as_runtime_result()
    }

    fn set_end_of_constant(&mut self, addr: usize) -> GarnishLangRuntimeResult {
        self.end_of_constant_data = addr;
        Ok(())
    }

    fn get_end_of_constant_data(&self) -> usize {
        self.end_of_constant_data
    }

    fn remove_non_constant_data(&mut self) -> GarnishLangRuntimeResult {
        self.data = Vec::from(&self.data[..self.end_of_constant_data]);

        Ok(())
    }

    fn get_data_len(&self) -> usize {
        self.data.len()
    }

    fn set_result(&mut self, result: Option<usize>) -> GarnishLangRuntimeResult {
        self.current_result = result;
        Ok(())
    }

    fn get_result(&self) -> Option<usize> {
        self.current_result
    }

    fn push_input(&mut self, addr: usize) -> GarnishLangRuntimeResult {
        self.inputs.push(addr);
        Ok(())
    }

    fn pop_input(&mut self) -> GarnishLangRuntimeResult<usize> {
        self.inputs.pop().as_runtime_result()
    }

    fn get_input(&self, index: usize) -> GarnishLangRuntimeResult<usize> {
        self.inputs.get(index).cloned().as_runtime_result()
    }

    fn get_input_count(&self) -> usize {
        self.inputs.len()
    }

    fn get_current_input(&self) -> GarnishLangRuntimeResult<usize> {
        self.inputs.last().cloned().as_runtime_result()
    }

    fn push_instruction(&mut self, instruction: InstructionData) -> GarnishLangRuntimeResult {
        self.instructions.push(instruction);
        Ok(())
    }

    fn get_instruction(&self, index: usize) -> GarnishLangRuntimeResult<&InstructionData> {
        self.instructions.get(index).as_runtime_result()
    }

    fn set_instruction_cursor(&mut self, index: usize) -> GarnishLangRuntimeResult {
        self.instruction_cursor = index;
        Ok(())
    }

    fn advance_instruction_cursor(&mut self) -> GarnishLangRuntimeResult {
        self.instruction_cursor += 1;
        Ok(())
    }

    fn get_instruction_cursor(&self) -> GarnishLangRuntimeResult<usize> {
        Ok(self.instruction_cursor)
    }

    fn get_instruction_len(&self) -> usize {
        self.instructions.len()
    }

    fn push_jump_point(&mut self, index: usize) -> GarnishLangRuntimeResult {
        if index >= self.instructions.len() {
            return Err(error(format!(
                "Specified jump point {:?} is out of bounds of instructions with length {:?}",
                index,
                self.instructions.len()
            )));
        }

        self.expression_table.push(index);
        Ok(())
    }

    fn get_jump_point(&self, index: usize) -> GarnishLangRuntimeResult<usize> {
        self.expression_table.get(index).cloned().as_runtime_result()
    }

    fn push_jump_path(&mut self, index: usize) -> GarnishLangRuntimeResult {
        self.jump_path.push(index);
        Ok(())
    }

    fn pop_jump_path(&mut self) -> GarnishLangRuntimeResult<usize> {
        self.jump_path.pop().as_runtime_result()
    }

    fn get_jump_path(&self, index: usize) -> GarnishLangRuntimeResult<usize> {
        self.jump_path.get(index).cloned().as_runtime_result()
    }
}
