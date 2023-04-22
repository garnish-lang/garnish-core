use garnish_traits::Instruction;

use crate::runtime::internals::concatenation_len;
use crate::runtime::list::{is_value_association, iterate_concatenation_internal};
use crate::{
    get_range, next_ref, next_two_raw_ref, push_unit, ExpressionDataType, GarnishLangRuntimeContext, GarnishLangRuntimeData, GarnishNumber,
    OrNumberError, RuntimeError, TypeConstants,
};

pub(crate) fn type_of<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    let a = next_ref(this)?;
    let t = this.get_data_type(a)?;
    this.add_type(t).and_then(|r| this.push_register(r))?;

    Ok(())
}

pub(crate) fn type_cast<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<(), RuntimeError<Data::Error>> {
    let (right, left) = next_two_raw_ref(this)?;

    let (left_type, mut right_type) = (this.get_data_type(left)?, this.get_data_type(right)?);

    if right_type == ExpressionDataType::Type {
        // correct actual type we want to cast to
        right_type = this.get_type(right)?;
    }

    match (left_type, right_type) {
        // NoOp re-push left to register
        (l, r) if l == r => this.push_register(left)?,

        // Casts that defer to data object and only expect an addr to push
        (ExpressionDataType::CharList, ExpressionDataType::Byte) => {
            this.add_byte_from(left).and_then(|r| this.push_register(r))?;
        }
        (ExpressionDataType::CharList, ExpressionDataType::Number) => {
            this.add_number_from(left).and_then(|r| this.push_register(r))?;
        }
        (_, ExpressionDataType::CharList) => {
            this.add_char_list_from(left).and_then(|r| this.push_register(r))?;
        }
        (_, ExpressionDataType::ByteList) => {
            this.add_byte_list_from(left).and_then(|r| this.push_register(r))?;
        }
        (_, ExpressionDataType::Symbol) => {
            this.add_symbol_from(left).and_then(|r| this.push_register(r))?;
        }
        // Primitives
        (ExpressionDataType::Number, ExpressionDataType::Char) => {
            primitive_cast(this, left, Data::get_number, Data::number_to_char, Data::add_char)?;
        }
        (ExpressionDataType::Number, ExpressionDataType::Byte) => {
            primitive_cast(this, left, Data::get_number, Data::number_to_byte, Data::add_byte)?;
        }
        (ExpressionDataType::Char, ExpressionDataType::Number) => {
            primitive_cast(this, left, Data::get_char, Data::char_to_number, Data::add_number)?;
        }
        (ExpressionDataType::Char, ExpressionDataType::Byte) => {
            primitive_cast(this, left, Data::get_char, Data::char_to_byte, Data::add_byte)?;
        }
        (ExpressionDataType::Byte, ExpressionDataType::Number) => {
            primitive_cast(this, left, Data::get_byte, Data::byte_to_number, Data::add_number)?;
        }
        (ExpressionDataType::Byte, ExpressionDataType::Char) => {
            primitive_cast(this, left, Data::get_byte, Data::byte_to_char, Data::add_char)?;
        }
        (ExpressionDataType::CharList, ExpressionDataType::Char) => {
            let len = this.get_char_list_len(left)?;
            if len == Data::Size::one() {
                this.get_char_list_item(left, Data::Number::zero())
                    .and_then(|c| this.add_char(c))
                    .and_then(|r| this.push_register(r))?;
            } else {
                push_unit(this)?;
            }
        }
        (ExpressionDataType::Range, ExpressionDataType::List) => {
            let (start, end) = this.get_range(left)?;
            let len = end - start + Data::Size::one();
            let (start, end, _) = get_range(this, left)?;
            let mut count = start;

            this.start_list(len)?;
            while count <= end {
                let addr = this.add_number(count)?;
                this.add_to_list(addr, false)?;
                count = count.increment().or_num_err()?;
            }

            this.end_list().and_then(|r| this.push_register(r))?
        }
        (ExpressionDataType::CharList, ExpressionDataType::List) => {
            let len = this.get_char_list_len(left)?;
            list_from_char_list(this, left, Data::Number::zero(), Data::size_to_number(len))?;
        }
        (ExpressionDataType::ByteList, ExpressionDataType::List) => {
            let len = this.get_byte_list_len(left)?;
            list_from_byte_list(this, left, Data::Number::zero(), Data::size_to_number(len))?;
        }
        // TODO
        // (ExpressionDataType::Slice, ExpressionDataType::CharList) => {}
        // (ExpressionDataType::Slice, ExpressionDataType::ByteList) => {}
        (ExpressionDataType::Slice, ExpressionDataType::List) => {
            let (value, range) = this.get_slice(left)?;
            let (start, end, len) = get_range(this, range)?;
            match this.get_data_type(value)? {
                ExpressionDataType::List => {
                    let len = this.get_list_len(value)?;

                    this.start_list(len)?;

                    let mut i = start;

                    while i <= end {
                        let addr = this.get_list_item(value, i)?;
                        let is_associative = match this.get_data_type(addr)? {
                            ExpressionDataType::Pair => {
                                let (left, _) = this.get_pair(addr)?;
                                match this.get_data_type(left)? {
                                    ExpressionDataType::Symbol => true,
                                    _ => false,
                                }
                            }
                            _ => false,
                        };

                        this.add_to_list(addr, is_associative)?;
                        i = i.increment().or_num_err()?;
                    }

                    this.end_list().and_then(|r| this.push_register(r))?
                }
                ExpressionDataType::CharList => {
                    list_from_char_list(this, value, start, end.increment().or_num_err()?)?;
                }
                ExpressionDataType::ByteList => {
                    list_from_byte_list(this, value, start, end.increment().or_num_err()?)?;
                }
                ExpressionDataType::Concatenation => {
                    this.start_list(Data::number_to_size(len).or_num_err()?)?;

                    iterate_concatenation_internal(
                        this,
                        value,
                        |this, current_index, addr| {
                            let list_len = Data::size_to_number(this.get_list_len(addr)?);
                            let list_end = current_index.plus(list_len).or_num_err()?;

                            if start > list_end {
                                return Ok(None);
                            }

                            if end <= current_index {
                                // providing value will end iteration
                                // even tho we don't need the return value
                                return Ok(Some(addr));
                            }

                            let adjusted_start = if current_index > start {
                                Data::Number::zero()
                            } else {
                                start.subtract(current_index).or_num_err()?
                            };

                            let adjusted_end = if end > list_end {
                                list_len.decrement().or_num_err()?
                            } else {
                                end.subtract(current_index).or_num_err()?
                            };

                            if adjusted_start < list_end && adjusted_end >= adjusted_start {
                                let mut index = adjusted_start;

                                while index <= adjusted_end {
                                    let item_addr = this.get_list_item(addr, index)?;
                                    let is_associative = is_value_association(this, item_addr)?;
                                    this.add_to_list(item_addr, is_associative)?;

                                    index = index.increment().or_num_err()?;
                                }
                            }

                            Ok(None)
                        },
                        |this, current_index, addr| {
                            if current_index < start {
                                return Ok(None);
                            }

                            if current_index >= end {
                                // providing value will end iteration
                                // even tho we don't need the return value
                                return Ok(Some(addr));
                            }

                            let is_associative = is_value_association(this, addr)?;
                            this.add_to_list(addr, is_associative)?;
                            Ok(None)
                        },
                    )?;

                    let addr = this.end_list()?;
                    this.push_register(addr)?;
                }
                _ => push_unit(this)?,
            }
        }
        // TODO
        // (ExpressionDataType::Concatenation, ExpressionDataType::CharList) => {}
        // (ExpressionDataType::Concatenation, ExpressionDataType::ByteList) => {}
        (ExpressionDataType::Concatenation, ExpressionDataType::List) => {
            let len = concatenation_len(this, left)?;
            this.start_list(len)?;
            iterate_concatenation_internal(
                this,
                left,
                |this, _, addr| {
                    let len = Data::size_to_number(this.get_list_len(addr)?);
                    let mut index = Data::Number::zero();

                    while index < len {
                        let item_addr = this.get_list_item(addr, index)?;
                        let is_associative = is_value_association(this, item_addr)?;
                        this.add_to_list(item_addr, is_associative)?;

                        index = index.increment().or_num_err()?;
                    }

                    Ok(None)
                },
                |this, _, addr| {
                    let is_associative = is_value_association(this, addr)?;
                    this.add_to_list(addr, is_associative)?;
                    Ok(None)
                },
            )?;

            let addr = this.end_list()?;
            this.push_register(addr)?;
        }
        // Unit and Boolean
        (ExpressionDataType::Unit, ExpressionDataType::True) | (ExpressionDataType::False, ExpressionDataType::True) => {
            this.add_false().and_then(|r| this.push_register(r))?;
        }
        (ExpressionDataType::Unit, ExpressionDataType::False) => this.add_true().and_then(|r| this.push_register(r))?,

        // Final Catches
        (ExpressionDataType::Unit, _) => push_unit(this)?,
        (_, ExpressionDataType::False) => this.add_false().and_then(|r| this.push_register(r))?,
        (_, ExpressionDataType::True) => this.add_true().and_then(|r| this.push_register(r))?,
        (l, r) => match context {
            None => push_unit(this)?,
            Some(c) => {
                if !c.defer_op(this, Instruction::ApplyType, (l, left), (r, right))? {
                    push_unit(this)?
                }
            }
        },
    }

    Ok(())
}

