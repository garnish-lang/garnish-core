use crate::{GarnishData, GarnishDataType, TypeConstants};

pub type CloneHandler<Data> = fn(<Data as GarnishData>::Size, &Data, &mut Data) -> Result<<Data as GarnishData>::Size, <Data as GarnishData>::Error>;

pub fn clone_data<Data: GarnishData>(data_addr: Data::Size, from: &Data, to: &mut Data) -> Result<Data::Size, Data::Error> {
    clone_data_with_handlers_internal(data_addr, from, to, None, None)
}

pub fn clone_data_with_custom_handler<Data: GarnishData>(
    data_addr: Data::Size,
    from: &Data,
    to: &mut Data,
    custom_handler: CloneHandler<Data>,
) -> Result<Data::Size, Data::Error> {
    clone_data_with_handlers_internal(data_addr, from, to, Some(custom_handler), None)
}

pub fn clone_data_with_invalid_handler<Data: GarnishData>(
    data_addr: Data::Size,
    from: &Data,
    to: &mut Data,
    invalid_handler: CloneHandler<Data>,
) -> Result<Data::Size, Data::Error> {
    clone_data_with_handlers_internal(data_addr, from, to, None, Some(invalid_handler))
}

pub fn clone_data_with_handlers<Data: GarnishData>(
    data_addr: Data::Size,
    from: &Data,
    to: &mut Data,
    custom_handler: CloneHandler<Data>,
    invalid_handler: CloneHandler<Data>,
) -> Result<Data::Size, Data::Error> {
    clone_data_with_handlers_internal(data_addr, from, to, Some(custom_handler), Some(invalid_handler))
}

pub trait GarnishCloneHandler<Data: GarnishData> {
    fn clone_data(&mut self, addr: Data::Size, from: &Data, to: &mut Data) -> Result<Data::Size, Data::Error> {
        match from.get_data_type(addr.clone())? {
            GarnishDataType::Invalid => self.clone_invalid(addr, from, to),
            GarnishDataType::Unit => self.clone_unit(addr, from, to),
            GarnishDataType::Number => self.clone_number(addr, from, to),
            GarnishDataType::Type => self.clone_type(addr, from, to),
            GarnishDataType::Char => self.clone_char(addr, from, to),
            GarnishDataType::CharList => self.clone_char_list(addr, from, to),
            GarnishDataType::Byte => self.clone_byte(addr, from, to),
            GarnishDataType::ByteList => self.clone_byte_list(addr, from, to),
            GarnishDataType::Symbol => self.clone_symbol(addr, from, to),
            GarnishDataType::SymbolList => self.clone_symbol_list(addr, from, to),
            GarnishDataType::Pair => self.clone_pair(addr, from, to),
            GarnishDataType::Range => self.clone_range(addr, from, to),
            GarnishDataType::Concatenation => self.clone_concatenation(addr, from, to),
            GarnishDataType::Slice => self.clone_slice(addr, from, to),
            GarnishDataType::Partial => todo!(),
            GarnishDataType::List => self.clone_list(addr, from, to),
            GarnishDataType::Expression => self.clone_expression(addr, from, to),
            GarnishDataType::External => self.clone_external(addr, from, to),
            GarnishDataType::True => self.clone_true(addr, from, to),
            GarnishDataType::False => self.clone_false(addr, from, to),
            GarnishDataType::Custom => self.clone_custom(addr, from, to),
        }
    }

    fn clone_invalid(&mut self, addr: Data::Size, from: &Data, to: &mut Data) -> Result<Data::Size, Data::Error>;

    fn clone_custom(&mut self, addr: Data::Size, from: &Data, to: &mut Data) -> Result<Data::Size, Data::Error>;

    fn clone_unit(&mut self, _: Data::Size, _: &Data, to: &mut Data) -> Result<Data::Size, Data::Error> {
        to.add_unit()
    }

    fn clone_number(&mut self, addr: Data::Size, from: &Data, to: &mut Data) -> Result<Data::Size, Data::Error> {
        to.add_number(from.get_number(addr.clone())?)
    }

    fn clone_type(&mut self, addr: Data::Size, from: &Data, to: &mut Data) -> Result<Data::Size, Data::Error> {
        to.add_type(from.get_type(addr.clone())?)
    }

