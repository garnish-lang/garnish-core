use std::collections::HashMap;
use std::vec;

use crate::expression_data::*;
use crate::instruction::*;
use crate::result::{error, GarnishLangRuntimeResult, GarnishLangRuntimeState};
use crate::GarnishLangRuntimeData;
use log::trace;

#[derive(Debug)]
pub struct GarnishLangRuntime {
    data: Vec<ExpressionData>,
    reference_stack: Vec<usize>,
    instructions: Vec<InstructionData>,
    instruction_cursor: usize,
    results: Vec<usize>,
    jump_path: Vec<usize>,
    inputs: Vec<usize>,
    symbols: HashMap<String, u64>,
}

impl GarnishLangRuntime {
    pub fn new() -> Self {
        GarnishLangRuntime {
            data: vec![],
            reference_stack: vec![],
            instructions: vec![InstructionData {
                instruction: Instruction::EndExecution,
                data: None,
            }],
            instruction_cursor: 1,
            results: vec![],
            jump_path: vec![],
            inputs: vec![],
            symbols: HashMap::new(),
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

    pub fn get_data(&self, index: usize) -> Option<&ExpressionData> {
        self.data.get(index)
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

    pub fn get_result(&self, i: usize) -> Option<&ExpressionData> {
        match self.results.get(i) {
            None => None,
            Some(index) => self.data.get(*index),
        }
    }

    pub fn clear_results(&mut self) -> GarnishLangRuntimeResult {
        self.results.clear();
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

        self.add_reference_data(match self.inputs.last() {
            None => Err(error(format!("No inputs available to put reference.")))?,
            Some(r) => *r,
        })
    }

    pub fn push_input(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Push Input");

        self.inputs.push(match self.data.len() > 0 {
            true => self.data.len() - 1,
            false => Err(error(format!("No data available to push as input.")))?,
        });

        Ok(())
    }

    pub fn put_result(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Put Result");

        self.add_reference_data(match self.results.last() {
            None => Err(error(format!("No inputs available to put reference.")))?,
            Some(r) => *r,
        })
    }

    pub fn perform_addition(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Addition");
        match self.data.len() {
            0 | 1 => Err(error(format!("Not enough data to perform addition operation."))),
            // 2 and greater
            _ => {
                let right_addr = self.addr_of_raw_data(self.data.len() - 1)?;
                let left_addr = self.addr_of_raw_data(self.data.len() - 2)?;
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

                        self.add_data(ExpressionData::integer(left + right))?;

                        Ok(())
                    }
                    (l, r) => Err(error(format!("Cannot perform addition with types {:?} and {:?}", l, r))),
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
                self.results.push(self.addr_of_raw_data(self.data.len() - 1)?);
            }
            Some(jump_point) => {
                self.instruction_cursor = jump_point;
            }
        }

        Ok(())
    }

    pub fn output_result(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Output Result");
        match self.data.len() {
            0 => Err(error(format!("Not enough data to perform output result operation."))),
            n => {
                self.results.push(self.addr_of_raw_data(n - 1)?);
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
        match self.data.len() {
            0 | 1 => Err(error(format!("Not enough data to perform addition operation."))),
            // 2 and greater
            _ => {
                let right_addr = self.addr_of_raw_data(self.data.len() - 1)?;
                let left_addr = self.addr_of_raw_data(self.data.len() - 2)?;
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

    pub fn execute_current_instruction(&mut self) -> GarnishLangRuntimeResult<GarnishLangRuntimeData> {
        match self.instructions.get(self.instruction_cursor) {
            None => Err(error(format!("No instructions left.")))?,
            Some(instruction_data) => match instruction_data.instruction {
                Instruction::PerformAddition => self.perform_addition()?,
                Instruction::PutInput => self.put_input()?,
                Instruction::PutResult => self.put_result()?,
                Instruction::PushInput => self.push_input()?,
                Instruction::EndExpression => self.end_expression()?,
                Instruction::EqualityComparison => self.equality_comparison()?,
                Instruction::ExecuteExpression => match instruction_data.data {
                    None => Err(error(format!("No address given with execute expression instruction.")))?,
                    Some(i) => self.execute_expression(i)?,
                },
                Instruction::JumpIfTrue => match instruction_data.data {
                    None => Err(error(format!("No address given with execute expression instruction.")))?,
                    Some(i) => self.jump_if_true(i)?,
                },
                Instruction::JumpIfFalse => match instruction_data.data {
                    None => Err(error(format!("No address given with execute expression instruction.")))?,
                    Some(i) => self.jump_if_false(i)?,
                },
                Instruction::Put => match instruction_data.data {
                    None => Err(error(format!("No address given with put instruction.")))?,
                    Some(i) => self.put(i)?,
                },
                Instruction::EndExecution => self.end_execution()?,
                Instruction::Jump => match instruction_data.data {
                    None => Err(error(format!("No address given with execute expression instruction.")))?,
                    Some(i) => self.jump(i)?,
                },
                Instruction::MakePair => self.make_pair()?,
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

    fn addr_of_raw_data<'a>(&self, addr: usize) -> GarnishLangRuntimeResult<usize> {
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
}

#[cfg(test)]
mod tests {
    use crate::{ExpressionData, ExpressionDataType, GarnishLangRuntime, Instruction};

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
        runtime.perform_addition().unwrap();

        assert_eq!(runtime.data.get(0).unwrap().bytes, 30i64.to_le_bytes());
    }

    #[test]
    fn perform_addition_through_references() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::reference(0)).unwrap();
        runtime.add_data(ExpressionData::reference(1)).unwrap();
        runtime.perform_addition().unwrap();

        assert_eq!(runtime.data.get(2).unwrap().bytes, 30i64.to_le_bytes());
    }

    #[test]
    fn perform_addition_with_non_integers() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol(&"sym1".to_string(), 1)).unwrap();
        runtime.add_data(ExpressionData::symbol(&"sym2".to_string(), 2)).unwrap();
        let result = runtime.perform_addition();

        assert!(result.is_err());
    }

    #[test]
    fn output_result() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.output_result().unwrap();

        assert_eq!(runtime.get_result(0).unwrap().bytes, 10i64.to_le_bytes());
    }

    #[test]
    fn clear_results() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.output_result().unwrap();

        runtime.clear_results().unwrap();

        assert!(runtime.results.is_empty());
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
        assert_eq!(runtime.get_result(0).unwrap().bytes, 10i64.to_le_bytes());
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
    }

    #[test]
    fn put_input() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.add_input_reference(1).unwrap();

        runtime.put_input().unwrap();

        assert_eq!(runtime.data.get(2).unwrap(), &ExpressionData::reference(1));
    }

    #[test]
    fn push_input() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.push_input().unwrap();

        assert_eq!(runtime.inputs.get(0).unwrap(), &1);
    }

    #[test]
    fn put_result() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.results.push(1);

        runtime.put_result().unwrap();

        assert_eq!(runtime.data.get(2).unwrap(), &ExpressionData::reference(1));
    }

    #[test]
    fn execute_current_instruction() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.execute_current_instruction().unwrap();

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

        runtime.add_instruction(Instruction::EqualityComparison, None).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.data.get(0).unwrap().as_symbol_value().unwrap(), 1);
    }

    #[test]
    fn equality_false() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

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
}
