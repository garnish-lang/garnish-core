use crate::runtime::list::get_access_addr;
use crate::runtime::utilities::{next_ref, push_unit};
use garnish_lang_traits::{GarnishData, GarnishDataType, Instruction, RuntimeError};

pub fn access<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let right_addr = next_ref(this)?;
    let left_addr = next_ref(this)?;

    match (this.get_data_type(left_addr.clone())?, this.get_data_type(right_addr.clone())?) {
        (GarnishDataType::Symbol, GarnishDataType::Symbol)
        | (GarnishDataType::Symbol, GarnishDataType::SymbolList)
        | (GarnishDataType::SymbolList, GarnishDataType::Symbol)
        | (GarnishDataType::SymbolList, GarnishDataType::SymbolList) => this.merge_to_symbol_list(left_addr, right_addr).and_then(|i| this.push_register(i))?,
        (GarnishDataType::Pair, GarnishDataType::Number)
        | (GarnishDataType::Pair, GarnishDataType::Symbol)
        | (GarnishDataType::List, GarnishDataType::Number)
        | (GarnishDataType::List, GarnishDataType::Symbol)
        | (GarnishDataType::CharList, GarnishDataType::Number)
        | (GarnishDataType::CharList, GarnishDataType::Symbol)
        | (GarnishDataType::ByteList, GarnishDataType::Number)
        | (GarnishDataType::ByteList, GarnishDataType::Symbol)
        | (GarnishDataType::SymbolList, GarnishDataType::Number)
        | (GarnishDataType::Range, GarnishDataType::Number)
        | (GarnishDataType::Range, GarnishDataType::Symbol)
        | (GarnishDataType::Concatenation, GarnishDataType::Number)
        | (GarnishDataType::Concatenation, GarnishDataType::Symbol)
        | (GarnishDataType::Slice, GarnishDataType::Number)
        | (GarnishDataType::Slice, GarnishDataType::Symbol) => match get_access_addr(this, right_addr, left_addr)? {
            None => push_unit(this)?,
            Some(i) => this.push_register(i)?,
        },
        (l, r) => {
            if !this.defer_op(Instruction::Access, (l, left_addr), (r, right_addr))? {
                push_unit(this)?
            }
        }
    }

    Ok(None)
}

#[cfg(test)]
mod defer_op {
    use crate::ops::access;
    use crate::runtime::tests::MockGarnishData;
    use garnish_lang_traits::{GarnishData, GarnishDataType, Instruction};

    #[test]
    fn add_defer_op() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Symbol, GarnishDataType::Number]);

        mock_data.stub_get_instruction_cursor = |_| 1;
        mock_data.stub_defer_op = |data, instruction, left, right| {
            assert_eq!(instruction, Instruction::Access);
            assert_eq!(left, (GarnishDataType::Symbol, 0));
            assert_eq!(right, (GarnishDataType::Number, 1));
            data.registers.push(200);
            Ok(true)
        };

        access(&mut mock_data).unwrap();

        assert_eq!(mock_data.pop_register().unwrap(), Some(200));
    }
}

#[cfg(test)]
mod tests {
    use crate::runtime::access::access;
    use crate::runtime::tests::MockGarnishData;
    use garnish_lang_traits::{GarnishDataType, SymbolListPart};

