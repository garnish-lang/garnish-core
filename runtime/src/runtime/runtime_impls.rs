use crate::runtime::apply::*;
use crate::runtime::arithmetic::{absolute_value, add, divide, integer_divide, multiply, opposite, power, remainder, subtract};
use crate::runtime::bitwise::{bitwise_and, bitwise_left_shift, bitwise_not, bitwise_or, bitwise_right_shift, bitwise_xor};
use crate::runtime::casting::{type_cast, type_of};
use crate::runtime::comparison::{greater_than, greater_than_or_equal, less_than, less_than_or_equal};
use crate::runtime::context::GarnishLangRuntimeContext;
use crate::runtime::data::{GarnishLangRuntimeData, TypeConstants};
use crate::runtime::equality::equal;
use crate::runtime::equality::{not_equal, type_equal};
use crate::runtime::error::*;
use crate::runtime::instruction::*;
use crate::runtime::internals::{access_left_internal, access_length_internal, access_right_internal};
use crate::runtime::jumps::{end_expression, jump, jump_if_false, jump_if_true};
use crate::runtime::link::{append_link, prepend_link};
use crate::runtime::list::*;
use crate::runtime::logical::{and, not, or, xor};
use crate::runtime::pair::make_pair;
use crate::runtime::put::{push_input, push_result, put, put_input};
use crate::runtime::range::{make_end_exclusive_range, make_exclusive_range, make_range, make_start_exclusive_range};
use crate::runtime::resolve::resolve;
use crate::runtime::result::*;
use crate::runtime::sideeffect::*;

use crate::runtime::GarnishLangRuntimeInfo;
use crate::runtime::GarnishRuntime;
use log::trace;
use crate::runtime::concat::concat;

pub struct SimpleGarnishRuntime<Data: GarnishLangRuntimeData> {
    data: Data
}

impl<Data: GarnishLangRuntimeData> SimpleGarnishRuntime<Data> {
    pub fn new(data: Data) -> SimpleGarnishRuntime<Data> {
        SimpleGarnishRuntime { data }
    }

    pub fn get_data(&self) -> &Data {
        &self.data
    }

    pub fn get_data_mut(&mut self) -> &mut Data {
        &mut self.data
    }
}

