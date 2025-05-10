use crate::{GarnishDataType, GarnishData, TypeConstants};

pub type CloneHandler<Data> = fn(<Data as GarnishData>::Size, &Data, &mut Data) -> Result<<Data as GarnishData>::Size, <Data as GarnishData>::Error>;

pub fn clone_data<Data: GarnishData>(
    data_addr: Data::Size,
    from: &Data,
    to: &mut Data,
) -> Result<Data::Size, Data::Error> {
    clone_data_with_handlers_internal(
        data_addr,
        from,
        to,
        None,
        None,
    )
}

pub fn clone_data_with_custom_handler<Data: GarnishData>(
    data_addr: Data::Size,
    from: &Data,
    to: &mut Data,
    custom_handler: CloneHandler<Data>,
) -> Result<Data::Size, Data::Error> {
    clone_data_with_handlers_internal(
        data_addr,
        from,
        to,
        Some(custom_handler),
        None
    )
}

pub fn clone_data_with_invalid_handler<Data: GarnishData>(
    data_addr: Data::Size,
    from: &Data,
    to: &mut Data,
    invalid_handler: CloneHandler<Data>,
) -> Result<Data::Size, Data::Error> {
    clone_data_with_handlers_internal(
        data_addr,
        from,
        to,
        None,
        Some(invalid_handler),
    )
}

pub fn clone_data_with_handlers<Data: GarnishData>(
    data_addr: Data::Size,
    from: &Data,
    to: &mut Data,
    custom_handler: CloneHandler<Data>,
    invalid_handler: CloneHandler<Data>, // to be implemented
) -> Result<Data::Size, Data::Error> {
    clone_data_with_handlers_internal(
        data_addr,
        from,
        to,
        Some(custom_handler),
        Some(invalid_handler),
    )
}

fn clone_data_with_handlers_internal<Data: GarnishData>(
    data_addr: Data::Size,
    from: &Data,
    to: &mut Data,
    custom_handler: Option<CloneHandler<Data>>,
    invalid_handler: Option<CloneHandler<Data>>, // to be implemented
) -> Result<Data::Size, Data::Error> {
    match from.get_data_type(data_addr.clone())? {
        GarnishDataType::Invalid => match invalid_handler {
            None => to.add_unit(),
            Some(handler) => handler(data_addr.clone(), from, to)
        }
        GarnishDataType::Custom => match custom_handler {
            None => to.add_unit(),
            Some(handler) => handler(data_addr.clone(), from, to)
        }
        GarnishDataType::Unit => to.add_unit(),
        GarnishDataType::Number => to.add_number(from.get_number(data_addr.clone())?),
        GarnishDataType::Type => to.add_type(from.get_type(data_addr.clone())?),
        GarnishDataType::Char => to.add_char(from.get_char(data_addr.clone())?),
        GarnishDataType::CharList => {
            let len = from.get_char_list_len(data_addr.clone())?;
            let iter =
                Data::make_number_iterator_range(Data::Number::zero(), Data::size_to_number(len));
            to.start_char_list()?;
            for i in iter {
                to.add_to_char_list(from.get_char_list_item(data_addr.clone(), i)?)?;
            }

            to.end_char_list()
        }
        GarnishDataType::Byte => to.add_byte(from.get_byte(data_addr.clone())?),
        GarnishDataType::ByteList => {
            let len = from.get_byte_list_len(data_addr.clone())?;
            let iter =
                Data::make_number_iterator_range(Data::Number::zero(), Data::size_to_number(len));
            to.start_byte_list()?;
            for i in iter {
                to.add_to_byte_list(from.get_byte_list_item(data_addr.clone(), i)?)?;
            }

            to.end_byte_list()
        }
        GarnishDataType::Symbol => to.add_symbol(from.get_symbol(data_addr.clone())?),
        GarnishDataType::SymbolList => {
            let mut iter = from.get_symbol_list_iter(data_addr.clone());

            match iter.next() {
                None => to.add_unit(),
                Some(first) => {
                    let sym = from.get_symbol_list_item(data_addr.clone(), first)?;
                    let sym1_addr = to.add_symbol(sym)?;
                    
                    match iter.next() {
                        None => Ok(sym1_addr),
                        Some(second) => {
                            let sym = from.get_symbol_list_item(data_addr.clone(), second)?;
                            let sym2_addr = to.add_symbol(sym)?;
                            let mut previous = to.merge_to_symbol_list(sym1_addr, sym2_addr)?;

                            while let Some(addr) = iter.next() {
                                let sym = from.get_symbol_list_item(data_addr.clone(), addr)?;
                                let sym_addr = to.add_symbol(sym)?;
                                let sym_list = to.merge_to_symbol_list(previous, sym_addr)?;
                                previous = sym_list;
                            }

                            Ok(previous)
                        }
                    }
                }
            }
        },
        GarnishDataType::Pair => from.get_pair(data_addr.clone()).and_then(|(left, right)| {
            let to_left = clone_data_with_handlers_internal(left, from, to, custom_handler, invalid_handler)?;
            let to_right = clone_data_with_handlers_internal(right, from, to, custom_handler, invalid_handler)?;
            to.add_pair((to_left, to_right))
        }),
        GarnishDataType::Range => from.get_range(data_addr.clone()).and_then(|(left, right)| {
            let to_left = clone_data_with_handlers_internal(left, from, to, custom_handler, invalid_handler)?;
            let to_right = clone_data_with_handlers_internal(right, from, to, custom_handler, invalid_handler)?;
            to.add_range(to_left, to_right)
        }),
        GarnishDataType::Concatenation => {
            from.get_concatenation(data_addr.clone()).and_then(|(left, right)| {
                let to_left = clone_data_with_handlers_internal(left, from, to, custom_handler, invalid_handler)?;
                let to_right = clone_data_with_handlers_internal(right, from, to, custom_handler, invalid_handler)?;
                to.add_concatenation(to_left, to_right)
            })
        }
        GarnishDataType::Slice => from.get_slice(data_addr.clone()).and_then(|(left, right)| {
            let to_left = clone_data_with_handlers_internal(left, from, to, custom_handler, invalid_handler)?;
            let to_right = clone_data_with_handlers_internal(right, from, to, custom_handler, invalid_handler)?;
            to.add_slice(to_left, to_right)
        }),
        GarnishDataType::List => {
            let len = from.get_list_len(data_addr.clone())?;
            let iter =
                Data::make_number_iterator_range(Data::Number::zero(), Data::size_to_number(len.clone()));

            let mut items = vec![];
            for i in iter {
                let addr = from
                    .get_list_item(data_addr.clone(), i)
                    .and_then(|addr| clone_data_with_handlers_internal(addr, from, to, custom_handler, invalid_handler))?;
                let is_association = match to.get_data_type(addr.clone())? {
                    GarnishDataType::Pair => {
                        let (left, _right) = to.get_pair(addr.clone())?;
                        match to.get_data_type(left)? {
                            GarnishDataType::Symbol => true,
                            _ => false
                        }
                    }
                    _ => false
                };
                items.push((addr, is_association));
            }

            to.start_list(len.clone())?;
            for (addr, is_association) in items {
                to.add_to_list(addr, is_association)?;
            }
            to.end_list()
        }
        GarnishDataType::Expression => to.add_expression(from.get_expression(data_addr.clone())?),
        GarnishDataType::External => to.add_external(from.get_external(data_addr.clone())?),
        GarnishDataType::True => to.add_true(),
        GarnishDataType::False => to.add_false(),
    }
}