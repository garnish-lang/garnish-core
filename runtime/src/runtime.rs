use std::collections::HashMap;
use std::vec;

use crate::expression_data::*;
use crate::instruction::*;
use crate::result::{error, GarnishLangRuntimeResult, GarnishLangRuntimeState};
use crate::GarnishLangRuntimeData;
use crate::GarnishLangRuntimeError;
use log::trace;

pub trait GarnishLangRuntimeContext {
    fn resolve(&mut self, symbol_addr: usize, runtime: &mut GarnishLangRuntime) -> GarnishLangRuntimeResult<bool>;
    fn apply(&mut self, external_value: usize, input_addr: usize, runtime: &mut GarnishLangRuntime) -> GarnishLangRuntimeResult<bool>;
}

pub struct EmptyContext {}

impl GarnishLangRuntimeContext for EmptyContext {
    fn resolve(&mut self, _: usize, _: &mut GarnishLangRuntime) -> GarnishLangRuntimeResult<bool> {
        Ok(false)
    }

    fn apply(&mut self, _: usize, _: usize, _: &mut GarnishLangRuntime) -> GarnishLangRuntimeResult<bool> {
        Ok(false)
    }
}

#[derive(Debug)]
pub struct GarnishLangRuntime {
    data: Vec<ExpressionData>,
    end_of_constant_data: usize,
    reference_stack: Vec<usize>,
    instructions: Vec<InstructionData>,
    instruction_cursor: usize,
    current_result: Option<usize>,
    jump_path: Vec<usize>,
    inputs: Vec<usize>,
    symbols: HashMap<String, u64>,
    expression_table: Vec<usize>,
}

impl GarnishLangRuntime {
    pub fn new() -> Self {
        GarnishLangRuntime {
            data: vec![],
            end_of_constant_data: 0,
            reference_stack: vec![],
            instructions: vec![InstructionData {
                instruction: Instruction::EndExecution,
                data: None,
            }],
            instruction_cursor: 1,
            current_result: None,
            jump_path: vec![],
            inputs: vec![],
            symbols: HashMap::new(),
            expression_table: vec![],
        }
    }

    pub fn add_data(&mut self, data: ExpressionData) -> GarnishLangRuntimeResult<usize> {
        // Check if give a reference of reference
        // flatten reference to point to non-Reference data
        let data = match data.get_type() {
            ExpressionDataType::Reference => match self.data.get(data.as_reference().unwrap()) {
                None => Err(error(format!("Reference given doesn't not exist in data.")))?,
                Some(d) => match d.get_type() {
                    ExpressionDataType::Reference => d.clone(),
                    _ => data,
                },
            },
            ExpressionDataType::Symbol => {
                self.symbols.extend(data.symbols.clone());
                data
            }
            _ => data,
        };

        let addr = self.data.len();
        self.data.push(data);
        Ok(addr)
    }

    pub fn end_constant_data(&mut self) -> GarnishLangRuntimeResult {
        self.end_of_constant_data = self.data.len();

        Ok(())
    }

    pub fn get_end_of_constant_data(&self) -> usize {
        self.end_of_constant_data
    }

    pub fn get_data(&self, index: usize) -> Option<&ExpressionData> {
        self.data.get(index)
    }

    pub fn get_data_len(&self) -> usize {
        self.data.len()
    }

    pub fn add_expression(&mut self, index: usize) -> GarnishLangRuntimeResult {
        match index < self.instructions.len() {
            false => Err(error(format!("No instruction at {:?} to register as expression.", index)))?,
            true => {
                self.expression_table.push(index);
                Ok(())
            }
        }
    }

    pub fn add_reference_data(&mut self, reference: usize) -> GarnishLangRuntimeResult {
        self.data.push(ExpressionData::reference(reference));
        Ok(())
    }

    pub fn remove_data(&mut self, from: usize) -> GarnishLangRuntimeResult {
        match from < self.data.len() {
            true => {
                self.data = Vec::from(&self.data[..from]);
                Ok(())
            }
            false => Err(error(format!("Given address is beyond data size."))),
        }
    }

    pub fn add_instruction(&mut self, instruction: Instruction, data: Option<usize>) -> GarnishLangRuntimeResult {
        self.instructions.push(InstructionData { instruction, data });
        Ok(())
    }

    pub fn add_input_reference(&mut self, reference: usize) -> GarnishLangRuntimeResult {
        match reference < self.data.len() {
            false => Err(error(format!("Input reference beyond bounds of data."))),
            true => {
                self.inputs.push(reference);
                Ok(())
            }
        }
    }

    pub fn get_instruction(&self, i: usize) -> Option<&InstructionData> {
        self.instructions.get(i)
    }

    pub fn get_current_instruction(&self) -> Option<&InstructionData> {
        self.instructions.get(self.instruction_cursor)
    }

    pub fn get_result(&self) -> Option<&ExpressionData> {
        match self.current_result {
            None => None,
            Some(i) => self.data.get(i),
        }
    }

    pub fn clear_result(&mut self) -> GarnishLangRuntimeResult {
        self.current_result = None;
        Ok(())
    }

    pub fn set_instruction_cursor(&mut self, i: usize) -> GarnishLangRuntimeResult {
        match i >= self.instructions.len() {
            true => Err(error(format!("Instruction doesn't exist."))),
            false => {
                self.instruction_cursor = i;
                Ok(())
            }
        }
    }

    pub fn end_execution(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - End Execution");
        self.instruction_cursor = self.instructions.len();

        Ok(())
    }

    pub fn put(&mut self, i: usize) -> GarnishLangRuntimeResult {
        trace!("Instruction - Put | Data - {:?}", i);
        self.reference_stack.push(self.data.len());
        self.add_reference_data(i)
    }

    pub fn put_input(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Put Input");

        self.reference_stack.push(self.data.len());
        self.add_reference_data(match self.inputs.last() {
            None => Err(error(format!("No inputs available to put reference.")))?,
            Some(r) => *r,
        })
    }

    pub fn push_input(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Push Input");
        let r = self.addr_of_raw_data(match self.data.len() > 0 {
            true => self.data.len() - 1,
            false => Err(error(format!("No data available to push as input.")))?,
        })?;

        self.inputs.push(r);
        self.current_result = Some(r);

        Ok(())
    }

