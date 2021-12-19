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

pub use crate::runtime::utilities::*;
pub use context::{EmptyContext, GarnishLangRuntimeContext};
pub use data::GarnishLangRuntimeData;

use crate::result::{error, GarnishLangRuntimeResult, GarnishLangRuntimeState};
use crate::runtime::apply::*;
use crate::runtime::comparisons::equality_comparison;
use crate::runtime::list::*;
use crate::{instruction::*, ExpressionDataType};
use crate::{GarnishLangRuntimeInfo, NestInto};
use log::trace;

pub trait GarnishRuntime<Data: GarnishLangRuntimeData> {
    fn end_execution(&mut self) -> GarnishLangRuntimeResult<Data::Error>;
    fn execute_current_instruction<T: GarnishLangRuntimeContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> GarnishLangRuntimeResult<Data::Error, GarnishLangRuntimeInfo>;

    fn apply<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> GarnishLangRuntimeResult<Data::Error>;
    fn reapply(&mut self, index: usize) -> GarnishLangRuntimeResult<Data::Error>;
    fn empty_apply<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> GarnishLangRuntimeResult<Data::Error>;

    fn perform_addition(&mut self) -> GarnishLangRuntimeResult<Data::Error>;

    fn equality_comparison(&mut self) -> GarnishLangRuntimeResult<Data::Error>;

    fn jump(&mut self, index: usize) -> GarnishLangRuntimeResult<Data::Error>;
    fn jump_if_true(&mut self, index: usize) -> GarnishLangRuntimeResult<Data::Error>;
    fn jump_if_false(&mut self, index: usize) -> GarnishLangRuntimeResult<Data::Error>;
    fn end_expression(&mut self) -> GarnishLangRuntimeResult<Data::Error>;

    fn make_list(&mut self, len: usize) -> GarnishLangRuntimeResult<Data::Error>;
    fn access(&mut self) -> GarnishLangRuntimeResult<Data::Error>;
    fn access_left_internal(&mut self) -> GarnishLangRuntimeResult<Data::Error>;
    fn access_right_internal(&mut self) -> GarnishLangRuntimeResult<Data::Error>;
    fn access_length_internal(&mut self) -> GarnishLangRuntimeResult<Data::Error>;

    fn make_pair(&mut self) -> GarnishLangRuntimeResult<Data::Error>;

    fn put(&mut self, i: usize) -> GarnishLangRuntimeResult<Data::Error>;
    fn put_input(&mut self) -> GarnishLangRuntimeResult<Data::Error>;
    fn push_input(&mut self) -> GarnishLangRuntimeResult<Data::Error>;
    fn put_result(&mut self) -> GarnishLangRuntimeResult<Data::Error>;
    fn push_result(&mut self) -> GarnishLangRuntimeResult<Data::Error>;

    fn resolve<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> GarnishLangRuntimeResult<Data::Error>;
}

