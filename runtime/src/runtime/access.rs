use crate::runtime::list::get_access_addr;
use crate::runtime::utilities::{next_ref, push_unit};
use garnish_lang_traits::{GarnishContext, GarnishData, GarnishDataType, Instruction, RuntimeError};

pub(crate) fn access<Data: GarnishData, T: GarnishContext<Data>>(
    this: &mut Data,
    context: Option<&mut T>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let right_addr = next_ref(this)?;
    let left_addr = next_ref(this)?;

    match (this.get_data_type(left_addr.clone())?, this.get_data_type(right_addr.clone())?) {
        (GarnishDataType::Symbol, GarnishDataType::Symbol)
        | (GarnishDataType::Symbol, GarnishDataType::SymbolList)
        | (GarnishDataType::SymbolList, GarnishDataType::Symbol)
        | (GarnishDataType::SymbolList, GarnishDataType::SymbolList) => {
            this.merge_to_symbol_list(left_addr, right_addr).and_then(|i| this.push_register(i))?
        }
        (GarnishDataType::List, GarnishDataType::Number)
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
        (l, r) => match context {
            None => push_unit(this)?,
            Some(c) => {
                if !c.defer_op(this, Instruction::Access, (l, left_addr), (r, right_addr))? {
                    push_unit(this)?
                }
            }
        },
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use crate::runtime::access::access;
    use crate::runtime::tests::MockGarnishData;
    use garnish_lang_traits::{GarnishDataType, NO_CONTEXT};

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

        let result = access(&mut mock_data, NO_CONTEXT).unwrap();

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

        let result = access(&mut mock_data, NO_CONTEXT).unwrap();

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

        let result = access(&mut mock_data, NO_CONTEXT).unwrap();

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

        let result = access(&mut mock_data, NO_CONTEXT).unwrap();

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
        mock_data.stub_get_symbol_list_item = |_, _, index| Ok((index + 1) as u32 * 10);
        mock_data.stub_add_symbol = |_, sym| {
            assert_eq!(sym, 20);
            Ok(5)
        };
        mock_data.stub_push_register = |_, i| {
            assert_eq!(i, 5);
            Ok(())
        };

        let result = access(&mut mock_data, NO_CONTEXT).unwrap();

        assert_eq!(result, None);
    }
}
