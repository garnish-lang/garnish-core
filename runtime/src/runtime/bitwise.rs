use crate::runtime::arithmetic::{perform_op, perform_unary_op};
use garnish_lang_traits::{GarnishContext, GarnishData, GarnishNumber, Instruction, RuntimeError};

pub fn bitwise_not<Data: GarnishData, Context: GarnishContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_unary_op(this, Instruction::BitwiseNot, Data::Number::bitwise_not, context)
}

pub fn bitwise_and<Data: GarnishData, Context: GarnishContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_op(this, Instruction::BitwiseAnd, Data::Number::bitwise_and, context)
}

pub fn bitwise_or<Data: GarnishData, Context: GarnishContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_op(this, Instruction::BitwiseOr, Data::Number::bitwise_or, context)
}

pub fn bitwise_xor<Data: GarnishData, Context: GarnishContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_op(this, Instruction::BitwiseXor, Data::Number::bitwise_xor, context)
}

pub fn bitwise_left_shift<Data: GarnishData, Context: GarnishContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_op(this, Instruction::BitwiseShiftLeft, Data::Number::bitwise_shift_left, context)
}

pub fn bitwise_right_shift<Data: GarnishData, Context: GarnishContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_op(this, Instruction::BitwiseShiftRight, Data::Number::bitwise_shift_right, context)
}
