use crate::runtime::range::range_len;
use crate::{
    get_range, next_ref, push_unit, state_error, ErrorType, ExpressionDataType, GarnishLangRuntimeContext, GarnishLangRuntimeData, GarnishNumber,
    Instruction, OrNumberError, RuntimeError, TypeConstants,
};

pub(crate) fn make_list<Data: GarnishLangRuntimeData>(this: &mut Data, len: Data::Size) -> Result<(), RuntimeError<Data::Error>> {
    if len > this.get_register_len() {
        state_error(format!("Not enough register values to make list of length {:?}", len))?
    }

    this.start_list(len)?;

    let mut count = this.get_register_len() - len;
    let end = this.get_register_len();
    // look into getting this to work with a range value
    while count < end {
        let r = match this.get_register(count) {
            None => state_error(format!("No register value at {:?} when making list", count))?,
            Some(r) => r,
        };

        let is_associative = match this.get_data_type(r)? {
            ExpressionDataType::Pair => {
                let (left, _right) = this.get_pair(r)?;
                match this.get_data_type(left)? {
                    ExpressionDataType::Symbol => true,
                    _ => false,
                }
            }
            _ => false,
        };

        this.add_to_list(r, is_associative)?;

        count += Data::Size::one();
    }

    // remove used registers
    count = Data::Size::zero();
    while count < len {
        this.pop_register();
        count += Data::Size::one();
    }

    this.end_list().and_then(|r| this.push_register(r))?;

    Ok(())
}

pub(crate) fn access<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<(), RuntimeError<Data::Error>> {
    let right_ref = next_ref(this)?;
    let left_ref = next_ref(this)?;

    match get_access_addr(this, right_ref, left_ref) {
        Err(e) => match e.get_type() {
            ErrorType::UnsupportedOpTypes => match context {
                None => push_unit(this)?,
                Some(c) => {
                    if !c.defer_op(
                        this,
                        Instruction::Access,
                        (this.get_data_type(left_ref)?, left_ref),
                        (this.get_data_type(right_ref)?, right_ref),
                    )? {
                        push_unit(this)?
                    }
                }
            },
            _ => Err(e)?,
        },
        Ok(i) => match i {
            None => push_unit(this)?,
            Some(i) => this.push_register(i)?,
        },
    }

    Ok(())
}

pub(crate) fn get_access_addr<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    right: Data::Size,
    left: Data::Size,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    match this.get_data_type(right)? {
        ExpressionDataType::Number => {
            let i = this.get_number(right)?;
            access_with_integer(this, i, left)
        }
        ExpressionDataType::Symbol => {
            let sym = this.get_symbol(right)?;
            access_with_symbol(this, sym, left)
        }
        _ => Err(RuntimeError::unsupported_types()),
    }
}

pub(crate) fn access_with_integer<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    index: Data::Number,
    value: Data::Size,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    match this.get_data_type(value)? {
        ExpressionDataType::List => index_list(this, value, index),
        ExpressionDataType::CharList => index_char_list(this, value, index),
        ExpressionDataType::ByteList => index_byte_list(this, value, index),
        ExpressionDataType::Range => {
            let (start, end) = this.get_range(value)?;
            match (this.get_data_type(start)?, this.get_data_type(end)?) {
                (ExpressionDataType::Number, ExpressionDataType::Number) => {
                    let start_int = this.get_number(start)?;
                    let end_int = this.get_number(end)?;
                    let len = range_len::<Data>(start_int, end_int)?;

                    if index >= len {
                        return Ok(None);
                    } else {
                        let result = start_int.plus(index).or_num_err()?;
                        let addr = this.add_number(result)?;
                        Ok(Some(addr))
                    }
                }
                _ => Ok(None),
            }
        }
        ExpressionDataType::Slice => {
            let (value, range) = this.get_slice(value)?;
            let (start, _, _) = get_range(this, range)?;
            let adjusted_index = start.plus(index).or_num_err()?;

            match this.get_data_type(value)? {
                ExpressionDataType::Link => index_link(this, value, adjusted_index),
                ExpressionDataType::List => index_list(this, value, adjusted_index),
                ExpressionDataType::CharList => index_char_list(this, value, adjusted_index),
                ExpressionDataType::ByteList => index_byte_list(this, value, adjusted_index),
                t => state_error(format!("Invalid value for slice {:?}", t)),
            }
        }
        ExpressionDataType::Link => index_link(this, value, index),
        _ => Err(RuntimeError::unsupported_types()),
    }
}

