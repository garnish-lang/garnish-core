use log::trace;

use crate::{next_ref, push_unit, ExpressionDataType, GarnishLangRuntimeData, GarnishLangRuntimeResult, NestInto, TypeConstants};

pub(crate) fn make_list<Data: GarnishLangRuntimeData>(this: &mut Data, len: Data::Size) -> GarnishLangRuntimeResult<Data::Error> {
    trace!("Instruction - Make List | Length - {:?}", len);

    this.start_list(len).nest_into()?;

    // look into getting this to work with a range value
    let mut count = Data::Size::zero();
    while count < len {
        next_ref(this).and_then(|r| {
            this.get_data_type(r)
                .and_then(|t| match t {
                    ExpressionDataType::Pair => this.get_pair(r).and_then(|(left, _)| {
                        this.get_data_type(left).and_then(|t| match t {
                            ExpressionDataType::Symbol => Ok(true),
                            _ => Ok(false),
                        })
                    }),
                    _ => Ok(false), // Other values are just simple items
                })
                .and_then(|is_associative| this.add_to_list(r, is_associative))
                .nest_into()
        })?;

        count += Data::Size::one();
    }

    this.end_list().and_then(|r| this.push_register(r)).nest_into()
}

pub(crate) fn access<Data: GarnishLangRuntimeData>(this: &mut Data) -> GarnishLangRuntimeResult<Data::Error> {
    trace!("Instruction - Access");

    let right_ref = next_ref(this)?;
    let left_ref = next_ref(this)?;

    match get_access_addr(this, right_ref, left_ref)? {
        None => push_unit(this),
        Some(i) => this.push_register(i).nest_into(),
    }
}

pub(crate) fn access_left_internal<Data: GarnishLangRuntimeData>(this: &mut Data) -> GarnishLangRuntimeResult<Data::Error> {
    trace!("Instruction - Access Left Internal");
    next_ref(this).and_then(|r| match this.get_data_type(r).nest_into()? {
        ExpressionDataType::Pair => this.get_pair(r).and_then(|(left, _)| this.push_register(left)).nest_into(),
        _ => push_unit(this),
    })
}

pub(crate) fn access_right_internal<Data: GarnishLangRuntimeData>(this: &mut Data) -> GarnishLangRuntimeResult<Data::Error> {
    trace!("Instruction - Access Right Internal");
    next_ref(this).and_then(|r| match this.get_data_type(r).nest_into()? {
        ExpressionDataType::Pair => this.get_pair(r).and_then(|(_, right)| this.push_register(right)).nest_into(),
        _ => push_unit(this),
    })
}

pub(crate) fn access_length_internal<Data: GarnishLangRuntimeData>(this: &mut Data) -> GarnishLangRuntimeResult<Data::Error> {
    trace!("Instruction - Access Length Internal");
    next_ref(this).and_then(|r| match this.get_data_type(r).nest_into()? {
        ExpressionDataType::List => this
            .get_list_len(r)
            .and_then(|len| this.add_integer(Data::size_to_integer(len)).and_then(|r| this.push_register(r)))
            .nest_into(),
        _ => push_unit(this),
    })
}

