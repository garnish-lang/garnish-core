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

use crate::runtime::arithmetic::perform_addition;
use crate::runtime::jumps::{end_expression, jump, jump_if_false, jump_if_true};
use crate::runtime::pair::make_pair;
use crate::runtime::put::{push_input, push_result, put, put_input};
use crate::GarnishLangRuntimeInfo;
use apply::*;
use comparisons::equality_comparison;
use instruction::*;
use list::*;
use result::*;

pub trait GarnishRuntime<Data: GarnishLangRuntimeData> {
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
    fn execute_current_instruction<T: GarnishLangRuntimeContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> Result<GarnishLangRuntimeInfo, RuntimeError<Data::Error>> {
        let (instruction, data) = match self.get_instruction(self.get_instruction_cursor()) {
            None => return Ok(GarnishLangRuntimeInfo::new(GarnishLangRuntimeState::End)),
            Some(v) => v,
        };


        let mut next_instruction = self.get_instruction_cursor() + Data::Size::one();

        match instruction {
            Instruction::PerformAddition => self.perform_addition()?,
            Instruction::PutValue => self.put_value()?,
            Instruction::PushValue => self.push_value()?,
            Instruction::UpdateValue => self.update_value()?,
            Instruction::EndExpression => self.end_expression()?,
            Instruction::EqualityComparison => self.equality_comparison()?,
            Instruction::Put => match data {
                None => instruction_error(instruction, self.get_instruction_cursor())?,
                Some(i) => self.put(i)?,
            },
            Instruction::MakePair => self.make_pair()?,
            Instruction::MakeList => match data {
                None => instruction_error(instruction, self.get_instruction_cursor())?,
                Some(i) => self.make_list(i)?,
            },
            Instruction::Access => self.access()?,
            Instruction::Resolve => match data {
                None => instruction_error(instruction, self.get_instruction_cursor())?,
                Some(i) => self.resolve(i, context)?,
            },
            Instruction::AccessLeftInternal => self.access_left_internal()?,
            Instruction::AccessRightInternal => self.access_right_internal()?,
            Instruction::AccessLengthInternal => self.access_length_internal()?,
            Instruction::Apply => {
                // sets instruction cursor for us
                self.apply(context)?;
                next_instruction = self.get_instruction_cursor();
            },
            Instruction::EmptyApply => {
                self.empty_apply(context)?;
                next_instruction = self.get_instruction_cursor();
            },
            Instruction::Reapply => match data {
                None => instruction_error(instruction, self.get_instruction_cursor())?,
                Some(i) => {
                    self.reapply(i)?;
                    next_instruction = self.get_instruction_cursor();
                },
            },
            Instruction::JumpIfTrue => match data {
                None => instruction_error(instruction, self.get_instruction_cursor())?,
                Some(i) => {
                    self.jump_if_true(i)?;
                    next_instruction = self.get_instruction_cursor();
                }
            },
            Instruction::JumpIfFalse => match data {
                None => instruction_error(instruction, self.get_instruction_cursor())?,
                Some(i) => {
                    self.jump_if_false(i)?;
                    next_instruction = self.get_instruction_cursor();
                }
            },
            Instruction::JumpTo => match data {
                None => instruction_error(instruction, self.get_instruction_cursor())?,
                Some(i) => {
                    self.jump(i)?;
                    next_instruction = self.get_instruction_cursor();
                }
            },
        };

        match next_instruction >= self.get_instruction_len() {
            true => Ok(GarnishLangRuntimeInfo::new(GarnishLangRuntimeState::End)),
            false => {
                self.set_instruction_cursor(next_instruction)?;
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
    use crate::{
        runtime::{context::EmptyContext, GarnishRuntime},
        GarnishLangRuntimeData, GarnishLangRuntimeState, Instruction, SimpleRuntimeData, ExpressionDataType
    };

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
    fn execute_current_instruction() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_integer(10).unwrap();
        let d2 = runtime.add_integer(20).unwrap();
        let start = runtime.get_data_len();

        let i1 = runtime.push_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.set_instruction_cursor(i1).unwrap();

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

    #[test]
    fn execute_current_instruction_apply() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_integer(10).unwrap();
        let exp1 = runtime.add_expression(0).unwrap();
        let int2 = runtime.add_integer(20).unwrap();

        // 1
        let i1 = runtime.push_instruction(Instruction::Put, Some(int1)).unwrap();
        runtime.push_instruction(Instruction::PutValue, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::EndExpression, None).unwrap();

        // 5
        runtime.push_instruction(Instruction::Put, Some(exp1)).unwrap();
        runtime.push_instruction(Instruction::Put, Some(int2)).unwrap();
        let i2 = runtime.push_instruction(Instruction::Apply, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.push_jump_point(i1).unwrap();

        runtime.push_register(exp1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.set_instruction_cursor(i2).unwrap();

        runtime.execute_current_instruction::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_value(0).unwrap(), int2);
        assert_eq!(runtime.get_instruction_cursor(), i1);
        assert_eq!(runtime.get_jump_path(0).unwrap(), i2);
    }

    #[test]
    fn execute_current_instruction_empty_apply() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_integer(10).unwrap();
        let exp1 = runtime.add_expression(0).unwrap();

        // 1
        let i1 = runtime.push_instruction(Instruction::Put, Some(int1)).unwrap();
        runtime.push_instruction(Instruction::PutValue, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::EndExpression, None).unwrap();

        // 5
        runtime.push_instruction(Instruction::Put, Some(exp1)).unwrap();
        let i2 = runtime.push_instruction(Instruction::EmptyApply, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.push_jump_point(i1).unwrap();

        runtime.push_register(exp1).unwrap();

        runtime.set_instruction_cursor(i2).unwrap();

        runtime.execute_current_instruction::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_value(0).unwrap()).unwrap(), ExpressionDataType::Unit);
        assert_eq!(runtime.get_instruction_cursor(), i1);
        assert_eq!(runtime.get_jump_path(0).unwrap(), i2);
    }

    #[test]
    fn execute_current_instruction_reapply_if_true() {
        let mut runtime = SimpleRuntimeData::new();

        let true1 = runtime.add_true().unwrap();
        let _exp1 = runtime.add_expression(0).unwrap();
        let int1 = runtime.add_integer(20).unwrap();
        let _int2 = runtime.add_integer(30).unwrap();
        let int3 = runtime.add_integer(40).unwrap();

        // 1
        runtime.push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.push_instruction(Instruction::Put, Some(2)).unwrap();
        runtime.push_instruction(Instruction::Apply, None).unwrap();

        // 4
        let i1 = runtime.push_instruction(Instruction::Put, Some(0)).unwrap();
        runtime.push_instruction(Instruction::PutValue, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PutValue, None).unwrap();
        let i2 = runtime.push_instruction(Instruction::Reapply, Some(0)).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::EndExpression, None).unwrap();

        runtime.push_jump_point(i1).unwrap();

        runtime.push_register(true1).unwrap();
        runtime.push_register(int3).unwrap();

        runtime.push_value_stack(int1).unwrap();

        runtime.set_instruction_cursor(i2).unwrap();

        runtime.execute_current_instruction::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_value_stack_len(), 1);
        assert_eq!(runtime.get_value(0).unwrap(), int3);
        assert_eq!(runtime.get_instruction_cursor(), i1);
    }

    #[test]
    fn execute_current_instruction_jump_if_true_when_true() {
        let mut runtime = SimpleRuntimeData::new();

        let ta = runtime.add_true().unwrap();
        let i1 = runtime.push_instruction(Instruction::JumpIfTrue, Some(0)).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        let i2 = runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.push_jump_point(i2).unwrap();

        runtime.set_instruction_cursor(i1).unwrap();

        runtime.push_register(ta).unwrap();

        runtime.execute_current_instruction::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_register_len(), 0);
        assert_eq!(runtime.get_data_len(), 3);
        assert_eq!(runtime.get_instruction_cursor(), i2);
    }

    #[test]
    fn execute_current_instruction_jump_if_false_when_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let ua = runtime.add_unit().unwrap();
        let i1 = runtime.push_instruction(Instruction::JumpIfFalse, Some(0)).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        let i2 = runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.push_jump_point(i2).unwrap();

        runtime.set_instruction_cursor(i1).unwrap();

        runtime.push_register(ua).unwrap();

        runtime.execute_current_instruction::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_register_len(), 0);
        assert_eq!(runtime.get_data_len(), 3);
        assert_eq!(runtime.get_instruction_cursor(), i2);
    }

    #[test]
    fn execute_current_instruction_jump_if_false_when_false() {
        let mut runtime = SimpleRuntimeData::new();

        let fa = runtime.add_false().unwrap();
        let i1 = runtime.push_instruction(Instruction::JumpIfFalse, Some(0)).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        let i2 = runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.push_jump_point(i2).unwrap();

        runtime.set_instruction_cursor(i1).unwrap();

        runtime.push_register(fa).unwrap();

        runtime.execute_current_instruction::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_register_len(), 0);
        assert_eq!(runtime.get_data_len(), 3);
        assert_eq!(runtime.get_instruction_cursor(), i2);
    }
}
