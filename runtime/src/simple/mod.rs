use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::{collections::HashMap, hash::Hasher};

use crate::{symbol_value, AnyData, DataCoersion, EmptyContext, ExpressionData, ExpressionDataType, ExternalData, GarnishLangRuntimeContext, GarnishLangRuntimeData, GarnishLangRuntimeState, GarnishRuntime, Instruction, InstructionData, IntegerData, ListData, PairData, RuntimeError, SimpleData, SimpleDataList, SymbolData, FloatData, CharData, CharListData, ByteData, ByteListData, RangeData, SliceData};

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
    current_char_list: Option<String>,
    current_byte_list: Option<Vec<u8>>,
    symbols: HashMap<String, u64>,
    cache: HashMap<u64, usize>,
    lease_stack: Vec<usize>,
}

impl SimpleRuntimeData {
    pub fn new() -> Self {
        SimpleRuntimeData {
            register: vec![],
            simple_data: SimpleDataList::default(),
            end_of_constant_data: 0,
            values: vec![],
            instruction_cursor: 0,
            instructions: vec![],
            expression_table: vec![],
            jump_path: vec![],
            current_list: None,
            current_char_list: None,
            current_byte_list: None,
            symbols: HashMap::new(),
            cache: HashMap::new(),
            lease_stack: vec![],
        }
    }

    pub(crate) fn get(&self, index: usize) -> Result<&AnyData, DataError> {
        match self.simple_data.get(index) {
            None => Err(format!("No data at addr {:?}", index))?,
            Some(d) => Ok(d),
        }
    }

    pub fn get_symbols(&self) -> &HashMap<String, u64> {
        &self.symbols
    }

