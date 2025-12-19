use garnish_lang_traits::{Extents, Instruction, SymbolListPart};
use garnish_lang_traits::helpers::iterate_concatenation_mut;

use crate::runtime::error::OrNumberError;
use crate::runtime::internals::concatenation_len;
use crate::runtime::list::is_value_association;
use crate::runtime::utilities::{get_range, next_ref, next_two_raw_ref, push_unit};
use garnish_lang_traits::{GarnishContext, GarnishData, GarnishDataFactory, GarnishDataType, GarnishNumber, RuntimeError, TypeConstants};

pub fn type_of<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let a = next_ref(this)?;
    let t = this.get_data_type(a)?;
    this.add_type(t).and_then(|r| this.push_register(r))?;

    Ok(None)
}

pub fn type_cast<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
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
            primitive_cast(this, left, Data::get_number, <Data as GarnishData>::DataFactory::number_to_char, Data::add_char)?;
        }
        (GarnishDataType::Number, GarnishDataType::Byte) => {
            primitive_cast(this, left, Data::get_number, <Data as GarnishData>::DataFactory::number_to_byte, Data::add_byte)?;
        }
        (GarnishDataType::Char, GarnishDataType::Number) => {
            primitive_cast(this, left, Data::get_char, <Data as GarnishData>::DataFactory::char_to_number, Data::add_number)?;
        }
        (GarnishDataType::Char, GarnishDataType::Byte) => {
            primitive_cast(this, left, Data::get_char, <Data as GarnishData>::DataFactory::char_to_byte, Data::add_byte)?;
        }
        (GarnishDataType::Byte, GarnishDataType::Number) => {
            primitive_cast(this, left, Data::get_byte, <Data as GarnishData>::DataFactory::byte_to_number, Data::add_number)?;
        }
        (GarnishDataType::Byte, GarnishDataType::Char) => {
            primitive_cast(this, left, Data::get_byte, <Data as GarnishData>::DataFactory::byte_to_char, Data::add_char)?;
        }
        (GarnishDataType::CharList, GarnishDataType::Char) => {
            let len = this.get_char_list_len(left.clone())?;
            if len == Data::Size::one() {
                let mut iter = this.get_char_list_iter(left.clone(), Extents::new(Data::Number::zero(), Data::Number::max_value()))?;
                match iter.next() {
                    Some(c) => this.add_char(c).and_then(|r| this.push_register(r))?,
                    None => push_unit(this)?,
                }
            } else {
                push_unit(this)?;
            }
        }
        (GarnishDataType::SymbolList, GarnishDataType::List) => {
            let mut iter = this.get_symbol_list_iter(left.clone(), Extents::new(Data::Number::zero(), Data::Number::max_value()))?;
            let len = this.get_symbol_list_len(left.clone())?;

            let mut list_index = this.start_list(len)?;
            while let Some(part) = iter.next() {
                let item_index = match part {
                    SymbolListPart::Symbol(sym) => this.add_symbol(sym)?,
                    SymbolListPart::Number(num) => this.add_number(num)?,
                };
                list_index = this.add_to_list(list_index.clone(), item_index)?;
            }
            this.end_list(list_index).and_then(|r| this.push_register(r))?
        }
        (GarnishDataType::Range, GarnishDataType::List) => {
            let (start, end) = this.get_range(left.clone())?;
            let len = end - start + Data::Size::one();
            let (start, end, _) = get_range(this, left)?;
            let mut count = start;

            let mut list_index = this.start_list(len)?;
            while count <= end {
                let addr = this.add_number(count.clone())?;
                list_index = this.add_to_list(list_index.clone(), addr)?;
                count = count.increment().or_num_err()?;
            }

            this.end_list(list_index).and_then(|r| this.push_register(r))?
        }
        (GarnishDataType::CharList, GarnishDataType::List) => {
            let len = this.get_char_list_len(left.clone())?;
            list_from_char_list(this, left, Data::Number::zero(), <Data as GarnishData>::DataFactory::size_to_number(len))?;
        }
        (GarnishDataType::ByteList, GarnishDataType::List) => {
            let len = this.get_byte_list_len(left.clone())?;
            list_from_byte_list(this, left, Data::Number::zero(), <Data as GarnishData>::DataFactory::size_to_number(len))?;
        }
        (GarnishDataType::Concatenation, GarnishDataType::List) => {
            let len = concatenation_len(this, left.clone())?;
            let mut list_index = this.start_list(len)?;
            iterate_concatenation_mut(this, left, |this, _, addr| {
                list_index = this.add_to_list(list_index.clone(), addr)?;
                Ok(None)
            })?;

            let addr = this.end_list(list_index)?;
            this.push_register(addr)?;
        }
        (GarnishDataType::Slice, GarnishDataType::List) => {
            let (value, range) = this.get_slice(left)?;
            let (start, end, len) = get_range(this, range)?;
            match this.get_data_type(value.clone())? {
                GarnishDataType::List => {
                    let len = this.get_list_len(value.clone())?;

                    let mut list_index = this.start_list(len)?;

                    let mut i = start;

                    while i <= end {
                        let addr = match this.get_list_item(value.clone(), i.clone())? {
                            Some(addr) => addr,
                            None => {
                                this.add_unit()?
                            }
                        };

                        list_index = this.add_to_list(list_index.clone(), addr)?;
                        i = i.increment().or_num_err()?;
                    }

                    this.end_list(list_index).and_then(|r| this.push_register(r))?
                }
                GarnishDataType::CharList => {
                    list_from_char_list(this, value, start, end.increment().or_num_err()?)?;
                }
                GarnishDataType::ByteList => {
                    list_from_byte_list(this, value, start, end.increment().or_num_err()?)?;
                }
                GarnishDataType::Concatenation => {
                    let mut list_index = this.start_list(<Data as GarnishData>::DataFactory::number_to_size(len).or_num_err()?)?;

                    iterate_concatenation_mut(this, value, |this, current_index, addr| {
                        if current_index < start {
                            return Ok(None);
                        }

                        if current_index > end {
                            // providing value will end iteration
                            // even tho we don't need the return value
                            return Ok(Some(addr));
                        }

                        list_index = this.add_to_list(list_index.clone(), addr)?;
                        Ok(None)
                    })?;

                    let addr = this.end_list(list_index)?;
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
        (l, r) => {
            if !this.defer_op(Instruction::ApplyType, (l, left), (r, right))? {
                push_unit(this)?
            }
        }
    }

    Ok(None)
}

pub(crate) fn list_from_char_list<Data: GarnishData>(this: &mut Data, byte_list_addr: Data::Size, start: Data::Number, end: Data::Number) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let len = this.get_char_list_len(byte_list_addr.clone())?;
    let mut count = start;

    let mut list_index = this.start_list(len)?;
    while count < end {
        let c = this.get_char_list_item(byte_list_addr.clone(), count.clone())?;
        let addr = match c {
            Some(c) => this.add_char(c)?,
            None => this.add_unit()?,
        };
        list_index = this.add_to_list(list_index.clone(), addr)?;

        count = count.increment().or_num_err()?;
    }

    this.end_list(list_index).and_then(|r| this.push_register(r))?;

    Ok(None)
}

