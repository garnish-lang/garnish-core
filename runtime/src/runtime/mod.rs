mod apply;
mod arithmetic;
mod comparisons;
mod context;
mod data;
mod error;
pub mod instruction;
mod jumps;
mod list;
mod pair;
mod put;
mod resolve;
pub mod result;
pub mod types;
mod utilities;

pub use context::{EmptyContext, GarnishLangRuntimeContext};
pub use data::{GarnishLangRuntimeData, TypeConstants};
pub use error::*;
pub(crate) use utilities::*;

use log::trace;

use crate::runtime::arithmetic::perform_addition;
use crate::runtime::jumps::{end_expression, jump, jump_if_false, jump_if_true};
use crate::runtime::pair::make_pair;
use crate::runtime::put::{push_input, push_result, put, put_input};
use crate::{GarnishLangRuntimeInfo};
use apply::*;
use comparisons::equality_comparison;
use instruction::*;
use list::*;
use result::*;

pub trait GarnishRuntime<Data: GarnishLangRuntimeData> {
    fn end_execution(&mut self) -> Result<(), RuntimeError<Data::Error>>;
    fn execute_current_instruction<T: GarnishLangRuntimeContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> Result<GarnishLangRuntimeInfo, RuntimeError<Data::Error>>;

    fn apply<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>>;
    fn reapply(&mut self, index: Data::Size) -> Result<(), RuntimeError<Data::Error>>;
    fn empty_apply<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>>;

    fn perform_addition(&mut self) -> Result<(), RuntimeError<Data::Error>>;

    fn equality_comparison(&mut self) -> Result<(), RuntimeError<Data::Error>>;

    fn jump(&mut self, index: Data::Size) -> Result<(), RuntimeError<Data::Error>>;
    fn jump_if_true(&mut self, index: Data::Size) -> Result<(), RuntimeError<Data::Error>>;
    fn jump_if_false(&mut self, index: Data::Size) -> Result<(), RuntimeError<Data::Error>>;
    fn end_expression(&mut self) -> Result<(), RuntimeError<Data::Error>>;

    fn make_list(&mut self, len: Data::Size) -> Result<(), RuntimeError<Data::Error>>;
    fn access(&mut self) -> Result<(), RuntimeError<Data::Error>>;
    fn access_left_internal(&mut self) -> Result<(), RuntimeError<Data::Error>>;
    fn access_right_internal(&mut self) -> Result<(), RuntimeError<Data::Error>>;
    fn access_length_internal(&mut self) -> Result<(), RuntimeError<Data::Error>>;

    fn make_pair(&mut self) -> Result<(), RuntimeError<Data::Error>>;

    fn put(&mut self, i: Data::Size) -> Result<(), RuntimeError<Data::Error>>;
    fn put_value(&mut self) -> Result<(), RuntimeError<Data::Error>>;
    fn push_value(&mut self) -> Result<(), RuntimeError<Data::Error>>;
    fn update_value(&mut self) -> Result<(), RuntimeError<Data::Error>>;