    fn clone_char(&mut self, addr: Data::Size, from: &Data, to: &mut Data) -> Result<Data::Size, Data::Error> {
        to.add_char(from.get_char(addr.clone())?)
    }

    fn clone_char_list(&mut self, addr: Data::Size, from: &Data, to: &mut Data) -> Result<Data::Size, Data::Error> {
        let len = from.get_char_list_len(addr.clone())?;
        let iter = Data::make_number_iterator_range(Data::Number::zero(), Data::size_to_number(len));
        to.start_char_list()?;
        for i in iter {
            to.add_to_char_list(from.get_char_list_item(addr.clone(), i)?)?;
        }

        to.end_char_list()
    }

    fn clone_byte(&mut self, addr: Data::Size, from: &Data, to: &mut Data) -> Result<Data::Size, Data::Error> {
        to.add_byte(from.get_byte(addr.clone())?)
    }

    fn clone_byte_list(&mut self, addr: Data::Size, from: &Data, to: &mut Data) -> Result<Data::Size, Data::Error> {
        let len = from.get_byte_list_len(addr.clone())?;
        let iter = Data::make_number_iterator_range(Data::Number::zero(), Data::size_to_number(len));
        to.start_byte_list()?;
        for i in iter {
            to.add_to_byte_list(from.get_byte_list_item(addr.clone(), i)?)?;
        }

        to.end_byte_list()
    }

    fn clone_symbol(&mut self, addr: Data::Size, from: &Data, to: &mut Data) -> Result<Data::Size, Data::Error> {
        to.add_symbol(from.get_symbol(addr.clone())?)
    }

    fn clone_symbol_list(&mut self, addr: Data::Size, from: &Data, to: &mut Data) -> Result<Data::Size, Data::Error> {
        let mut iter = from.get_symbol_list_iter(addr.clone());

        match iter.next() {
            None => to.add_unit(),
            Some(first) => {
                let sym = from.get_symbol_list_item(addr.clone(), first)?;
                let sym1_addr = to.add_symbol(sym)?;

                match iter.next() {
                    None => Ok(sym1_addr),
                    Some(second) => {
                        let sym = from.get_symbol_list_item(addr.clone(), second)?;
                        let sym2_addr = to.add_symbol(sym)?;
                        let mut previous = to.merge_to_symbol_list(sym1_addr, sym2_addr)?;

                        while let Some(index) = iter.next() {
                            let sym = from.get_symbol_list_item(addr.clone(), index)?;
                            let sym_addr = to.add_symbol(sym)?;
                            let sym_list = to.merge_to_symbol_list(previous, sym_addr)?;
                            previous = sym_list;
                        }

                        Ok(previous)
                    }
                }
            }
        }
    }

    fn clone_pair(&mut self, addr: Data::Size, from: &Data, to: &mut Data) -> Result<Data::Size, Data::Error> {
        from.get_pair(addr.clone()).and_then(|(left, right)| {
            let to_left = self.clone_data(left, from, to)?;
            let to_right = self.clone_data(right, from, to)?;
            to.add_pair((to_left, to_right))
        })
    }

    fn clone_range(&mut self, addr: Data::Size, from: &Data, to: &mut Data) -> Result<Data::Size, Data::Error> {
        from.get_range(addr.clone()).and_then(|(left, right)| {
            let to_left = self.clone_data(left, from, to)?;
            let to_right = self.clone_data(right, from, to)?;
            to.add_range(to_left, to_right)
        })
    }

    fn clone_concatenation(&mut self, addr: Data::Size, from: &Data, to: &mut Data) -> Result<Data::Size, Data::Error> {
        from.get_concatenation(addr.clone()).and_then(|(left, right)| {
            let to_left = self.clone_data(left, from, to)?;
            let to_right = self.clone_data(right, from, to)?;
            to.add_concatenation(to_left, to_right)
        })
    }

    fn clone_slice(&mut self, addr: Data::Size, from: &Data, to: &mut Data) -> Result<Data::Size, Data::Error> {
        from.get_slice(addr.clone()).and_then(|(left, right)| {
            let to_left = self.clone_data(left, from, to)?;
            let to_right = self.clone_data(right, from, to)?;
            to.add_slice(to_left, to_right)
        })
    }

