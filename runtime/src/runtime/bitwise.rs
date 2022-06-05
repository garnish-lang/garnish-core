use crate::runtime::arithmetic::{perform_op, perform_unary_op};
use crate::{GarnishLangRuntimeContext, GarnishLangRuntimeData, GarnishNumber, RuntimeError};
use garnish_traits::Instruction;

pub fn bitwise_not<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<(), RuntimeError<Data::Error>> {
    perform_unary_op(this, Instruction::BitwiseNot, Data::Number::bitwise_not, context)
}

pub fn bitwise_and<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, Instruction::BitwiseAnd, Data::Number::bitwise_and, context)
}

pub fn bitwise_or<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, Instruction::BitwiseOr, Data::Number::bitwise_or, context)
}

pub fn bitwise_xor<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, Instruction::BitwiseXor, Data::Number::bitwise_xor, context)
}

pub fn bitwise_left_shift<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, Instruction::BitwiseShiftLeft, Data::Number::bitwise_shift_left, context)
}

pub fn bitwise_right_shift<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, Instruction::BitwiseShiftRight, Data::Number::bitwise_shift_right, context)
}