fn index_list<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    list: Data::Size,
    index: Data::Number,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    if index < Data::Number::zero() {
        Ok(None)
    } else {
        if index >= Data::size_to_integer(this.get_list_len(list)?) {
            Ok(None)
        } else {
            Ok(Some(this.get_list_item(list, index)?))
        }
    }
}

fn index_char_list<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    list: Data::Size,
    index: Data::Number,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    if index < Data::Number::zero() {
        Ok(None)
    } else {
        if index >= Data::size_to_integer(this.get_char_list_len(list)?) {
            Ok(None)
        } else {
            let c = this.get_char_list_item(list, index)?;
            let addr = this.add_char(c)?;
            Ok(Some(addr))
        }
    }
}

fn index_byte_list<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    list: Data::Size,
    index: Data::Number,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    if index < Data::Number::zero() {
        Ok(None)
    } else {
        if index >= Data::size_to_integer(this.get_byte_list_len(list)?) {
            Ok(None)
        } else {
            let c = this.get_byte_list_item(list, index)?;
            let addr = this.add_byte(c)?;
            Ok(Some(addr))
        }
    }
}

pub(crate) fn iterate_link_internal<Data: GarnishLangRuntimeData, Callback>(
    this: &mut Data,
    link: Data::Size,
    mut func: Callback,
) -> Result<(), RuntimeError<Data::Error>>
where
    Callback: FnMut(&mut Data, Data::Size, Data::Number) -> Result<bool, RuntimeError<Data::Error>>,
{
    let mut current_index = Data::Number::zero();
    // keep track of starting point to any pushed values later
    let start_register = this.get_register_len();

    this.push_register(link)?;

    while this.get_register_len() > start_register {
        match this.pop_register() {
            None => state_error(format!("Popping more registers than placed during linking indexing."))?,
            Some(r) => match this.get_data_type(r)? {
                // flatten all links, pushing their vals to the register
                // val of link should never be another link
                ExpressionDataType::Link => {
                    let (val, linked, is_append) = this.get_link(r)?;

                    match this.get_data_type(linked)? {
                        ExpressionDataType::Unit => {
                            // linked of type unit means, only push value
                            this.push_register(val)?;
                        }
                        ExpressionDataType::Link => {
                            if is_append {
                                // linked value is previous, gets checked first, pushed last
                                this.push_register(val)?;
                                this.push_register(linked)?;
                            } else {
                                // linked value is next, gets resolved last, pushed first
                                this.push_register(linked)?;
                                this.push_register(val)?;
                            }
                        }
                        t => state_error(format!("Invalid linked type {:?}", t))?,
                    }
                }
                _ => {
                    if func(this, r, current_index)? {
                        break;
                    } else {
                        current_index = current_index.increment().or_num_err()?;
                    }
                }
            },
        }
    }

    while this.get_register_len() > start_register {
        this.pop_register();
    }

    Ok(())
}