    #[test]
    fn access_pair_with_number() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Number, GarnishDataType::Symbol, GarnishDataType::Pair, GarnishDataType::Number]);

        mock_data.stub_get_number = |_, i| {
            assert_eq!(i, 3);
            Ok(0)
        };
        mock_data.stub_get_pair = |_, i| {
            assert_eq!(i, 2);
            Ok((1, 0))
        };
        mock_data.stub_push_register = |_, i| {
            assert_eq!(i, 2);
            Ok(())
        };

        let result = access(&mut mock_data).unwrap();

        assert_eq!(result, None);
    }

    #[test]
    fn access_pair_with_non_zero_number() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Number, GarnishDataType::Symbol, GarnishDataType::Pair, GarnishDataType::Number]);

        mock_data.stub_get_number = |_, i| {
            assert_eq!(i, 3);
            Ok(1)
        };
        mock_data.stub_get_pair = |_, i| {
            assert_eq!(i, 2);
            Ok((1, 0))
        };
        mock_data.stub_add_unit = |_| Ok(100);
        mock_data.stub_push_register = |_, i| {
            assert_eq!(i, 100);
            Ok(())
        };

        let result = access(&mut mock_data).unwrap();

        assert_eq!(result, None);
    }

    #[test]
    fn access_pair_with_symbol() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Number, GarnishDataType::Pair, GarnishDataType::Symbol]);

        mock_data.stub_get_symbol = |_, i| {
            assert_eq!(i, 2);
            Ok(0)
        };
        mock_data.stub_get_pair = |_, i| {
            assert_eq!(i, 1);
            Ok((2, 0))
        };
        mock_data.stub_push_register = |_, i| {
            assert_eq!(i, 0);
            Ok(())
        };

        let result = access(&mut mock_data).unwrap();

        assert_eq!(result, None);
    }

    #[test]
    fn access_pair_with_non_matching_symbol() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Number, GarnishDataType::Symbol, GarnishDataType::Pair, GarnishDataType::Symbol]);

        mock_data.stub_get_symbol = |_, i| {
            if i == 3 {
                Ok(40)
            } else {
                assert_eq!(i, 1);
                Ok(30)
            }
        };
        mock_data.stub_get_pair = |_, i| {
            assert_eq!(i, 2);
            Ok((1, 0))
        };
        mock_data.stub_add_unit = |_| Ok(100);
        mock_data.stub_push_register = |_, i| {
            assert_eq!(i, 100);
            Ok(())
        };

        let result = access(&mut mock_data).unwrap();

        assert_eq!(result, None);
    }

    #[test]
    fn create_symbol_list() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Symbol, GarnishDataType::Symbol]);

        mock_data.stub_merge_to_symbol_list = |_, first, second| {
            assert_eq!(first, 0);
            assert_eq!(second, 1);
            Ok(30)
        };
        mock_data.stub_push_register = |_, i| {
            assert_eq!(i, 30);
            Ok(())
        };

        let result = access(&mut mock_data).unwrap();

        assert_eq!(result, None);
    }

    #[test]
    fn access_symbol_list_with_number() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::SymbolList, GarnishDataType::Number]);

        mock_data.stub_get_number = |_, num| {
            assert_eq!(num, 1);
            Ok(1)
        };
        mock_data.stub_get_symbol_list_len = |_, _| Ok(2);
        mock_data.stub_get_symbol_list_item = |_, _, index| Ok(SymbolListPart::Symbol((index + 1) as u32 * 100u32));
        mock_data.stub_add_symbol = |_, sym| {
            assert_eq!(sym, 20);
            Ok(5)
        };
        mock_data.stub_push_register = |_, i| {
            assert_eq!(i, 5);
            Ok(())
        };

        let result = access(&mut mock_data).unwrap();

        assert_eq!(result, None);
    }

    #[test]
    fn extend_symbol_list_from_left() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Symbol, GarnishDataType::SymbolList]);

        mock_data.stub_merge_to_symbol_list = |_, first, second| {
            assert_eq!(first, 0);
            assert_eq!(second, 1);
            Ok(30)
        };
        mock_data.stub_push_register = |_, i| {
            assert_eq!(i, 30);
            Ok(())
        };

        let result = access(&mut mock_data).unwrap();

        assert_eq!(result, None);
    }

    #[test]
    fn extend_symbol_list_from_right() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::SymbolList, GarnishDataType::Symbol]);

        mock_data.stub_merge_to_symbol_list = |_, first, second| {
            assert_eq!(first, 0);
            assert_eq!(second, 1);
            Ok(30)
        };
        mock_data.stub_push_register = |_, i| {
            assert_eq!(i, 30);
            Ok(())
        };

        let result = access(&mut mock_data).unwrap();

        assert_eq!(result, None);
    }

    #[test]
    fn merge_symbol_lists() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::SymbolList, GarnishDataType::SymbolList]);

        mock_data.stub_merge_to_symbol_list = |_, first, second| {
            assert_eq!(first, 0);
            assert_eq!(second, 1);
            Ok(30)
        };
        mock_data.stub_push_register = |_, i| {
            assert_eq!(i, 30);
            Ok(())
        };

        let result = access(&mut mock_data).unwrap();

        assert_eq!(result, None);
    }
}
