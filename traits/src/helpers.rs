//! Helper functions created for core libraries that don't have an implementation of [GarnishData].
//! These might be removed when an iterator interface is created for reading data instead of manual indexing.
//!

use crate::{GarnishDataType, GarnishData, GarnishNumber, RuntimeError, TypeConstants};

/// Iterates through a concatenation at the given address, calling provided 'check_fn' function for each item.
pub fn iterate_concatenation_mut<Data: GarnishData, CheckFn>(
    this: &mut Data,
    addr: Data::Size,
    #[allow(unused_mut)] // removing causes compiler error
    mut check_fn: CheckFn,
) -> Result<(Option<Data::Size>, Data::Size), RuntimeError<Data::Error>>
where
    CheckFn: FnMut(&mut Data, Data::Number, Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>,
{
    iterate_concatenation_mut_with_method(this, addr, check_fn, Data::get_concatenation)
}

/// Iterates through a concatenation at the given address in reverse, calling provided 'check_fn' function for each item.
pub fn iterate_rev_concatenation_mut<Data: GarnishData, CheckFn>(
    this: &mut Data,
    addr: Data::Size,
    #[allow(unused_mut)] // removing causes compiler error
    mut check_fn: CheckFn,
) -> Result<(Option<Data::Size>, Data::Size), RuntimeError<Data::Error>>
    where
        CheckFn: FnMut(&mut Data, Data::Number, Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>,
{
    iterate_concatenation_mut_with_method(this, addr, check_fn, get_rev_concatentation)
}

fn get_rev_concatentation<Data: GarnishData>(
    this: &Data,
    addr: Data::Size,
) -> Result<(Data::Size, Data::Size), Data::Error> {
    let (left, right) = this.get_concatenation(addr)?;
    Ok((right, left))
}

/// Iterates through a concatenation at the given address where a custom 'get_method' is needed, calling provided 'check_fn' function for each item.
///
/// 'get_method' should return the next 2 addresses to process
///
/// [`iterate_concatenation_mut`] uses [`GarnishData::get_concatenation`]
///
/// [`iterate_rev_concatenation_mut`] uses function that reverses return value of [`GarnishData::get_concatenation`]
pub fn iterate_concatenation_mut_with_method<Data: GarnishData, CheckFn, GetFn>(
    this: &mut Data,
    addr: Data::Size,
    mut check_fn: CheckFn,
    get_method: GetFn,
) -> Result<(Option<Data::Size>, Data::Size), RuntimeError<Data::Error>>
where
    CheckFn: FnMut(&mut Data, Data::Number, Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>,
    GetFn: Fn(&Data, Data::Size) -> Result<(Data::Size, Data::Size), Data::Error>,
{
    let (current, next) = get_method(this, addr)?;
    let start_register = this.get_register_len();
    let mut index = Data::Size::zero();

    this.push_register(next)?;
    this.push_register(current)?;

    let mut result = None;

    while this.get_register_len() > start_register {
        match this.pop_register()? {
            None => Err(RuntimeError::new(
                format!("Popping more registers than placed during concatenation indexing.").as_str(),
            ))?,
            Some(r) => {
                let mut temp_result = None;
                match this.get_data_type(r.clone())? {
                    GarnishDataType::Concatenation => {
                        let (current, next) = get_method(this, r.clone())?;
                        this.push_register(next)?;
                        this.push_register(current)?;
                    }
                    GarnishDataType::List => {
                        let len = this.get_list_len(r.clone())?;
                        let mut i = Data::Size::zero();

                        while i < len {
                            let sub_index = Data::size_to_number(i.clone());
                            let item = this.get_list_item(r.clone(), sub_index.clone())?;
                            temp_result = check_fn(
                                this,
                                (Data::size_to_number(index.clone())).plus(sub_index.clone()).ok_or(RuntimeError::new("Number error"))?,
                                item,
                            )?;
                            match temp_result {
                                Some(_) => break,
                                None => i = i.clone() + Data::Size::one(),
                            }
                        }
                        index = index.clone() + len;
                    }
                    _ => {
                        temp_result = check_fn(this, Data::size_to_number(index.clone()), r.clone())?;
                        index += Data::Size::one();
                    }
                }

                match temp_result {
                    Some(_) => {
                        result = temp_result;
                        break;
                    }
                    _ => (), // continue
                }
            }
        }
    }

    // clear borrowed registers
    while this.get_register_len() > start_register {
        this.pop_register()?;
    }

    Ok((result, index))
}