pub(crate) fn iterate_link_internal_rev<Data: GarnishLangRuntimeData, Callback>(
    this: &mut Data,
    link: Data::Size,
    mut func: Callback,
) -> Result<(), RuntimeError<Data::Error>>
where
    Callback: FnMut(&mut Data, Data::Size, Data::Number) -> Result<bool, RuntimeError<Data::Error>>,
{
    let mut current_index = Data::Number::zero();
    // keep track of starting point to any pushed values later
    let start_register = this.get_register_len();

    this.push_register(link)?;

    while this.get_register_len() > start_register {
        match this.pop_register() {
            None => state_error(format!("Popping more registers than placed during linking indexing."))?,
            Some(r) => match this.get_data_type(r)? {
                // flatten all links, pushing their vals to the register
                // val of link should never be another link
                ExpressionDataType::Link => {
                    let (val, linked, is_append) = this.get_link(r)?;

                    match this.get_data_type(linked)? {
                        ExpressionDataType::Unit => {
                            // linked of type unit means, only push value
                            this.push_register(val)?;
                        }
                        ExpressionDataType::Link => {
                            if is_append {
                                // linked value is previous, gets checked first, pushed last
                                this.push_register(linked)?;
                                this.push_register(val)?;
                            } else {
                                // linked value is next, gets resolved last, pushed first
                                this.push_register(val)?;
                                this.push_register(linked)?;
                            }
                        }
                        t => state_error(format!("Invalid linked type {:?}", t))?,
                    }
                }
                _ => {
                    if func(this, r, current_index)? {
                        break;
                    } else {
                        current_index = current_index.increment().or_num_err()?;
                    }
                }
            },
        }
    }

    while this.get_register_len() > start_register {
        this.pop_register();
    }

    Ok(())
}

pub(crate) fn index_link<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    link: Data::Size,
    index: Data::Number,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let mut item: Option<Data::Size> = None;
    iterate_link_internal(this, link, |_runtime, item_addr, current_index| {
        if current_index == index {
            item = Some(item_addr);
            Ok(true)
        } else {
            Ok(false)
        }
    })?;

    Ok(item)
}

fn access_with_symbol<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    sym: Data::Symbol,
    value: Data::Size,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    match this.get_data_type(value)? {
        ExpressionDataType::List => Ok(this.get_list_item_with_symbol(value, sym)?),
        ExpressionDataType::Slice => {
            let (value, range) = this.get_slice(value)?;
            let (start, end, _) = get_range(this, range)?;

            match this.get_data_type(value)? {
                ExpressionDataType::Link => sym_access_links_slices(this, start, value, sym, end),
                ExpressionDataType::List => {
                    // in order to limit to slice range need to check items manually
                    // can't push, in case any of the items are a Link or Slice
                    let mut i = start;
                    let mut item: Option<Data::Size> = None;
                    while i <= end {
                        let list_item = this.get_list_item(value, i)?;
                        match this.get_data_type(list_item)? {
                            ExpressionDataType::Pair => {
                                let (left, right) = this.get_pair(list_item)?;
                                match this.get_data_type(left)? {
                                    ExpressionDataType::Symbol => {
                                        if this.get_symbol(left)? == sym {
                                            item = Some(right);
                                            // found item break both loops
                                            break;
                                        }
                                    }
                                    _ => (),
                                }
                            }
                            _ => (),
                        }

                        i = i.increment().or_num_err()?;
                    }

                    Ok(item)
                }
                t => state_error(format!("Invalid value for slice {:?}", t)),
            }
        }
        ExpressionDataType::Link => sym_access_links_slices(this, Data::Number::zero(), value, sym, Data::Number::max_value()),
        _ => Err(RuntimeError::unsupported_types()),
    }
}

