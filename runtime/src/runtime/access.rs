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