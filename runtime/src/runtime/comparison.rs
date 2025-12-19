use crate::runtime::error::OrNumberError;
use crate::runtime::utilities::{get_range, next_two_raw_ref, push_boolean, push_unit};
use garnish_lang_traits::{GarnishDataType, GarnishData, GarnishDataFactory, GarnishNumber, RuntimeError, TypeConstants};
use std::cmp::Ordering;

pub fn less_than<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_comparison(this, Ordering::Greater).and_then(|result| match result {
        Some(result) => push_boolean(this, result.is_lt()),
        None => push_unit(this),
    })?;

    Ok(None)
}

pub fn less_than_or_equal<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_comparison(this, Ordering::Greater).and_then(|result| match result {
        Some(result) => push_boolean(this, result.is_le()),
        None => push_unit(this),
    })?;

    Ok(None)
}

pub fn greater_than<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_comparison(this, Ordering::Less).and_then(|result| match result {
        Some(result) => push_boolean(this, result.is_gt()),
        None => push_unit(this),
    })?;

    Ok(None)
}

pub fn greater_than_or_equal<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_comparison(this, Ordering::Less).and_then(|result| match result {
        Some(result) => push_boolean(this, result.is_ge()),
        None => push_unit(this),
    })?;

    Ok(None)
}

fn perform_comparison<Data: GarnishData>(this: &mut Data, false_ord: Ordering) -> Result<Option<Ordering>, RuntimeError<Data::Error>> {
    let (right, left) = next_two_raw_ref(this)?;

    let result = match (this.get_data_type(left.clone())?, this.get_data_type(right.clone())?) {
        (GarnishDataType::Number, GarnishDataType::Number) => this.get_number(left)?.partial_cmp(&this.get_number(right)?),
        (GarnishDataType::Char, GarnishDataType::Char) => this.get_char(left)?.partial_cmp(&this.get_char(right)?),
        (GarnishDataType::Byte, GarnishDataType::Byte) => this.get_byte(left)?.partial_cmp(&this.get_byte(right)?),
        (GarnishDataType::CharList, GarnishDataType::CharList) => cmp_list(
            this,
            left,
            right,
            Data::Number::zero(),
            Data::Number::zero(),
            Data::get_char_list_item,
            Data::get_char_list_len,
        )?,
        (GarnishDataType::ByteList, GarnishDataType::ByteList) => cmp_list(
            this,
            left,
            right,
            Data::Number::zero(),
            Data::Number::zero(),
            Data::get_byte_list_item,
            Data::get_byte_list_len,
        )?,
        (GarnishDataType::Slice, GarnishDataType::Slice) => {
            let (left_value, left_range) = this.get_slice(left)?;
            let (right_value, right_range) = this.get_slice(right)?;

            match (this.get_data_type(left_value.clone())?, this.get_data_type(right_value.clone())?) {
                (GarnishDataType::ByteList, GarnishDataType::ByteList) => {
                    let (start1, ..) = get_range(this, left_range)?;
                    let (start2, ..) = get_range(this, right_range)?;

                    cmp_list(
                        this,
                        left_value,
                        right_value,
                        start1,
                        start2,
                        Data::get_byte_list_item,
                        Data::get_byte_list_len,
                    )?
                }
                (GarnishDataType::CharList, GarnishDataType::CharList) => {
                    let (start1, ..) = get_range(this, left_range)?;
                    let (start2, ..) = get_range(this, right_range)?;

                    cmp_list(
                        this,
                        left_value,
                        right_value,
                        start1,
                        start2,
                        Data::get_char_list_item,
                        Data::get_char_list_len,
                    )?
                }
                _ => return Ok(Some(false_ord)),
            }
        }
        _ => return Ok(Some(false_ord)),
    };

    Ok(result)
}

fn cmp_list<Data: GarnishData, T: PartialOrd, GetFunc, LenFunc>(
    this: &mut Data,
    left: Data::Size,
    right: Data::Size,
    left_start: Data::Number,
    right_start: Data::Number,
    get_func: GetFunc,
    len_func: LenFunc,
) -> Result<Option<Ordering>, RuntimeError<Data::Error>>
where
    GetFunc: Fn(&Data, Data::Size, Data::Number) -> Result<T, Data::Error>,
    LenFunc: Fn(&Data, Data::Size) -> Result<Data::Size, Data::Error>,
{
    let (len1, len2) = (<Data as GarnishData>::DataFactory::size_to_number(len_func(this, left.clone())?), <Data as GarnishData>::DataFactory::size_to_number(len_func(this, right.clone())?));

    let mut left_index = left_start;
    let mut right_index = right_start;

    while left_index < len1 && right_index < len2 {
        match get_func(this, left.clone(), left_index.clone())?.partial_cmp(&get_func(this, right.clone(), right_index.clone())?) {
            Some(Ordering::Equal) => (),
            Some(non_eq) => return Ok(Some(non_eq)),
            None => (), // deferr
        }

        left_index = left_index.increment().or_num_err()?;
        right_index = right_index.increment().or_num_err()?;
    }

    Ok(len1.partial_cmp(&len2))
}
