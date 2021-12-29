use std::collections::HashMap;

use crate::{
    EmptyContext, ExpressionData, ExpressionDataType, GarnishLangRuntimeContext, GarnishLangRuntimeData, GarnishLangRuntimeError,
    GarnishLangRuntimeState, GarnishRuntime, Instruction, InstructionData,
};

pub mod expression_data;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SimpleRuntimeData {
    register: Vec<usize>,
    data: Vec<ExpressionData>,
    end_of_constant_data: usize,
    values: Vec<usize>,
    instructions: Vec<InstructionData>,
    instruction_cursor: usize,
    expression_table: Vec<usize>,
    jump_path: Vec<usize>,
    current_list: Option<(Vec<usize>, Vec<usize>)>,
    symbols: HashMap<String, u64>,
}

impl SimpleRuntimeData {
    pub fn new() -> Self {
        SimpleRuntimeData {
            register: vec![],
            data: vec![ExpressionData::unit()],
            end_of_constant_data: 0,
            values: vec![],
            instruction_cursor: 0,
            instructions: vec![InstructionData::new(Instruction::EndExecution, None)],
            expression_table: vec![],
            jump_path: vec![],
            current_list: None,
            symbols: HashMap::new(),
        }
    }

    pub fn get(&self, index: usize) -> Result<&ExpressionData, String> {
        match self.data.get(index) {
            None => Err(format!("No data at addr {:?}", index)),
            Some(d) => Ok(d),
        }
    }

    pub fn get_symbols(&self) -> &HashMap<String, u64> {
        &self.symbols
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

    pub fn execute_all_instructions(&mut self) -> Result<(), GarnishLangRuntimeError<String>> {
        loop {
            match self.execute_current_instruction::<EmptyContext>(None) {
                Err(e) => return Err(e),
                Ok(data) => match data.get_state() {
                    GarnishLangRuntimeState::Running => (),
                    GarnishLangRuntimeState::End => return Ok(()),
                },
            }
        }
    }

    pub fn execute_all_instructions_with_context<Context: GarnishLangRuntimeContext<Self>>(
        &mut self,
        context: &mut Context,
    ) -> Result<(), GarnishLangRuntimeError<String>> {
        loop {
            match self.execute_current_instruction(Some(context)) {
                Err(e) => return Err(e),
                Ok(data) => match data.get_state() {
                    GarnishLangRuntimeState::Running => (),
                    GarnishLangRuntimeState::End => return Ok(()),
                },
            }
        }
    }
}

impl GarnishLangRuntimeData for SimpleRuntimeData {
    type Error = String;

    fn get_data_type(&self, index: usize) -> Result<ExpressionDataType, Self::Error> {
        Ok(self.get(index)?.get_type())
    }

    fn get_integer(&self, index: usize) -> Result<i64, Self::Error> {
        self.get(index)?.as_integer()
    }

    fn get_reference(&self, index: usize) -> Result<usize, Self::Error> {
        self.get(index)?.as_reference()
    }

    fn get_symbol(&self, index: usize) -> Result<u64, Self::Error> {
        self.get(index)?.as_symbol_value()
    }

    fn get_expression(&self, index: usize) -> Result<usize, Self::Error> {
        self.get(index)?.as_expression()
    }

    fn get_external(&self, index: usize) -> Result<usize, Self::Error> {
        self.get(index)?.as_external()
    }

    fn get_pair(&self, index: usize) -> Result<(usize, usize), Self::Error> {
        self.get(index)?.as_pair()
    }

    fn get_list_len(&self, index: usize) -> Result<usize, Self::Error> {
        Ok(self.get(index)?.as_list()?.0.len())
    }

    fn get_list_item(&self, list_index: usize, item_index: usize) -> Result<usize, Self::Error> {
        match self.get(list_index)?.as_list()?.0.get(item_index) {
            None => Err(format!("No list item at index {:?} for list at addr {:?}", item_index, list_index)),
            Some(v) => Ok(*v),
        }
    }

    fn get_list_associations_len(&self, index: usize) -> Result<usize, Self::Error> {
        Ok(self.get(index)?.as_list()?.1.len())
    }

