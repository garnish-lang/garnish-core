use garnish_lang_traits::Instruction;
use log::trace;

use crate::runtime::utilities::{next_ref, next_two_raw_ref, push_number, push_unit};
use garnish_lang_traits::{ExpressionDataType, GarnishLangRuntimeContext, GarnishLangRuntimeData, GarnishNumber, RuntimeError, TypeConstants};

pub fn add<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_op(this, Instruction::Add, Data::Number::plus, context)
}

pub fn subtract<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_op(this, Instruction::Subtract, Data::Number::subtract, context)
}

pub fn multiply<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_op(this, Instruction::Multiply, Data::Number::multiply, context)
}

pub fn power<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_op(this, Instruction::Power, Data::Number::power, context)
}

pub fn divide<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_op(this, Instruction::Power, Data::Number::divide, context)
}

pub fn integer_divide<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_op(this, Instruction::IntegerDivide, Data::Number::integer_divide, context)
}

pub fn remainder<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_op(this, Instruction::Remainder, Data::Number::remainder, context)
}

pub fn absolute_value<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_unary_op(this, Instruction::AbsoluteValue, Data::Number::absolute_value, context)
}

pub fn opposite<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_unary_op(this, Instruction::Opposite, Data::Number::opposite, context)
}

pub(crate) fn perform_unary_op<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>, Op>(
    this: &mut Data,
    op_name: Instruction,
    op: Op,
    context: Option<&mut Context>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>
where
    Op: FnOnce(Data::Number) -> Option<Data::Number>,
{
    let addr = next_ref(this)?;

    let t = this.get_data_type(addr)?;
    trace!("Attempting {:?} on {:?} at {:?}", op_name, t, addr,);

    match t {
        ExpressionDataType::Number => {
            let value = this.get_number(addr)?;

            match op(value) {
                Some(result) => push_number(this, result)?,
                None => push_unit(this)?,
            }
        }
        l => match context {
            None => push_unit(this)?,
            Some(c) => {
                if !c.defer_op(this, op_name, (l, addr), (ExpressionDataType::Unit, Data::Size::zero()))? {
                    push_unit(this)?
                }
            }
        },
    }

    Ok(None)
}

pub(crate) fn perform_op<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>, Op>(
    this: &mut Data,
    op_name: Instruction,
    op: Op,
    context: Option<&mut Context>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>
where
    Op: FnOnce(Data::Number, Data::Number) -> Option<Data::Number>,
{
    let (right_addr, left_addr) = next_two_raw_ref(this)?;

    let types = (this.get_data_type(left_addr)?, this.get_data_type(right_addr)?);
    trace!(
        "Attempting {:?} between {:?} at {:?} and {:?} at {:?}",
        op_name,
        types.0,
        left_addr,
        types.1,
        right_addr
    );

    match types {
        (ExpressionDataType::Number, ExpressionDataType::Number) => {
            let left = this.get_number(left_addr)?;
            let right = this.get_number(right_addr)?;

            match op(left, right) {
                Some(result) => push_number(this, result)?,
                None => push_unit(this)?,
            }
        }
        (l, r) => match context {
            None => push_unit(this)?,
            Some(c) => {
                if !c.defer_op(this, op_name, (l, left_addr), (r, right_addr))? {
                    push_unit(this)?
                }
            }
        },
    }

    Ok(None)
}
