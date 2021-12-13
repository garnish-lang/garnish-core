mod apply;
mod arithmetic;
mod comparisons;
mod data;
mod jumps;
mod list;
mod pair;
mod put;
mod resolve;

use std::collections::HashMap;
use std::vec;

use crate::expression_data::*;
use crate::instruction::*;
use crate::result::{error, GarnishLangRuntimeResult, GarnishLangRuntimeState};
use crate::GarnishLangRuntimeData;
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

    pub fn add_expression(&mut self, index: usize) -> GarnishLangRuntimeResult {
        match index < self.instructions.len() {
            false => Err(error(format!("No instruction at {:?} to register as expression.", index)))?,
            true => {
                self.expression_table.push(index);
                Ok(())
            }
        }
    }

    pub fn add_instruction(&mut self, instruction: Instruction, data: Option<usize>) -> GarnishLangRuntimeResult {
        self.instructions.push(InstructionData { instruction, data });
        Ok(())
    }

    pub fn get_instruction(&self, i: usize) -> Option<&InstructionData> {
        self.instructions.get(i)
    }

    pub fn get_current_instruction(&self) -> Option<&InstructionData> {
        self.instructions.get(self.instruction_cursor)
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

    pub fn add_input_reference(&mut self, reference: usize) -> GarnishLangRuntimeResult {
        match reference < self.data.len() {
            false => Err(error(format!("Input reference beyond bounds of data."))),
            true => {
                self.inputs.push(reference);
                Ok(())
            }
        }
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

    pub fn end_execution(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - End Execution");
        self.instruction_cursor = self.instructions.len();

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
                Instruction::EqualityComparison => self.equality_comparison()?,
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
                Instruction::JumpTo => match instruction_data.data {
                    None => Err(error(format!("No address given with jump instruction.")))?,
                    Some(i) => self.jump(i)?,
                },
                Instruction::MakePair => self.make_pair()?,
                Instruction::MakeList => match instruction_data.data {
                    None => Err(error(format!("No address given with make list instruction.")))?,
                    Some(i) => self.make_list(i)?,
                },
                Instruction::Apply => self.apply(context)?,
                Instruction::EmptyApply => self.empty_apply(context)?,
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
    use crate::{EmptyContext, ExpressionData, GarnishLangRuntime, Instruction};

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
    fn clear_result() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.push_result().unwrap();

        runtime.clear_result().unwrap();

        assert!(runtime.current_result.is_none());
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
}