    fn resolve<T: GarnishLangRuntimeContext<Data>>(&mut self, data: Data::Size, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>>;
}

impl<Data> GarnishRuntime<Data> for Data
where
    Data: GarnishLangRuntimeData,
{
    fn end_execution(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        trace!("Instruction - End Execution");
        Ok(self.set_instruction_cursor(self.get_instruction_len())?)
    }

    fn execute_current_instruction<T: GarnishLangRuntimeContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> Result<GarnishLangRuntimeInfo, RuntimeError<Data::Error>> {
        let (instruction, data) = match self.get_instruction(self.get_instruction_cursor()) {
            None => return Ok(GarnishLangRuntimeInfo::new(GarnishLangRuntimeState::End)),
            Some(v) => v
        };
        
        match instruction {
            Instruction::PerformAddition => self.perform_addition()?,
            Instruction::PutValue => self.put_value()?,
            Instruction::PushValue => self.push_value()?,
            Instruction::UpdateValue => self.update_value()?,
            Instruction::EndExpression => self.end_expression()?,
            Instruction::EqualityComparison => self.equality_comparison()?,
            Instruction::JumpIfTrue => match data {
                None => instruction_error(instruction, self.get_instruction_cursor())?,
                Some(i) => self.jump_if_true(i)?,
            },
            Instruction::JumpIfFalse => match data {
                None => instruction_error(instruction, self.get_instruction_cursor())?,
                Some(i) => self.jump_if_false(i)?,
            },
            Instruction::Put => match data {
                None => instruction_error(instruction, self.get_instruction_cursor())?,
                Some(i) => self.put(i)?,
            },
            Instruction::EndExecution => self.end_execution()?,
            Instruction::JumpTo => match data {
                None => instruction_error(instruction, self.get_instruction_cursor())?,
                Some(i) => self.jump(i)?,
            },
            Instruction::MakePair => self.make_pair()?,
            Instruction::MakeList => match data {
                None => instruction_error(instruction, self.get_instruction_cursor())?,
                Some(i) => self.make_list(i)?,
            },
            Instruction::Apply => self.apply(context)?,
            Instruction::EmptyApply => self.empty_apply(context)?,
            Instruction::Reapply => match data {
                None => instruction_error(instruction, self.get_instruction_cursor())?,
                Some(i) => self.reapply(i)?,
            },
            Instruction::Access => self.access()?,
            Instruction::Resolve => match data {
                None => instruction_error(instruction, self.get_instruction_cursor())?,
                Some(i) => self.resolve(i, context)?,
            },
            Instruction::AccessLeftInternal => self.access_left_internal()?,
            Instruction::AccessRightInternal => self.access_right_internal()?,
            Instruction::AccessLengthInternal => self.access_length_internal()?,
        };

        match self.get_instruction_cursor() + Data::Size::one() >= self.get_instruction_len() {
            true => Ok(GarnishLangRuntimeInfo::new(GarnishLangRuntimeState::End)),
            false => {
                self.set_instruction_cursor(self.get_instruction_cursor() + Data::Size::one())
                    ?;
                Ok(GarnishLangRuntimeInfo::new(GarnishLangRuntimeState::Running))
            }
        }
    }

    // Apply

    fn apply<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>> {
        apply(self, context)
    }

    fn reapply(&mut self, index: Data::Size) -> Result<(), RuntimeError<Data::Error>> {
        reapply(self, index)
    }

    fn empty_apply<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>> {
        empty_apply(self, context)
    }

    //
    // Arithmetic
    //

    fn perform_addition(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        perform_addition(self)
    }

    //
    // Comparison
    //

    fn equality_comparison(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        equality_comparison(self)
    }

    //
    // Jumps
    //

    fn jump(&mut self, index: Data::Size) -> Result<(), RuntimeError<Data::Error>> {
        jump(self, index)
    }

    fn jump_if_true(&mut self, index: Data::Size) -> Result<(), RuntimeError<Data::Error>> {
        jump_if_true(self, index)
    }

    fn jump_if_false(&mut self, index: Data::Size) -> Result<(), RuntimeError<Data::Error>> {
        jump_if_false(self, index)
    }

    fn end_expression(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        end_expression(self)
    }

    //
    // List
    //

    fn make_list(&mut self, len: Data::Size) -> Result<(), RuntimeError<Data::Error>> {
        make_list(self, len)
    }

    fn access(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        access(self)
    }

    fn access_left_internal(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        access_left_internal(self)
    }

    fn access_right_internal(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        access_right_internal(self)
    }

    fn access_length_internal(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        access_length_internal(self)
    }

    //
    // Pair
    //

    fn make_pair(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        make_pair(self)
    }

    //
    // Put
    //

    fn put(&mut self, i: Data::Size) -> Result<(), RuntimeError<Data::Error>> {
        put(self, i)
    }

    fn put_value(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        put_input(self)
    }

    fn push_value(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        push_input(self)
    }

    fn update_value(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        push_result(self)
    }

    //
    // Resolve
    //

    fn resolve<T: GarnishLangRuntimeContext<Data>>(&mut self, data: Data::Size, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>> {
        resolve::resolve(self, data, context)
    }
}

#[cfg(test)]
mod tests {
    use crate::{runtime::{context::EmptyContext, GarnishRuntime}, GarnishLangRuntimeData, Instruction, SimpleRuntimeData, GarnishLangRuntimeState};

    #[test]
    fn create_runtime() {
        SimpleRuntimeData::new();
    }

    #[test]
    fn add_jump_point() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.push_instruction(Instruction::EndExpression, None).unwrap();
        runtime.push_jump_point(1).unwrap();

        assert_eq!(runtime.get_jump_points().len(), 1);
        assert_eq!(*runtime.get_jump_points().get(0).unwrap(), 1);
    }

    #[test]
    fn add_jump_point_out_of_bounds() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.push_instruction(Instruction::EndExpression, None).unwrap();
        let result = runtime.push_jump_point(5);

        assert!(result.is_err());
    }

    #[test]
    fn add_instruction() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.push_instruction(Instruction::Put, Some(0)).unwrap();

        assert_eq!(runtime.get_instructions().len(), 2);
    }

    #[test]
    fn add_input_reference() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_integer(10).unwrap();
        runtime.push_value_stack(0).unwrap();

        assert_eq!(runtime.get_value(0).unwrap().to_owned(), 0);
    }

    #[test]
    fn add_input_reference_with_data_addr() {
        let mut runtime = SimpleRuntimeData::new();

        let _i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(10).unwrap();

        runtime.push_value_stack(i2).unwrap();

        assert_eq!(runtime.get_value(0).unwrap().to_owned(), i2);
    }

    #[test]
    fn get_instruction() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.push_instruction(Instruction::Put, None).unwrap();

        assert_eq!(runtime.get_instruction(1).unwrap().0, Instruction::Put);
    }

    #[test]
    fn get_current_instruction() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.push_instruction(Instruction::Put, None).unwrap();

        runtime.set_instruction_cursor(1).unwrap();

        assert_eq!(runtime.get_current_instruction().unwrap().0, Instruction::Put);
    }

    #[test]
    fn set_instruction_cursor() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.push_instruction(Instruction::Put, None).unwrap();
        runtime.push_instruction(Instruction::Put, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.set_instruction_cursor(3).unwrap();

        assert_eq!(runtime.get_current_instruction().unwrap().0, Instruction::PerformAddition);
    }

    #[test]
    fn end_execution() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.push_instruction(Instruction::Put, None).unwrap();
        runtime.push_instruction(Instruction::Put, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.set_instruction_cursor(3).unwrap();

        runtime.end_execution().unwrap();

        assert_eq!(runtime.get_instruction_cursor(), 4);
    }

    #[test]
    fn execute_current_instruction() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let start = runtime.get_data_len();

        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.push_register(i1).unwrap();
        runtime.push_register(i2).unwrap();

        runtime.set_instruction_cursor(1).unwrap();

        runtime.execute_current_instruction::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), start);
        assert_eq!(runtime.get_integer(start).unwrap(), 30);
    }

    #[test]
    fn execute_current_instruction_with_cursor_past_len() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let _start = runtime.get_data_len();

        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.push_register(i1).unwrap();
        runtime.push_register(i2).unwrap();

        runtime.set_instruction_cursor(10).unwrap();

        let result = runtime.execute_current_instruction::<EmptyContext>(None).unwrap();

        assert_eq!(result.get_state(), GarnishLangRuntimeState::End);
    }
}