impl<Data> GarnishRuntime<Data> for SimpleGarnishRuntime<Data>
where
    Data: GarnishLangRuntimeData,
{
    fn execute_current_instruction<T: GarnishLangRuntimeContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> Result<GarnishLangRuntimeInfo, RuntimeError<Data::Error>> {
        let (instruction, data) = match self.get_data().get_instruction(self.get_data().get_instruction_cursor()) {
            None => return Ok(GarnishLangRuntimeInfo::new(GarnishLangRuntimeState::End)),
            Some(v) => v,
        };

        trace!(
            "Executing instruction {:?} at {:?} with data {:?}",
            instruction,
            self.get_data().get_instruction_cursor(),
            data
        );

        let mut next_instruction = self.get_data().get_instruction_cursor() + Data::Size::one();

        match instruction {
            Instruction::Add => self.add(context)?,
            Instruction::Subtract => self.subtract(context)?,
            Instruction::Multiply => self.multiply(context)?,
            Instruction::Divide => self.divide(context)?,
            Instruction::IntegerDivide => self.integer_divide(context)?,
            Instruction::Power => self.power(context)?,
            Instruction::Opposite => self.opposite(context)?,
            Instruction::AbsoluteValue => self.absolute_value(context)?,
            Instruction::Remainder => self.remainder(context)?,
            Instruction::BitwiseNot => self.bitwise_not(context)?,
            Instruction::BitwiseAnd => self.bitwise_and(context)?,
            Instruction::BitwiseOr => self.bitwise_or(context)?,
            Instruction::BitwiseXor => self.bitwise_xor(context)?,
            Instruction::BitwiseShiftLeft => self.bitwise_left_shift(context)?,
            Instruction::BitwiseShiftRight => self.bitwise_right_shift(context)?,
            Instruction::And => self.and()?,
            Instruction::Or => self.or()?,
            Instruction::Xor => self.xor()?,
            Instruction::Not => self.not()?,
            Instruction::PutValue => self.put_value()?,
            Instruction::PushValue => self.push_value()?,
            Instruction::UpdateValue => self.update_value()?,
            Instruction::StartSideEffect => self.start_side_effect()?,
            Instruction::EndSideEffect => self.end_side_effect()?,
            Instruction::TypeOf => self.type_of()?,
            Instruction::ApplyType => self.type_cast(context)?,
            Instruction::TypeEqual => self.type_equal()?,
            Instruction::Equal => self.equal()?,
            Instruction::NotEqual => self.not_equal()?,
            Instruction::LessThan => self.less_than()?,
            Instruction::LessThanOrEqual => self.less_than_or_equal()?,
            Instruction::GreaterThan => self.greater_than()?,
            Instruction::GreaterThanOrEqual => self.greater_than_or_equal()?,
            Instruction::MakePair => self.make_pair()?,
            Instruction::Access => self.apply(context)?,
            Instruction::AccessLeftInternal => self.access_left_internal(context)?,
            Instruction::AccessRightInternal => self.access_right_internal(context)?,
            Instruction::AccessLengthInternal => self.access_length_internal(context)?,
            Instruction::MakeRange => self.make_range()?,
            Instruction::MakeStartExclusiveRange => self.make_start_exclusive_range()?,
            Instruction::MakeEndExclusiveRange => self.make_end_exclusive_range()?,
            Instruction::MakeExclusiveRange => self.make_exclusive_range()?,
            Instruction::AppendLink => self.append_link()?,
            Instruction::PrependLink => self.prepend_link()?,
            Instruction::Concat => self.concat()?,
            Instruction::Put => match data {
                None => instruction_error(instruction, self.get_data().get_instruction_cursor())?,
                Some(i) => self.put(i)?,
            },
            Instruction::MakeList => match data {
                None => instruction_error(instruction, self.get_data().get_instruction_cursor())?,
                Some(i) => self.make_list(i)?,
            },
            Instruction::Resolve => match data {
                None => instruction_error(instruction, self.get_data().get_instruction_cursor())?,
                Some(i) => self.resolve(i, context)?,
            },
            Instruction::EndExpression => {
                self.end_expression()?;
                next_instruction = self.get_data().get_instruction_cursor();
            }
            Instruction::Apply => {
                // sets instruction cursor for us
                self.apply(context)?;
                next_instruction = self.get_data().get_instruction_cursor();
            }
            Instruction::EmptyApply => {
                self.empty_apply(context)?;
                next_instruction = self.get_data().get_instruction_cursor();
            }
            Instruction::Reapply => match data {
                None => instruction_error(instruction, self.get_data().get_instruction_cursor())?,
                Some(i) => {
                    self.reapply(i)?;
                    next_instruction = self.get_data().get_instruction_cursor();
                }
            },
            Instruction::JumpIfTrue => match data {
                None => instruction_error(instruction, self.get_data().get_instruction_cursor())?,
                Some(i) => {
                    self.jump_if_true(i)?;
                    next_instruction = self.get_data().get_instruction_cursor();
                }
            },
            Instruction::JumpIfFalse => match data {
                None => instruction_error(instruction, self.get_data().get_instruction_cursor())?,
                Some(i) => {
                    self.jump_if_false(i)?;
                    next_instruction = self.get_data().get_instruction_cursor();
                }
            },
            Instruction::JumpTo => match data {
                None => instruction_error(instruction, self.get_data().get_instruction_cursor())?,
                Some(i) => {
                    self.jump(i)?;
                    next_instruction = self.get_data().get_instruction_cursor();
                }
            },
        };

        match next_instruction >= self.get_data().get_instruction_len() {
            true => Ok(GarnishLangRuntimeInfo::new(GarnishLangRuntimeState::End)),
            false => {
                self.get_data_mut().set_instruction_cursor(next_instruction)?;
                Ok(GarnishLangRuntimeInfo::new(GarnishLangRuntimeState::Running))
            }
        }
    }

    // Apply

    fn apply<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>> {
        apply(self.get_data_mut(), context)
    }

    fn reapply(&mut self, index: Data::Size) -> Result<(), RuntimeError<Data::Error>> {
        reapply(self.get_data_mut(), index)
    }

    fn empty_apply<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>> {
        empty_apply(self.get_data_mut(), context)
    }

    //
    // Arithmetic
    //

    fn add<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>> {
        add(self.get_data_mut(), context)
    }

    fn subtract<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>> {
        subtract(self.get_data_mut(), context)
    }

    fn multiply<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>> {
        multiply(self.get_data_mut(), context)
    }

    fn power<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>> {
        power(self.get_data_mut(), context)
    }

    fn divide<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>> {
        divide(self.get_data_mut(), context)
    }

    fn integer_divide<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>> {
        integer_divide(self.get_data_mut(), context)
    }

    fn remainder<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>> {
        remainder(self.get_data_mut(), context)
    }

    fn absolute_value<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>> {
        absolute_value(self.get_data_mut(), context)
    }

    fn opposite<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>> {
        opposite(self.get_data_mut(), context)
    }

    //
    // Bitwise
    //

    fn bitwise_not<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>> {
        bitwise_not(self.get_data_mut(), context)
    }

    fn bitwise_and<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>> {
        bitwise_and(self.get_data_mut(), context)
    }

    fn bitwise_or<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>> {
        bitwise_or(self.get_data_mut(), context)
    }

    fn bitwise_xor<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>> {
        bitwise_xor(self.get_data_mut(), context)
    }

    fn bitwise_left_shift<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>> {
        bitwise_left_shift(self.get_data_mut(), context)
    }

    fn bitwise_right_shift<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>> {
        bitwise_right_shift(self.get_data_mut(), context)
    }

    //
    // Logical
    //

    fn and(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        and(self.get_data_mut())
    }

    fn or(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        or(self.get_data_mut())
    }

    fn xor(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        xor(self.get_data_mut())
    }

    fn not(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        not(self.get_data_mut())
    }

    //
    // Type Ops
    //

    fn type_of(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        type_of(self.get_data_mut())
    }

    fn type_cast<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>> {
        type_cast(self.get_data_mut(), context)
    }

    fn type_equal(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        type_equal(self.get_data_mut())
    }

    //
    // Comparison
    //

    fn equal(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        equal(self.get_data_mut())
    }

    fn not_equal(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        not_equal(self.get_data_mut())
    }

    fn less_than(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        less_than(self.get_data_mut())
    }

    fn less_than_or_equal(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        less_than_or_equal(self.get_data_mut())
    }

    fn greater_than(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        greater_than(self.get_data_mut())
    }

    fn greater_than_or_equal(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        greater_than_or_equal(self.get_data_mut())
    }

    //
    // Jumps
    //

    fn jump(&mut self, index: Data::Size) -> Result<(), RuntimeError<Data::Error>> {
        jump(self.get_data_mut(), index)
    }

    fn jump_if_true(&mut self, index: Data::Size) -> Result<(), RuntimeError<Data::Error>> {
        jump_if_true(self.get_data_mut(), index)
    }

    fn jump_if_false(&mut self, index: Data::Size) -> Result<(), RuntimeError<Data::Error>> {
        jump_if_false(self.get_data_mut(), index)
    }

    fn end_expression(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        end_expression(self.get_data_mut())
    }

    //
    // List
    //

    fn make_list(&mut self, len: Data::Size) -> Result<(), RuntimeError<Data::Error>> {
        make_list(self.get_data_mut(), len)
    }

    fn access_left_internal<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>> {
        access_left_internal(self.get_data_mut(), context)
    }

    fn access_right_internal<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>> {
        access_right_internal(self.get_data_mut(), context)
    }

    fn access_length_internal<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>> {
        access_length_internal(self.get_data_mut(), context)
    }

    //
    // Range
    //

    fn make_range(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        make_range(self.get_data_mut())
    }

    fn make_start_exclusive_range(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        make_start_exclusive_range(self.get_data_mut())
    }

    fn make_end_exclusive_range(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        make_end_exclusive_range(self.get_data_mut())
    }

    fn make_exclusive_range(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        make_exclusive_range(self.get_data_mut())
    }

    //
    // Link
    //

    fn append_link(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        append_link(self.get_data_mut())
    }

    fn prepend_link(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        prepend_link(self.get_data_mut())
    }

    //
    // Pair
    //

    fn make_pair(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        make_pair(self.get_data_mut())
    }

    //
    // Concatenation
    //

    fn concat(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        concat(self.get_data_mut())
    }

    //
    // Put
    //

    fn put(&mut self, i: Data::Size) -> Result<(), RuntimeError<Data::Error>> {
        put(self.get_data_mut(), i)
    }

    fn put_value(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        put_input(self.get_data_mut())
    }

    fn push_value(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        push_input(self.get_data_mut())
    }

    fn update_value(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        push_result(self.get_data_mut())
    }

    //
    // Resolve
    //

    fn resolve<T: GarnishLangRuntimeContext<Data>>(&mut self, data: Data::Size, context: Option<&mut T>) -> Result<(), RuntimeError<Data::Error>> {
        resolve(self.get_data_mut(), data, context)
    }

    //
    // Side Effect
    //

    fn start_side_effect(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        start_side_effect(self.get_data_mut())
    }

    fn end_side_effect(&mut self) -> Result<(), RuntimeError<Data::Error>> {
        end_side_effect(self.get_data_mut())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        runtime::{context::EmptyContext, GarnishRuntime},
        ExpressionDataType, GarnishLangRuntimeData, GarnishLangRuntimeState, Instruction,
    };
    use crate::simple::SimpleRuntimeData;
    use crate::testing_utilites::create_simple_runtime;

    #[test]
    fn create_data() {
        SimpleRuntimeData::new();
    }

    #[test]
    fn add_jump_point() {
                let mut runtime = create_simple_runtime();


        runtime.get_data_mut().push_instruction(Instruction::EndExpression, None).unwrap();
        runtime.get_data_mut().push_jump_point(1).unwrap();

        assert_eq!(runtime.get_data_mut().get_jump_points().len(), 1);
        assert_eq!(*runtime.get_data_mut().get_jump_points().get(0).unwrap(), 1);
    }

    #[test]
    fn add_input_reference() {
                let mut runtime = create_simple_runtime();


        runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().push_value_stack(0).unwrap();

        assert_eq!(runtime.get_data_mut().get_value(0).unwrap().to_owned(), 0);
    }

    #[test]
    fn add_input_reference_with_data_addr() {
                let mut runtime = create_simple_runtime();


        let _i1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i2 = runtime.get_data_mut().add_number(10.into()).unwrap();

        runtime.get_data_mut().push_value_stack(i2).unwrap();

        assert_eq!(runtime.get_data_mut().get_value(0).unwrap().to_owned(), i2);
    }

    #[test]
    fn execute_current_instruction() {
                let mut runtime = create_simple_runtime();


        let d1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let d2 = runtime.get_data_mut().add_number(20.into()).unwrap();

        let i1 = runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.get_data_mut().set_instruction_cursor(i1).unwrap();

        runtime.execute_current_instruction::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 30.into());
    }

    #[test]
    fn execute_current_instruction_with_cursor_past_len() {
                let mut runtime = create_simple_runtime();


        let i1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i2 = runtime.get_data_mut().add_number(20.into()).unwrap();
        let _start = runtime.get_data_mut().get_data_len();

        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();

        runtime.get_data_mut().push_register(i1).unwrap();
        runtime.get_data_mut().push_register(i2).unwrap();

        runtime.get_data_mut().set_instruction_cursor(10).unwrap();

        let result = runtime.execute_current_instruction::<EmptyContext>(None).unwrap();

        assert_eq!(result.get_state(), GarnishLangRuntimeState::End);
    }

    #[test]
    fn execute_current_instruction_apply() {
                let mut runtime = create_simple_runtime();


        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let exp1 = runtime.get_data_mut().add_expression(0).unwrap();
        let int2 = runtime.get_data_mut().add_number(20.into()).unwrap();

        // 1
        let i1 = runtime.get_data_mut().push_instruction(Instruction::Put, Some(int1)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::PutValue, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::EndExpression, None).unwrap();

        // 5
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(exp1)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(int2)).unwrap();
        let i2 = runtime.get_data_mut().push_instruction(Instruction::Apply, None).unwrap();
        let i3 = runtime.get_data_mut().push_instruction(Instruction::EndExpression, None).unwrap();

        runtime.get_data_mut().push_jump_point(i1).unwrap();

        runtime.get_data_mut().push_register(exp1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.get_data_mut().set_instruction_cursor(i2).unwrap();

        runtime.execute_current_instruction::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_data_mut().get_value(0).unwrap(), int2);
        assert_eq!(runtime.get_data_mut().get_instruction_cursor(), i1);
        assert_eq!(runtime.get_data_mut().get_jump_path(0).unwrap(), i3);
    }

    #[test]
    fn execute_current_instruction_empty_apply() {
                let mut runtime = create_simple_runtime();


        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let exp1 = runtime.get_data_mut().add_expression(0).unwrap();

        // 1
        let i1 = runtime.get_data_mut().push_instruction(Instruction::Put, Some(int1)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::PutValue, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::EndExpression, None).unwrap();

        // 5
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(exp1)).unwrap();
        let i2 = runtime.get_data_mut().push_instruction(Instruction::EmptyApply, None).unwrap();
        let i3 = runtime.get_data_mut().push_instruction(Instruction::EndExpression, None).unwrap();

        runtime.get_data_mut().push_jump_point(i1).unwrap();

        runtime.get_data_mut().push_register(exp1).unwrap();

        runtime.get_data_mut().set_instruction_cursor(i2).unwrap();

        runtime.execute_current_instruction::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_value(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::Unit);
        assert_eq!(runtime.get_data_mut().get_instruction_cursor(), i1);
        assert_eq!(runtime.get_data_mut().get_jump_path(0).unwrap(), i3);
    }

    #[test]
    fn execute_current_instruction_reapply_if_true() {
                let mut runtime = create_simple_runtime();


        let true1 = runtime.get_data_mut().add_true().unwrap();
        let _exp1 = runtime.get_data_mut().add_expression(0).unwrap();
        let int1 = runtime.get_data_mut().add_number(20.into()).unwrap();
        let _int2 = runtime.get_data_mut().add_number(30.into()).unwrap();
        let int3 = runtime.get_data_mut().add_number(40.into()).unwrap();

        // 1
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(2)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Apply, None).unwrap();

        // 4
        let i1 = runtime.get_data_mut().push_instruction(Instruction::Put, Some(0)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::PutValue, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::PutValue, None).unwrap();
        let i2 = runtime.get_data_mut().push_instruction(Instruction::Reapply, Some(0)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::EndExpression, None).unwrap();

        runtime.get_data_mut().push_jump_point(i1).unwrap();

        runtime.get_data_mut().push_register(true1).unwrap();
        runtime.get_data_mut().push_register(int3).unwrap();

        runtime.get_data_mut().push_value_stack(int1).unwrap();

        runtime.get_data_mut().set_instruction_cursor(i2).unwrap();

        runtime.execute_current_instruction::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_data_mut().get_value_stack_len(), 1);
        assert_eq!(runtime.get_data_mut().get_value(0).unwrap(), int3);
        assert_eq!(runtime.get_data_mut().get_instruction_cursor(), i1);
    }

    #[test]
    fn execute_current_instruction_jump_if_true_when_true() {
                let mut runtime = create_simple_runtime();


        let ta = runtime.get_data_mut().add_true().unwrap();
        let i1 = runtime.get_data_mut().push_instruction(Instruction::JumpIfTrue, Some(0)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        let i2 = runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();

        runtime.get_data_mut().push_jump_point(i2).unwrap();

        runtime.get_data_mut().set_instruction_cursor(i1).unwrap();

        runtime.get_data_mut().push_register(ta).unwrap();

        runtime.execute_current_instruction::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_data_mut().get_register_len(), 0);
        assert_eq!(runtime.get_data_mut().get_data_len(), 3);
        assert_eq!(runtime.get_data_mut().get_instruction_cursor(), i2);
    }

    #[test]
    fn execute_current_instruction_jump_if_false_when_unit() {
                let mut runtime = create_simple_runtime();


        let ua = runtime.get_data_mut().add_unit().unwrap();
        let i1 = runtime.get_data_mut().push_instruction(Instruction::JumpIfFalse, Some(0)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        let i2 = runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();

        runtime.get_data_mut().push_jump_point(i2).unwrap();

        runtime.get_data_mut().set_instruction_cursor(i1).unwrap();

        runtime.get_data_mut().push_register(ua).unwrap();

        runtime.execute_current_instruction::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_data_mut().get_register_len(), 0);
        assert_eq!(runtime.get_data_mut().get_data_len(), 3);
        assert_eq!(runtime.get_data_mut().get_instruction_cursor(), i2);
    }

    #[test]
    fn execute_current_instruction_jump_if_false_when_false() {
                let mut runtime = create_simple_runtime();


        let fa = runtime.get_data_mut().add_false().unwrap();
        let i1 = runtime.get_data_mut().push_instruction(Instruction::JumpIfFalse, Some(0)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        let i2 = runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();

        runtime.get_data_mut().push_jump_point(i2).unwrap();

        runtime.get_data_mut().set_instruction_cursor(i1).unwrap();

        runtime.get_data_mut().push_register(fa).unwrap();

        runtime.execute_current_instruction::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_data_mut().get_register_len(), 0);
        assert_eq!(runtime.get_data_mut().get_data_len(), 3);
        assert_eq!(runtime.get_data_mut().get_instruction_cursor(), i2);
    }

    #[test]
    fn execute_current_instruction_end_expression() {
                let mut runtime = create_simple_runtime();


        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(1)).unwrap();
        let i1 = runtime.get_data_mut().push_instruction(Instruction::EndExpression, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(1)).unwrap();

        runtime.get_data_mut().set_instruction_cursor(i1).unwrap();
        runtime.get_data_mut().push_register(int1).unwrap();

        runtime.execute_current_instruction::<EmptyContext>(None).unwrap();

        let i = runtime.get_data_mut().get_current_value().unwrap();
        assert_eq!(runtime.get_data_mut().get_instruction_cursor(), runtime.get_data_mut().get_instruction_len());
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 10.into());
    }

    #[test]
    fn execute_current_instructionend_expression_with_path() {
                let mut runtime = create_simple_runtime();


        runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::EndExpression, Some(0)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::EmptyApply, None).unwrap();

        runtime.get_data_mut().push_register(1).unwrap();
        runtime.get_data_mut().push_jump_path(4).unwrap();
        runtime.get_data_mut().set_instruction_cursor(2).unwrap();

        runtime.execute_current_instruction::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_data_mut().get_instruction_cursor(), 4);
    }
}
