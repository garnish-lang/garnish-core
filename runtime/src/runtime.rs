use std::collections::HashMap;

use log::{trace};
use crate::GarnishLangRuntimeData;
use crate::instruction::*;
use crate::expression_data::*;
use crate::result::{error, GarnishLangRuntimeResult, GarnishLangRuntimeState};

#[derive(Debug)]
pub struct GarnishLangRuntime {
    data: Vec<ExpressionData>,
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
            instructions: vec![InstructionData { instruction: Instruction::EndExecution, data: None }],
            instruction_cursor: 1,
            results: vec![],
            jump_path: vec![],
            inputs: vec![],
            symbols: HashMap::new()
        }
    }

    pub fn add_data(&mut self, data: ExpressionData) -> GarnishLangRuntimeResult {
        // Check if give a reference of reference
        // flatten reference to point to non-Reference data
        let data = match data.get_type() {
            ExpressionDataType::Reference => match self.data.get(data.as_reference().unwrap()) {
                None => Err(error(format!("Reference given doesn't not exist in data.")))?,
                Some(d) => match d.get_type() {
                    ExpressionDataType::Reference => d.clone(),
                    _ => data
                }
            }
            ExpressionDataType::Symbol => {
                self.symbols.extend(data.symbols.clone());
                data
            }
            _ => data
        };

        self.data.push(data);
        Ok(())
    }

    pub fn add_reference_data(&mut self, reference: usize) -> GarnishLangRuntimeResult{
        self.data.push(ExpressionData::reference(reference));
        Ok(())
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
            Some(index) => self.data.get(*index)
        }
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
        self.add_reference_data(i)
    }

    pub fn put_input(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Put Input");

        self.add_reference_data(match self.inputs.last() {
            None => Err(error(format!("No inputs available to put reference.")))?,
            Some(r) => *r
        })
    }

    pub fn push_input(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Push Input");

        self.inputs.push(match self.data.len() > 0 {
            true => self.data.len() - 1,
            false => Err(error(format!("No data available to push as input.")))?
        });

        Ok(())
    }

    pub fn put_result(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Put Result");

        self.add_reference_data(match self.results.last() {
            None => Err(error(format!("No inputs available to put reference.")))?,
            Some(r) => *r
        })
    }

    pub fn perform_addition(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Addition");
        match self.data.len() {
            0 | 1 => Err(error(format!("Not enough data to perform addition operation."))),
            // 2 and greater
            _ => {
                let right_data = &self.data.pop().unwrap();
                let left_data = &self.data.pop().unwrap();

                let right_data = match right_data.get_type() {
                    ExpressionDataType::Reference => match self.data.get(match right_data.as_reference() {
                        Ok(v) => v,
                        Err(e) => Err(error(e))?
                    }) {
                        None => Err(error(format!("Reference value doesn't reference existing value.")))?,
                        Some(data) => data
                    },
                    _ => right_data
                };

                let left_data = match left_data.get_type() {
                    ExpressionDataType::Reference => match self.data.get(match left_data.as_reference() {
                        Ok(v) => v,
                        Err(e) => Err(error(e))?
                    }) {
                        None => Err(error(format!("Reference value doesn't reference existing value.")))?,
                        Some(data) => data
                    },
                    _ => left_data
                };

                let left = match left_data.as_integer() {
                    Ok(v) => v,
                    Err(e) => Err(error(e))?
                };
                let right = match right_data.as_integer() {
                    Ok(v) => v,
                    Err(e) => Err(error(e))?
                };

                trace!("Performing {:?} + {:?}", right, left);

                self.add_data(ExpressionData::integer(left + right))?;

                Ok(())
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

    pub fn execute_if_true(&mut self, index: usize) -> GarnishLangRuntimeResult {
        trace!("Instruction - Execute Expression If True | Data - {:?}", index);
        match index > 0 && index <= self.instructions.len() {
            false => Err(error(format!("Given index is out of bounds."))),
            true => {
                match self.data.last() {
                    None => Err(error(format!("No data available.")))?,
                    Some(d) => match d.get_type() {
                        ExpressionDataType::Symbol => match d.as_symbol_value() {
                            Err(e) => Err(error(e))?,
                            Ok(v) => match v {
                                0 => (),
                                _ => {
                                    self.jump_path.push(self.instruction_cursor);
                                    self.instruction_cursor = index - 1;
                                }
                            }
                        }
                        ExpressionDataType::Unit => (),
                        _ => {
                            self.jump_path.push(self.instruction_cursor);
                            self.instruction_cursor = index - 1;
                        }
                    }
                };
                Ok(())
            }
        }
    }

    pub fn execute_if_false(&mut self, index: usize) -> GarnishLangRuntimeResult {
        trace!("Instruction - Execute Expression If True | Data - {:?}", index);
        match index > 0 && index <= self.instructions.len() {
            false => Err(error(format!("Given index is out of bounds."))),
            true => {
                match self.data.last() {
                    None => Err(error(format!("No data available.")))?,
                    Some(d) => match d.get_type() {
                        ExpressionDataType::Symbol => match d.as_symbol_value() {
                            Err(e) => Err(error(e))?,
                            Ok(v) => match v {
                                0 => {
                                    self.jump_path.push(self.instruction_cursor);
                                    self.instruction_cursor = index - 1;
                                }
                                _ => ()
                            }
                        }
                        ExpressionDataType::Unit => {
                            self.jump_path.push(self.instruction_cursor);
                            self.instruction_cursor = index - 1;
                        }
                        _ => ()
                    }
                };
                Ok(())
            }
        }
    }

    pub fn end_expression(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - End Expression");
        match self.jump_path.pop() {
            None => {
                self.instruction_cursor += 1;
                self.results.push(self.data.len() - 1);
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
                self.results.push(n - 1);
                Ok(())
            }
        }
    }

    pub fn execute_current_instruction(&mut self) -> GarnishLangRuntimeResult<GarnishLangRuntimeData> {
        match self.instructions.get(self.instruction_cursor) {
            None => Err(error(format!("No instructions left.")))?,
            Some(instruction_data) => {
                match instruction_data.instruction {
                    Instruction::PerformAddition => self.perform_addition()?,
                    Instruction::EndExpression => self.end_expression()?,
                    Instruction::ExecuteExpression => match instruction_data.data {
                        None => Err(error(format!("No address given with execute expression instruction.")))?,
                        Some(i ) => self.execute_expression(i)?
                    }
                    Instruction::ExecuteIfTrue => match instruction_data.data {
                        None => Err(error(format!("No address given with execute expression instruction.")))?,
                        Some(i ) => self.execute_if_true(i)?
                    }
                    Instruction::ExecuteIfFalse => match instruction_data.data {
                        None => Err(error(format!("No address given with execute expression instruction.")))?,
                        Some(i ) => self.execute_if_false(i)?
                    }
                    Instruction::Put => match instruction_data.data {
                        None => Err(error(format!("No address given with put instruction.")))?,
                        Some(i ) => self.put(i)?
                    }
                    Instruction::EndExecution => self.end_execution()?
                }
            }
        }

        self.advance_instruction()
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
    use crate::{ExpressionData, GarnishLangRuntime, Instruction};

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
    fn perform_addition_with_references() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::reference(0)).unwrap();
        runtime.add_data(ExpressionData::reference(1)).unwrap();
        runtime.perform_addition().unwrap();

        assert_eq!(runtime.data.get(2).unwrap().bytes, 30i64.to_le_bytes());
    }

    #[test]
    fn output_result() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.output_result().unwrap();

        assert_eq!(runtime.get_result(0).unwrap().bytes, 10i64.to_le_bytes()); 
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
    fn execute_if_true_when_true() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol(&"true".to_string(), 1)).unwrap();
        runtime.add_instruction(Instruction::ExecuteIfTrue, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.execute_if_true(3).unwrap();

        assert_eq!(runtime.instruction_cursor, 2);
    }

    #[test]
    fn execute_if_true_when_unit() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::unit()).unwrap();
        runtime.add_instruction(Instruction::ExecuteIfTrue, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.execute_if_true(3).unwrap();

        assert_eq!(runtime.instruction_cursor, 1);
    }

    #[test]
    fn execute_if_true_when_false() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol(&"false".to_string(), 0)).unwrap();
        runtime.add_instruction(Instruction::ExecuteIfTrue, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.execute_if_true(3).unwrap();

        assert_eq!(runtime.instruction_cursor, 1);
    }

    #[test]
    fn execute_if_false_when_true() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol(&"true".to_string(), 1)).unwrap();
        runtime.add_instruction(Instruction::ExecuteIfFalse, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.execute_if_false(3).unwrap();

        assert_eq!(runtime.instruction_cursor, 1);
    }

    #[test]
    fn execute_if_false_when_unit() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::unit()).unwrap();
        runtime.add_instruction(Instruction::ExecuteIfFalse, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.execute_if_false(3).unwrap();

        assert_eq!(runtime.instruction_cursor, 2);
    }

    #[test]
    fn execute_if_false_when_false() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol(&"false".to_string(), 0)).unwrap();
        runtime.add_instruction(Instruction::ExecuteIfFalse, Some(3)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.execute_if_false(3).unwrap();

        assert_eq!(runtime.instruction_cursor, 2);
    }
}