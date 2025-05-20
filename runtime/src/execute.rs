use log::trace;
use garnish_lang_traits::{GarnishContext, GarnishData, Instruction, RuntimeError, TypeConstants};
use crate::runtime::access::access;
use crate::runtime::apply::{apply, empty_apply, reapply};
use crate::runtime::arithmetic::{absolute_value, add, divide, integer_divide, multiply, opposite, power, remainder, subtract};
use crate::runtime::bitwise::{bitwise_and, bitwise_left_shift, bitwise_not, bitwise_or, bitwise_right_shift, bitwise_xor};
use crate::runtime::casting::{type_cast, type_of};
use crate::runtime::comparison::{greater_than, greater_than_or_equal, less_than, less_than_or_equal};
use crate::runtime::concat::concat;
use crate::runtime::equality::{equal, not_equal, type_equal};
use crate::runtime::internals::{access_left_internal, access_length_internal, access_right_internal};
use crate::runtime::jumps::{end_expression, jump, jump_if_false, jump_if_true};
use crate::runtime::list::make_list;
use crate::runtime::logical::{and, not, or, tis, xor};
use crate::runtime::pair::make_pair;
use crate::runtime::put::{push_value, put, put_value, update_value};
use crate::runtime::range::{make_end_exclusive_range, make_exclusive_range, make_range, make_start_exclusive_range};
use crate::runtime::resolve::resolve;
use crate::runtime::sideeffect::{end_side_effect, start_side_effect};

/// State that the runtime is currently in.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum SimpleRuntimeState {
    Running,
    End,
}

/// Information about the runtime, returned after execution an instruction.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct SimpleRuntimeInfo {
    state: SimpleRuntimeState,
}

/// Information about the current execution state of a runtime.
impl SimpleRuntimeInfo {
    pub fn new(state: SimpleRuntimeState) -> Self {
        SimpleRuntimeInfo { state }
    }

    pub fn get_state(&self) -> SimpleRuntimeState {
        self.state
    }
}

pub fn execute_current_instruction<Data: GarnishData, T: GarnishContext<Data>>(
    data: &mut Data,
    context: Option<&mut T>,
) -> Result<SimpleRuntimeInfo, RuntimeError<Data::Error>> {
    let (instruction, instruction_data) = match data.get_instruction(data.get_instruction_cursor()) {
        None => return Ok(SimpleRuntimeInfo::new(SimpleRuntimeState::End)),
        Some(v) => v,
    };

    trace!(
            "Executing instruction {:?} at {:?} with data {:?}",
            instruction,
            data.get_instruction_cursor(),
            instruction_data
        );

    let next_instruction = match instruction {
        Instruction::Invalid => None,
        Instruction::Add => add(data, context)?,
        Instruction::Subtract => subtract(data, context)?,
        Instruction::Multiply => multiply(data, context)?,
        Instruction::Divide => divide(data, context)?,
        Instruction::IntegerDivide => integer_divide(data, context)?,
        Instruction::Power => power(data, context)?,
        Instruction::Opposite => opposite(data, context)?,
        Instruction::AbsoluteValue => absolute_value(data, context)?,
        Instruction::Remainder => remainder(data, context)?,
        Instruction::BitwiseNot => bitwise_not(data, context)?,
        Instruction::BitwiseAnd => bitwise_and(data, context)?,
        Instruction::BitwiseOr => bitwise_or(data, context)?,
        Instruction::BitwiseXor => bitwise_xor(data, context)?,
        Instruction::BitwiseShiftLeft => bitwise_left_shift(data, context)?,
        Instruction::BitwiseShiftRight => bitwise_right_shift(data, context)?,
        Instruction::Xor => xor(data)?,
        Instruction::Not => not(data)?,
        Instruction::Tis => tis(data)?,
        Instruction::PutValue => put_value(data)?,
        Instruction::PushValue => push_value(data)?,
        Instruction::UpdateValue => update_value(data)?,
        Instruction::StartSideEffect => start_side_effect(data)?,
        Instruction::EndSideEffect => end_side_effect(data)?,
        Instruction::TypeOf => type_of(data)?,
        Instruction::ApplyType => type_cast(data, context)?,
        Instruction::TypeEqual => type_equal(data)?,
        Instruction::Equal => equal(data)?,
        Instruction::NotEqual => not_equal(data)?,
        Instruction::LessThan => less_than(data)?,
        Instruction::LessThanOrEqual => less_than_or_equal(data)?,
        Instruction::GreaterThan => greater_than(data)?,
        Instruction::GreaterThanOrEqual => greater_than_or_equal(data)?,
        Instruction::MakePair => make_pair(data)?,
        Instruction::Access => access(data, context)?,
        Instruction::AccessLeftInternal => access_left_internal(data, context)?,
        Instruction::AccessRightInternal => access_right_internal(data, context)?,
        Instruction::AccessLengthInternal => access_length_internal(data, context)?,
        Instruction::MakeRange => make_range(data)?,
        Instruction::MakeStartExclusiveRange => make_start_exclusive_range(data)?,
        Instruction::MakeEndExclusiveRange => make_end_exclusive_range(data)?,
        Instruction::MakeExclusiveRange => make_exclusive_range(data)?,
        Instruction::Concat => concat(data)?,
        Instruction::EndExpression => end_expression(data)?,
        Instruction::Apply => apply(data, context)?,
        Instruction::PartialApply => unimplemented!(),
        Instruction::EmptyApply => empty_apply(data, context)?,
        Instruction::And => match instruction_data {
            None => crate::runtime::error::instruction_error(instruction, data.get_instruction_cursor())?,
            Some(i) => and(data, i)?,
        },
        Instruction::Or => match instruction_data {
            None => crate::runtime::error::instruction_error(instruction, data.get_instruction_cursor())?,
            Some(i) => or(data, i)?,
        },
        Instruction::Put => match instruction_data {
            None => crate::runtime::error::instruction_error(instruction, data.get_instruction_cursor())?,
            Some(i) => put(data, i)?,
        },
        Instruction::MakeList => match instruction_data {
            None => crate::runtime::error::instruction_error(instruction, data.get_instruction_cursor())?,
            Some(i) => make_list(data, i)?,
        },
        Instruction::Resolve => match instruction_data {
            None => crate::runtime::error::instruction_error(instruction, data.get_instruction_cursor())?,
            Some(i) => resolve(data, i, context)?,
        },
        Instruction::Reapply => match instruction_data {
            None => crate::runtime::error::instruction_error(instruction, data.get_instruction_cursor())?,
            Some(i) => reapply(data, i)?,
        },
        Instruction::JumpIfTrue => match instruction_data {
            None => crate::runtime::error::instruction_error(instruction, data.get_instruction_cursor())?,
            Some(i) => jump_if_true(data, i)?,
        },
        Instruction::JumpIfFalse => match instruction_data {
            None => crate::runtime::error::instruction_error(instruction, data.get_instruction_cursor())?,
            Some(i) => jump_if_false(data, i)?,
        },
        Instruction::JumpTo => match instruction_data {
            None => crate::runtime::error::instruction_error(instruction, data.get_instruction_cursor())?,
            Some(i) => jump(data, i)?,
        },
    };

    let next_instruction = match next_instruction {
        Some(i) => i,
        None => data.get_instruction_cursor() + Data::Size::one(),
    };

    match next_instruction >= data.get_instruction_len() {
        true => Ok(SimpleRuntimeInfo::new(SimpleRuntimeState::End)),
        false => {
            data.set_instruction_cursor(next_instruction)?;
            Ok(SimpleRuntimeInfo::new(SimpleRuntimeState::Running))
        }
    }
}