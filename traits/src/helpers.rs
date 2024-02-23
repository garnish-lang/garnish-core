use crate::{ExpressionDataType, GarnishLangRuntimeData, GarnishNumber, RuntimeError, TypeConstants};

pub fn iterate_concatenation_mut<Data: GarnishLangRuntimeData, CheckFn>(
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

pub fn iterate_rev_concatenation_mut<Data: GarnishLangRuntimeData, CheckFn>(
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

fn get_rev_concatentation<Data: GarnishLangRuntimeData>(
    this: &Data,
    addr: Data::Size,
) -> Result<(Data::Size, Data::Size), Data::Error> {
    let (left, right) = this.get_concatenation(addr)?;
    Ok((right, left))
}

pub fn iterate_concatenation_mut_with_method<Data: GarnishLangRuntimeData, CheckFn, GetFn>(
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
        match this.pop_register() {
            None => Err(RuntimeError::new(
                format!("Popping more registers than placed during concatenation indexing.").as_str(),
            ))?,
            Some(r) => {
                let mut temp_result = None;
                match this.get_data_type(r)? {
                    ExpressionDataType::Concatenation => {
                        let (current, next) = get_method(this, r)?;
                        this.push_register(next)?;
                        this.push_register(current)?;
                    }
                    ExpressionDataType::List => {
                        let len = this.get_list_len(r)?;
                        let mut i = Data::Size::zero();

                        while i < len {
                            let sub_index = Data::size_to_number(i);
                            let item = this.get_list_item(r, sub_index)?;
                            temp_result = check_fn(
                                this,
                                (Data::size_to_number(index)).plus(sub_index).ok_or(RuntimeError::new("Number error"))?,
                                item,
                            )?;
                            match temp_result {
                                Some(_) => break,
                                None => i = i + Data::Size::one(),
                            }
                        }
                        index = index + len;
                    }
                    _ => {
                        temp_result = check_fn(this, Data::size_to_number(index), r)?;
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
        this.pop_register();
    }

    Ok((result, index))
}
