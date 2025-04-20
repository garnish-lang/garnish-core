use garnish_lang_traits::{GarnishContext, GarnishData, GarnishDataType, Instruction, RuntimeError};
use crate::runtime::list::get_access_addr;
use crate::runtime::utilities::{next_ref, push_unit};

pub(crate) fn access<Data: GarnishData, T: GarnishContext<Data>>(
    this: &mut Data,
    context: Option<&mut T>,
) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let right_addr = next_ref(this)?;
    let left_addr = next_ref(this)?;

    match (this.get_data_type(left_addr.clone())?, this.get_data_type(right_addr.clone())?) {
        (GarnishDataType::Symbol, GarnishDataType::Symbol) => {
            this.merge_to_symbol_list(left_addr, right_addr)
                .and_then(|i| this.push_register(i))?
        }
        (GarnishDataType::List, GarnishDataType::Number)
        | (GarnishDataType::List, GarnishDataType::Symbol)
        | (GarnishDataType::CharList, GarnishDataType::Number)
        | (GarnishDataType::CharList, GarnishDataType::Symbol)
        | (GarnishDataType::ByteList, GarnishDataType::Number)
        | (GarnishDataType::ByteList, GarnishDataType::Symbol)
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
    use garnish_lang_traits::{GarnishDataType, NO_CONTEXT};
    use crate::runtime::access::access;
    use crate::runtime::tests::{MockGarnishData};

    struct StackData {
        addrs: Vec<i32>
    }

    impl Default for StackData {
        fn default() -> Self {
            StackData {
                addrs: vec![]
            }
        }
    }

    #[test]
    fn create_symbol_list() {
        let mut mock_data = MockGarnishData::default_with_data(StackData {
            addrs: vec![10, 20]
        });

        mock_data.stub_pop_register = |data| { Ok(Some(data.addrs.pop().unwrap())) };
        mock_data.stub_get_data_type = |_, _| Ok(GarnishDataType::Symbol);
        mock_data.stub_merge_to_symbol_list = |_, first, second| {
            assert_eq!(first, 10);
            assert_eq!(second, 20);
            Ok(30)
        };
        mock_data.stub_push_register = |_, i| {
            assert_eq!(i, 30);
            Ok(())
        };

        let result = access(&mut mock_data, NO_CONTEXT).unwrap();

        assert_eq!(result, None);
    }
}