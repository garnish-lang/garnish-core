mod apply;
mod arithmetic;
mod comparisons;
mod context;
mod data;
mod jumps;
mod list;
mod pair;
mod put;
mod resolve;
mod utilities;

pub use context::*;
pub use data::{GarnishLangRuntimeData, SimpleRuntimeData};

use crate::instruction::*;
use crate::result::{error, GarnishLangRuntimeResult, GarnishLangRuntimeState};
use crate::GarnishLangRuntimeInfo;
use log::trace;

use self::context::GarnishLangRuntimeContext;

#[derive(Debug)]
pub struct GarnishLangRuntime<Data> {
    data: Data,
}

impl GarnishLangRuntime<SimpleRuntimeData> {
    pub fn simple() -> Self {
        GarnishLangRuntime::<SimpleRuntimeData>::new()
    }
}

impl<Data> GarnishLangRuntime<Data>
where
    Data: GarnishLangRuntimeData,
{
    pub fn new() -> Self {
        GarnishLangRuntime { data: Data::new() }
    }

    pub fn new_with_data(data: Data) -> Self {
        GarnishLangRuntime { data }
    }

    pub fn add_expression(&mut self, index: usize) -> GarnishLangRuntimeResult {
        self.data.push_jump_point(index)
    }

    pub fn add_instruction(&mut self, instruction: Instruction, data: Option<usize>) -> GarnishLangRuntimeResult {
        self.data.push_instruction(InstructionData { instruction, data })
    }

    pub fn get_instruction(&self, i: usize) -> GarnishLangRuntimeResult<&InstructionData> {
        self.data.get_instruction(i)
    }

    pub fn get_current_instruction(&self) -> GarnishLangRuntimeResult<&InstructionData> {
        self.data.get_instruction(self.data.get_instruction_cursor()?)
    }

    pub fn set_instruction_cursor(&mut self, i: usize) -> GarnishLangRuntimeResult {
        self.data.set_instruction_cursor(i)
    }

    pub fn add_input_reference(&mut self, reference: usize) -> GarnishLangRuntimeResult {
        match reference < self.data.get_data_len() {
            false => Err(error(format!("Input reference beyond bounds of data."))),
            true => self.data.push_input(reference),
        }
    }

    pub fn get_result(&self) -> Option<usize> {
        self.data.get_result()
    }

    pub fn clear_result(&mut self) -> GarnishLangRuntimeResult {
        self.data.set_result(None)
    }

    pub fn end_execution(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - End Execution");
        self.data.set_instruction_cursor(self.data.get_instruction_len())?;

        Ok(())
    }

    pub fn execute_current_instruction<T: GarnishLangRuntimeContext>(
        &mut self,
        context: Option<&mut T>,
    ) -> GarnishLangRuntimeResult<GarnishLangRuntimeInfo> {
        let instruction_data = self.data.get_instruction(self.data.get_instruction_cursor()?)?;
        match instruction_data.instruction {
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
        };

        self.advance_instruction()
    }

    fn advance_instruction(&mut self) -> GarnishLangRuntimeResult<GarnishLangRuntimeInfo> {
        match self.data.get_instruction_cursor()? + 1 >= self.data.get_instruction_len() {
            true => Ok(GarnishLangRuntimeInfo::new(GarnishLangRuntimeState::End)),
            false => {
                self.data.advance_instruction_cursor()?;
                Ok(GarnishLangRuntimeInfo::new(GarnishLangRuntimeState::Running))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        runtime::{context::EmptyContext, data::GarnishLangRuntimeData},
        ExpressionData, ExpressionDataType, GarnishLangRuntime, Instruction,
    };

    #[test]
    fn create_runtime() {
        GarnishLangRuntime::simple();
    }

    #[test]
    fn default_data_for_new() {
        let runtime = GarnishLangRuntime::simple();

        assert_eq!(runtime.get_instruction(0).unwrap().instruction, Instruction::EndExecution);
        assert_eq!(runtime.get_data_len(), 1);
        assert_eq!(runtime.data.get_data_type(0).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn add_expression() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_instruction(Instruction::EndExpression, None).unwrap();
        runtime.add_expression(1).unwrap();

        assert_eq!(runtime.data.get_jump_points().len(), 1);
        assert_eq!(*runtime.data.get_jump_points().get(0).unwrap(), 1);
    }

    #[test]
    fn add_expression_out_of_bounds() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_instruction(Instruction::EndExpression, None).unwrap();
        let result = runtime.add_expression(5);

        assert!(result.is_err());
    }

    #[test]
    fn add_instruction() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_instruction(Instruction::Put, Some(0)).unwrap();

        assert_eq!(runtime.data.get_instructions().len(), 2);
    }

    #[test]
    fn add_input_reference() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_input_reference(0).unwrap();

        assert_eq!(runtime.data.get_input(0).unwrap().to_owned(), 0);
    }

    #[test]
    fn add_input_reference_with_data_addr() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        let addr = runtime.add_data(ExpressionData::integer(10)).unwrap();

        runtime.add_input_reference(addr).unwrap();

        assert_eq!(runtime.data.get_input(0).unwrap().to_owned(), 2);
    }

    #[test]
    fn get_instruction() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_instruction(Instruction::Put, None).unwrap();

        assert_eq!(runtime.get_instruction(1).unwrap().get_instruction(), Instruction::Put);
    }

    #[test]
    fn get_current_instruction() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_instruction(Instruction::Put, None).unwrap();

        runtime.data.set_instruction_cursor(1).unwrap();

        assert_eq!(runtime.get_current_instruction().unwrap().get_instruction(), Instruction::Put);
    }

    #[test]
    fn advance_instruction() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_instruction(Instruction::Put, None).unwrap();
        runtime.add_instruction(Instruction::EndExpression, None).unwrap();

        runtime.data.set_instruction_cursor(1).unwrap();

        runtime.advance_instruction().unwrap();

        assert_eq!(runtime.get_current_instruction().unwrap().get_instruction(), Instruction::EndExpression);
    }

    #[test]
    fn set_instruction_cursor() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_instruction(Instruction::Put, None).unwrap();
        runtime.add_instruction(Instruction::Put, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.set_instruction_cursor(3).unwrap();

        assert_eq!(runtime.get_current_instruction().unwrap().get_instruction(), Instruction::PerformAddition);
    }

    #[test]
    fn end_execution() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_instruction(Instruction::Put, None).unwrap();
        runtime.add_instruction(Instruction::Put, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.set_instruction_cursor(3).unwrap();

        runtime.end_execution().unwrap();

        assert_eq!(runtime.data.get_instruction_cursor().unwrap(), 4);
    }

    #[test]
    fn clear_result() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();

        runtime.data.push_register(1).unwrap();
        runtime.push_result().unwrap();

        runtime.clear_result().unwrap();

        assert!(runtime.data.get_result().is_none());
    }

    #[test]
    fn execute_current_instruction() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.data.push_register(1).unwrap();
        runtime.data.push_register(2).unwrap();

        runtime.set_instruction_cursor(1).unwrap();

        runtime.execute_current_instruction::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.data.get_register(), &vec![3]);
        assert_eq!(runtime.data.get_integer(3).unwrap(), 30);
    }
}
