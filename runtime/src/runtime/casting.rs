use crate::runtime::internals::{link_len, link_len_size};
use crate::runtime::list::{iterate_link_internal, iterate_link_internal_rev};
use crate::{get_range, next_two_raw_ref, push_unit, ExpressionDataType, GarnishLangRuntimeData, GarnishNumber, OrNumberError, RuntimeError, TypeConstants, GarnishLangRuntimeContext, Instruction};

pub(crate) fn type_cast<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<(), RuntimeError<Data::Error>> {
    let (right, left) = next_two_raw_ref(this)?;

    match (this.get_data_type(left)?, this.get_data_type(right)?) {
        (ExpressionDataType::Link, ExpressionDataType::Link) => {
            let (_, _, from_is_append) = this.get_link(left)?;
            let (_, _, to_is_append) = this.get_link(right)?;

            if from_is_append == to_is_append {
                // NoOp
                this.push_register(left)?;
            } else {
                // reverse link
                let len = link_len(this, left)?;
                let mut last = this.add_unit()?;

                if to_is_append {
                    iterate_link_start_end_internal(this, left, Data::Number::zero(), len, |this, addr, _index| {
                        last = this.add_link(addr, last, to_is_append)?;
                        Ok(false)
                    })?;
                } else {
                    iterate_link_start_end_internal_rev(this, left, Data::Number::zero(), len, |this, addr, _index| {
                        last = this.add_link(addr, last, to_is_append)?;
                        Ok(false)
                    })?;
                }

                this.push_register(last)?;
            }
        }
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
            primitive_cast(this, left, Data::get_number, Data::integer_to_char, Data::add_char)?;
        }
        (ExpressionDataType::Number, ExpressionDataType::Byte) => {
            primitive_cast(this, left, Data::get_number, Data::integer_to_byte, Data::add_byte)?;
        }
        (ExpressionDataType::Char, ExpressionDataType::Number) => {
            primitive_cast(this, left, Data::get_char, Data::char_to_integer, Data::add_number)?;
        }
        (ExpressionDataType::Char, ExpressionDataType::Byte) => {
            primitive_cast(this, left, Data::get_char, Data::char_to_byte, Data::add_byte)?;
        }
        (ExpressionDataType::Byte, ExpressionDataType::Number) => {
            primitive_cast(this, left, Data::get_byte, Data::byte_to_integer, Data::add_number)?;
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
        (ExpressionDataType::Link, ExpressionDataType::List) => {
            let len = link_len_size(this, left)?;
            list_from_link(this, left, Data::Number::zero(), Data::size_to_integer(len))?;
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
            list_from_char_list(this, left, Data::Number::zero(), Data::size_to_integer(len))?;
        }
        (ExpressionDataType::ByteList, ExpressionDataType::List) => {
            let len = this.get_byte_list_len(left)?;
            list_from_byte_list(this, left, Data::Number::zero(), Data::size_to_integer(len))?;
        }
        (ExpressionDataType::Slice, ExpressionDataType::List) => {
            let (value, range) = this.get_slice(left)?;
            let (start, end, _) = get_range(this, range)?;
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
                ExpressionDataType::Link => {
                    list_from_link(this, value, start, end.increment().or_num_err()?)?;
                }
                ExpressionDataType::CharList => {
                    list_from_char_list(this, value, start, end.increment().or_num_err()?)?;
                }
                ExpressionDataType::ByteList => {
                    list_from_byte_list(this, value, start, end.increment().or_num_err()?)?;
                }
                _ => push_unit(this)?,
            }
        }
        (ExpressionDataType::List, ExpressionDataType::Link) => {
            let len = Data::size_to_integer(this.get_list_len(left)?);
            create_link(this, right, Data::Number::zero(), len, |this, index| Ok(this.get_list_item(left, index)?))?;
        }
        (ExpressionDataType::Range, ExpressionDataType::Link) => {
            let (start, end, _) = get_range(this, left)?;
            create_link(this, right, start, end.increment().or_num_err()?, |this, index| {
                Ok(this.add_number(index)?)
            })?;
        }
        (ExpressionDataType::CharList, ExpressionDataType::Link) => {
            let len = this.get_char_list_len(left)?;
            create_link(this, right, Data::Number::zero(), Data::size_to_integer(len), |this, index| {
                let c = this.get_char_list_item(left, index)?;
                Ok(this.add_char(c)?)
            })?;
        }
        (ExpressionDataType::ByteList, ExpressionDataType::Link) => {
            let len = this.get_byte_list_len(left)?;
            create_link(this, right, Data::Number::zero(), Data::size_to_integer(len), |this, index| {
                let c = this.get_byte_list_item(left, index)?;
                Ok(this.add_byte(c)?)
            })?;
        }
        (ExpressionDataType::Slice, ExpressionDataType::Link) => {
            let (_, _, to_is_append) = this.get_link(right)?;
            let (value, range) = this.get_slice(left)?;
            let (start, end, _) = get_range(this, range)?;

            match this.get_data_type(value)? {
                ExpressionDataType::List => {
                    create_link(this, right, start, end.increment().or_num_err()?, |this, index| {
                        Ok(this.get_list_item(value, index)?)
                    })?;
                }
                ExpressionDataType::Link => {
                    let mut last = this.add_unit()?;

                    if to_is_append {
                        iterate_link_start_end_internal(this, value, start, end.increment().or_num_err()?, |this, addr, _index| {
                            last = this.add_link(addr, last, to_is_append)?;
                            Ok(false)
                        })?;
                    } else {
                        iterate_link_start_end_internal_rev(this, value, start, end.increment().or_num_err()?, |this, addr, _index| {
                            last = this.add_link(addr, last, to_is_append)?;
                            Ok(false)
                        })?;
                    }

                    this.push_register(last)?;
                }
                ExpressionDataType::CharList => {
                    create_link(this, right, start, end.increment().or_num_err()?, |this, index| {
                        let c = this.get_char_list_item(value, index)?;
                        Ok(this.add_char(c)?)
                    })?;
                }
                ExpressionDataType::ByteList => {
                    create_link(this, right, start, end.increment().or_num_err()?, |this, index| {
                        let c = this.get_byte_list_item(value, index)?;
                        Ok(this.add_byte(c)?)
                    })?;
                }
                _ => push_unit(this)?,
            }
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

pub(crate) fn create_link<Data: GarnishLangRuntimeData, GetFunc>(
    this: &mut Data,
    link_addr: Data::Size,
    start: Data::Number,
    end: Data::Number,
    mut get_item: GetFunc,
) -> Result<(), RuntimeError<Data::Error>>
where
    GetFunc: FnMut(&mut Data, Data::Number) -> Result<Data::Size, RuntimeError<Data::Error>>,
{
    let (_, _, is_append) = this.get_link(link_addr)?;
    let mut last = this.add_unit()?;

    if is_append {
        let mut i = start;

        while i < end {
            let addr = get_item(this, i)?;
            last = this.add_link(addr, last, is_append)?;
            i = i.increment().or_num_err()?;
        }
    } else {
        let mut i = end.decrement().or_num_err()?;

        while i >= start {
            let addr = get_item(this, i)?;
            last = this.add_link(addr, last, is_append)?;
            i = i.decrement().or_num_err()?;
        }
    }

    this.push_register(last)?;

    Ok(())
}

pub(crate) fn iterate_link_start_end_internal<Data: GarnishLangRuntimeData, Callback>(
    this: &mut Data,
    link_addr: Data::Size,
    start: Data::Number,
    end: Data::Number,
    mut func: Callback,
) -> Result<(), RuntimeError<Data::Error>>
where
    Callback: FnMut(&mut Data, Data::Size, Data::Number) -> Result<bool, RuntimeError<Data::Error>>,
{
    let mut skip = start;

    iterate_link_internal(this, link_addr, |this, addr, current_index| {
        if skip > Data::Number::zero() {
            skip = skip.decrement().or_num_err()?;
            return Ok(false);
        }

        if current_index >= end {
            return Ok(true);
        }

        func(this, addr, current_index)
    })
}

pub(crate) fn iterate_link_start_end_internal_rev<Data: GarnishLangRuntimeData, Callback>(
    this: &mut Data,
    link_addr: Data::Size,
    start: Data::Number,
    end: Data::Number,
    mut func: Callback,
) -> Result<(), RuntimeError<Data::Error>>
where
    Callback: FnMut(&mut Data, Data::Size, Data::Number) -> Result<bool, RuntimeError<Data::Error>>,
{
    let mut skip = start;

    iterate_link_internal_rev(this, link_addr, |this, addr, current_index| {
        if skip > Data::Number::zero() {
            skip = skip.decrement().or_num_err()?;
            return Ok(false);
        }

        if current_index >= end {
            return Ok(true);
        }

        func(this, addr, current_index)
    })
}

pub(crate) fn list_from_link<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    link_addr: Data::Size,
    start: Data::Number,
    end: Data::Number,
) -> Result<(), RuntimeError<Data::Error>> {
    let len = link_len_size(this, link_addr)?;
    this.start_list(len)?;
    let mut skip = start;

    iterate_link_internal(this, link_addr, |this, addr, current_index| {
        if skip > Data::Number::zero() {
            skip = skip.decrement().or_num_err()?;
            return Ok(false);
        }

        if current_index >= end {
            return Ok(true);
        }

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
        Ok(false)
    })?;

    this.end_list().and_then(|r| this.push_register(r))?;

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

#[cfg(test)]
mod deferring {
    use crate::runtime::GarnishRuntime;
    use crate::testing_utilites::{deferred_op};

    #[test]
    fn type_cast() {
        deferred_op(|runtime, context| {
            runtime.type_cast(Some(context)).unwrap();
        })
    }
}

#[cfg(test)]
mod simple {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, SimpleRuntimeData, NO_CONTEXT};

    #[test]
    fn no_op_cast_expression() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_expression(10).unwrap();
        let d2 = runtime.add_expression(10).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_expression(runtime.get_register(0).unwrap()).unwrap(), 10);
    }

    #[test]
    fn cast_to_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let int = runtime.add_number(10).unwrap();
        let unit = runtime.add_unit().unwrap();

        runtime.push_register(int).unwrap();
        runtime.push_register(unit).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn cast_to_true() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_number(10).unwrap();
        let d2 = runtime.add_true().unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn cast_unit_to_true() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_unit().unwrap();
        let d2 = runtime.add_true().unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn cast_false_to_true() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_false().unwrap();
        let d2 = runtime.add_true().unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn cast_to_false() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_number(10).unwrap();
        let d2 = runtime.add_false().unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn cast_unit_to_false() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_unit().unwrap();
        let d2 = runtime.add_false().unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn cast_true_to_false() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_true().unwrap();
        let d2 = runtime.add_false().unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }
}