    pub fn put_result(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Put Result");

        match self.current_result {
            None => Err(error(format!("No result available to put reference.")))?,
            Some(i) => {
                self.reference_stack.push(self.data.len());
                self.add_reference_data(i)?;
            }
        }

        Ok(())
    }

    pub fn perform_addition(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Addition");
        match self.reference_stack.len() {
            0 | 1 => Err(error(format!("Not enough data to perform addition operation."))),
            // 2 and greater
            _ => {
                let right_ref = self.reference_stack.pop().unwrap();
                let left_ref = self.reference_stack.pop().unwrap();
                let right_addr = self.addr_of_raw_data(right_ref)?;
                let left_addr = self.addr_of_raw_data(left_ref)?;
                let right_data = self.get_data_internal(right_addr)?;
                let left_data = self.get_data_internal(left_addr)?;

                match (left_data.get_type(), right_data.get_type()) {
                    (ExpressionDataType::Integer, ExpressionDataType::Integer) => {
                        let left = match left_data.as_integer() {
                            Ok(v) => v,
                            Err(e) => Err(error(e))?,
                        };
                        let right = match right_data.as_integer() {
                            Ok(v) => v,
                            Err(e) => Err(error(e))?,
                        };

                        trace!("Performing {:?} + {:?}", left, right);

                        self.data.pop();
                        self.data.pop();

                        self.reference_stack.push(self.data.len());
                        self.add_data(ExpressionData::integer(left + right))?;

                        Ok(())
                    }
                    _ => {
                        self.reference_stack.push(self.data.len());
                        self.add_data(ExpressionData::unit())?;

                        Ok(())
                    }
                }
            }
        }
    }

    pub fn execute_expression(&mut self, index: usize) -> GarnishLangRuntimeResult {
        trace!("Instruction - Execute Expression | Data - {:?}", index);
        match index > 0 && index <= self.instructions.len() {
            false => Err(error(format!("Given index is out of bounds."))),
            true => {
                self.jump_path.push(self.instruction_cursor);
                self.instruction_cursor = index - 1;
                Ok(())
            }
        }
    }

    pub fn jump_if_true(&mut self, index: usize) -> GarnishLangRuntimeResult {
        trace!("Instruction - Execute Expression If True | Data - {:?}", index);
        match index > 0 && index <= self.instructions.len() {
            false => Err(error(format!("Given index is out of bounds."))),
            true => {
                let i = self.addr_of_raw_data(self.data.len() - 1)?;
                let d = self.get_data_internal(i)?;
                let remove_data = match self.get_instruction(self.instruction_cursor + 1) {
                    None => true,
                    Some(instruction) => match instruction.instruction {
                        Instruction::JumpIfFalse => false,
                        _ => true,
                    },
                };

                match d.get_type() {
                    ExpressionDataType::Symbol => match d.as_symbol_value() {
                        Err(e) => Err(error(e))?,
                        Ok(v) => match v {
                            0 => (),
                            _ => {
                                trace!(
                                    "Jumping from symbol {:?} with value {:?}",
                                    d.as_symbol_name().unwrap_or("[NO_SYMBOL_NAME]".to_string()),
                                    v
                                );
                                self.instruction_cursor = index - 1;
                            }
                        },
                    },
                    ExpressionDataType::Unit => (),
                    _ => {
                        trace!(
                            "Jumping from non Unit value of type {:?} with addr {:?}",
                            d.get_type(),
                            self.data.len() - 1
                        );
                        self.instruction_cursor = index - 1;
                    }
                };

                if remove_data {
                    self.data.pop();
                }

                Ok(())
            }
        }
    }

    pub fn jump_if_false(&mut self, index: usize) -> GarnishLangRuntimeResult {
        trace!("Instruction - Execute Expression If False | Data - {:?}", index);
        match index > 0 && index <= self.instructions.len() {
            false => Err(error(format!("Given index is out of bounds."))),
            true => {
                let i = self.addr_of_raw_data(self.data.len() - 1)?;
                let d = self.get_data_internal(i)?;
                let remove_data = match self.get_instruction(self.instruction_cursor + 1) {
                    None => true,
                    Some(instruction) => match instruction.instruction {
                        Instruction::JumpIfTrue => false,
                        _ => true,
                    },
                };

                match d.get_type() {
                    ExpressionDataType::Symbol => match d.as_symbol_value() {
                        Err(e) => Err(error(e))?,
                        Ok(v) => match v {
                            0 => {
                                trace!(
                                    "Jumping from Zero symbol with name {:?}",
                                    d.as_symbol_name().unwrap_or("[NO_SYMBOL_NAME]".to_string())
                                );
                                self.instruction_cursor = index - 1;
                            }
                            _ => (),
                        },
                    },
                    ExpressionDataType::Unit => {
                        trace!("Jumping from Unit value with addr {:?}", self.data.len() - 1);
                        self.instruction_cursor = index - 1;
                    }
                    _ => (),
                };

                if remove_data {
                    self.data.pop();
                }

                Ok(())
            }
        }
    }

    pub fn end_expression(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - End Expression");
        match self.jump_path.pop() {
            None => {
                self.instruction_cursor += 1;
                self.current_result = Some(self.addr_of_raw_data(self.data.len() - 1)?);
            }
            Some(jump_point) => {
                self.instruction_cursor = jump_point;
            }
        }

        Ok(())
    }

    pub fn push_result(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Output Result");
        match self.data.len() {
            0 => Err(error(format!("Not enough data to perform output result operation."))),
            n => {
                self.current_result = Some(self.addr_of_raw_data(n - 1)?);
                Ok(())
            }
        }
    }

    pub fn jump(&mut self, index: usize) -> GarnishLangRuntimeResult {
        trace!("Instruction - Jump | Data - {:?}", index);
        match index > 0 && index <= self.instructions.len() {
            false => Err(error(format!("Given index is out of bounds."))),
            true => {
                self.instruction_cursor = index - 1;
                Ok(())
            }
        }
    }