fn sym_access_links_slices<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    start_count: Data::Number,
    start_value: Data::Size,
    sym: Data::Symbol,
    limit: Data::Number,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let mut item: Option<Data::Size> = None;
    let mut skip = start_count;
    let mut count = Data::Number::zero();
    // keep track of starting point to any pushed values later
    let start_register = this.get_register_len();

    this.push_register(start_value)?;

    while this.get_register_len() > start_register {
        if count > limit {
            break;
        }

        match this.pop_register() {
            None => state_error(format!("Popping more registers than placed during linking indexing."))?,
            Some(r) => match this.get_data_type(r)? {
                ExpressionDataType::Link => {
                    let (val, linked, is_append) = this.get_link(r)?;

                    match this.get_data_type(linked)? {
                        ExpressionDataType::Unit => {
                            // linked of type unit means, only push value
                            this.push_register(val)?;
                        }
                        ExpressionDataType::Link => {
                            if is_append {
                                // linked value is previous, gets checked first, pushed last
                                this.push_register(val)?;
                                this.push_register(linked)?;
                            } else {
                                // linked value is next, gets resolved last, pushed first
                                this.push_register(linked)?;
                                this.push_register(val)?;
                            }
                        }
                        t => state_error(format!("Invalid linked type {:?}", t))?,
                    }
                }
                ExpressionDataType::Pair => {
                    // skip values
                    if skip > Data::Number::zero() {
                        skip = skip.decrement().or_num_err()?;
                        continue;
                    }

                    let (left, right) = this.get_pair(r)?;
                    match this.get_data_type(left)? {
                        ExpressionDataType::Symbol => {
                            // if this sym equals search sym, return 'right' value
                            // else, continue checking
                            if this.get_symbol(left)? == sym {
                                item = Some(right);
                                break;
                            } else {
                                count = count.increment().or_num_err()?;
                            }
                        }
                        // not an associations, continue on
                        _ => {
                            count = count.increment().or_num_err()?;
                        }
                    }
                }
                // nothing to do for any other values
                _ => {
                    // skip values
                    if skip > Data::Number::zero() {
                        skip = skip.decrement().or_num_err()?;
                        continue;
                    }

                    count = count.increment().or_num_err()?;
                }
            },
        }
    }

    // remove remaining registers added during operation
    while this.get_register_len() > start_register {
        this.pop_register();
    }

    Ok(item)
}

#[cfg(test)]
mod deferring {
    use crate::runtime::GarnishRuntime;
    use crate::testing_utilites::deferred_op;

    #[test]
    fn access() {
        deferred_op(|runtime, context| {
            runtime.access(Some(context)).unwrap();
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, Instruction, SimpleRuntimeData, NO_CONTEXT};

    #[test]
    fn make_list() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_number(10).unwrap();
        let i2 = runtime.add_number(20).unwrap();
        let i3 = runtime.add_number(20).unwrap();
        let start = runtime.get_data_len();

        runtime.push_register(i1).unwrap();
        runtime.push_register(i2).unwrap();
        runtime.push_register(i3).unwrap();

        runtime.push_instruction(Instruction::MakeList, Some(3)).unwrap();

        runtime.make_list(3).unwrap();

        assert_eq!(runtime.get_list_len(start).unwrap(), 3);
        assert_eq!(runtime.get_list_item(start, 0).unwrap(), i1);
        assert_eq!(runtime.get_list_item(start, 1).unwrap(), i2);
        assert_eq!(runtime.get_list_item(start, 2).unwrap(), i3);
    }

    #[test]
    fn make_list_no_refs_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_number(10).unwrap();
        runtime.add_number(20).unwrap();
        runtime.add_number(20).unwrap();

        runtime.push_instruction(Instruction::MakeList, Some(3)).unwrap();

        let result = runtime.make_list(3);

        assert!(result.is_err());
    }

    #[test]
    fn make_list_with_associations() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_number(10).unwrap();
        let i3 = runtime.add_symbol("two").unwrap();
        let i4 = runtime.add_number(20).unwrap();
        let i5 = runtime.add_symbol("three").unwrap();
        let i6 = runtime.add_number(30).unwrap();
        // 6
        let i7 = runtime.add_pair((i1, i2)).unwrap();
        let i8 = runtime.add_pair((i3, i4)).unwrap();
        let i9 = runtime.add_pair((i5, i6)).unwrap();

        let start = runtime.get_data_len();

        runtime.push_register(i7).unwrap();
        runtime.push_register(i8).unwrap();
        runtime.push_register(i9).unwrap();

        runtime.push_instruction(Instruction::MakeList, Some(3)).unwrap();

        runtime.make_list(3).unwrap();

