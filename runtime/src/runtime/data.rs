// use log::trace;

use crate::{error, ExpressionData, ExpressionDataType, GarnishLangRuntime, GarnishLangRuntimeResult, Instruction, InstructionData, RuntimeResult};

pub trait GarnishLangRuntimeDataPool {
    fn new() -> Self;

    fn set_end_of_constant(&mut self, addr: usize) -> GarnishLangRuntimeResult;
    fn get_end_of_constant_data(&self) -> usize;

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

impl<Data> GarnishLangRuntime<Data>
where
    Data: GarnishLangRuntimeDataPool,
{
    pub fn get_data_pool(&self) -> &Data {
        &self.heap
    }

    pub fn add_data(&mut self, data: ExpressionData) -> GarnishLangRuntimeResult<usize> {
        // Check if give a reference of reference
        // flatten reference to point to non-Reference data
        let data = match data.get_type() {
            ExpressionDataType::Reference => {
                let ref_addr = data.as_reference().as_runtime_result()?;
                match self.heap.get_data_type(ref_addr)? {
                    ExpressionDataType::Reference => ExpressionData::reference(self.heap.get_reference(ref_addr)?),
                    _ => data,
                }
            }
            ExpressionDataType::Symbol => {
                // self.symbols.extend(data.symbols.clone());
                data
            }
            _ => data,
        };

        let addr = self.heap.get_data_len();
        self.heap.add_data(data.clone())?;
        Ok(addr)
    }

    pub fn end_constant_data(&mut self) -> GarnishLangRuntimeResult {
        self.heap.set_end_of_constant(self.heap.get_data_len())
    }

    pub fn get_end_of_constant_data(&self) -> usize {
        self.heap.get_end_of_constant_data()
    }

    pub fn add_data_ref(&mut self, data: ExpressionData) -> GarnishLangRuntimeResult<usize> {
        let addr = self.add_data(data)?;
        self.heap.push_register(addr).unwrap();
        Ok(addr)
    }

    pub fn get_data_len(&self) -> usize {
        self.heap.get_data_len()
    }

    pub fn add_reference_data(&mut self, reference: usize) -> GarnishLangRuntimeResult<usize> {
        self.add_data(ExpressionData::reference(reference))
    }

    // move to GarnishLangRuntimeDataPool trait
    // pub fn remove_data(&mut self, from: usize) -> GarnishLangRuntimeResult {
    //     match from < self.get_data_len() {
    //         true => {
    //             self.data = Vec::from(&self.data[..from]);
    //             Ok(())
    //         }
    //         false => Err(error(format!("Given address is beyond data size."))),
    //     }
    // }

    pub(crate) fn next_ref(&mut self) -> GarnishLangRuntimeResult<usize> {
        self.heap.pop_register()
    }

    pub(crate) fn next_two_raw_ref(&mut self) -> GarnishLangRuntimeResult<(usize, usize)> {
        let first_ref = self.next_ref()?;
        let second_ref = self.next_ref()?;

        Ok((self.addr_of_raw_data(first_ref)?, self.addr_of_raw_data(second_ref)?))
    }

    pub(crate) fn addr_of_raw_data(&self, addr: usize) -> GarnishLangRuntimeResult<usize> {
        Ok(match self.heap.get_data_type(addr)? {
            ExpressionDataType::Reference => self.heap.get_reference(addr)?,
            _ => addr,
        })
    }

    // push utilities

    pub fn push_unit(&mut self) -> GarnishLangRuntimeResult {
        self.heap.add_unit().and_then(|v| self.heap.push_register(v))
    }

    pub fn push_integer(&mut self, value: i64) -> GarnishLangRuntimeResult {
        self.heap.add_integer(value).and_then(|v| self.heap.push_register(v))
    }

    pub fn push_boolean(&mut self, value: bool) -> GarnishLangRuntimeResult {
        match value {
            true => self.heap.add_true(),
            false => self.heap.add_false(),
        }
        .and_then(|v| self.heap.push_register(v))
    }

    pub fn push_list(&mut self, list: Vec<usize>, associations: Vec<usize>) -> GarnishLangRuntimeResult {
        self.heap.add_list(list, associations).and_then(|v| self.heap.push_register(v))
    }

    pub fn push_reference(&mut self, value: usize) -> GarnishLangRuntimeResult {
        self.heap.add_reference(value).and_then(|v| self.heap.push_register(v))
    }

    pub fn push_pair(&mut self, left: usize, right: usize) -> GarnishLangRuntimeResult {
        self.heap.add_pair((left, right)).and_then(|v| self.heap.push_register(v))
    }
}

#[cfg(test)]
mod tests {
    use crate::{runtime::data::GarnishLangRuntimeDataPool, ExpressionData, GarnishLangRuntime};

    #[test]
    fn add_data() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(100)).unwrap();