#[cfg(test)]
mod primitive {
    use crate::testing_utilites::add_char_list;
    use crate::{runtime::GarnishRuntime, GarnishLangRuntimeData, SimpleRuntimeData, NO_CONTEXT};

    #[test]
    fn integer_to_char() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_number('a' as i32).unwrap();
        let d2 = runtime.add_char('\0').unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let expected = SimpleRuntimeData::integer_to_char('a' as i32).unwrap();

        assert_eq!(runtime.get_char(runtime.get_register(0).unwrap()).unwrap(), expected);
    }

    #[test]
    fn integer_to_byte() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_number(10).unwrap();
        let d2 = runtime.add_byte(0).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let expected = SimpleRuntimeData::integer_to_byte(10).unwrap();

        assert_eq!(runtime.get_byte(runtime.get_register(0).unwrap()).unwrap(), expected);
    }

    #[test]
    fn char_to_integer() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_char('a').unwrap();
        let d2 = runtime.add_number(0).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let expected = SimpleRuntimeData::char_to_integer('a').unwrap();

        assert_eq!(runtime.get_number(runtime.get_register(0).unwrap()).unwrap(), expected);
    }

    #[test]
    fn char_to_byte() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_char('a').unwrap();
        let d2 = runtime.add_byte(0).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let expected = SimpleRuntimeData::char_to_byte('a').unwrap();

        assert_eq!(runtime.get_byte(runtime.get_register(0).unwrap()).unwrap(), expected);
    }

    #[test]
    fn byte_to_integer() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_byte('a' as u8).unwrap();
        let d2 = runtime.add_number(0).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let expected = SimpleRuntimeData::byte_to_integer('a' as u8).unwrap();

        assert_eq!(runtime.get_number(runtime.get_register(0).unwrap()).unwrap(), expected);
    }

    #[test]
    fn byte_to_char() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_byte('a' as u8).unwrap();
        let d2 = runtime.add_char('a').unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let expected = SimpleRuntimeData::byte_to_char('a' as u8).unwrap();

        assert_eq!(runtime.get_char(runtime.get_register(0).unwrap()).unwrap(), expected);
    }

    #[test]
    fn char_list_to_byte() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_char_list(&mut runtime, "100");
        let d2 = runtime.add_byte(0).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_byte(runtime.get_register(0).unwrap()).unwrap(), 100);
    }

    #[test]
    fn char_list_to_char() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_char_list(&mut runtime, "c");
        let d2 = runtime.add_char('a').unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_char(runtime.get_register(0).unwrap()).unwrap(), 'c');
    }

    #[test]
    fn char_list_to_number() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_char_list(&mut runtime, "100");
        let d2 = runtime.add_number(0).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_number(runtime.get_register(0).unwrap()).unwrap(), 100);
    }
}