    pub fn equality_comparison(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Equality Comparison");
        match self.reference_stack.len() {
            0 | 1 => Err(error(format!("Not enough data to perform addition operation."))),
            // 2 and greater
            _ => {
                let right_ref = self.reference_stack.pop().unwrap();
                let left_ref = self.reference_stack.pop().unwrap();
                let right_addr = self.addr_of_raw_data(right_ref)?;
                let left_addr = self.addr_of_raw_data(left_ref)?;
                let right_data = self.get_data_internal(right_addr)?;
                let left_data = self.get_data_internal(left_addr)?;

                let result = match (left_data.get_type(), right_data.get_type()) {
                    (ExpressionDataType::Integer, ExpressionDataType::Integer) => {
                        let left = match left_data.as_integer() {
                            Ok(v) => v,
                            Err(e) => Err(error(e))?,
                        };
                        let right = match right_data.as_integer() {
                            Ok(v) => v,
                            Err(e) => Err(error(e))?,
                        };

                        trace!("Comparing {:?} == {:?}", left, right);

                        left == right
                    }
                    (l, r) => Err(error(format!("Comparison between types not implemented {:?} and {:?}", l, r)))?,
                };

                self.data.pop();
                self.data.pop();

                self.add_data(match result {
                    true => ExpressionData::symbol(&"true".to_string(), 1),
                    false => ExpressionData::symbol(&"false".to_string(), 0),
                })?;

                Ok(())
            }
        }
    }

    pub fn make_pair(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Make Pair");
        match self.reference_stack.len() {
            0 | 1 => Err(error(format!("Not enough data to make a pair value."))),
            _ => {
                let right_ref = self.reference_stack.pop().unwrap();
                let left_ref = self.reference_stack.pop().unwrap();

                let right_addr = self.addr_of_raw_data(right_ref)?;
                let left_addr = self.addr_of_raw_data(left_ref)?;

                self.reference_stack.push(self.data.len());
                self.add_data(ExpressionData::pair(left_addr, right_addr))?;

                Ok(())
            }
        }
    }

    pub fn make_list(&mut self, len: usize) -> GarnishLangRuntimeResult {
        trace!("Instruction - Make List | Length - {:?}", len);
        match self.reference_stack.len() >= len {
            false => Err(error(format!("Not enough references to make list of size {:?}", len))),
            true => {
                let mut list = vec![];
                let mut associative_list = vec![];

                for _ in 0..len {
                    let r = match self.reference_stack.pop() {
                        Some(i) => self.addr_of_raw_data(i)?,
                        None => Err(error(format!("Not enough references for list of len {:?}", len)))?,
                    };

                    let data = self.get_data_internal(r)?;

                    match data.get_type() {
                        ExpressionDataType::Pair => {
                            let pair = self.get_data_internal(r)?;
                            let left = self.addr_of_raw_data(match pair.as_pair() {
                                Err(e) => Err(error(e))?,
                                Ok((left, _)) => left,
                            })?;

                            let left_data = self.get_data_internal(left)?;
                            match left_data.get_type() {
                                ExpressionDataType::Symbol => associative_list.push(r),
                                _ => (),
                            }
                        }
                        _ => (), // Other values are just simple items
                    }

                    list.push(r);
                }

                list.reverse();

                // reorder associative values by modulo value
                let mut ordered = vec![0usize; associative_list.len()];
                for index in 0..associative_list.len() {
                    let item = associative_list[index];
                    let mut i = item % associative_list.len();
                    let mut count = 0;
                    while ordered[i] != 0 {
                        i += 1;
                        if i >= associative_list.len() {
                            i = 0;
                        }

                        count += 1;
                        if count > associative_list.len() {
                            return Err(error(format!("Could not place associative value")));
                        }
                    }

                    ordered[i] = item;
                }

                self.reference_stack.push(self.data.len());
                self.add_data(ExpressionData::list(list, ordered))?;

                Ok(())
            }
        }
    }

    pub fn apply<T: GarnishLangRuntimeContext>(&mut self, context: Option<&mut T>) -> GarnishLangRuntimeResult {
        trace!("Instruction - Apply");
        match self.reference_stack.len() {
            0 | 1 => Err(error(format!("Not enough references to perform apply."))),
            _ => {
                let right_ref = self.reference_stack.pop().unwrap();
                let left_ref = self.reference_stack.pop().unwrap();

                let right_addr = self.addr_of_raw_data(right_ref)?;
                let left_addr = self.addr_of_raw_data(left_ref)?;

                let left_data = self.get_data_internal(left_addr)?;
                match left_data.get_type() {
                    ExpressionDataType::Expression => {
                        let expression_index = match left_data.as_expression() {
                            Err(err) => Err(error(err))?,
                            Ok(v) => v,
                        };

                        let next_instruction = match self.expression_table.get(expression_index) {
                            Some(v) => *v,
                            None => Err(error(format!("No expression registered at index {:?}.", expression_index)))?,
                        };

                        // Expression stores index of expression table, look up actual instruction index

                        self.jump_path.push(self.instruction_cursor);
                        self.set_instruction_cursor(next_instruction - 1)?;
                        self.inputs.push(right_addr);

                        Ok(())
                    }
                    ExpressionDataType::External => {
                        let external_value = match left_data.as_external() {
                            Err(err) => Err(error(err))?,
                            Ok(v) => v,
                        };

                        match context {
                            None => {
                                self.reference_stack.push(self.data.len());
                                self.add_data(ExpressionData::unit())?;
                                Ok(())
                            }
                            Some(c) => match c.apply(external_value, right_addr, self)? {
                                true => Ok(()),
                                false => {
                                    self.reference_stack.push(self.data.len());
                                    self.add_data(ExpressionData::unit())?;
                                    Ok(())
                                }
                            },
                        }
                    }
                    _ => Err(error(format!(
                        "Data type {:?} not supported on left side of apply operation.",
                        left_data.get_type()
                    ))),
                }
            }
        }
    }