        assert_eq!(runtime.get_data_len(), 2);
    }

    #[test]
    fn add_data_ref() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data_ref(ExpressionData::integer(100)).unwrap();

        assert_eq!(runtime.heap.get_register(), &vec![1]);
        assert_eq!(runtime.get_data_len(), 2);
    }

    #[test]
    fn get_data() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(100)).unwrap();
        runtime.add_data(ExpressionData::integer(200)).unwrap();

        assert_eq!(runtime.heap.get_integer(2).unwrap(), 200);
    }

    #[test]
    fn end_constant_data() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(100)).unwrap();
        runtime.add_data(ExpressionData::integer(200)).unwrap();
        runtime.end_constant_data().unwrap();

        assert_eq!(runtime.get_end_of_constant_data(), 3);
    }

    #[test]
    fn add_data_returns_addr() {
        let mut runtime = GarnishLangRuntime::simple();

        assert_eq!(runtime.add_data(ExpressionData::integer(100)).unwrap(), 1);
        assert_eq!(runtime.add_data(ExpressionData::integer(100)).unwrap(), 2);
        assert_eq!(runtime.add_data(ExpressionData::integer(100)).unwrap(), 3);
        assert_eq!(runtime.add_data(ExpressionData::integer(100)).unwrap(), 4);
    }

    #[test]
    fn add_reference_of_reference_falls_through() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(100)).unwrap();
        runtime.add_data(ExpressionData::reference(1)).unwrap();
        runtime.add_data(ExpressionData::reference(2)).unwrap();

        assert_eq!(runtime.heap.get_reference(3).unwrap(), 1);
    }

    #[test]
    fn push_top_reference() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(100)).unwrap();
        runtime.add_reference_data(0).unwrap();

        assert_eq!(runtime.get_data_len(), 3);
    }

    // #[test]
    // fn remove_data() {
    //     let mut runtime = GarnishLangRuntime::simple();

    //     runtime.add_data(ExpressionData::symbol(&"false".to_string(), 0)).unwrap();
    //     runtime.add_data(ExpressionData::integer(10)).unwrap();
    //     runtime.add_data(ExpressionData::integer(20)).unwrap();
    //     runtime.add_data(ExpressionData::integer(20)).unwrap();
    //     let addr = runtime.add_data(ExpressionData::integer(20)).unwrap();
    //     runtime.add_data(ExpressionData::integer(20)).unwrap();
    //     runtime.add_data(ExpressionData::integer(20)).unwrap();

    //     runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

    //     runtime.remove_data(addr).unwrap();

    //     assert_eq!(runtime.get_data_len(), 5);
    // }

    // #[test]
    // fn remove_data_out_of_bounds() {
    //     let mut runtime = GarnishLangRuntime::simple();

    //     runtime.add_data(ExpressionData::symbol(&"false".to_string(), 0)).unwrap();
    //     runtime.add_data(ExpressionData::integer(10)).unwrap();
    //     runtime.add_data(ExpressionData::integer(20)).unwrap();
    //     runtime.add_data(ExpressionData::integer(20)).unwrap();
    //     runtime.add_data(ExpressionData::integer(20)).unwrap();
    //     runtime.add_data(ExpressionData::integer(20)).unwrap();

    //     runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

    //     let result = runtime.remove_data(10);

    //     assert!(result.is_err());
    // }
}

#[cfg(test)]
mod internal {
    // use crate::{ExpressionData, GarnishLangRuntime};

    // #[test]
    // fn next_ref_data() {
    //     let mut runtime = GarnishLangRuntime::simple();
    //     runtime.add_data(ExpressionData::integer(10)).unwrap();
    //     runtime.add_data(ExpressionData::integer(20)).unwrap();

    //     runtime.reference_stack.push(2);

    //     let result = runtime.next_ref_data();

    //     assert_eq!(result.unwrap().as_integer().unwrap(), 20);
    // }

    // #[test]
    // fn next_ref_data_no_ref_is_error() {
    //     let mut runtime = GarnishLangRuntime::simple();
    //     runtime.add_data(ExpressionData::integer(10)).unwrap();
    //     runtime.add_data(ExpressionData::integer(20)).unwrap();

    //     let result = runtime.next_ref_data();

    //     assert!(result.is_err());
    // }

    // #[test]
    // fn next_two_ref_data() {
    //     let mut runtime = GarnishLangRuntime::simple();
    //     runtime.add_data(ExpressionData::integer(10)).unwrap();
    //     runtime.add_data(ExpressionData::integer(20)).unwrap();

    //     runtime.reference_stack.push(1);
    //     runtime.reference_stack.push(2);

    //     let result = runtime.next_two_ref_data();

    //     let (first, second) = result.unwrap();

    //     assert_eq!(first.as_integer().unwrap(), 20);
    //     assert_eq!(second.as_integer().unwrap(), 10);
    // }

    // #[test]
    // fn next_two_ref_data_one_ref_is_error() {
    //     let mut runtime = GarnishLangRuntime::simple();
    //     runtime.add_data(ExpressionData::integer(10)).unwrap();
    //     runtime.add_data(ExpressionData::integer(20)).unwrap();

    //     runtime.reference_stack.push(1);

    //     let result = runtime.next_two_ref_data();

    //     assert!(result.is_err());
    // }

    // #[test]
    // fn next_two_ref_data_zero_refs_is_error() {
    //     let mut runtime = GarnishLangRuntime::simple();
    //     runtime.add_data(ExpressionData::integer(10)).unwrap();
    //     runtime.add_data(ExpressionData::integer(20)).unwrap();

    //     let result = runtime.next_two_ref_data();

    //     assert!(result.is_err());
    // }
}