#[cfg(test)]
mod lists {
    use crate::testing_utilites::{add_byte_list, add_char_list, add_links_with_start, add_list_with_start, add_range};
    use crate::{runtime::GarnishRuntime, GarnishLangRuntimeData, SimpleRuntimeData, NO_CONTEXT};
    use crate::simple::symbol_value;

    #[test]
    fn link_to_list() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_links_with_start(&mut runtime, 10, true, 20);
        let list = add_list_with_start(&mut runtime, 1, 0);

        runtime.push_register(d1).unwrap();
        runtime.push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();
        let len = runtime.get_list_len(addr).unwrap();
        assert_eq!(len, 10);

        for i in 0..10 {
            let item_addr = runtime.get_list_item(addr, i).unwrap();
            let (left, right) = runtime.get_pair(item_addr).unwrap();
            let s = symbol_value(format!("val{}", 20 + i).as_ref());
            assert_eq!(runtime.get_symbol(left).unwrap(), s);
            assert_eq!(runtime.get_number(right).unwrap(), 20 + i);

            let association = runtime.get_list_item_with_symbol(addr, s).unwrap().unwrap();
            assert_eq!(runtime.get_number(association).unwrap(), 20 + i)
        }
    }

    #[test]
    fn range_to_list() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_range(&mut runtime, 10, 20);
        let list = add_list_with_start(&mut runtime, 1, 0);

        runtime.push_register(d1).unwrap();
        runtime.push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();
        let len = runtime.get_list_len(addr).unwrap();
        assert_eq!(len, 11);

        for i in 0..10 {
            let item_addr = runtime.get_list_item(addr, i).unwrap();
            assert_eq!(runtime.get_number(item_addr).unwrap(), 10 + i);
        }
    }

    #[test]
    fn char_list_to_list() {
        let mut runtime = SimpleRuntimeData::new();

        let input = "characters";
        let d1 = add_char_list(&mut runtime, input);
        let list = add_list_with_start(&mut runtime, 1, 0);

        runtime.push_register(d1).unwrap();
        runtime.push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();
        let expected = SimpleRuntimeData::parse_char_list(input);
        let mut result = vec![];

        for i in 0..input.len() {
            let item_addr = runtime.get_list_item(addr, i as i32).unwrap();
            let item = runtime.get_char(item_addr).unwrap();
            result.push(item);
        }

        assert_eq!(result, expected);
    }

    #[test]
    fn byte_list_to_list() {
        let mut runtime = SimpleRuntimeData::new();

        let input = "characters";
        let d1 = add_byte_list(&mut runtime, input);
        let list = add_list_with_start(&mut runtime, 1, 0);

        runtime.push_register(d1).unwrap();
        runtime.push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();
        let expected = SimpleRuntimeData::parse_byte_list(input);
        let mut result = vec![];

        for i in 0..input.len() {
            let item_addr = runtime.get_list_item(addr, i as i32).unwrap();
            let item = runtime.get_byte(item_addr).unwrap();
            result.push(item);
        }

        assert_eq!(result, expected);
    }

    #[test]
    fn slice_of_link_to_list() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_links_with_start(&mut runtime, 10, true, 20);
        let d2 = add_range(&mut runtime, 2, 7);
        let d3 = runtime.add_slice(d1, d2).unwrap();
        let list = add_list_with_start(&mut runtime, 1, 0);

        runtime.push_register(d3).unwrap();
        runtime.push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();
        let len = runtime.get_list_len(addr).unwrap();
        assert_eq!(len, 6);

        for i in 0..6 {
            let value = 22 + i;
            let item_addr = runtime.get_list_item(addr, i).unwrap();
            let (left, right) = runtime.get_pair(item_addr).unwrap();
            let s = symbol_value(format!("val{}", value).as_ref());
            assert_eq!(runtime.get_symbol(left).unwrap(), s);
            assert_eq!(runtime.get_number(right).unwrap(), value);

            let association = runtime.get_list_item_with_symbol(addr, s).unwrap().unwrap();
            assert_eq!(runtime.get_number(association).unwrap(), value)
        }
    }

    #[test]
    fn slice_of_list_to_list() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_list_with_start(&mut runtime, 10, 20);
        let d2 = add_range(&mut runtime, 2, 7);
        let d3 = runtime.add_slice(d1, d2).unwrap();
        let list = add_list_with_start(&mut runtime, 1, 0);

        runtime.push_register(d3).unwrap();
        runtime.push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();
        let len = runtime.get_list_len(addr).unwrap();
        assert_eq!(len, 6);

        for i in 0..6 {
            let value = 22 + i;
            let item_addr = runtime.get_list_item(addr, i).unwrap();
            let (left, right) = runtime.get_pair(item_addr).unwrap();
            let s = symbol_value(format!("val{}", value).as_ref());
            assert_eq!(runtime.get_symbol(left).unwrap(), s);
            assert_eq!(runtime.get_number(right).unwrap(), value);

            let association = runtime.get_list_item_with_symbol(addr, s).unwrap().unwrap();
            assert_eq!(runtime.get_number(association).unwrap(), value)
        }
    }

    #[test]
    fn slice_of_char_list_to_list() {
        let mut runtime = SimpleRuntimeData::new();

        let input = "characters";
        let d1 = add_char_list(&mut runtime, input);
        let d2 = add_range(&mut runtime, 2, 7); // "aracte"
        let d3 = runtime.add_slice(d1, d2).unwrap();
        let list = add_list_with_start(&mut runtime, 1, 0);

        runtime.push_register(d3).unwrap();
        runtime.push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();
        let expected: Vec<char> = SimpleRuntimeData::parse_char_list(input).iter().skip(2).take(6).map(|c| *c).collect();
        let mut result = vec![];

        for i in 0..expected.len() {
            let item_addr = runtime.get_list_item(addr, i as i32).unwrap();
            let item = runtime.get_char(item_addr).unwrap();
            result.push(item);
        }

        assert_eq!(result, expected);
    }

    #[test]
    fn slice_of_byte_list_to_list() {
        let mut runtime = SimpleRuntimeData::new();

        let input = "characters";
        let d1 = add_byte_list(&mut runtime, input);
        let d2 = add_range(&mut runtime, 2, 7);
        let d3 = runtime.add_slice(d1, d2).unwrap();
        let list = add_list_with_start(&mut runtime, 1, 0);

        runtime.push_register(d3).unwrap();
        runtime.push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();
        let expected: Vec<u8> = SimpleRuntimeData::parse_byte_list(input).iter().skip(2).take(6).map(|c| *c).collect();
        let mut result = vec![];

        for i in 0..expected.len() {
            let item_addr = runtime.get_list_item(addr, i as i32).unwrap();
            let item = runtime.get_byte(item_addr).unwrap();
            result.push(item);
        }

        assert_eq!(result, expected);
    }
}