    pub fn reapply(&mut self, index: usize) -> GarnishLangRuntimeResult {
        trace!("Instruction - Reapply | Data - {:?}", index);
        match self.reference_stack.len() {
            0 => Err(error(format!("Not enough references to perform reapply."))),
            _ => {
                let right_ref = self.reference_stack.pop().unwrap();

                let right_addr = self.addr_of_raw_data(right_ref)?;

                self.set_instruction_cursor(index - 1)?;
                self.inputs.pop();
                self.inputs.push(right_addr);

                Ok(())
            }
        }
    }

    pub fn access(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Access");
        match self.reference_stack.len() {
            0 | 1 => Err(error(format!("Not enough references to perform access operation."))),
            _ => {
                let right_ref = self.reference_stack.pop().unwrap();
                let left_ref = self.reference_stack.pop().unwrap();

                match self.get_access_addr(right_ref, left_ref)? {
                    None => {
                        self.reference_stack.push(self.data.len());
                        self.add_data(ExpressionData::unit())?;
                    }
                    Some(i) => {
                        self.add_reference_data(i)?;
                    }
                }
                Ok(())
            }
        }
    }

    pub fn resolve<T: GarnishLangRuntimeContext>(&mut self, context: Option<&mut T>) -> GarnishLangRuntimeResult {
        trace!("Instruction - Resolve");
        let r = self.reference_stack.pop().unwrap();
        let addr = self.addr_of_raw_data(r)?;

        // check result
        match self.current_result {
            None => (),
            Some(result_ref) => match self.get_access_addr(addr, result_ref)? {
                None => (),
                Some(i) => {
                    self.add_reference_data(i)?;
                    return Ok(());
                }
            },
        }

        // check input
        match self.inputs.last() {
            None => (),
            Some(list_ref) => match self.get_access_addr(addr, *list_ref)? {
                None => (),
                Some(i) => {
                    self.add_reference_data(i)?;
                    return Ok(());
                }
            },
        }

        // check context
        match context {
            None => (),
            Some(c) => match c.resolve(r, self)? {
                true => return Ok(()), // context resovled end look up
                false => (),           // not resolved fall through
            },
        }

        // default to unit
        self.reference_stack.push(self.data.len());
        self.add_data(ExpressionData::unit())?;
        Ok(())
    }

    pub fn execute_current_instruction<T: GarnishLangRuntimeContext>(
        &mut self,
        context: Option<&mut T>,
    ) -> GarnishLangRuntimeResult<GarnishLangRuntimeData> {
        match self.instructions.get(self.instruction_cursor) {
            None => Err(error(format!("No instructions left.")))?,
            Some(instruction_data) => match instruction_data.instruction {
                Instruction::PerformAddition => self.perform_addition()?,
                Instruction::PutInput => self.put_input()?,
                Instruction::PutResult => self.put_result()?,
                Instruction::PushInput => self.push_input()?,
                Instruction::PushResult => self.push_result()?,
                Instruction::EndExpression => self.end_expression()?,
                Instruction::Return => todo!(),
                Instruction::ReturnTo => todo!(),
                Instruction::EqualityComparison => self.equality_comparison()?,
                Instruction::ExecuteExpression => match instruction_data.data {
                    None => Err(error(format!("No address given with execute expression instruction.")))?,
                    Some(i) => self.execute_expression(i)?,
                },
                Instruction::JumpIfTrue => match instruction_data.data {
                    None => Err(error(format!("No address given with jump if true instruction.")))?,
                    Some(i) => self.jump_if_true(i)?,
                },
                Instruction::JumpIfFalse => match instruction_data.data {
                    None => Err(error(format!("No address given with jump if false instruction.")))?,
                    Some(i) => self.jump_if_false(i)?,
                },
                Instruction::Put => match instruction_data.data {
                    None => Err(error(format!("No address given with put instruction.")))?,
                    Some(i) => self.put(i)?,
                },
                Instruction::EndExecution => self.end_execution()?,
                Instruction::Jump => match instruction_data.data {
                    None => Err(error(format!("No address given with jump instruction.")))?,
                    Some(i) => self.jump(i)?,
                },
                Instruction::MakePair => self.make_pair()?,
                Instruction::MakeList => match instruction_data.data {
                    None => Err(error(format!("No address given with make list instruction.")))?,
                    Some(i) => self.make_list(i)?,
                },
                Instruction::Apply => self.apply(context)?,
                Instruction::Reapply => match instruction_data.data {
                    None => Err(error(format!("No address given with reapply instruction.")))?,
                    Some(i) => self.reapply(i)?,
                },
                Instruction::Access => self.access()?,
                Instruction::Resolve => self.resolve(context)?,
            },
        }

        self.advance_instruction()
    }

    fn get_data_internal(&self, index: usize) -> GarnishLangRuntimeResult<&ExpressionData> {
        match self.data.get(index) {
            None => Err(error(format!("No data at addr {:?}", index))),
            Some(d) => Ok(d),
        }
    }

    fn addr_of_raw_data(&self, addr: usize) -> GarnishLangRuntimeResult<usize> {
        Ok(match self.data.get(addr) {
            None => Err(error(format!("No data at addr {:?}", addr)))?,
            Some(d) => match d.get_type() {
                ExpressionDataType::Reference => match d.as_reference() {
                    Err(e) => Err(error(e))?,
                    Ok(i) => i,
                },
                _ => addr,
            },
        })
    }

    fn advance_instruction(&mut self) -> GarnishLangRuntimeResult<GarnishLangRuntimeData> {
        match self.instruction_cursor + 1 >= self.instructions.len() {
            true => Ok(GarnishLangRuntimeData::new(GarnishLangRuntimeState::End)),
            false => {
                self.instruction_cursor += 1;
                Ok(GarnishLangRuntimeData::new(GarnishLangRuntimeState::Running))
            }
        }
    }

