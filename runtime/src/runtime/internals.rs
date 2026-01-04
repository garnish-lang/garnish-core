use crate::runtime::error::state_error;
use crate::runtime::range::range_len;
use crate::runtime::utilities::{next_ref, push_number, push_unit};
use garnish_lang_traits::helpers::iterate_concatenation_mut;
use garnish_lang_traits::Instruction;
use garnish_lang_traits::{GarnishData, GarnishDataFactory, GarnishDataType, RuntimeError, TypeConstants};

pub fn access_left_internal<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let r = next_ref(this)?;
    match this.get_data_type(r.clone())? {
        GarnishDataType::Pair => {
            let (left, _) = this.get_pair(r)?;
            this.push_register(left)?;
        }
        GarnishDataType::Range => {
            let (start, _) = this.get_range(r)?;
            match this.get_data_type(start.clone())? {
                GarnishDataType::Number => {
                    this.push_register(start)?;
                }
                _ => push_unit(this)?,
            }
        }
        GarnishDataType::Slice => {
            let (value, _) = this.get_slice(r)?;
            this.push_register(value)?;
        }
        GarnishDataType::Concatenation => {
            let (left, _) = this.get_concatenation(r)?;
            this.push_register(left)?;
        }
        t => {
            if !this.defer_op(Instruction::AccessLeftInternal, (t, r), (GarnishDataType::Unit, Data::Size::zero()))? {
                push_unit(this)?
            }
        }
    }

    Ok(None)
}

pub fn access_right_internal<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let r = next_ref(this)?;
    match this.get_data_type(r.clone())? {
        GarnishDataType::Pair => {
            let (_, right) = this.get_pair(r)?;
            this.push_register(right)?;
        }
        GarnishDataType::Range => {
            let (_, end) = this.get_range(r)?;
            match this.get_data_type(end.clone())? {
                GarnishDataType::Number => {
                    this.push_register(end)?;
                }
                _ => push_unit(this)?,
            }
        }
        GarnishDataType::Slice => {
            let (_, range) = this.get_slice(r)?;
            this.push_register(range)?;
        }
        GarnishDataType::Concatenation => {
            let (_, right) = this.get_concatenation(r)?;
            this.push_register(right)?;
        }
        t => {
            if !this.defer_op(Instruction::AccessRightInternal, (t, r), (GarnishDataType::Unit, Data::Size::zero()))? {
                push_unit(this)?
            }
        }
    }

    Ok(None)
}

pub fn access_length_internal<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let r = next_ref(this)?;
    match this.get_data_type(r.clone())? {
        GarnishDataType::Pair => {
            let (left, _) = this.get_pair(r)?;
            match this.get_data_type(left.clone())? {
                GarnishDataType::Symbol => {
                    push_number(this, Data::Number::one())?;
                }
                _ => push_unit(this)?,
            }
        }
        GarnishDataType::List => {
            let len = <Data as GarnishData>::DataFactory::size_to_number(this.get_list_len(r)?);
            push_number(this, len)?;
        }
        GarnishDataType::CharList => {
            let len = <Data as GarnishData>::DataFactory::size_to_number(this.get_char_list_len(r)?);
            push_number(this, len)?;
        }
        GarnishDataType::ByteList => {
            let len = <Data as GarnishData>::DataFactory::size_to_number(this.get_byte_list_len(r)?);
            push_number(this, len)?;
        }
        GarnishDataType::Range => {
            let (start, end) = this.get_range(r)?;
            match (this.get_data_type(end.clone())?, this.get_data_type(start.clone())?) {
                (GarnishDataType::Number, GarnishDataType::Number) => {
                    let start_int = this.get_number(start)?;
                    let end_int = this.get_number(end)?;
                    let result = range_len::<Data>(start_int, end_int)?;

                    let addr = this.add_number(result)?;
                    this.push_register(addr)?;
                }
                _ => push_unit(this)?,
            }
        }
        GarnishDataType::Slice => {
            let (_, range_addr) = this.get_slice(r)?;
            let (start, end) = this.get_range(range_addr)?;
            match (this.get_data_type(start.clone())?, this.get_data_type(end.clone())?) {
                (GarnishDataType::Number, GarnishDataType::Number) => {
                    let start = this.get_number(start)?;
                    let end = this.get_number(end)?;
                    let addr = this.add_number(range_len::<Data>(start, end)?)?;
                    this.push_register(addr)?;
                }
                (s, e) => state_error(format!("Non integer values used for range {:?} {:?}", s, e))?,
            }
        }
        GarnishDataType::Concatenation => {
            let count = concatenation_len(this, r)?;
            let addr = this.add_number(<Data as GarnishData>::DataFactory::size_to_number(count))?;
            this.push_register(addr)?;
        }
        t => {
            if !this.defer_op(Instruction::AccessLengthInternal, (t, r), (GarnishDataType::Unit, Data::Size::zero()))? {
                push_unit(this)?
            }
        }
    }

    Ok(None)
}