    pub fn get_registers(&self) -> &Vec<usize> {
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
    type Symbol = u64;
    type Char = char;
    type Byte = u8;
    type Integer = i32;
    type Float = f64;
    type Size = usize;

    fn get_data_type(&self, index: usize) -> Result<ExpressionDataType, Self::Error> {
        let d = self.get(index)?;

        Ok(d.get_data_type())
    }

    fn get_integer(&self, index: usize) -> Result<i32, Self::Error> {
        Ok(self.get(index)?.as_integer()?.value())
    }

    fn get_float(&self, index: usize) -> Result<f64, Self::Error> {
        Ok(self.get(index)?.as_float()?.value())
    }

    fn get_char(&self, index: Self::Size) -> Result<Self::Char, Self::Error> {
        Ok(self.get(index)?.as_char()?.value())
    }

    fn get_byte(&self, addr: Self::Size) -> Result<Self::Byte, Self::Error> {
        Ok(self.get(addr)?.as_byte()?.value())
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

    fn get_range(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size, bool, bool), Self::Error> {
        let range = self.get(addr)?.as_range()?;
        Ok((range.start(), range.end(), range.exclude_start(), range.exclude_end()))
    }

    fn get_slice(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error> {
        let slice = self.get(addr)?.as_slice()?;
        Ok((slice.list(), slice.range()))
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

    fn get_char_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        Ok(self.get(addr)?.as_char_list()?.value().len())
    }

    fn get_char_list_item(&self, addr: Self::Size, item_index: Self::Integer) -> Result<Self::Char, Self::Error> {
        match self.get(addr)?.as_char_list()?.value().chars().nth(item_index as usize) {
            None => Err(format!("No character at index {:?} for char list at {:?}", item_index, addr))?,
            Some(c) => Ok(c)
        }
    }

    fn get_byte_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error> {
        Ok(self.get(addr)?.as_byte_list()?.value().len())
    }

    fn get_byte_list_item(&self, addr: Self::Size, item_index: Self::Integer) -> Result<Self::Byte, Self::Error> {
        match self.get(addr)?.as_byte_list()?.value().get(item_index as usize) {
            None => Err(format!("No character at index {:?} for char list at {:?}", item_index, addr))?,
            Some(c) => Ok(*c)
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

    fn add_float(&mut self, value: f64) -> Result<usize, Self::Error> {
        self.cache_add(FloatData::from(value))
    }

    fn add_char(&mut self, value: Self::Char) -> Result<Self::Size, Self::Error> {
        self.cache_add(CharData::from(value))
    }

    fn add_byte(&mut self, value: Self::Byte) -> Result<Self::Size, Self::Error> {
        self.cache_add(ByteData::from(value))
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

    fn add_range(&mut self, start: Self::Size, end: Self::Size, excludes_start: bool, excludes_end: bool) -> Result<Self::Size, Self::Error> {
        self.simple_data.push(RangeData::from((start, end, excludes_start, excludes_end)));
        Ok(self.simple_data.len() - 1)
    }

    fn add_slice(&mut self, list: Self::Size, range: Self::Size) -> Result<Self::Size, Self::Error> {
        self.simple_data.push(SliceData::from((list, range)));
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

                self.simple_data.push(ListData::from_items(items.to_vec(), ordered));
                Ok(self.simple_data.len() - 1)
            }
        }
    }

    fn start_char_list(&mut self) -> Result<(), Self::Error> {
        self.current_char_list = Some(String::new());
        Ok(())
    }

    fn add_to_char_list(&mut self, c: Self::Char) -> Result<(), Self::Error> {
        match &mut self.current_char_list {
            None => Err(format!("Attempting to add to unstarted char list."))?,
            Some(s) => s.push(c)
        }

        Ok(())
    }

    fn end_char_list(&mut self) -> Result<Self::Size, Self::Error> {
        let data = match &self.current_char_list {
            None => Err(format!("Attempting to end unstarted char list."))?,
            Some(s) => CharListData::from(s.clone())
        };

        let addr = self.cache_add(data)?;

        self.current_char_list = None;

        Ok(addr)
    }

    fn start_byte_list(&mut self) -> Result<(), Self::Error> {
        self.current_byte_list = Some(Vec::new());
        Ok(())
    }

    fn add_to_byte_list(&mut self, c: Self::Byte) -> Result<(), Self::Error> {
        match &mut self.current_byte_list {
            None => Err(format!("Attempting to add to unstarted byte list."))?,
            Some(l) => l.push(c)
        }

        Ok(())
    }

    fn end_byte_list(&mut self) -> Result<Self::Size, Self::Error> {
        let data = match &self.current_byte_list {
            None => Err(format!("Attempting to end unstarted byte list."))?,
            Some(l) => ByteListData::from(l.clone())
        };

        let addr = self.cache_add(data)?;

        self.current_byte_list = None;

        Ok(addr)
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

    fn get_register_len(&self) -> Self::Size {
        self.register.len()
    }

    fn push_register(&mut self, addr: usize) -> Result<(), Self::Error> {
        self.register.push(addr);
        Ok(())
    }

    fn get_register(&self, addr: Self::Size) -> Option<Self::Size>{
        self.register.get(addr).cloned()
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

    fn push_instruction(&mut self, instruction: Instruction, data: Option<usize>) -> Result<usize, Self::Error> {
        self.instructions.push(InstructionData::new(instruction, data));
        Ok(self.instructions.len() - 1)
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
        // if index >= self.instructions.len() {
        //     Err(format!(
        //         "Specified jump point {:?} is out of bounds of instructions with length {:?}",
        //         index,
        //         self.instructions.len()
        //     ))?;
        // }

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

    fn integer_to_float(from: Self::Integer) -> Self::Float {
        from as Self::Float
    }

    // Parsing

    fn parse_char_list(from: &str) -> Vec<Self::Char> {
        from.chars().collect()
    }

    fn parse_byte_list(from: &str) -> Vec<Self::Byte> {
        from.bytes().collect()
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
    use crate::{ExpressionDataType, GarnishLangRuntimeData, SimpleRuntimeData, Instruction};

    #[test]
    fn type_of() {
        let mut runtime = SimpleRuntimeData::new();
        runtime.add_integer(10).unwrap();

        assert_eq!(runtime.get_data_type(3).unwrap(), ExpressionDataType::Integer);
    }

    // #[test]
    // fn add_jump_point_out_of_bounds() {
    //     let mut runtime = SimpleRuntimeData::new();
    //
    //     runtime.push_instruction(Instruction::EndExpression, None).unwrap();
    //     let result = runtime.push_jump_point(5);
    //
    //     assert!(result.is_err());
    // }

    #[test]
    fn add_instruction() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.push_instruction(Instruction::Put, Some(0)).unwrap();

        assert_eq!(runtime.get_instructions().len(), 1);
    }


    #[test]
    fn get_instruction() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.push_instruction(Instruction::Put, None).unwrap();

        assert_eq!(runtime.get_instruction(0).unwrap().0, Instruction::Put);
    }

    #[test]
    fn get_current_instruction() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.push_instruction(Instruction::Put, None).unwrap();

        runtime.set_instruction_cursor(0).unwrap();

        assert_eq!(runtime.get_current_instruction().unwrap().0, Instruction::Put);
    }

    #[test]
    fn set_instruction_cursor() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.push_instruction(Instruction::Put, None).unwrap();
        runtime.push_instruction(Instruction::Put, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.set_instruction_cursor(2).unwrap();

        assert_eq!(runtime.get_current_instruction().unwrap().0, Instruction::PerformAddition);
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
    fn floats() {
        let mut runtime = SimpleRuntimeData::new();

        let start = runtime.get_data_len();
        let i1 = runtime.add_float(10.0).unwrap();
        let i2 = runtime.add_float(20.0).unwrap();
        let i3 = runtime.add_float(10.0).unwrap();

        assert_eq!(i1, start);
        assert_eq!(i2, start + 1);
        assert_eq!(i3, i1);

        assert_eq!(runtime.get_data_len(), 5);
        assert_eq!(runtime.simple_data.get(3).unwrap().as_float().unwrap().value(), 10.0);
        assert_eq!(runtime.simple_data.get(4).unwrap().as_float().unwrap().value(), 20.0);
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