    fn get_list_association(&self, list_index: usize, item_index: usize) -> Result<usize, Self::Error> {
        match self.get(list_index)?.as_list()?.1.get(item_index) {
            None => Err(format!("No list item at index {:?} for list at addr {:?}", item_index, list_index)),
            Some(v) => Ok(*v),
        }
    }

    fn add_integer(&mut self, value: i64) -> Result<usize, Self::Error> {
        self.data.push(ExpressionData::integer(value));
        Ok(self.data.len() - 1)
    }

    fn add_symbol(&mut self, value: &str) -> Result<usize, Self::Error> {
        let sym = ExpressionData::symbol_from_string(value);
        for (key, value) in sym.get_symbols() {
            self.symbols.insert(key.clone(), value.clone());
        }

        self.data.push(sym);
        Ok(self.data.len() - 1)
    }

    fn add_expression(&mut self, value: usize) -> Result<usize, Self::Error> {
        self.data.push(ExpressionData::expression(value));
        Ok(self.data.len() - 1)
    }

    fn add_external(&mut self, value: usize) -> Result<usize, Self::Error> {
        self.data.push(ExpressionData::external(value));
        Ok(self.data.len() - 1)
    }

    fn add_pair(&mut self, value: (usize, usize)) -> Result<usize, Self::Error> {
        self.data.push(ExpressionData::pair(value.0, value.1));
        Ok(self.data.len() - 1)
    }

    fn add_data(&mut self, data: ExpressionData) -> Result<usize, Self::Error> {
        self.data.push(data);
        Ok(self.data.len() - 1)
    }

    fn add_unit(&mut self) -> Result<usize, Self::Error> {
        self.data.push(ExpressionData::unit());
        Ok(self.data.len() - 1)
    }

    fn add_true(&mut self) -> Result<usize, Self::Error> {
        self.data.push(ExpressionData::boolean_true());
        Ok(self.data.len() - 1)
    }

    fn add_false(&mut self) -> Result<usize, Self::Error> {
        self.data.push(ExpressionData::boolean_false());
        Ok(self.data.len() - 1)
    }

    fn add_list(&mut self, value: Vec<usize>, associations: Vec<usize>) -> Result<usize, Self::Error> {
        self.data.push(ExpressionData::list(value, associations));
        Ok(self.data.len() - 1)
    }

    fn start_list(&mut self, _: usize) -> Result<(), Self::Error> {
        self.current_list = Some((vec![], vec![]));
        Ok(())
    }

    fn add_to_list(&mut self, addr: usize, is_associative: bool) -> Result<(), Self::Error> {
        match &mut self.current_list {
            None => Err(format!("Not currently creating a list.")),
            Some((items, associations)) => {
                items.push(addr);

                if is_associative {
                    associations.push(addr);
                }

                Ok(())
            }
        }
    }

    fn end_list(&mut self) -> Result<usize, Self::Error> {
        match &mut self.current_list {
            None => Err(format!("Not currently creating a list.")),
            Some((items, associations)) => {
                // reorder associative values by modulo value
                let mut ordered = vec![0usize; associations.len()];
                for index in 0..associations.len() {
                    let item = associations[index];
                    let mut i = item % associations.len();
                    let mut count = 0;
                    while ordered[i] != 0 {
                        i += 1;
                        if i >= associations.len() {
                            i = 0;
                        }

                        count += 1;
                        if count > associations.len() {
                            return Err(format!("Could not place associative value"));
                        }
                    }

                    ordered[i] = item;
                }

                items.reverse();

                self.data.push(ExpressionData::list(items.to_vec(), ordered));
                Ok(self.data.len() - 1)
            }
        }
    }