pub(crate) fn concatenation_len<Data: GarnishData>(this: &mut Data, addr: Data::Size) -> Result<Data::Size, RuntimeError<Data::Error>> {
    Ok(iterate_concatenation_mut(this, addr, |_, _, _| Ok(None))?.1)
}

#[cfg(test)]
mod defer_op {
    use crate::ops::{access_left_internal, access_length_internal, access_right_internal};
    use crate::runtime::tests::MockGarnishData;
    use garnish_lang_traits::{GarnishData, GarnishDataType, Instruction};

    #[test]
    fn access_length_of_pair() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Number, GarnishDataType::Symbol, GarnishDataType::Pair]);

        mock_data.stub_get_pair = |_, i| {
            assert_eq!(i, 2);
            Ok((1, 0))
        };
        mock_data.stub_get_symbol = |_, _i| Ok(100);
        mock_data.stub_add_number = |_, num| {
            assert_eq!(num, 1);
            Ok(100)
        };
        mock_data.stub_push_register = |data, i| {
            assert_eq!(i, 100);
            data.registers.push(i);
            Ok(())
        };
        
        access_length_internal(&mut mock_data).unwrap();

        assert_eq!(mock_data.pop_register().unwrap(), Some(100));
    }

    #[test]
    fn access_length_of_pair_with_symbol() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Number, GarnishDataType::CharList, GarnishDataType::Pair]);

        mock_data.stub_get_pair = |_, i| {
            assert_eq!(i, 2);
            Ok((1, 0))
        };
        mock_data.stub_get_symbol = |_, _i| Ok(100);
        mock_data.stub_add_unit = |_| {
            Ok(200)
        };
        mock_data.stub_push_register = |data, i| {
            assert_eq!(i, 200);
            data.registers.push(i);
            Ok(())
        };

        access_length_internal(&mut mock_data).unwrap();

        assert_eq!(mock_data.pop_register().unwrap(), Some(200));
    }

    #[test]
    fn access_left_defer_op() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Symbol, GarnishDataType::Number]);

        mock_data.stub_get_instruction_cursor = |_| 1;
        mock_data.stub_defer_op = |data, instruction, left, right| {
            assert_eq!(instruction, Instruction::AccessLeftInternal);
            assert_eq!(left, (GarnishDataType::Number, 1));
            assert_eq!(right, (GarnishDataType::Unit, 0));
            data.registers.push(200);
            Ok(true)
        };

        access_left_internal(&mut mock_data).unwrap();

        assert_eq!(mock_data.pop_register().unwrap(), Some(200));
    }

    #[test]
    fn access_right_defer_op() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Symbol, GarnishDataType::Number]);

        mock_data.stub_get_instruction_cursor = |_| 1;
        mock_data.stub_defer_op = |data, instruction, left, right| {
            assert_eq!(instruction, Instruction::AccessRightInternal);
            assert_eq!(left, (GarnishDataType::Number, 1));
            assert_eq!(right, (GarnishDataType::Unit, 0));
            data.registers.push(200);
            Ok(true)
        };

        access_right_internal(&mut mock_data).unwrap();

        assert_eq!(mock_data.pop_register().unwrap(), Some(200));
    }

    #[test]
    fn access_length_defer_op() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Symbol, GarnishDataType::Number]);

        mock_data.stub_get_instruction_cursor = |_| 1;
        mock_data.stub_defer_op = |data, instruction, left, right| {
            assert_eq!(instruction, Instruction::AccessLengthInternal);
            assert_eq!(left, (GarnishDataType::Number, 1));
            assert_eq!(right, (GarnishDataType::Unit, 0));
            data.registers.push(200);
            Ok(true)
        };

        access_length_internal(&mut mock_data).unwrap();

        assert_eq!(mock_data.pop_register().unwrap(), Some(200));
    }
}