pub(crate) fn get_access_addr<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    sym: Data::Size,
    list: Data::Size,
) -> GarnishLangRuntimeResult<Data::Error, Option<Data::Size>> {
    let sym_ref = sym;
    let list_ref = list;

    match (this.get_data_type(list_ref).nest_into()?, this.get_data_type(sym_ref).nest_into()?) {
        (ExpressionDataType::List, ExpressionDataType::Symbol) => {
            let sym_val = this.get_symbol(sym_ref).nest_into()?;

            Ok(this.get_list_item_with_symbol(list_ref, sym_val).nest_into()?)
        }
        (ExpressionDataType::List, ExpressionDataType::Integer) => {
            let i = this.get_integer(sym).nest_into()?;

            if i < Data::Integer::zero() {
                Ok(None)
            } else {
                let i = i;
                if i >= Data::size_to_integer(this.get_list_len(list).nest_into()?) {
                    Ok(None)
                } else {
                    Ok(Some(this.get_list_item(list, i).nest_into()?))
                }
            }
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use crate::{runtime::GarnishRuntime, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

    #[test]
    fn make_list() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let i3 = runtime.add_integer(20).unwrap();
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

        runtime.add_integer(10).unwrap();
        runtime.add_integer(20).unwrap();
        runtime.add_integer(20).unwrap();

        runtime.push_instruction(Instruction::MakeList, Some(3)).unwrap();

        let result = runtime.make_list(3);

        assert!(result.is_err());
    }

    #[test]
    fn make_list_with_associations() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_symbol("two").unwrap();
        let i4 = runtime.add_integer(20).unwrap();
        let i5 = runtime.add_symbol("three").unwrap();
        let i6 = runtime.add_integer(30).unwrap();
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
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_symbol("one").unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i4).unwrap();
        runtime.push_register(i5).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_register().get(0).unwrap(), &i2);
    }

    #[test]
    fn access_with_number() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_integer(0).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i4).unwrap();
        runtime.push_register(i5).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_register().get(0).unwrap(), &i3);
    }

    #[test]
    fn access_with_number_out_of_bounds_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_integer(10).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i4).unwrap();
        runtime.push_register(i5).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_register().get(0).unwrap(), &0);
    }

    #[test]
    fn access_with_number_negative_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_integer(-1).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i4).unwrap();
        runtime.push_register(i5).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_register().get(0).unwrap(), &0);
    }

    #[test]
    fn access_pair_left() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();

        runtime.push_instruction(Instruction::AccessLeftInternal, None).unwrap();

        runtime.push_register(i3).unwrap();

        runtime.access_left_internal().unwrap();

        assert_eq!(runtime.get_register().get(0).unwrap(), &i1);
    }

    #[test]
    fn access_left_internal_incompatible_type_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();

        runtime.push_instruction(Instruction::AccessLeftInternal, None).unwrap();

        runtime.push_register(i1).unwrap();

        runtime.access_left_internal().unwrap();

        assert_eq!(runtime.get_register().get(0).unwrap(), &0);
    }

    #[test]
    fn access_pair_right() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();

        runtime.push_instruction(Instruction::AccessRightInternal, None).unwrap();

        runtime.push_register(i3).unwrap();

        runtime.access_right_internal().unwrap();

        assert_eq!(runtime.get_register().get(0).unwrap(), &i2);
    }

    #[test]
    fn access_right_internal_incompatible_type_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();

        runtime.push_instruction(Instruction::AccessRightInternal, None).unwrap();

        runtime.push_register(i1).unwrap();

        runtime.access_right_internal().unwrap();

        assert_eq!(runtime.get_register().get(0).unwrap(), &0);
    }

    #[test]
    fn access_list_length() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let start = runtime.get_data_len();

        runtime.push_instruction(Instruction::AccessLengthInternal, None).unwrap();

        runtime.push_register(i4).unwrap();

        runtime.access_length_internal().unwrap();

        assert_eq!(runtime.get_integer(start).unwrap(), 1);
        assert_eq!(runtime.get_register().get(0).unwrap(), &start);
    }

    #[test]
    fn access_length_internal_incompatible_type_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();

        runtime.push_instruction(Instruction::AccessLengthInternal, None).unwrap();

        runtime.push_register(i1).unwrap();

        runtime.access_length_internal().unwrap();

        assert_eq!(runtime.get_register().get(0).unwrap(), &0);
    }

    #[test]
    fn access_non_list_on_left_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_symbol("one").unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i1).unwrap();
        runtime.push_register(i2).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_register().get(0).unwrap(), &0);
    }

    #[test]
    fn access_non_symbol_on_right_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_expression(10).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i4).unwrap();
        runtime.push_register(i5).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_register().get(0).unwrap(), &0);
    }

    #[test]
    fn access_no_refs_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let _i4 = runtime.end_list().unwrap();
        let _i5 = runtime.add_symbol("one").unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        let result = runtime.access();

        assert!(result.is_err());
    }

    #[test]
    fn access_with_non_existent_key() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("one").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();
        runtime.start_list(1).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        let i4 = runtime.end_list().unwrap();
        let i5 = runtime.add_symbol("two").unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(i4).unwrap();
        runtime.push_register(i5).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_register().len(), 1);
        assert_eq!(runtime.get_register().get(0).unwrap(), &0);
    }
}
