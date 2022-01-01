use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::{collections::HashMap, hash::Hasher};

use crate::{
    symbol_value, AnyData, DataCoersion, EmptyContext, ExpressionData, ExpressionDataType, ExternalData, FalseData, GarnishLangRuntimeContext,
    GarnishLangRuntimeData, GarnishLangRuntimeState, GarnishRuntime, Instruction, InstructionData, IntegerData, ListData, PairData, RuntimeError,
    SimpleData, SimpleDataList, SymbolData, TrueData, UnitData,
};

pub mod data;

#[derive(Debug)]
pub struct SimpleRuntimeData {
    register: Vec<usize>,
    simple_data: SimpleDataList,
    end_of_constant_data: usize,
    values: Vec<usize>,
    instructions: Vec<InstructionData>,
    instruction_cursor: usize,
    expression_table: Vec<usize>,
    jump_path: Vec<usize>,
    current_list: Option<(Vec<usize>, Vec<usize>)>,
    symbols: HashMap<String, u64>,
    cache: HashMap<u64, usize>,
    lease_stack: Vec<usize>
}

impl SimpleRuntimeData {
    pub fn new() -> Self {
        SimpleRuntimeData {
            register: vec![],
            simple_data: SimpleDataList::default(),
            end_of_constant_data: 0,
            values: vec![],
            instruction_cursor: 0,
            instructions: vec![InstructionData::new(Instruction::EndExecution, None)],
            expression_table: vec![],
            jump_path: vec![],
            current_list: None,
            symbols: HashMap::new(),
            cache: HashMap::new(),
            lease_stack: vec![]
        }
    }

    pub fn get(&self, index: usize) -> Result<&AnyData, DataError> {
        match self.simple_data.get(index) {
            None => Err(format!("No data at addr {:?}", index))?,
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

    pub fn get_data(&self) -> &SimpleDataList {
        &self.simple_data
    }

    pub fn execute_all_instructions(&mut self) -> Result<(), RuntimeError<DataError>> {
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
    ) -> Result<(), RuntimeError<DataError>> {
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

    pub fn set_end_of_constant(&mut self, addr: usize) -> Result<(), DataError> {
        self.end_of_constant_data = addr;
        Ok(())
    }

    pub fn get_end_of_constant_data(&self) -> usize {
        self.end_of_constant_data
    }

    pub fn get_jump_path(&self, index: usize) -> Option<usize> {
        self.jump_path.get(index).cloned()
    }

    pub fn get_current_instruction(&self) -> Option<(Instruction, Option<usize>)> {
        self.get_instruction(self.get_instruction_cursor())
    }

    pub fn advance_instruction_cursor(&mut self) -> Result<(), String> {
        self.instruction_cursor += 1;
        Ok(())
    }

    fn cache_add<T: SimpleData>(&mut self, value: T) -> Result<usize, DataError> {
        let mut h = DefaultHasher::new();
        value.hash(&mut h);
        value.get_type().hash(&mut h);
        let hv = h.finish();

        match self.cache.get(&hv) {
            Some(addr) => Ok(*addr),
            None => {
                let addr = self.simple_data.len();
                self.simple_data.push(value);
                self.cache.insert(hv, addr);
                Ok(addr)
            }
        }
    }
}

#[derive(Debug)]
pub struct DataError {
    message: String,
}

impl Display for DataError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message.as_str())
    }
}

impl std::error::Error for DataError {}

impl From<String> for DataError {
    fn from(s: String) -> Self {
        DataError { message: s }
    }
}

impl GarnishLangRuntimeData for SimpleRuntimeData {
    type Error = DataError;
    type DataLease = usize;
    type Integer = i32;
    type Symbol = u64;
    type Size = usize;

    fn get_data_type(&self, index: usize) -> Result<ExpressionDataType, Self::Error> {
        let d = self.get(index)?;
        let i = &d.data;
        let t = if i.is::<UnitData>() {
            ExpressionDataType::Unit
        } else if i.is::<TrueData>() {
            ExpressionDataType::True
        } else if i.is::<FalseData>() {
            ExpressionDataType::False
        } else if i.is::<IntegerData>() {
            ExpressionDataType::Integer
        } else if i.is::<SymbolData>() {
            ExpressionDataType::Symbol
        } else if i.is::<ExternalData>() {
            ExpressionDataType::External
        } else if i.is::<ExpressionData>() {
            ExpressionDataType::Expression
        } else if i.is::<PairData>() {
            ExpressionDataType::Pair
        } else if i.is::<ListData>() {
            ExpressionDataType::List
        } else {
            Err(format!("No data type for object at index {:?}.", index))?
        };

        Ok(t)
    }