        assert_eq!(runtime.get_list_len(start).unwrap(), 3);
        assert_eq!(runtime.get_list_item(start, 0).unwrap(), i7);
        assert_eq!(runtime.get_list_item(start, 1).unwrap(), i8);
        assert_eq!(runtime.get_list_item(start, 2).unwrap(), i9);

        assert_eq!(runtime.get_list_associations_len(start).unwrap(), 3);
        assert_eq!(runtime.get_list_association(start, 0).unwrap(), i7);
        assert_eq!(runtime.get_list_association(start, 1).unwrap(), i8);
        assert_eq!(runtime.get_list_association(start, 2).unwrap(), i9);
    }

    #[test]
    fn access() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_number(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_symbol("one").unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i4).unwrap();
        runtime.push_register(i5).unwrap();

        runtime.access(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), i2);
    }

    #[test]
    fn access_with_integer() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_number(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_number(0).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i4).unwrap();
        runtime.push_register(i5).unwrap();

        runtime.access(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), i3);
    }

    #[test]
    fn access_char_list_with_integer() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.start_char_list().unwrap();
        runtime.add_to_char_list('a').unwrap();
        runtime.add_to_char_list('b').unwrap();
        runtime.add_to_char_list('c').unwrap();
        let d1 = runtime.end_char_list().unwrap();
        let d2 = runtime.add_number(2).unwrap();
        let start = runtime.get_data_len();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.access(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), start);
        assert_eq!(runtime.get_char(start).unwrap(), 'c');
    }

    #[test]
    fn access_byte_list_with_integer() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.start_byte_list().unwrap();
        runtime.add_to_byte_list(10).unwrap();
        runtime.add_to_byte_list(20).unwrap();
        runtime.add_to_byte_list(30).unwrap();
        let d1 = runtime.end_byte_list().unwrap();
        let d2 = runtime.add_number(2).unwrap();
        let start = runtime.get_data_len();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.access(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), start);
        assert_eq!(runtime.get_byte(start).unwrap(), 30);
    }

    #[test]
    fn access_with_integer_out_of_bounds_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_number(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_number(10).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i4).unwrap();
        runtime.push_register(i5).unwrap();

        runtime.access(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn access_with_number_negative_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_number(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_number(-1).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i4).unwrap();
        runtime.push_register(i5).unwrap();

        runtime.access(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn access_non_list_on_left_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_number(10).unwrap();
        let i2 = runtime.add_symbol("one").unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i1).unwrap();
        runtime.push_register(i2).unwrap();

        runtime.access(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn access_non_symbol_on_right_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_number(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_expression(10).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i4).unwrap();
        runtime.push_register(i5).unwrap();

        runtime.access(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn access_no_refs_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_number(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let _i4 = runtime.end_list().unwrap();
        let _i5 = runtime.add_symbol("one").unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        let result = runtime.access(NO_CONTEXT);

        assert!(result.is_err());
    }

    #[test]
    fn access_with_non_existent_key() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_number(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_symbol("two").unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i4).unwrap();
        runtime.push_register(i5).unwrap();

        runtime.access(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }
}

#[cfg(test)]
mod ranges {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, Instruction, SimpleRuntimeData, NO_CONTEXT};

    #[test]
    fn access_with_integer() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_number(10).unwrap();
        let i2 = runtime.add_number(20).unwrap();
        let i3 = runtime.add_range(i1, i2).unwrap();
        let i4 = runtime.add_number(5).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i3).unwrap();
        runtime.push_register(i4).unwrap();

        runtime.access(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_number(runtime.get_register(0).unwrap()).unwrap(), 15);
    }

    #[test]
    fn access_with_integer_out_of_range() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_number(10).unwrap();
        let i2 = runtime.add_number(20).unwrap();
        let i3 = runtime.add_range(i1, i2).unwrap();
        let i4 = runtime.add_number(30).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i3).unwrap();
        runtime.push_register(i4).unwrap();

        runtime.access(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }
}