#[cfg(test)]
mod links {
    use crate::runtime::internals::{link_len, link_len_size};
    use crate::testing_utilites::{add_byte_list, add_char_list, add_links_with_start, add_list_with_start, add_range};
    use crate::{iterate_link, runtime::GarnishRuntime, GarnishLangRuntimeData, SimpleRuntimeData, NO_CONTEXT};
    use crate::simple::symbol_value;

    #[test]
    fn list_to_link_append() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_list_with_start(&mut runtime, 10, 20);
        let list = add_links_with_start(&mut runtime, 1, true, 0);

        runtime.push_register(d1).unwrap();
        runtime.push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();
        let len = link_len_size(&mut runtime, addr).unwrap();
        assert_eq!(len, 10);

        iterate_link(&mut runtime, addr, |runtime, addr, current_index| {
            let (left, right) = runtime.get_pair(addr).unwrap();
            let s = symbol_value(format!("val{}", 20 + current_index).as_ref());

            assert_eq!(runtime.get_symbol(left).unwrap(), s);
            assert_eq!(runtime.get_number(right).unwrap(), 20 + current_index);
            Ok(false)
        })
        .unwrap();
    }

    #[test]
    fn list_to_link_prepend() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_list_with_start(&mut runtime, 10, 20);
        let list = add_links_with_start(&mut runtime, 1, false, 0);

        runtime.push_register(d1).unwrap();
        runtime.push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();
        let len = link_len_size(&mut runtime, addr).unwrap();
        assert_eq!(len, 10);

        iterate_link(&mut runtime, addr, |runtime, addr, current_index| {
            let (left, right) = runtime.get_pair(addr).unwrap();
            let s = symbol_value(format!("val{}", 20 + current_index).as_ref());

            assert_eq!(runtime.get_symbol(left).unwrap(), s);
            assert_eq!(runtime.get_number(right).unwrap(), 20 + current_index);
            Ok(false)
        })
        .unwrap();
    }

    #[test]
    fn range_to_link_append() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_range(&mut runtime, 0, 10);
        let list = add_links_with_start(&mut runtime, 1, true, 0);

        runtime.push_register(d1).unwrap();
        runtime.push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();
        let len = link_len_size(&mut runtime, addr).unwrap();
        assert_eq!(len, 11);

        iterate_link(&mut runtime, addr, |runtime, addr, current_index| {
            assert_eq!(runtime.get_number(addr).unwrap(), current_index);
            Ok(false)
        })
        .unwrap();
    }

    #[test]
    fn range_to_link_prepend() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_range(&mut runtime, 0, 10);
        let list = add_links_with_start(&mut runtime, 1, false, 0);

        runtime.push_register(d1).unwrap();
        runtime.push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();
        let len = link_len_size(&mut runtime, addr).unwrap();
        assert_eq!(len, 11);

        iterate_link(&mut runtime, addr, |runtime, addr, current_index| {
            assert_eq!(runtime.get_number(addr).unwrap(), current_index);
            Ok(false)
        })
        .unwrap();
    }

    #[test]
    fn char_list_to_link_append() {
        let mut runtime = SimpleRuntimeData::new();

        let input = "characters";
        let d1 = add_char_list(&mut runtime, input);
        let list = add_links_with_start(&mut runtime, 1, true, 0);

        runtime.push_register(d1).unwrap();
        runtime.push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();
        let len = link_len_size(&mut runtime, addr).unwrap();
        assert_eq!(len, input.len());

        iterate_link(&mut runtime, addr, |runtime, addr, current_index| {
            assert_eq!(runtime.get_char(addr).unwrap(), input.chars().nth(current_index as usize).unwrap());
            Ok(false)
        })
        .unwrap();
    }

    #[test]
    fn char_list_to_link_prepend() {
        let mut runtime = SimpleRuntimeData::new();

        let input = "characters";
        let d1 = add_char_list(&mut runtime, input);
        let list = add_links_with_start(&mut runtime, 1, false, 0);

        runtime.push_register(d1).unwrap();
        runtime.push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();
        let len = link_len_size(&mut runtime, addr).unwrap();
        assert_eq!(len, input.len());

        iterate_link(&mut runtime, addr, |runtime, addr, current_index| {
            assert_eq!(runtime.get_char(addr).unwrap(), input.chars().nth(current_index as usize).unwrap());
            Ok(false)
        })
        .unwrap();
    }

    #[test]
    fn byte_list_to_link_append() {
        let mut runtime = SimpleRuntimeData::new();

        let input = "characters";
        let d1 = add_byte_list(&mut runtime, input);
        let list = add_links_with_start(&mut runtime, 1, true, 0);

        runtime.push_register(d1).unwrap();
        runtime.push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();
        let len = link_len_size(&mut runtime, addr).unwrap();
        assert_eq!(len, input.len());

        iterate_link(&mut runtime, addr, |runtime, addr, current_index| {
            assert_eq!(runtime.get_byte(addr).unwrap(), input.chars().nth(current_index as usize).unwrap() as u8);
            Ok(false)
        })
        .unwrap();
    }

    #[test]
    fn byte_list_to_link_prepend() {
        let mut runtime = SimpleRuntimeData::new();

        let input = "characters";
        let d1 = add_byte_list(&mut runtime, input);
        let list = add_links_with_start(&mut runtime, 1, false, 0);

        runtime.push_register(d1).unwrap();
        runtime.push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();
        let len = link_len_size(&mut runtime, addr).unwrap();
        assert_eq!(len, input.len());

        iterate_link(&mut runtime, addr, |runtime, addr, current_index| {
            assert_eq!(runtime.get_byte(addr).unwrap(), input.chars().nth(current_index as usize).unwrap() as u8);
            Ok(false)
        })
        .unwrap();
    }

    #[test]
    fn link_prepend_to_link_append() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_links_with_start(&mut runtime, 10, false, 20);
        let list = add_links_with_start(&mut runtime, 1, true, 0);

        runtime.push_register(d1).unwrap();
        runtime.push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();
        let len = link_len_size(&mut runtime, addr).unwrap();
        assert_eq!(len, 10);

        iterate_link(&mut runtime, addr, |runtime, addr, current_index| {
            let (left, right) = runtime.get_pair(addr).unwrap();
            let s = symbol_value(format!("val{}", 20 + current_index).as_ref());

            assert_eq!(runtime.get_symbol(left).unwrap(), s);
            assert_eq!(runtime.get_number(right).unwrap(), 20 + current_index);
            Ok(false)
        })
        .unwrap();
    }

    #[test]
    fn link_append_to_link_prepend() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_links_with_start(&mut runtime, 10, true, 20);
        let list = add_links_with_start(&mut runtime, 1, false, 0);

        runtime.push_register(d1).unwrap();
        runtime.push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();
        let len = link_len_size(&mut runtime, addr).unwrap();
        assert_eq!(len, 10);

        iterate_link(&mut runtime, addr, |runtime, addr, current_index| {
            let (left, right) = runtime.get_pair(addr).unwrap();
            let s = symbol_value(format!("val{}", 20 + current_index).as_ref());

            assert_eq!(runtime.get_symbol(left).unwrap(), s);
            assert_eq!(runtime.get_number(right).unwrap(), 20 + current_index);
            Ok(false)
        })
        .unwrap();
    }

    #[test]
    fn slice_of_append_link_to_prepend_link() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_links_with_start(&mut runtime, 10, true, 20);
        let d2 = add_range(&mut runtime, 2, 7);
        let d3 = runtime.add_slice(d1, d2).unwrap();
        let list = add_links_with_start(&mut runtime, 1, false, 0);

        runtime.push_register(d3).unwrap();
        runtime.push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();
        let len = link_len(&mut runtime, addr).unwrap();
        assert_eq!(len, 6);

        iterate_link(&mut runtime, addr, |runtime, addr, current_index| {
            let value = 22 + current_index;
            let (left, right) = runtime.get_pair(addr).unwrap();
            let s = symbol_value(format!("val{}", value).as_ref());

            assert_eq!(runtime.get_symbol(left).unwrap(), s);
            assert_eq!(runtime.get_number(right).unwrap(), value);
            Ok(false)
        })
        .unwrap();
    }

    #[test]
    fn slice_of_prepend_link_to_append_link() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_links_with_start(&mut runtime, 10, false, 20);
        let d2 = add_range(&mut runtime, 2, 7);
        let d3 = runtime.add_slice(d1, d2).unwrap();
        let list = add_links_with_start(&mut runtime, 1, true, 0);

        runtime.push_register(d3).unwrap();
        runtime.push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();
        let len = link_len(&mut runtime, addr).unwrap();
        assert_eq!(len, 6);

        iterate_link(&mut runtime, addr, |runtime, addr, current_index| {
            let value = 22 + current_index;
            let (left, right) = runtime.get_pair(addr).unwrap();
            let s = symbol_value(format!("val{}", value).as_ref());

            assert_eq!(runtime.get_symbol(left).unwrap(), s);
            assert_eq!(runtime.get_number(right).unwrap(), value);
            Ok(false)
        })
        .unwrap();
    }

    #[test]
    fn slice_of_list_to_append_link() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_list_with_start(&mut runtime, 10, 20);
        let d2 = add_range(&mut runtime, 2, 7);
        let d3 = runtime.add_slice(d1, d2).unwrap();
        let list = add_links_with_start(&mut runtime, 1, true, 0);

        runtime.push_register(d3).unwrap();
        runtime.push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();
        let len = link_len(&mut runtime, addr).unwrap();
        assert_eq!(len, 6);

        iterate_link(&mut runtime, addr, |runtime, addr, current_index| {
            let value = 22 + current_index;
            let (left, right) = runtime.get_pair(addr).unwrap();
            let s = symbol_value(format!("val{}", value).as_ref());

            assert_eq!(runtime.get_symbol(left).unwrap(), s);
            assert_eq!(runtime.get_number(right).unwrap(), value);
            Ok(false)
        })
        .unwrap();
    }

    #[test]
    fn slice_of_list_to_prepend_link() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_list_with_start(&mut runtime, 10, 20);
        let d2 = add_range(&mut runtime, 2, 7);
        let d3 = runtime.add_slice(d1, d2).unwrap();
        let list = add_links_with_start(&mut runtime, 1, false, 0);

        runtime.push_register(d3).unwrap();
        runtime.push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();
        let len = link_len(&mut runtime, addr).unwrap();
        assert_eq!(len, 6);

        iterate_link(&mut runtime, addr, |runtime, addr, current_index| {
            let value = 22 + current_index;
            let (left, right) = runtime.get_pair(addr).unwrap();
            let s = symbol_value(format!("val{}", value).as_ref());

            assert_eq!(runtime.get_symbol(left).unwrap(), s);
            assert_eq!(runtime.get_number(right).unwrap(), value);
            Ok(false)
        })
        .unwrap();
    }

    #[test]
    fn slice_of_char_list_to_link_append() {
        let mut runtime = SimpleRuntimeData::new();

        let input = "characters";
        let d1 = add_char_list(&mut runtime, input);
        let d2 = add_range(&mut runtime, 2, 7); // "aracte"
        let d3 = runtime.add_slice(d1, d2).unwrap();
        let list = add_links_with_start(&mut runtime, 1, true, 0);

        runtime.push_register(d3).unwrap();
        runtime.push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();

        iterate_link(&mut runtime, addr, |runtime, addr, current_index| {
            let i = 2 + current_index as usize;
            assert_eq!(runtime.get_char(addr).unwrap(), input.chars().nth(i).unwrap());
            Ok(false)
        })
        .unwrap();
    }

    #[test]
    fn slice_of_char_list_to_link_prepend() {
        let mut runtime = SimpleRuntimeData::new();

        let input = "characters";
        let d1 = add_char_list(&mut runtime, input);
        let d2 = add_range(&mut runtime, 2, 7); // "aracte"
        let d3 = runtime.add_slice(d1, d2).unwrap();
        let list = add_links_with_start(&mut runtime, 1, false, 0);

        runtime.push_register(d3).unwrap();
        runtime.push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();

        iterate_link(&mut runtime, addr, |runtime, addr, current_index| {
            let i = 2 + current_index as usize;
            assert_eq!(runtime.get_char(addr).unwrap(), input.chars().nth(i).unwrap());
            Ok(false)
        })
        .unwrap();
    }

    #[test]
    fn slice_of_byte_list_to_link_append() {
        let mut runtime = SimpleRuntimeData::new();

        let input = "characters";
        let d1 = add_byte_list(&mut runtime, input);
        let d2 = add_range(&mut runtime, 2, 7);
        let d3 = runtime.add_slice(d1, d2).unwrap();
        let list = add_links_with_start(&mut runtime, 1, true, 0);

        runtime.push_register(d3).unwrap();
        runtime.push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();

        iterate_link(&mut runtime, addr, |runtime, addr, current_index| {
            let i = 2 + current_index as usize;
            assert_eq!(runtime.get_byte(addr).unwrap(), input.chars().nth(i).unwrap() as u8);
            Ok(false)
        })
        .unwrap();
    }

    #[test]
    fn slice_of_byte_list_to_link_prepend() {
        let mut runtime = SimpleRuntimeData::new();

        let input = "characters";
        let d1 = add_byte_list(&mut runtime, input);
        let d2 = add_range(&mut runtime, 2, 7);
        let d3 = runtime.add_slice(d1, d2).unwrap();
        let list = add_links_with_start(&mut runtime, 1, false, 0);

        runtime.push_register(d3).unwrap();
        runtime.push_register(list).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();

        iterate_link(&mut runtime, addr, |runtime, addr, current_index| {
            let i = 2 + current_index as usize;
            assert_eq!(runtime.get_byte(addr).unwrap(), input.chars().nth(i).unwrap() as u8);
            Ok(false)
        })
        .unwrap();
    }
}