    fn get_list_item_with_symbol(&self, list_addr: usize, sym: u64) -> Result<Option<usize>, Self::Error> {
        let assocations_len = self.get_list_associations_len(list_addr)?;

        if assocations_len == 0 {
            return Ok(None);
        }

        let mut i = sym as usize % assocations_len;
        let mut count = 0;

        loop {
            // check to make sure item has same symbol
            let association_ref = self.get_list_association(list_addr, i)?;

            // should have symbol on left
            match self.get_data_type(association_ref)? {
                ExpressionDataType::Pair => {
                    let (left, right) = self.get_pair(association_ref)?;

                    let left_ref = left;

                    match self.get_data_type(left_ref)? {
                        ExpressionDataType::Symbol => {
                            let v = self.get_symbol(left_ref)?;

                            if v == sym {
                                // found match
                                // insert pair right as value
                                return Ok(Some(right));
                            }
                        }
                        t => Err(format!("Association created with non-symbol type {:?} on pair left.", t))?,
                    }
                }
                t => Err(format!("Association created with non-pair type {:?}.", t))?,
            }

            i += 1;
            if i >= assocations_len {
                i = 0;
            }

            count += 1;
            if count > assocations_len {
                return Ok(None);
            }
        }
    }

    fn push_register(&mut self, addr: usize) -> Result<(), Self::Error> {
        self.register.push(addr);
        Ok(())
    }

    fn pop_register(&mut self) -> Option<usize> {
        self.register.pop()
    }

    fn set_end_of_constant(&mut self, addr: usize) -> Result<(), Self::Error> {
        self.end_of_constant_data = addr;
        Ok(())
    }

    fn get_end_of_constant_data(&self) -> usize {
        self.end_of_constant_data
    }

    fn remove_non_constant_data(&mut self) -> Result<(), Self::Error> {
        self.data = Vec::from(&self.data[..self.end_of_constant_data]);

        Ok(())
    }

    fn get_data_len(&self) -> usize {
        self.data.len()
    }

    fn push_value_stack(&mut self, addr: usize) -> Result<(), Self::Error> {
        self.values.push(addr);
        Ok(())
    }

    fn pop_value_stack(&mut self) -> Option<usize> {
        self.values.pop()
    }

    fn get_value(&self, index: usize) -> Option<usize> {
        self.values.get(index).cloned()
    }

    fn get_value_mut(&mut self, index: usize) -> Option<&mut usize> {
        self.values.get_mut(index)
    }

    fn get_value_count(&self) -> usize {
        self.values.len()
    }

    fn get_current_value(&self) -> Option<usize> {
        self.values.last().cloned()
    }

    fn get_current_value_mut(&mut self) -> Option<&mut usize> {
        self.values.last_mut()
    }

    fn push_instruction(&mut self, instruction: Instruction, data: Option<usize>) -> Result<(), Self::Error> {
        self.instructions.push(InstructionData::new(instruction, data));
        Ok(())
    }

    fn get_instruction(&self, index: usize) -> Option<&InstructionData> {
        self.instructions.get(index)
    }

    fn set_instruction_cursor(&mut self, index: usize) -> Result<(), Self::Error> {
        self.instruction_cursor = index;
        Ok(())
    }

    fn advance_instruction_cursor(&mut self) -> Result<(), Self::Error> {
        self.instruction_cursor += 1;
        Ok(())
    }

    fn get_current_instruction(&self) -> Option<&InstructionData> {
        self.get_instruction(self.get_instruction_cursor())
    }

    fn get_instruction_cursor(&self) -> usize {
        self.instruction_cursor
    }

    fn get_instruction_len(&self) -> usize {
        self.instructions.len()
    }

    fn push_jump_point(&mut self, index: usize) -> Result<(), Self::Error> {
        if index >= self.instructions.len() {
            return Err(format!(
                "Specified jump point {:?} is out of bounds of instructions with length {:?}",
                index,
                self.instructions.len()
            ));
        }

        self.expression_table.push(index);
        Ok(())
    }

    fn get_jump_point(&self, index: usize) -> Option<usize> {
        self.expression_table.get(index).cloned()
    }

    fn get_jump_point_count(&self) -> usize {
        self.expression_table.len()
    }

    fn push_jump_path(&mut self, index: usize) -> Result<(), Self::Error> {
        self.jump_path.push(index);
        Ok(())
    }

    fn pop_jump_path(&mut self) -> Option<usize> {
        self.jump_path.pop()
    }

    fn get_jump_path(&self, index: usize) -> Option<usize> {
        self.jump_path.get(index).cloned()
    }

    fn get_jump_point_mut(&mut self, index: usize) -> Option<&mut usize> {
        self.expression_table.get_mut(index)
    }
}
