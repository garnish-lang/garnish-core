use crate::runtime::error::state_error;
use crate::runtime::range::range_len;
use crate::runtime::utilities::{next_ref, push_number, push_unit};
use garnish_lang_traits::helpers::iterate_concatenation_mut;
use garnish_lang_traits::Instruction;
use garnish_lang_traits::{GarnishDataType, GarnishContext, GarnishData, RuntimeError, TypeConstants};

pub(crate) fn access_left_internal<Data: GarnishData, Context: GarnishContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let r = next_ref(this)?;
    match this.get_data_type(r)? {
        GarnishDataType::Pair => {
            let (left, _) = this.get_pair(r)?;
            this.push_register(left)?;
        }
        GarnishDataType::Range => {
            let (start, _) = this.get_range(r)?;
            match this.get_data_type(start)? {
                GarnishDataType::Number => {
                    this.push_register(start)?;
                }
                _ => push_unit(this)?,
            }
        }
        GarnishDataType::Slice => {
            let (value, _) = this.get_slice(r)?;
            this.push_register(value)?;
        }
        GarnishDataType::Concatenation => {
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
                    (GarnishDataType::Unit, Data::Size::zero()),
                )? {
                    push_unit(this)?
                }
            }
        },
    }

    Ok(None)
}

pub(crate) fn access_right_internal<Data: GarnishData, Context: GarnishContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let r = next_ref(this)?;
    match this.get_data_type(r)? {
        GarnishDataType::Pair => {
            let (_, right) = this.get_pair(r)?;
            this.push_register(right)?;
        }
        GarnishDataType::Range => {
            let (_, end) = this.get_range(r)?;
            match this.get_data_type(end)? {
                GarnishDataType::Number => {
                    this.push_register(end)?;
                }
                _ => push_unit(this)?,
            }
        }
        GarnishDataType::Slice => {
            let (_, range) = this.get_slice(r)?;
            this.push_register(range)?;
        }
        GarnishDataType::Concatenation => {
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
                    (GarnishDataType::Unit, Data::Size::zero()),
                )? {
                    push_unit(this)?
                }
            }
        },
    }

    Ok(None)
}

pub(crate) fn access_length_internal<Data: GarnishData, Context: GarnishContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let r = next_ref(this)?;
    match this.get_data_type(r)? {
        GarnishDataType::List => {
            let len = Data::size_to_number(this.get_list_len(r)?);
            push_number(this, len)?;
        }
        GarnishDataType::CharList => {
            let len = Data::size_to_number(this.get_char_list_len(r)?);
            push_number(this, len)?;
        }
        GarnishDataType::ByteList => {
            let len = Data::size_to_number(this.get_byte_list_len(r)?);
            push_number(this, len)?;
        }
        GarnishDataType::Range => {
            let (start, end) = this.get_range(r)?;
            match (this.get_data_type(end)?, this.get_data_type(start)?) {
                (GarnishDataType::Number, GarnishDataType::Number) => {
                    let start_int = this.get_number(start)?;
                    let end_int = this.get_number(end)?;
                    let result = range_len::<Data>(start_int, end_int)?;

                    let addr = this.add_number(result)?;
                    this.push_register(addr)?;
                }
                _ => push_unit(this)?,
            }
        }
        GarnishDataType::Slice => {
            let (_, range_addr) = this.get_slice(r)?;
            let (start, end) = this.get_range(range_addr)?;
            match (this.get_data_type(start)?, this.get_data_type(end)?) {
                (GarnishDataType::Number, GarnishDataType::Number) => {
                    let start = this.get_number(start)?;
                    let end = this.get_number(end)?;
                    let addr = this.add_number(range_len::<Data>(start, end)?)?;
                    this.push_register(addr)?;
                }
                (s, e) => state_error(format!("Non integer values used for range {:?} {:?}", s, e))?,
            }
        }
        GarnishDataType::Concatenation => {
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
                    (GarnishDataType::Unit, Data::Size::zero()),
                )? {
                    push_unit(this)?
                }
            }
        },
    }

    Ok(None)
}

pub(crate) fn concatenation_len<Data: GarnishData>(this: &mut Data, addr: Data::Size) -> Result<Data::Size, RuntimeError<Data::Error>> {
    Ok(iterate_concatenation_mut(this, addr, |_, _, _| Ok(None))?.1)
}
