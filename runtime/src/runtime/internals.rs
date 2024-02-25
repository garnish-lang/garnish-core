use crate::runtime::error::state_error;
use crate::runtime::range::range_len;
use crate::runtime::utilities::{next_ref, push_number, push_unit};
use garnish_lang_traits::helpers::iterate_concatenation_mut;
use garnish_lang_traits::Instruction;
use garnish_lang_traits::{ExpressionDataType, GarnishLangRuntimeContext, GarnishLangRuntimeData, RuntimeError, TypeConstants};

pub(crate) fn access_left_internal<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let r = next_ref(this)?;
    match this.get_data_type(r)? {
        ExpressionDataType::Pair => {
            let (left, _) = this.get_pair(r)?;
            this.push_register(left)?;
        }
        ExpressionDataType::Range => {
            let (start, _) = this.get_range(r)?;
            match this.get_data_type(start)? {
                ExpressionDataType::Number => {
                    this.push_register(start)?;
                }
                _ => push_unit(this)?,
            }
        }
        ExpressionDataType::Slice => {
            let (value, _) = this.get_slice(r)?;
            this.push_register(value)?;
        }
        ExpressionDataType::Concatenation => {
            let (left, _) = this.get_concatenation(r)?;
            this.push_register(left)?;
        }
        t => match context {
            None => push_unit(this)?,
            Some(c) => {
                if !c.defer_op(
                    this,
                    Instruction::AccessLeftInternal,
                    (t, r),
                    (ExpressionDataType::Unit, Data::Size::zero()),
                )? {
                    push_unit(this)?
                }
            }
        },
    }

    Ok(None)
}

pub(crate) fn access_right_internal<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let r = next_ref(this)?;
    match this.get_data_type(r)? {
        ExpressionDataType::Pair => {
            let (_, right) = this.get_pair(r)?;
            this.push_register(right)?;
        }
        ExpressionDataType::Range => {
            let (_, end) = this.get_range(r)?;
            match this.get_data_type(end)? {
                ExpressionDataType::Number => {
                    this.push_register(end)?;
                }
                _ => push_unit(this)?,
            }
        }
        ExpressionDataType::Slice => {
            let (_, range) = this.get_slice(r)?;
            this.push_register(range)?;
        }
        ExpressionDataType::Concatenation => {
            let (_, right) = this.get_concatenation(r)?;
            this.push_register(right)?;
        }
        t => match context {
            None => push_unit(this)?,
            Some(c) => {
                if !c.defer_op(
                    this,
                    Instruction::AccessRightInternal,
                    (t, r),
                    (ExpressionDataType::Unit, Data::Size::zero()),
                )? {
                    push_unit(this)?
                }
            }
        },
    }

    Ok(None)
}

pub(crate) fn access_length_internal<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let r = next_ref(this)?;
    match this.get_data_type(r)? {
        ExpressionDataType::List => {
            let len = Data::size_to_number(this.get_list_len(r)?);
            push_number(this, len)?;
        }
        ExpressionDataType::CharList => {
            let len = Data::size_to_number(this.get_char_list_len(r)?);
            push_number(this, len)?;
        }
        ExpressionDataType::ByteList => {
            let len = Data::size_to_number(this.get_byte_list_len(r)?);
            push_number(this, len)?;
        }
        ExpressionDataType::Range => {
            let (start, end) = this.get_range(r)?;
            match (this.get_data_type(end)?, this.get_data_type(start)?) {
                (ExpressionDataType::Number, ExpressionDataType::Number) => {
                    let start_int = this.get_number(start)?;
                    let end_int = this.get_number(end)?;
                    let result = range_len::<Data>(start_int, end_int)?;

                    let addr = this.add_number(result)?;
                    this.push_register(addr)?;
                }
                _ => push_unit(this)?,
            }
        }
        ExpressionDataType::Slice => {
            let (_, range_addr) = this.get_slice(r)?;
            let (start, end) = this.get_range(range_addr)?;
            match (this.get_data_type(start)?, this.get_data_type(end)?) {
                (ExpressionDataType::Number, ExpressionDataType::Number) => {
                    let start = this.get_number(start)?;
                    let end = this.get_number(end)?;
                    let addr = this.add_number(range_len::<Data>(start, end)?)?;
                    this.push_register(addr)?;
                }
                (s, e) => state_error(format!("Non integer values used for range {:?} {:?}", s, e))?,
            }
        }
        ExpressionDataType::Concatenation => {
            let count = concatenation_len(this, r)?;
            let addr = this.add_number(Data::size_to_number(count))?;
            this.push_register(addr)?;
        }
        t => match context {
            None => push_unit(this)?,
            Some(c) => {
                if !c.defer_op(
                    this,
                    Instruction::AccessLengthInternal,
                    (t, r),
                    (ExpressionDataType::Unit, Data::Size::zero()),
                )? {
                    push_unit(this)?
                }
            }
        },
    }

    Ok(None)
}

pub(crate) fn concatenation_len<Data: GarnishLangRuntimeData>(this: &mut Data, addr: Data::Size) -> Result<Data::Size, RuntimeError<Data::Error>> {
    Ok(iterate_concatenation_mut(this, addr, |_, _, _| Ok(None))?.1)
}