#[cfg(test)]
mod deferred {
    use crate::{ExpressionDataType, GarnishLangRuntimeData, GarnishRuntime, NO_CONTEXT, SimpleRuntimeData};

    #[test]
    fn char_list() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_unit().unwrap();

        runtime.start_char_list().unwrap();
        let s = runtime.end_char_list().unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(s).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();
        let len = runtime.get_char_list_len(addr).unwrap();

        let expected = "()";
        let mut chars = String::new();

        for i in 0..len {
            let c = runtime.get_char_list_item(addr, i as i32).unwrap();
            chars.push(c);
        }

        assert_eq!(chars, expected);
    }

    #[test]
    fn byte_list() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_unit().unwrap();

        runtime.start_byte_list().unwrap();
        let s = runtime.end_byte_list().unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(s).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();
        assert_eq!(runtime.get_data_type(addr).unwrap(), ExpressionDataType::ByteList);
    }

    #[test]
    fn symbols() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_unit().unwrap();

        let s = runtime.add_symbol("sym").unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(s).unwrap();

        runtime.type_cast(NO_CONTEXT).unwrap();

        let addr = runtime.get_register(0).unwrap();
        assert_eq!(runtime.get_data_type(addr).unwrap(), ExpressionDataType::Symbol);
    }
}