    fn get_integer(&self, index: usize) -> Result<i32, Self::Error> {
        Ok(self.get(index)?.as_integer()?.value())
    }

    fn get_symbol(&self, index: usize) -> Result<u64, Self::Error> {
        Ok(self.get(index)?.as_symbol()?.value())
    }

    fn get_expression(&self, index: usize) -> Result<usize, Self::Error> {
        Ok(self.get(index)?.as_expression()?.value())
    }

    fn get_external(&self, index: usize) -> Result<usize, Self::Error> {
        Ok(self.get(index)?.as_external()?.value())
    }

    fn get_pair(&self, index: usize) -> Result<(usize, usize), Self::Error> {
        let pair = self.get(index)?.as_pair()?;
        Ok((pair.left(), pair.right()))
    }

    fn get_list_len(&self, index: usize) -> Result<usize, Self::Error> {
        Ok(self.get(index)?.as_list()?.items().len())
    }

    fn get_list_item(&self, list_index: usize, item_index: i32) -> Result<usize, Self::Error> {
        match self.get(list_index)?.as_list()?.items().get(item_index as usize) {
            None => Err(format!("No list item at index {:?} for list at addr {:?}", item_index, list_index))?,
            Some(v) => Ok(*v),
        }
    }

    fn get_list_associations_len(&self, index: usize) -> Result<usize, Self::Error> {
        Ok(self.get(index)?.as_list()?.associations().len())
    }

    fn get_list_association(&self, list_index: usize, item_index: i32) -> Result<usize, Self::Error> {
        match self.get(list_index)?.as_list()?.associations().get(item_index as usize) {
            None => Err(format!("No list item at index {:?} for list at addr {:?}", item_index, list_index))?,
            Some(v) => Ok(*v),
        }
    }

    fn add_unit(&mut self) -> Result<usize, Self::Error> {
        Ok(0)
    }

    fn add_false(&mut self) -> Result<usize, Self::Error> {
        Ok(1)
    }

    fn add_true(&mut self) -> Result<usize, Self::Error> {
        Ok(2)
    }

    fn add_integer(&mut self, value: i32) -> Result<usize, Self::Error> {
        self.cache_add(IntegerData::from(value))
    }

    fn add_symbol(&mut self, value: &str) -> Result<usize, Self::Error> {
        let sym_val = symbol_value(value);
        self.symbols.insert(value.to_string(), sym_val);
        self.cache_add(SymbolData::from(sym_val))
    }

    fn add_expression(&mut self, value: usize) -> Result<usize, Self::Error> {
        self.cache_add(ExpressionData::from(value))
    }

    fn add_external(&mut self, value: usize) -> Result<usize, Self::Error> {
        self.cache_add(ExternalData::from(value))
    }

    fn add_pair(&mut self, value: (usize, usize)) -> Result<usize, Self::Error> {
        self.simple_data.push(PairData::from(value));
        Ok(self.simple_data.len() - 1)
    }

    fn start_list(&mut self, _: usize) -> Result<(), Self::Error> {
        self.current_list = Some((vec![], vec![]));
        Ok(())
    }

    fn add_to_list(&mut self, addr: usize, is_associative: bool) -> Result<(), Self::Error> {
        match &mut self.current_list {
            None => Err(format!("Not currently creating a list."))?,
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
            None => Err(format!("Not currently creating a list."))?,
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
                            Err(format!("Could not place associative value"))?;
                        }
                    }

