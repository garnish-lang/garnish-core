use garnish_lang_traits::helpers::iterate_concatenation_mut;
use garnish_lang_traits::Instruction;

use crate::runtime::error::OrNumberError;
use crate::runtime::internals::concatenation_len;
use crate::runtime::list::is_value_association;
use crate::runtime::utilities::{get_range, next_ref, next_two_raw_ref, push_unit};
use garnish_lang_traits::{GarnishDataType, GarnishContext, GarnishData, GarnishNumber, RuntimeError, TypeConstants};

pub(crate) fn type_of<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let a = next_ref(this)?;
    let t = this.get_data_type(a)?;
    this.add_type(t).and_then(|r| this.push_register(r))?;

    Ok(None)
}

pub(crate) fn type_cast<Data: GarnishData, Context: GarnishContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let (right, left) = next_two_raw_ref(this)?;

    let (left_type, mut right_type) = (this.get_data_type(left.clone())?, this.get_data_type(right.clone().clone())?);

    if right_type == GarnishDataType::Type {
        // correct actual type we want to cast to
        right_type = this.get_type(right.clone())?;
    }

    match (left_type, right_type) {
        // NoOp re-push left to register
        (l, r) if l == r => this.push_register(left)?,

        // Casts that defer to data object and only expect an addr to push
        (GarnishDataType::CharList, GarnishDataType::Byte) => {
            this.add_byte_from(left).and_then(|r| this.push_register(r))?;
        }
        (GarnishDataType::CharList, GarnishDataType::Number) => {
            this.add_number_from(left).and_then(|r| this.push_register(r))?;
        }
        (_, GarnishDataType::CharList) => {
            this.add_char_list_from(left).and_then(|r| this.push_register(r))?;
        }
        (_, GarnishDataType::ByteList) => {
            this.add_byte_list_from(left).and_then(|r| this.push_register(r))?;
        }
        (_, GarnishDataType::Symbol) => {
            this.add_symbol_from(left).and_then(|r| this.push_register(r))?;
        }
        // Primitives
        (GarnishDataType::Number, GarnishDataType::Char) => {
            primitive_cast(this, left, Data::get_number, Data::number_to_char, Data::add_char)?;
        }
        (GarnishDataType::Number, GarnishDataType::Byte) => {
            primitive_cast(this, left, Data::get_number, Data::number_to_byte, Data::add_byte)?;
        }
        (GarnishDataType::Char, GarnishDataType::Number) => {
            primitive_cast(this, left, Data::get_char, Data::char_to_number, Data::add_number)?;
        }
        (GarnishDataType::Char, GarnishDataType::Byte) => {
            primitive_cast(this, left, Data::get_char, Data::char_to_byte, Data::add_byte)?;
        }
        (GarnishDataType::Byte, GarnishDataType::Number) => {
            primitive_cast(this, left, Data::get_byte, Data::byte_to_number, Data::add_number)?;
        }
        (GarnishDataType::Byte, GarnishDataType::Char) => {
            primitive_cast(this, left, Data::get_byte, Data::byte_to_char, Data::add_char)?;
        }
        (GarnishDataType::CharList, GarnishDataType::Char) => {
            let len = this.get_char_list_len(left.clone())?;
            if len == Data::Size::one() {
                this.get_char_list_item(left, Data::Number::zero())
                    .and_then(|c| this.add_char(c))
                    .and_then(|r| this.push_register(r))?;
            } else {
                push_unit(this)?;
            }
        }
        (GarnishDataType::Range, GarnishDataType::List) => {
            let (start, end) = this.get_range(left.clone())?;
            let len = end - start + Data::Size::one();
            let (start, end, _) = get_range(this, left)?;
            let mut count = start;

            this.start_list(len)?;
            while count <= end {
                let addr = this.add_number(count.clone())?;
                this.add_to_list(addr, false)?;
                count = count.increment().or_num_err()?;
            }

            this.end_list().and_then(|r| this.push_register(r))?
        }
        (GarnishDataType::CharList, GarnishDataType::List) => {
            let len = this.get_char_list_len(left.clone())?;
            list_from_char_list(this, left, Data::Number::zero(), Data::size_to_number(len))?;
        }
        (GarnishDataType::ByteList, GarnishDataType::List) => {
            let len = this.get_byte_list_len(left.clone())?;
            list_from_byte_list(this, left, Data::Number::zero(), Data::size_to_number(len))?;
        }
        (GarnishDataType::Concatenation, GarnishDataType::List) => {
            let len = concatenation_len(this, left.clone())?;
            this.start_list(len)?;
            iterate_concatenation_mut(this, left, |this, _, addr| {
                let is_associative = is_value_association(this, addr.clone())?;
                this.add_to_list(addr, is_associative)?;
                Ok(None)
            })?;

            let addr = this.end_list()?;
            this.push_register(addr)?;
        }
        (GarnishDataType::Slice, GarnishDataType::List) => {
            let (value, range) = this.get_slice(left)?;
            let (start, end, len) = get_range(this, range)?;
            match this.get_data_type(value.clone())? {
                GarnishDataType::List => {
                    let len = this.get_list_len(value.clone())?;

                    this.start_list(len)?;

                    let mut i = start;

                    while i <= end {
                        let addr = this.get_list_item(value.clone(), i.clone())?;
                        let is_associative = match this.get_data_type(addr.clone().clone())? {
                            GarnishDataType::Pair => {
                                let (left, _) = this.get_pair(addr.clone())?;
                                match this.get_data_type(left)? {
                                    GarnishDataType::Symbol => true,
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
                GarnishDataType::CharList => {
                    list_from_char_list(this, value, start, end.increment().or_num_err()?)?;
                }
                GarnishDataType::ByteList => {
                    list_from_byte_list(this, value, start, end.increment().or_num_err()?)?;
                }
                GarnishDataType::Concatenation => {
                    this.start_list(Data::number_to_size(len).or_num_err()?)?;

                    iterate_concatenation_mut(this, value, |this, current_index, addr| {
                        if current_index < start {
                            return Ok(None);
                        }

                        if current_index > end {
                            // providing value will end iteration
                            // even tho we don't need the return value
                            return Ok(Some(addr));
                        }

                        let is_associative = is_value_association(this, addr.clone())?;
                        this.add_to_list(addr, is_associative)?;
                        Ok(None)
                    })?;

                    let addr = this.end_list()?;
                    this.push_register(addr)?;
                }
                _ => push_unit(this)?,
            }
        }
        // Unit and Boolean
        (GarnishDataType::Unit, GarnishDataType::True) | (GarnishDataType::False, GarnishDataType::True) => {
            this.add_false().and_then(|r| this.push_register(r))?;
        }
        (GarnishDataType::Unit, GarnishDataType::False) => this.add_true().and_then(|r| this.push_register(r))?,

        // Final Catches
        (GarnishDataType::Unit, _) => push_unit(this)?,
        (_, GarnishDataType::False) => this.add_false().and_then(|r| this.push_register(r))?,
        (_, GarnishDataType::True) => this.add_true().and_then(|r| this.push_register(r))?,
        (l, r) => match context {
            None => push_unit(this)?,
            Some(c) => {
                if !c.defer_op(this, Instruction::ApplyType, (l, left), (r, right))? {
                    push_unit(this)?
                }
            }
        },
    }

    Ok(None)
}

pub(crate) fn list_from_char_list<Data: GarnishData>(
    this: &mut Data,
    byte_list_addr: Data::Size,
    start: Data::Number,
    end: Data::Number,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let len = this.get_char_list_len(byte_list_addr.clone())?;
    let mut count = start;

    this.start_list(len)?;
    while count < end {
        let c = this.get_char_list_item(byte_list_addr.clone(), count.clone())?;
        let addr = this.add_char(c)?;
        this.add_to_list(addr, false)?;

        count = count.increment().or_num_err()?;
    }

    this.end_list().and_then(|r| this.push_register(r))?;

    Ok(None)
}

pub(crate) fn list_from_byte_list<Data: GarnishData>(
    this: &mut Data,
    byte_list_addr: Data::Size,
    start: Data::Number,
    end: Data::Number,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let len = this.get_byte_list_len(byte_list_addr.clone())?;
    let mut count = start;

    this.start_list(len)?;
    while count < end {
        let c = this.get_byte_list_item(byte_list_addr.clone(), count.clone())?;
        let addr = this.add_byte(c)?;
        this.add_to_list(addr, false)?;

        count = count.increment().or_num_err()?;
    }

    this.end_list().and_then(|r| this.push_register(r))?;

    Ok(None)
}

pub(crate) fn primitive_cast<Data: GarnishData, From, To, GetFunc, CastFunc, AddFunc>(
    this: &mut Data,
    addr: Data::Size,
    get: GetFunc,
    cast: CastFunc,
    add: AddFunc,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>
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

    Ok(None)
}