    fn clone_list(&mut self, addr: Data::Size, from: &Data, to: &mut Data) -> Result<Data::Size, Data::Error> {
        let len = from.get_list_len(addr.clone())?;
        let iter = Data::make_number_iterator_range(Data::Number::zero(), Data::size_to_number(len.clone()));

        let mut items = vec![];
        for i in iter {
            let addr = from.get_list_item(addr.clone(), i).and_then(|addr| self.clone_data(addr, from, to))?;
            let is_association = match to.get_data_type(addr.clone())? {
                GarnishDataType::Pair => {
                    let (left, _right) = to.get_pair(addr.clone())?;
                    match to.get_data_type(left)? {
                        GarnishDataType::Symbol => true,
                        _ => false,
                    }
                }
                _ => false,
            };
            items.push((addr, is_association));
        }

        to.start_list(len.clone())?;
        for (addr, is_association) in items {
            to.add_to_list(addr, is_association)?;
        }
        to.end_list()
    }

    fn clone_expression(&mut self, addr: Data::Size, from: &Data, to: &mut Data) -> Result<Data::Size, Data::Error> {
        to.add_expression(from.get_expression(addr.clone())?)
    }

    fn clone_external(&mut self, addr: Data::Size, from: &Data, to: &mut Data) -> Result<Data::Size, Data::Error> {
        to.add_external(from.get_external(addr.clone())?)
    }

    fn clone_true(&mut self, _: Data::Size, _: &Data, to: &mut Data) -> Result<Data::Size, Data::Error> {
        to.add_true()
    }

    fn clone_false(&mut self, _: Data::Size, _: &Data, to: &mut Data) -> Result<Data::Size, Data::Error> {
        to.add_false()
    }
}

pub struct StandardCloneHandler<Data: GarnishData> {
    invalid_handler: Option<CloneHandler<Data>>,
    custom_handler: Option<CloneHandler<Data>>,
}

impl<Data: GarnishData> StandardCloneHandler<Data> {
    pub fn new(invalid_handler: Option<CloneHandler<Data>>, custom_handler: Option<CloneHandler<Data>>) -> Self {
        StandardCloneHandler {
            invalid_handler,
            custom_handler,
        }
    }
}

impl<Data: GarnishData> Default for StandardCloneHandler<Data> {
    fn default() -> Self {
        Self {
            invalid_handler: None,
            custom_handler: None,
        }
    }
}

impl<Data: GarnishData> GarnishCloneHandler<Data> for StandardCloneHandler<Data> {
    fn clone_invalid(&mut self, addr: Data::Size, from: &Data, to: &mut Data) -> Result<Data::Size, Data::Error> {
        match self.invalid_handler {
            None => to.add_unit(),
            Some(handler) => handler(addr.clone(), from, to),
        }
    }

    fn clone_custom(&mut self, addr: Data::Size, from: &Data, to: &mut Data) -> Result<Data::Size, Data::Error> {
        match self.custom_handler {
            None => to.add_unit(),
            Some(handler) => handler(addr.clone(), from, to),
        }
    }
}

pub fn clone_data_with_handler<Data, Handler>(
    data_addr: Data::Size,
    from: &Data,
    to: &mut Data,
    mut handler: Handler,
) -> Result<Data::Size, Data::Error>
where
    Data: GarnishData,
    Handler: GarnishCloneHandler<Data>,
{
    handler.clone_data(data_addr, from, to)
}

pub fn clone_data_with_handler_default<Data, Handler>(data_addr: Data::Size, from: &Data, to: &mut Data) -> Result<Data::Size, Data::Error>
where
    Data: GarnishData,
    Handler: GarnishCloneHandler<Data> + Default,
{
    Handler::default().clone_data(data_addr, from, to)
}

fn clone_data_with_handlers_internal<Data: GarnishData>(
    data_addr: Data::Size,
    from: &Data,
    to: &mut Data,
    custom_handler: Option<CloneHandler<Data>>,
    invalid_handler: Option<CloneHandler<Data>>,
) -> Result<Data::Size, Data::Error> {
    StandardCloneHandler::new(invalid_handler, custom_handler).clone_data(data_addr, from, to)
}