                    ordered[i] = item;
                }

                items.reverse();

                self.simple_data.push(ListData::from_items(items.to_vec(), ordered));
                Ok(self.simple_data.len() - 1)
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
            let association_ref = self.get_list_association(list_addr, i as i32)?;

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

    fn get_data_len(&self) -> usize {
        self.simple_data.len()
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

    fn get_value_stack_len(&self) -> usize {
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

    fn get_instruction(&self, index: usize) -> Option<(Instruction, Option<usize>)> {
        self.instructions.get(index).and_then(|i| Some((i.instruction, i.data)))
    }

    fn set_instruction_cursor(&mut self, index: usize) -> Result<(), Self::Error> {
        self.instruction_cursor = index;
        Ok(())
    }

    fn get_instruction_cursor(&self) -> usize {
        self.instruction_cursor
    }

    fn get_instruction_len(&self) -> usize {
        self.instructions.len()
    }

    fn push_jump_point(&mut self, index: usize) -> Result<(), Self::Error> {
        if index >= self.instructions.len() {
            Err(format!(
                "Specified jump point {:?} is out of bounds of instructions with length {:?}",
                index,
                self.instructions.len()
            ))?;
        }

        self.expression_table.push(index);
        Ok(())
    }

    fn get_jump_point(&self, index: usize) -> Option<usize> {
        self.expression_table.get(index).cloned()
    }

    fn get_jump_table_len(&self) -> usize {
        self.expression_table.len()
    }

    fn push_jump_path(&mut self, index: usize) -> Result<(), Self::Error> {
        self.jump_path.push(index);
        Ok(())
    }

    fn pop_jump_path(&mut self) -> Option<usize> {
        self.jump_path.pop()
    }

    fn get_jump_point_mut(&mut self, index: usize) -> Option<&mut usize> {
        self.expression_table.get_mut(index)
    }

    fn size_to_integer(from: Self::Size) -> Self::Integer {
        from as Self::Integer
    }

    fn lease_tmp_stack(&mut self) -> Result<Self::DataLease, Self::Error> {
        Ok(1)
    }

    fn push_tmp_stack(&mut self, _lease: Self::DataLease, item: Self::Size) -> Result<(), Self::Error> {
        self.lease_stack.push(item);
        Ok(())
    }

    fn pop_tmp_stack(&mut self, _lease: Self::DataLease) -> Result<Option<Self::Size>, Self::Error> {
        Ok(self.lease_stack.pop())
    }

    fn release_tmp_stack(&mut self, _lease: Self::DataLease) -> Result<(), Self::Error> {
        self.lease_stack.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{ExpressionDataType, GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn type_of() {
        let mut runtime = SimpleRuntimeData::new();
        runtime.add_integer(10).unwrap();

        assert_eq!(runtime.get_data_type(3).unwrap(), ExpressionDataType::Integer);
    }
}

#[cfg(test)]
mod data_storage {
    use crate::{symbol_value, DataCoersion, FalseData, GarnishLangRuntimeData, SimpleRuntimeData, TrueData, UnitData};

    #[test]
    fn unit() {
        let mut runtime = SimpleRuntimeData::new();

        assert_eq!(runtime.add_unit().unwrap(), 0);
        assert_eq!(runtime.add_unit().unwrap(), 0);
        assert_eq!(runtime.add_unit().unwrap(), 0);

        assert_eq!(runtime.get_data_len(), 3);
        assert_eq!(runtime.simple_data.get(0).unwrap().as_unit().unwrap(), UnitData::new());
    }

    #[test]
    fn false_data() {
        let mut runtime = SimpleRuntimeData::new();

        assert_eq!(runtime.add_false().unwrap(), 1);
        assert_eq!(runtime.add_false().unwrap(), 1);
        assert_eq!(runtime.add_false().unwrap(), 1);

        assert_eq!(runtime.get_data_len(), 3);
        assert_eq!(runtime.simple_data.get(1).unwrap().as_false().unwrap(), FalseData::new());
    }

    #[test]
    fn true_data() {
        let mut runtime = SimpleRuntimeData::new();

        assert_eq!(runtime.add_true().unwrap(), 2);
        assert_eq!(runtime.add_true().unwrap(), 2);
        assert_eq!(runtime.add_true().unwrap(), 2);

        assert_eq!(runtime.get_data_len(), 3);
        assert_eq!(runtime.simple_data.get(2).unwrap().as_true().unwrap(), TrueData::new());
    }

    #[test]
    fn integers() {
        let mut runtime = SimpleRuntimeData::new();

        let start = runtime.get_data_len();
        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let i3 = runtime.add_integer(10).unwrap();

        assert_eq!(i1, start);
        assert_eq!(i2, start + 1);
        assert_eq!(i3, i1);

        assert_eq!(runtime.get_data_len(), 5);
        assert_eq!(runtime.simple_data.get(3).unwrap().as_integer().unwrap().value(), 10);
        assert_eq!(runtime.simple_data.get(4).unwrap().as_integer().unwrap().value(), 20);
    }

    #[test]
    fn symbols() {
        let mut runtime = SimpleRuntimeData::new();

        let start = runtime.get_data_len();
        let i1 = runtime.add_symbol("sym").unwrap();
        let i2 = runtime.add_symbol("value").unwrap();
        let i3 = runtime.add_symbol("sym").unwrap();

        assert_eq!(i1, start);
        assert_eq!(i2, start + 1);
        assert_eq!(i3, i1);

        assert_eq!(runtime.get_data_len(), 5);
        assert_eq!(runtime.simple_data.get(3).unwrap().as_symbol().unwrap().value(), symbol_value("sym"));
        assert_eq!(runtime.simple_data.get(4).unwrap().as_symbol().unwrap().value(), symbol_value("value"));
    }

    #[test]
    fn expression() {
        let mut runtime = SimpleRuntimeData::new();

        let start = runtime.get_data_len();
        let i1 = runtime.add_expression(10).unwrap();
        let i2 = runtime.add_expression(20).unwrap();
        let i3 = runtime.add_expression(10).unwrap();

        assert_eq!(i1, start);
        assert_eq!(i2, start + 1);
        assert_eq!(i3, i1);

        assert_eq!(runtime.get_data_len(), 5);
        assert_eq!(runtime.simple_data.get(3).unwrap().as_expression().unwrap().value(), 10);
        assert_eq!(runtime.simple_data.get(4).unwrap().as_expression().unwrap().value(), 20);
    }

    #[test]
    fn external() {
        let mut runtime = SimpleRuntimeData::new();

        let start = runtime.get_data_len();
        let i1 = runtime.add_external(10).unwrap();
        let i2 = runtime.add_external(20).unwrap();
        let i3 = runtime.add_external(10).unwrap();

        assert_eq!(i1, start);
        assert_eq!(i2, start + 1);
        assert_eq!(i3, i1);

        assert_eq!(runtime.get_data_len(), 5);
        assert_eq!(runtime.simple_data.get(3).unwrap().as_external().unwrap().value(), 10);
        assert_eq!(runtime.simple_data.get(4).unwrap().as_external().unwrap().value(), 20);
    }

    #[test]
    fn similar_values_cache_differently() {
        let mut runtime = SimpleRuntimeData::new();

        let start = runtime.get_data_len();
        let i1 = runtime.add_external(10).unwrap();
        let i2 = runtime.add_expression(10).unwrap();

        assert_eq!(i1, start);
        assert_eq!(i2, start + 1);
        assert_ne!(i1, i2);

        assert_eq!(runtime.get_data_len(), 5);
        assert_eq!(runtime.simple_data.get(3).unwrap().as_external().unwrap().value(), 10);
        assert_eq!(runtime.simple_data.get(4).unwrap().as_expression().unwrap().value(), 10);
    }
}

#[cfg(test)]
mod leases {
    use crate::{GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn lease_data() {
        let mut runtime = SimpleRuntimeData::new();

        let lease = runtime.lease_tmp_stack().unwrap();

        assert_eq!(lease, 1);
    }

    #[test]
    fn push_with_lease() {
        let mut runtime = SimpleRuntimeData::new();

        let lease = runtime.lease_tmp_stack().unwrap();
        runtime.push_tmp_stack(lease, 5).unwrap();

        assert_eq!(runtime.lease_stack.get(0).unwrap(), &5);
    }

    #[test]
    fn pop_with_lease() {
        let mut runtime = SimpleRuntimeData::new();

        let lease = runtime.lease_tmp_stack().unwrap();
        runtime.push_tmp_stack(lease, 5).unwrap();

        assert_eq!(runtime.pop_tmp_stack(lease).unwrap(), Some(5));
        assert!(runtime.lease_stack.is_empty());
    }

    #[test]
    fn end_lease() {
        let mut runtime = SimpleRuntimeData::new();

        let lease = runtime.lease_tmp_stack().unwrap();
        runtime.push_tmp_stack(lease, 5).unwrap();
        runtime.release_tmp_stack(lease).unwrap();

        assert!(runtime.lease_stack.is_empty());
    }
}