impl<Data> GarnishRuntime<Data> for Data
where
    Data: GarnishLangRuntimeData,
{
    fn end_execution(&mut self) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - End Execution");
        self.set_instruction_cursor(self.get_instruction_len()).nest_into()
    }

    fn execute_current_instruction<T: GarnishLangRuntimeContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> GarnishLangRuntimeResult<Data::Error, GarnishLangRuntimeInfo> {
        let instruction_data = self
            .get_instruction(self.get_instruction_cursor())
            .ok_or(error(format!("Attempted to execute instruction when no instructions remain.")))?;
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
            Instruction::AccessLeftInternal => self.access_left_internal()?,
            Instruction::AccessRightInternal => self.access_right_internal()?,
            Instruction::AccessLengthInternal => self.access_length_internal()?,
        };

        match self.get_instruction_cursor() + 1 >= self.get_instruction_len() {
            true => Ok(GarnishLangRuntimeInfo::new(GarnishLangRuntimeState::End)),
            false => {
                self.advance_instruction_cursor().nest_into()?;
                Ok(GarnishLangRuntimeInfo::new(GarnishLangRuntimeState::Running))
            }
        }
    }

    // Apply

    fn apply<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - Apply");
        apply_internal(self, context)
    }

    fn reapply(&mut self, index: usize) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - Reapply | Data - {:?}", index);

        let right_addr = next_ref(self)?;
        let point = self.get_jump_point(index).ok_or(error(format!("No jump point at index {:?}", index)))?;

        self.set_instruction_cursor(point - 1).nest_into()?;
        self.pop_input_stack()
            .ok_or(error(format!("Failed to pop input during reapply operation.")))?;
        self.push_input_stack(right_addr).nest_into()
    }

    fn empty_apply<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - Empty Apply");
        push_unit(self)?;

        apply_internal(self, context)
    }

    //
    // Arithmetic
    //

    fn perform_addition(&mut self) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - Addition");

        let (right_addr, left_addr) = next_two_raw_ref(self)?;

        match (self.get_data_type(left_addr).nest_into()?, self.get_data_type(right_addr).nest_into()?) {
            (ExpressionDataType::Integer, ExpressionDataType::Integer) => {
                let left = self.get_integer(left_addr).nest_into()?;
                let right = self.get_integer(right_addr).nest_into()?;

                trace!("Performing {:?} + {:?}", left, right);

                push_integer(self, left + right)
            }
            _ => push_unit(self),
        }
    }

    //
    // Comparison
    //

    fn equality_comparison(&mut self) -> GarnishLangRuntimeResult<Data::Error> {
        equality_comparison(self)
    }

    //
    // Jumps
    //

    fn jump(&mut self, index: usize) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - Jump | Data - {:?}", index);

        self.set_instruction_cursor(self.get_jump_point(index).ok_or(error(format!("No jump point at index {:?}", index)))? - 1)
            .nest_into()
    }

    fn jump_if_true(&mut self, index: usize) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - Execute Expression If True | Data - {:?}", index);
        let point = self.get_jump_point(index).ok_or(error(format!("No jump point at index {:?}.", index)))? - 1;
        let d = next_ref(self)?;

        match self.get_data_type(d).nest_into()? {
            ExpressionDataType::False | ExpressionDataType::Unit => {
                trace!(
                    "Not jumping from value of type {:?} with addr {:?}",
                    self.get_data_type(d).nest_into()?,
                    self.get_data_len() - 1
                );
            }
            // all other values are considered true
            t => {
                trace!("Jumping from value of type {:?} with addr {:?}", t, self.get_data_len() - 1);
                self.set_instruction_cursor(point).nest_into()?
            }
        };

        Ok(())
    }

    fn jump_if_false(&mut self, index: usize) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - Execute Expression If False | Data - {:?}", index);
        let point = self.get_jump_point(index).ok_or(error(format!("No jump point at index {:?}.", index)))? - 1;
        let d = next_ref(self)?;

        match self.get_data_type(d).nest_into()? {
            ExpressionDataType::False | ExpressionDataType::Unit => {
                trace!(
                    "Jumping from value of type {:?} with addr {:?}",
                    self.get_data_type(d).nest_into()?,
                    self.get_data_len() - 1
                );
                self.set_instruction_cursor(point).nest_into()?
            }
            t => {
                trace!("Not jumping from value of type {:?} with addr {:?}", t, self.get_data_len() - 1);
            }
        };

        Ok(())
    }

    fn end_expression(&mut self) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - End Expression");
        match self.pop_jump_path() {
            None => {
                // no more jumps, this should be the end of the entire execution
                let r = next_ref(self)?;
                self.advance_instruction_cursor().nest_into()?;
                self.set_result(Some(r)).nest_into()?;
            }
            Some(jump_point) => {
                self.set_instruction_cursor(jump_point).nest_into()?;
            }
        }

        Ok(())
    }

    //
    // List
    //

    fn make_list(&mut self, len: usize) -> GarnishLangRuntimeResult<Data::Error> {
        make_list(self, len)
    }

    fn access(&mut self) -> GarnishLangRuntimeResult<Data::Error> {
        access(self)
    }

    fn access_left_internal(&mut self) -> GarnishLangRuntimeResult<Data::Error> {
        access_left_internal(self)
    }

    fn access_right_internal(&mut self) -> GarnishLangRuntimeResult<Data::Error> {
        access_right_internal(self)
    }

    fn access_length_internal(&mut self) -> GarnishLangRuntimeResult<Data::Error> {
        access_length_internal(self)
    }

    //
    // Pair
    //

    fn make_pair(&mut self) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - Make Pair");

        let (right_addr, left_addr) = next_two_raw_ref(self)?;

        push_pair(self, left_addr, right_addr)
    }

    //
    // Put
    //

    fn put(&mut self, i: usize) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - Put | Data - {:?}", i);
        match i >= self.get_end_of_constant_data() {
            true => Err(error(format!(
                "Attempting to put reference to {:?} which is out of bounds of constant data that ends at {:?}.",
                i,
                self.get_end_of_constant_data()
            ))),
            false => self.push_register(i).nest_into(),
        }
    }

    fn put_input(&mut self) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - Put Input");

        match self.get_current_input() {
            None => push_unit(self),
            Some(i) => self.push_register(i).nest_into(),
        }
    }

    fn push_input(&mut self) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - Push Input");
        let r = next_ref(self)?;

        self.push_input_stack(r).nest_into()?;
        self.set_result(Some(r)).nest_into()
    }

    fn put_result(&mut self) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - Put Result");

        match self.get_result() {
            None => push_unit(self),
            Some(i) => self.push_register(i).nest_into(),
        }
    }

    fn push_result(&mut self) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - Output Result");

        let r = next_ref(self)?;
        self.set_result(Some(r)).nest_into()
    }

    //
    // Resolve
    //

    fn resolve<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> GarnishLangRuntimeResult<Data::Error> {
        resolve::resolve(self, context)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        runtime::{context::EmptyContext, GarnishRuntime},
        ExpressionData, ExpressionDataType, GarnishLangRuntimeData, Instruction, SimpleRuntimeData,
    };

    #[test]
    fn create_runtime() {
        SimpleRuntimeData::new();
    }

    #[test]
    fn default_data_for_new() {
        let runtime = SimpleRuntimeData::new();

        assert_eq!(runtime.get_instruction(0).unwrap().instruction, Instruction::EndExecution);
        assert_eq!(runtime.get_data_len(), 1);
        assert_eq!(runtime.get_data_type(0).unwrap(), ExpressionDataType::Unit);
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

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.push_input_stack(0).unwrap();

        assert_eq!(runtime.get_input(0).unwrap().to_owned(), 0);
    }

    #[test]
    fn add_input_reference_with_data_addr() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        let addr = runtime.add_data(ExpressionData::integer(10)).unwrap();

        runtime.push_input_stack(addr).unwrap();

        assert_eq!(runtime.get_input(0).unwrap().to_owned(), 2);
    }

    #[test]
    fn get_instruction() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.push_instruction(Instruction::Put, None).unwrap();

        assert_eq!(runtime.get_instruction(1).unwrap().get_instruction(), Instruction::Put);
    }

    #[test]
    fn get_current_instruction() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.push_instruction(Instruction::Put, None).unwrap();

        runtime.set_instruction_cursor(1).unwrap();

        assert_eq!(runtime.get_current_instruction().unwrap().get_instruction(), Instruction::Put);
    }

    #[test]
    fn set_instruction_cursor() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.push_instruction(Instruction::Put, None).unwrap();
        runtime.push_instruction(Instruction::Put, None).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.set_instruction_cursor(3).unwrap();

        assert_eq!(runtime.get_current_instruction().unwrap().get_instruction(), Instruction::PerformAddition);
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

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.push_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.push_register(1).unwrap();
        runtime.push_register(2).unwrap();

        runtime.set_instruction_cursor(1).unwrap();

        runtime.execute_current_instruction::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_register(), &vec![3]);
        assert_eq!(runtime.get_integer(3).unwrap(), 30);
    }
}