    fn get_access_addr(&self, sym: usize, list: usize) -> GarnishLangRuntimeResult<Option<usize>> {
        let sym_addr = self.addr_of_raw_data(sym)?;
        let list_addr = self.addr_of_raw_data(list)?;

        let sym_data = self.get_data_internal(sym_addr)?;
        let list_data = self.get_data_internal(list_addr)?;

        match (list_data.get_type(), sym_data.get_type()) {
            (ExpressionDataType::List, ExpressionDataType::Symbol) => {
                let sym_val = match sym_data.as_symbol_value() {
                    Err(e) => Err(error(e))?,
                    Ok(v) => v,
                };

                let (_, assocations) = match list_data.as_list() {
                    Err(e) => Err(error(e))?,
                    Ok(v) => v,
                };

                let mut i = sym_val as usize % assocations.len();
                let mut count = 0;

                loop {
                    // check to make sure item has same symbol
                    let r = self.addr_of_raw_data(assocations[i])?;
                    let data = self.get_data_internal(r)?; // this should be a pair

                    // should have symbol on left
                    match data.get_type() {
                        ExpressionDataType::Pair => {
                            match data.as_pair() {
                                Err(e) => Err(error(e))?,
                                Ok((left, right)) => {
                                    let left_r = self.addr_of_raw_data(left)?;
                                    let left_data = self.get_data_internal(left_r)?;

                                    match left_data.get_type() {
                                        ExpressionDataType::Symbol => {
                                            let v = match left_data.as_symbol_value() {
                                                Err(e) => Err(error(e))?,
                                                Ok(v) => v,
                                            };

                                            if v == sym_val {
                                                // found match
                                                // insert pair right as value
                                                return Ok(Some(right));
                                            }
                                        }
                                        t => Err(error(format!("Association created with non-symbol type {:?} on pair left.", t)))?,
                                    }
                                }
                            };
                        }
                        t => Err(error(format!("Association created with non-pair type {:?}.", t)))?,
                    }

                    i += 1;
                    if i >= assocations.len() {
                        i = 0;
                    }

                    count += 1;
                    if count > assocations.len() {
                        return Ok(None);
                    }
                }
            }
            (l, r) => Err(error(format!(
                "Access operation with {:?} on the left and {:?} on the right is not supported.",
                l, r
            )))?,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::result::error;

    use crate::{
        EmptyContext, ExpressionData, ExpressionDataType, GarnishLangRuntime, GarnishLangRuntimeContext, GarnishLangRuntimeResult, Instruction,
    };

    #[test]
    fn create_runtime() {
        GarnishLangRuntime::new();
    }

    #[test]
    fn end_execution_inserted_for_new() {
        let runtime = GarnishLangRuntime::new();

        assert_eq!(runtime.get_instruction(0).unwrap().instruction, Instruction::EndExecution);
    }

    #[test]
    fn add_data() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(100)).unwrap();

        assert_eq!(runtime.data.len(), 1);
    }