pub(crate) fn list_from_char_list<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    byte_list_addr: Data::Size,
    start: Data::Number,
    end: Data::Number,
) -> Result<(), RuntimeError<Data::Error>> {
    let len = this.get_char_list_len(byte_list_addr)?;
    let mut count = start;

    this.start_list(len)?;
    while count < end {
        let c = this.get_char_list_item(byte_list_addr, count)?;
        let addr = this.add_char(c)?;
        this.add_to_list(addr, false)?;

        count = count.increment().or_num_err()?;
    }

    this.end_list().and_then(|r| this.push_register(r))?;

    Ok(())
}

pub(crate) fn list_from_byte_list<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    byte_list_addr: Data::Size,
    start: Data::Number,
    end: Data::Number,
) -> Result<(), RuntimeError<Data::Error>> {
    let len = this.get_byte_list_len(byte_list_addr)?;
    let mut count = start;

    this.start_list(len)?;
    while count < end {
        let c = this.get_byte_list_item(byte_list_addr, count)?;
        let addr = this.add_byte(c)?;
        this.add_to_list(addr, false)?;

        count = count.increment().or_num_err()?;
    }

    this.end_list().and_then(|r| this.push_register(r))?;

    Ok(())
}

pub(crate) fn primitive_cast<Data: GarnishLangRuntimeData, From, To, GetFunc, CastFunc, AddFunc>(
    this: &mut Data,
    addr: Data::Size,
    get: GetFunc,
    cast: CastFunc,
    add: AddFunc,
) -> Result<(), RuntimeError<Data::Error>>
where
    GetFunc: Fn(&Data, Data::Size) -> Result<From, Data::Error>,
    CastFunc: Fn(From) -> Option<To>,
    AddFunc: FnOnce(&mut Data, To) -> Result<Data::Size, Data::Error>,
{
    let i = get(this, addr)?;
    match cast(i) {
        Some(i) => {
            let r = add(this, i)?;
            this.push_register(r)?;
        }
        None => push_unit(this)?,
    }

    Ok(())
}