#[cfg(test)]
mod slice {
    use crate::simple::symbol_value;
    use crate::testing_utilites::{add_integer_list, add_links, add_list, add_pair, add_range};
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, SimpleRuntimeData, NO_CONTEXT};

    #[test]
    fn index_slice_of_list() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_integer_list(&mut runtime, 10);
        let d2 = runtime.add_number(1).unwrap();
        let d3 = runtime.add_number(4).unwrap();
        let d4 = runtime.add_range(d2, d3).unwrap();
        let d5 = runtime.add_slice(d1, d4).unwrap();
        let d6 = runtime.add_number(2).unwrap();

        runtime.push_register(d5).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.access(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_number(runtime.get_register(0).unwrap()).unwrap(), 40);
    }

    #[test]
    fn index_slice_of_char_list() {
        let mut runtime = SimpleRuntimeData::new();

        let chars = SimpleRuntimeData::parse_char_list("abcde");

        runtime.start_char_list().unwrap();
        for c in chars {
            runtime.add_to_char_list(c).unwrap();
        }
        let list = runtime.end_char_list().unwrap();

        let range = add_range(&mut runtime, 1, 3);
        let slice = runtime.add_slice(list, range).unwrap();
        let d6 = runtime.add_number(2).unwrap();

        runtime.push_register(slice).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.access(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_char(runtime.get_register(0).unwrap()).unwrap(), 'd');
    }

    #[test]
    fn index_slice_of_byte_list() {
        let mut runtime = SimpleRuntimeData::new();

        let bytes = SimpleRuntimeData::parse_byte_list("abcde");

        runtime.start_byte_list().unwrap();
        for b in bytes {
            runtime.add_to_byte_list(b).unwrap();
        }
        let list = runtime.end_byte_list().unwrap();

        let range = add_range(&mut runtime, 1, 3);
        let slice = runtime.add_slice(list, range).unwrap();
        let d6 = runtime.add_number(2).unwrap();

        runtime.push_register(slice).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.access(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_byte(runtime.get_register(0).unwrap()).unwrap(), 'd' as u8);
    }

    #[test]
    fn sym_index_slice_of_list() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_list(&mut runtime, 10);
        let d2 = runtime.add_number(1).unwrap();
        let d3 = runtime.add_number(4).unwrap();
        let d4 = runtime.add_range(d2, d3).unwrap();
        let d5 = runtime.add_slice(d1, d4).unwrap();
        let d6 = runtime.add_symbol("val4").unwrap();

        runtime.push_register(d5).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.access(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_number(runtime.get_register(0).unwrap()).unwrap(), 50);
    }

    #[test]
    fn sym_index_slice_of_list_sym_not_in_slice_before() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_list(&mut runtime, 10);
        let d2 = runtime.add_number(2).unwrap();
        let d3 = runtime.add_number(4).unwrap();
        let d4 = runtime.add_range(d2, d3).unwrap();
        let d5 = runtime.add_slice(d1, d4).unwrap();
        let d6 = runtime.add_symbol("val1").unwrap();

        runtime.push_register(d5).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.access(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn sym_index_slice_of_list_sym_not_in_slice() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_list(&mut runtime, 10);
        let d2 = runtime.add_number(2).unwrap();
        let d3 = runtime.add_number(4).unwrap();
        let d4 = runtime.add_range(d2, d3).unwrap();
        let d5 = runtime.add_slice(d1, d4).unwrap();
        let d6 = runtime.add_symbol("val8").unwrap();

        runtime.push_register(d5).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.access(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn index_slice_of_links() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_links(&mut runtime, 10, true);
        let d2 = runtime.add_number(2).unwrap();
        let d3 = runtime.add_number(8).unwrap();
        let d4 = runtime.add_range(d2, d3).unwrap();
        let d5 = runtime.add_slice(d1, d4).unwrap();
        let d6 = runtime.add_number(2).unwrap();

        runtime.push_register(d5).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.access(NO_CONTEXT).unwrap();

        let (left, right) = runtime.get_pair(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(runtime.get_symbol(left).unwrap(), symbol_value("val4"));
        assert_eq!(runtime.get_number(right).unwrap(), 5);
    }

    #[test]
    fn sym_index_slice_of_links() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_links(&mut runtime, 10, true);
        let d2 = runtime.add_number(2).unwrap();
        let d3 = runtime.add_number(5).unwrap();
        let d4 = runtime.add_range(d2, d3).unwrap();
        let d5 = runtime.add_slice(d1, d4).unwrap();
        let d6 = runtime.add_symbol("val4").unwrap();

        runtime.push_register(d5).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.access(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_number(runtime.get_register(0).unwrap()).unwrap(), 5);
    }

    #[test]
    fn sym_index_slice_of_links_not_found() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_links(&mut runtime, 10, true);
        let d2 = runtime.add_number(2).unwrap();
        let d3 = runtime.add_number(5).unwrap();
        let d4 = runtime.add_range(d2, d3).unwrap();
        let d5 = runtime.add_slice(d1, d4).unwrap();
        let d6 = runtime.add_symbol("val8").unwrap();

        runtime.push_register(d5).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.access(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn sym_index_slice_of_links_mixed() {
        let mut runtime = SimpleRuntimeData::new();

        let unit = runtime.add_unit().unwrap();
        let d1 = runtime.add_number(100).unwrap();
        let d2 = add_pair(&mut runtime, "pair", 200);
        let d3 = runtime.add_number(300).unwrap();
        let d4 = runtime.add_pair((d1, d3)).unwrap();

        let link1 = runtime.add_link(d1, unit, true).unwrap();
        let link2 = runtime.add_link(d4, link1, true).unwrap();
        let link3 = runtime.add_link(d2, link2, true).unwrap();
        let link4 = runtime.add_link(d3, link3, true).unwrap();
        let link5 = runtime.add_link(d4, link4, true).unwrap();

        let start = runtime.add_number(1).unwrap();
        let end = runtime.add_number(4).unwrap();
        let range = runtime.add_range(start, end).unwrap();
        let slice = runtime.add_slice(link5, range).unwrap();
        let sym = runtime.add_symbol("pair").unwrap();

        runtime.push_register(slice).unwrap();
        runtime.push_register(sym).unwrap();

        runtime.access(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_number(runtime.get_register(0).unwrap()).unwrap(), 200);
    }
}

#[cfg(test)]
mod link {
    use crate::simple::symbol_value;
    use crate::testing_utilites::add_links;
    use crate::{GarnishLangRuntimeData, GarnishRuntime, SimpleRuntimeData, NO_CONTEXT};

    #[test]
    fn index_prepend_link_with_integer() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_links(&mut runtime, 10, false);
        let d2 = runtime.add_number(3).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.access(NO_CONTEXT).unwrap();

        let (left, right) = runtime.get_pair(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(runtime.get_number(right).unwrap(), 4);
        assert_eq!(runtime.get_symbol(left).unwrap(), symbol_value("val3"));
    }

    #[test]
    fn index_append_link_with_integer() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_links(&mut runtime, 10, true);
        let d2 = runtime.add_number(3).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.access(NO_CONTEXT).unwrap();

        let (left, right) = runtime.get_pair(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(runtime.get_number(right).unwrap(), 4);
        assert_eq!(runtime.get_symbol(left).unwrap(), symbol_value("val3"));
    }

    #[test]
    fn index_prepend_link_with_symbol() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_links(&mut runtime, 10, false);
        let d2 = runtime.add_symbol("val2").unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.access(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_number(runtime.get_register(0).unwrap()).unwrap(), 3);
    }

    #[test]
    fn index_append_link_with_symbol() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_links(&mut runtime, 10, true);
        let d2 = runtime.add_symbol("val2").unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.access(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_number(runtime.get_register(0).unwrap()).unwrap(), 3);
    }
}