    #[test]
    fn get_data() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(100)).unwrap();
        runtime.add_data(ExpressionData::integer(200)).unwrap();

        assert_eq!(runtime.get_data(1).unwrap().as_integer().unwrap(), 200);
    }

    #[test]
    fn end_constant_data() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(100)).unwrap();
        runtime.add_data(ExpressionData::integer(200)).unwrap();
        runtime.end_constant_data().unwrap();

        assert_eq!(runtime.get_end_of_constant_data(), 2);
    }

    #[test]
    fn add_expression() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_instruction(Instruction::EndExpression, None).unwrap();
        runtime.add_expression(1).unwrap();

        assert_eq!(runtime.expression_table.len(), 1);
        assert_eq!(*runtime.expression_table.get(0).unwrap(), 1);
    }

    #[test]
    fn add_expression_out_of_bounds() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_instruction(Instruction::EndExpression, None).unwrap();
        let result = runtime.add_expression(5);

        assert!(result.is_err());
    }

    #[test]
    fn add_data_returns_addr() {
        let mut runtime = GarnishLangRuntime::new();

        assert_eq!(runtime.add_data(ExpressionData::integer(100)).unwrap(), 0);
        assert_eq!(runtime.add_data(ExpressionData::integer(100)).unwrap(), 1);
        assert_eq!(runtime.add_data(ExpressionData::integer(100)).unwrap(), 2);
        assert_eq!(runtime.add_data(ExpressionData::integer(100)).unwrap(), 3);
    }

    #[test]
    fn add_reference_of_reference_falls_through() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(100)).unwrap();
        runtime.add_data(ExpressionData::reference(0)).unwrap();
        runtime.add_data(ExpressionData::reference(1)).unwrap();

        assert_eq!(runtime.data.get(2).unwrap().as_reference().unwrap(), 0);
    }

    #[test]
    fn push_top_reference() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(100)).unwrap();
        runtime.add_reference_data(0).unwrap();

        assert_eq!(runtime.data.len(), 2);
    }

    #[test]
    fn add_instruction() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_instruction(Instruction::Put, Some(0)).unwrap();

        assert_eq!(runtime.instructions.len(), 2);
    }

    #[test]
    fn add_input_reference() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_input_reference(0).unwrap();

        assert_eq!(runtime.inputs.get(0).unwrap().to_owned(), 0);
    }

    #[test]
    fn add_input_reference_with_data_addr() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        let addr = runtime.add_data(ExpressionData::integer(10)).unwrap();

        runtime.add_input_reference(addr).unwrap();

        assert_eq!(runtime.inputs.get(0).unwrap().to_owned(), 1);
    }

    #[test]
    fn get_instruction() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_instruction(Instruction::Put, None).unwrap();

        assert_eq!(runtime.get_instruction(1).unwrap().get_instruction(), Instruction::Put);
    }

    #[test]
    fn get_current_instruction() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_instruction(Instruction::Put, None).unwrap();

        assert_eq!(runtime.get_current_instruction().unwrap().get_instruction(), Instruction::Put);
    }

    #[test]
    fn advance_instruction() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_instruction(Instruction::Put, None).unwrap();
        runtime.add_instruction(Instruction::EndExpression, None).unwrap();

        runtime.advance_instruction().unwrap();

        assert_eq!(runtime.get_current_instruction().unwrap().get_instruction(), Instruction::EndExpression);
    }

    #[test]
    fn set_instruction_cursor() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_instruction(Instruction::Put, None).unwrap();
        runtime.add_instruction(Instruction::Put, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.set_instruction_cursor(3).unwrap();

        assert_eq!(runtime.get_current_instruction().unwrap().get_instruction(), Instruction::PerformAddition);
    }

    #[test]
    fn end_execution() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_instruction(Instruction::Put, None).unwrap();
        runtime.add_instruction(Instruction::Put, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.set_instruction_cursor(3).unwrap();

        runtime.end_execution().unwrap();

        assert_eq!(runtime.instruction_cursor, 4);
    }

    #[test]
    fn perform_addition() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        let result = runtime.perform_addition();

        assert!(result.is_err());
    }

    #[test]
    fn perform_addition_through_references() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_reference_data(0).unwrap();
        runtime.add_reference_data(1).unwrap();

        runtime.reference_stack.push(0);
        runtime.reference_stack.push(1);

        runtime.perform_addition().unwrap();

        assert_eq!(runtime.reference_stack, vec![2]);
        assert_eq!(runtime.data.get(2).unwrap().bytes, 30i64.to_le_bytes());
    }

    #[test]
    fn perform_addition_with_non_integers() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol(&"sym1".to_string(), 1)).unwrap();
        runtime.add_data(ExpressionData::symbol(&"sym2".to_string(), 2)).unwrap();

        runtime.reference_stack.push(0);
        runtime.reference_stack.push(1);

        runtime.perform_addition().unwrap();

        assert_eq!(runtime.reference_stack, vec![2]);
        assert_eq!(runtime.data.get(2).unwrap().get_type(), ExpressionDataType::Unit);
    }

    #[test]
    fn push_result() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_instruction(Instruction::PushResult, None).unwrap();
        runtime.push_result().unwrap();

        assert_eq!(runtime.get_result().unwrap().bytes, 10i64.to_le_bytes());
    }

    #[test]
    fn clear_result() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.push_result().unwrap();

        runtime.clear_result().unwrap();

        assert!(runtime.current_result.is_none());
    }

    #[test]
    fn execute_expression() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_instruction(Instruction::ExecuteExpression, Some(0)).unwrap();
        runtime.add_instruction(Instruction::Put, Some(0)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.instruction_cursor = 1;
        runtime.execute_expression(2).unwrap();

        assert_eq!(runtime.instruction_cursor, 1);
        assert_eq!(runtime.jump_path.get(0).unwrap().to_owned(), 1)
    }

    #[test]
    fn end_expression() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_instruction(Instruction::Put, Some(0)).unwrap();

        runtime.end_expression().unwrap();

        assert_eq!(runtime.instruction_cursor, 2);
        assert_eq!(runtime.get_result().unwrap().bytes, 10i64.to_le_bytes());
    }

    #[test]
    fn end_expression_with_path() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_instruction(Instruction::Put, Some(0)).unwrap();
        runtime.add_instruction(Instruction::Put, Some(0)).unwrap();
        runtime.add_instruction(Instruction::EndExpression, Some(0)).unwrap();
        runtime.add_instruction(Instruction::Put, Some(0)).unwrap();
        runtime.add_instruction(Instruction::ExecuteExpression, Some(0)).unwrap();

        runtime.jump_path.push(4);
        runtime.set_instruction_cursor(2).unwrap();
        runtime.end_expression().unwrap();

        assert_eq!(runtime.instruction_cursor, 4);
    }

    #[test]
    fn put() {
        let mut runtime = GarnishLangRuntime::new();
        runtime.add_data(ExpressionData::integer(10)).unwrap();

        runtime.put(0).unwrap();

        assert_eq!(runtime.data.get(1).unwrap().as_reference().unwrap(), 0);
        assert_eq!(*runtime.reference_stack.get(0).unwrap(), 1);
    }

    #[test]
    fn put_input() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.add_input_reference(1).unwrap();

        runtime.put_input().unwrap();

        assert_eq!(runtime.data.get(2).unwrap(), &ExpressionData::reference(1));
        assert_eq!(*runtime.reference_stack.get(0).unwrap(), 2);
    }

    #[test]
    fn push_input() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.push_input().unwrap();

        assert_eq!(runtime.inputs.get(0).unwrap(), &1);
        assert_eq!(runtime.current_result.unwrap(), 1usize);
    }

    #[test]
    fn put_result() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.current_result = Some(1);

        runtime.put_result().unwrap();

        assert_eq!(runtime.data.get(2).unwrap(), &ExpressionData::reference(1));
        assert_eq!(*runtime.reference_stack.get(0).unwrap(), 2);
    }

    #[test]
    fn execute_current_instruction() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.reference_stack.push(0);
        runtime.reference_stack.push(1);

        runtime.execute_current_instruction::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.data.get(0).unwrap().bytes, 30i64.to_le_bytes());
    }

    #[test]
    fn add_symbol() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol(&"false".to_string(), 0)).unwrap();

        let false_sym = runtime.symbols.get("false").unwrap();
        let false_data = runtime.data.get(0).unwrap();

        assert_eq!(false_sym, &0);
        assert_eq!(false_data.as_symbol_name().unwrap(), "false".to_string());
        assert_eq!(false_data.as_symbol_value().unwrap(), 0u64);
    }

    #[test]
    fn jump_if_true_when_true() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol(&"true".to_string(), 1)).unwrap();
        runtime.add_instruction(Instruction::JumpIfTrue, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.jump_if_true(3).unwrap();

        assert!(runtime.data.is_empty());
        assert_eq!(runtime.instruction_cursor, 2);
    }

    #[test]
    fn jump_if_true_when_unit() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::unit()).unwrap();
        runtime.add_instruction(Instruction::JumpIfTrue, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.jump_if_true(3).unwrap();

        assert!(runtime.data.is_empty());
        assert_eq!(runtime.instruction_cursor, 1);
    }

    #[test]
    fn jump_if_true_when_false() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol(&"false".to_string(), 0)).unwrap();
        runtime.add_instruction(Instruction::JumpIfTrue, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.jump_if_true(3).unwrap();

        assert!(runtime.data.is_empty());
        assert_eq!(runtime.instruction_cursor, 1);
    }

    #[test]
    fn jump_if_false_when_true() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol(&"true".to_string(), 1)).unwrap();
        runtime.add_instruction(Instruction::JumpIfFalse, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.jump_if_false(3).unwrap();

        assert!(runtime.data.is_empty());
        assert_eq!(runtime.instruction_cursor, 1);
    }

    #[test]
    fn jump_if_false_when_unit() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::unit()).unwrap();
        runtime.add_instruction(Instruction::JumpIfFalse, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.jump_if_false(3).unwrap();

        assert!(runtime.data.is_empty());
        assert_eq!(runtime.instruction_cursor, 2);
    }

    #[test]
    fn jump_if_false_when_false() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol(&"false".to_string(), 0)).unwrap();
        runtime.add_instruction(Instruction::JumpIfFalse, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.jump_if_false(3).unwrap();

        assert!(runtime.data.is_empty());
        assert_eq!(runtime.instruction_cursor, 2);
    }

    #[test]
    fn conditional_execute_double_check_removes_data_after_last_true_false() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol(&"false".to_string(), 0)).unwrap();
        runtime.add_instruction(Instruction::JumpIfTrue, Some(3)).unwrap();
        runtime.add_instruction(Instruction::JumpIfFalse, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.jump_if_true(4).unwrap();

        assert_eq!(runtime.data.len(), 1);

        runtime.jump_if_false(4).unwrap();

        assert!(runtime.data.is_empty());
        assert_eq!(runtime.instruction_cursor, 3);
    }

    #[test]
    fn conditional_execute_double_check_removes_data_after_last_false_true() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol(&"true".to_string(), 0)).unwrap();
        runtime.add_instruction(Instruction::JumpIfFalse, Some(3)).unwrap();
        runtime.add_instruction(Instruction::JumpIfTrue, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.jump_if_false(4).unwrap();

        assert_eq!(runtime.data.len(), 1);

        runtime.jump_if_true(4).unwrap();

        assert!(runtime.data.is_empty());
        assert_eq!(runtime.instruction_cursor, 3);
    }

    #[test]
    fn jump() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol(&"false".to_string(), 0)).unwrap();
        runtime.add_instruction(Instruction::JumpIfFalse, Some(3)).unwrap();
        runtime.add_instruction(Instruction::Jump, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.jump(4).unwrap();

        assert!(runtime.jump_path.is_empty());
        assert_eq!(runtime.instruction_cursor, 3);
    }

    #[test]
    fn remove_data() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol(&"false".to_string(), 0)).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        let addr = runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.remove_data(addr).unwrap();

        assert_eq!(runtime.data.len(), 4);
    }

    #[test]
    fn remove_data_out_of_bounds() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol(&"false".to_string(), 0)).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        let result = runtime.remove_data(10);

        assert!(result.is_err());
    }

    #[test]
    fn equality_true() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();

        runtime.reference_stack.push(0);
        runtime.reference_stack.push(1);

        runtime.add_instruction(Instruction::EqualityComparison, None).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.data.get(0).unwrap().as_symbol_value().unwrap(), 1);
    }

    #[test]
    fn equality_false() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.reference_stack.push(0);
        runtime.reference_stack.push(1);

        runtime.add_instruction(Instruction::EqualityComparison, None).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.data.get(0).unwrap().as_symbol_value().unwrap(), 0);
    }

    #[test]
    fn make_pair() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"my_symbol".to_string())).unwrap();

        runtime.reference_stack.push(0);
        runtime.reference_stack.push(1);

        runtime.add_instruction(Instruction::MakePair, None).unwrap();

        runtime.make_pair().unwrap();

        assert_eq!(runtime.data.get(2).unwrap().get_type(), ExpressionDataType::Pair);
        assert_eq!(runtime.data.get(2).unwrap().as_pair().unwrap(), (0, 1));

        assert_eq!(runtime.reference_stack.len(), 1);
        assert_eq!(*runtime.reference_stack.get(0).unwrap(), 2);
    }

    #[test]
    fn make_list() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.reference_stack.push(0);
        runtime.reference_stack.push(1);
        runtime.reference_stack.push(2);

        runtime.add_instruction(Instruction::MakeList, Some(3)).unwrap();

        runtime.make_list(3).unwrap();

        let list = runtime.data.get(3).unwrap().as_list().unwrap();
        assert_eq!(list, (vec![0, 1, 2], vec![]));
    }

    #[test]
    fn make_list_with_associations() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"two".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"three".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(30)).unwrap();
        // 6
        runtime.add_data(ExpressionData::pair(0, 1)).unwrap();
        runtime.add_data(ExpressionData::pair(2, 3)).unwrap();
        runtime.add_data(ExpressionData::pair(4, 5)).unwrap();

        runtime.reference_stack.push(6);
        runtime.reference_stack.push(7);
        runtime.reference_stack.push(8);

        runtime.add_instruction(Instruction::MakeList, Some(3)).unwrap();

        runtime.make_list(3).unwrap();

        println!("{:?}", runtime.data.get(9).unwrap());
        let list = runtime.data.get(9).unwrap().as_list().unwrap();
        assert_eq!(list, (vec![6, 7, 8], vec![6, 7, 8]));
    }

    #[test]
    fn apply() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::expression(0)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        // 1
        runtime.add_instruction(Instruction::Put, Some(0)).unwrap();
        runtime.add_instruction(Instruction::PutInput, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::EndExpression, None).unwrap();

        // 5
        runtime.add_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.add_instruction(Instruction::Put, Some(2)).unwrap();
        runtime.add_instruction(Instruction::Apply, None).unwrap();

        runtime.expression_table.push(1);

        runtime.reference_stack.push(1);
        runtime.reference_stack.push(2);

        runtime.set_instruction_cursor(7).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        assert_eq!(*runtime.inputs.get(0).unwrap(), 2);
        assert_eq!(runtime.instruction_cursor, 0);
        assert_eq!(*runtime.jump_path.get(0).unwrap(), 7);
    }

    #[test]
    fn reapply() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::expression(0)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(30)).unwrap();
        runtime.add_data(ExpressionData::integer(40)).unwrap();

        // 1
        runtime.add_instruction(Instruction::Put, Some(0)).unwrap();
        runtime.add_instruction(Instruction::PutInput, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PutResult, None).unwrap();
        runtime.add_instruction(Instruction::Reapply, None).unwrap();
        runtime.add_instruction(Instruction::EndExpression, None).unwrap();

        // 7
        runtime.add_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.add_instruction(Instruction::Put, Some(2)).unwrap();
        runtime.add_instruction(Instruction::Apply, None).unwrap();

        runtime.expression_table.push(1);

        runtime.reference_stack.push(4);

        runtime.inputs.push(2);
        runtime.current_result = Some(4);
        runtime.jump_path.push(9);

        runtime.set_instruction_cursor(5).unwrap();

        runtime.reapply(1).unwrap();

        assert_eq!(runtime.inputs.len(), 1);
        assert_eq!(*runtime.inputs.get(0).unwrap(), 4);
        assert_eq!(runtime.instruction_cursor, 0);
        assert_eq!(*runtime.jump_path.get(0).unwrap(), 9);
    }

    #[test]
    fn access() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(0, 1)).unwrap();
        runtime.add_data(ExpressionData::list(vec![2], vec![2])).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();

        runtime.add_instruction(Instruction::Access, None).unwrap();

        runtime.reference_stack.push(3);
        runtime.reference_stack.push(4);

        runtime.access().unwrap();

        assert_eq!(runtime.get_data(5).unwrap().as_reference().unwrap(), 1);
    }

    #[test]
    fn access_with_non_existent_key() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(0, 1)).unwrap();
        runtime.add_data(ExpressionData::list(vec![2], vec![2])).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"two".to_string())).unwrap();

        runtime.add_instruction(Instruction::Access, None).unwrap();

        runtime.reference_stack.push(3);
        runtime.reference_stack.push(4);

        runtime.access().unwrap();

        assert_eq!(runtime.reference_stack.len(), 1);
        assert_eq!(runtime.get_data(5).unwrap().get_type(), ExpressionDataType::Unit);
    }

    #[test]
    fn resolve_from_result() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(0, 1)).unwrap();
        runtime.add_data(ExpressionData::list(vec![2], vec![2])).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();

        runtime.add_instruction(Instruction::Resolve, None).unwrap();

        runtime.reference_stack.push(4);

        runtime.current_result = Some(3);

        runtime.resolve::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_data(5).unwrap().as_reference().unwrap(), 1);
    }

    #[test]
    fn resolve_from_input() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(0, 1)).unwrap();
        runtime.add_data(ExpressionData::list(vec![2], vec![2])).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();

        runtime.add_instruction(Instruction::Resolve, None).unwrap();

        runtime.reference_stack.push(4);

        runtime.add_input_reference(3).unwrap();

        runtime.resolve::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_data(5).unwrap().as_reference().unwrap(), 1);
    }

    #[test]
    fn resolve_not_found_is_unit() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(0, 1)).unwrap();
        runtime.add_data(ExpressionData::list(vec![2], vec![2])).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"two".to_string())).unwrap();

        runtime.add_instruction(Instruction::Resolve, None).unwrap();

        runtime.reference_stack.push(4);

        runtime.resolve::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_data(5).unwrap().get_type(), ExpressionDataType::Unit);
    }

    #[test]
    fn resolve_from_context() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();

        runtime.add_instruction(Instruction::Resolve, None).unwrap();

        runtime.reference_stack.push(0);

        struct MyContext {}

        impl GarnishLangRuntimeContext for MyContext {
            fn resolve(&mut self, sym_addr: usize, runtime: &mut GarnishLangRuntime) -> GarnishLangRuntimeResult<bool> {
                match runtime.get_data(sym_addr) {
                    None => Err(error(format!("Symbol address, {:?}, given to resolve not found in runtime.", sym_addr)))?,
                    Some(data) => match data.get_type() {
                        ExpressionDataType::Symbol => {
                            let addr = runtime.get_data_len();
                            runtime.add_data(ExpressionData::integer(100))?;
                            runtime.put(addr)?;
                            Ok(true)
                        }
                        t => Err(error(format!("Address given to resolve is of type {:?}. Expected symbol type.", t)))?,
                    },
                }
            }

            fn apply(&mut self, _: usize, _: usize, _: &mut GarnishLangRuntime) -> GarnishLangRuntimeResult<bool> {
                Ok(false)
            }
        }

        let mut context = MyContext {};

        runtime.resolve(Some(&mut context)).unwrap();

        assert_eq!(runtime.get_data(1).unwrap().as_integer().unwrap(), 100);
        assert_eq!(runtime.reference_stack.get(0).unwrap(), &2);
        assert_eq!(runtime.get_data(2).unwrap().as_reference().unwrap(), 1);
    }

    #[test]
    fn apply_from_context() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::external(3)).unwrap();
        runtime.add_data(ExpressionData::integer(100)).unwrap();

        runtime.add_instruction(Instruction::Resolve, None).unwrap();

        runtime.reference_stack.push(0);
        runtime.reference_stack.push(1);

        struct MyContext {}

        impl GarnishLangRuntimeContext for MyContext {
            fn resolve(&mut self, _: usize, _: &mut GarnishLangRuntime) -> GarnishLangRuntimeResult<bool> {
                Ok(false)
            }

            fn apply(&mut self, external_value: usize, input_addr: usize, runtime: &mut GarnishLangRuntime) -> GarnishLangRuntimeResult<bool> {
                assert_eq!(external_value, 3);

                let value = match runtime.get_data(input_addr) {
                    None => Err(error(format!("Input address given to external apply doesn't have data.")))?,
                    Some(data) => match data.get_type() {
                        ExpressionDataType::Integer => match data.as_integer() {
                            Err(e) => Err(error(e))?,
                            Ok(i) => i * 2,
                        },
                        _ => return Ok(false),
                    },
                };

                let addr = runtime.get_data_len();
                runtime.add_data(ExpressionData::integer(value))?;
                runtime.put(addr)?;
                Ok(true)
            }
        }

        let mut context = MyContext {};

        runtime.apply(Some(&mut context)).unwrap();

        assert_eq!(runtime.get_data(2).unwrap().as_integer().unwrap(), 200);
        assert_eq!(runtime.reference_stack.get(0).unwrap(), &3);
        assert_eq!(runtime.get_data(3).unwrap().as_reference().unwrap(), 2);
    }
}