pub(crate) fn list_from_byte_list<Data: GarnishData>(this: &mut Data, byte_list_addr: Data::Size, start: Data::Number, end: Data::Number) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let len = this.get_byte_list_len(byte_list_addr.clone())?;
    let mut count = start;

    let mut list_index = this.start_list(len)?;
    while count < end {
        let c = this.get_byte_list_item(byte_list_addr.clone(), count.clone())?;
        let addr = match c {
            Some(c) => this.add_byte(c)?,
            None => this.add_unit()?,
        };
        list_index = this.add_to_list(list_index.clone(), addr)?;

        count = count.increment().or_num_err()?;
    }

    this.end_list(list_index).and_then(|r| this.push_register(r))?;

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

#[cfg(test)]
mod tests {
    use crate::runtime::casting::type_cast;
    use crate::runtime::tests::{MockGarnishData, MockSymbolListPartIterator};
    use garnish_lang_traits::{GarnishDataType, SymbolListPart};

    #[test]
    fn symbol_list_to_list() {
        let mut data = MockGarnishData::new_basic_data(vec![GarnishDataType::SymbolList, GarnishDataType::List]);

        data.stub_get_symbol_list_iter = |_, _| MockSymbolListPartIterator::new(vec![SymbolListPart::Symbol(10), SymbolListPart::Symbol(11)]);
        data.stub_get_symbol_list_len = |_, _| Ok(2);
        data.stub_start_list = |_, _| Ok(0);
        data.stub_get_symbol_list_item = |_, _, i| Ok(Some(SymbolListPart::Symbol(i as u32 + 10)));
        data.stub_add_symbol = |_, sym| {
            assert!([10, 11].contains(&sym));
            Ok(sym as i32)
        };
        data.stub_add_to_list = |_, list_index, item_index| {
            assert!([10, 11].contains(&item_index));
            Ok(list_index as i32)
        };
        data.stub_end_list = |_, _| Ok(999);
        data.stub_push_register = |_, i| {
            assert_eq!(i, 999);
            Ok(())
        };

        let result = type_cast(&mut data).unwrap();

        assert_eq!(result, None);
    }
}

#[cfg(test)]
mod defer_op {
    use crate::ops::type_cast;
    use crate::runtime::tests::MockGarnishData;
    use garnish_lang_traits::{GarnishData, GarnishDataType, Instruction};

    #[test]
    fn add_defer_op() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Symbol, GarnishDataType::Number]);

        mock_data.stub_get_instruction_cursor = |_| 1;
        mock_data.stub_defer_op = |data, instruction, left, right| {
            assert_eq!(instruction, Instruction::ApplyType);
            assert_eq!(left, (GarnishDataType::Symbol, 0));
            assert_eq!(right, (GarnishDataType::Number, 1));
            data.registers.push(200);
            Ok(true)
        };

        type_cast(&mut mock_data).unwrap();

        assert_eq!(mock_data.pop_register().unwrap(), Some(200));
    }
}
