use log::trace;

use crate::{next_ref, push_unit, ExpressionDataType, GarnishLangRuntimeData, GarnishLangRuntimeResult, NestInto};

pub(crate) fn make_list<Data: GarnishLangRuntimeData>(this: &mut Data, len: usize) -> GarnishLangRuntimeResult<Data::Error> {
    trace!("Instruction - Make List | Length - {:?}", len);

    this.start_list(len).nest_into()?;

    for _ in 0..len {
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
        })?
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
            .and_then(|len| this.add_integer(len as i64).and_then(|r| this.push_register(r)))
            .nest_into(),
        _ => push_unit(this),
    })
}

pub(crate) fn get_access_addr<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    sym: usize,
    list: usize,
) -> GarnishLangRuntimeResult<Data::Error, Option<usize>> {
    let sym_ref = sym;
    let list_ref = list;

    match (this.get_data_type(list_ref).nest_into()?, this.get_data_type(sym_ref).nest_into()?) {
        (ExpressionDataType::List, ExpressionDataType::Symbol) => {
            let sym_val = this.get_symbol(sym_ref).nest_into()?;

            Ok(this.get_list_item_with_symbol(list_ref, sym_val).nest_into()?)
        }
        (ExpressionDataType::List, ExpressionDataType::Integer) => {
            let i = this.get_integer(sym).nest_into()?;

            if i < 0 {
                Ok(None)
            } else {
                let i = i as usize;
                if i >= this.get_list_len(list).nest_into()? {
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
    use crate::{runtime::GarnishRuntime, ExpressionData, ExpressionDataType, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

    #[test]
    fn make_list() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.push_register(1).unwrap();
        runtime.push_register(2).unwrap();
        runtime.push_register(3).unwrap();

        runtime.push_instruction(Instruction::MakeList, Some(3)).unwrap();

        runtime.make_list(3).unwrap();

        assert_eq!(runtime.get_list_len(4).unwrap(), 3);
        assert_eq!(runtime.get_list_item(4, 0).unwrap(), 1);
        assert_eq!(runtime.get_list_item(4, 1).unwrap(), 2);
        assert_eq!(runtime.get_list_item(4, 2).unwrap(), 3);
    }

    #[test]
    fn make_list_no_refs_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.push_instruction(Instruction::MakeList, Some(3)).unwrap();

        let result = runtime.make_list(3);

        assert!(result.is_err());
    }

    #[test]
    fn make_list_with_associations() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"two".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"three".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(30)).unwrap();
        // 6
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();
        runtime.add_data(ExpressionData::pair(3, 4)).unwrap();
        runtime.add_data(ExpressionData::pair(5, 6)).unwrap();

        runtime.push_register(7).unwrap();
        runtime.push_register(8).unwrap();
        runtime.push_register(9).unwrap();

        runtime.push_instruction(Instruction::MakeList, Some(3)).unwrap();

        runtime.make_list(3).unwrap();

        assert_eq!(runtime.get_list_len(10).unwrap(), 3);
        assert_eq!(runtime.get_list_item(10, 0).unwrap(), 7);
        assert_eq!(runtime.get_list_item(10, 1).unwrap(), 8);
        assert_eq!(runtime.get_list_item(10, 2).unwrap(), 9);

        assert_eq!(runtime.get_list_associations_len(10).unwrap(), 3);
        assert_eq!(runtime.get_list_association(10, 0).unwrap(), 9);
        assert_eq!(runtime.get_list_association(10, 1).unwrap(), 7);
        assert_eq!(runtime.get_list_association(10, 2).unwrap(), 8);
    }

    #[test]
    fn access() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();
        runtime.add_data(ExpressionData::list(vec![3], vec![3])).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(4).unwrap();
        runtime.push_register(5).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_register().get(0).unwrap(), &2);
    }

    #[test]
    fn access_with_number() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();
        runtime.add_data(ExpressionData::list(vec![3], vec![3])).unwrap();
        runtime.add_data(ExpressionData::integer(0)).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(4).unwrap();
        runtime.push_register(5).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_register().get(0).unwrap(), &3);
    }

    #[test]
    fn access_with_number_out_of_bounds_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();
        runtime.add_data(ExpressionData::list(vec![3], vec![3])).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(4).unwrap();
        runtime.push_register(5).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_data_type(6).unwrap(), ExpressionDataType::Unit);
        assert_eq!(runtime.get_register().get(0).unwrap(), &6);
    }

    #[test]
    fn access_with_number_negative_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();
        runtime.add_data(ExpressionData::list(vec![1, 2, 3], vec![1, 2, 3])).unwrap();
        runtime.add_data(ExpressionData::integer(-1)).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(4).unwrap();
        runtime.push_register(5).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_data_type(6).unwrap(), ExpressionDataType::Unit);
        assert_eq!(runtime.get_register().get(0).unwrap(), &6);
    }

    #[test]
    fn access_pair_left() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();

        runtime.push_instruction(Instruction::AccessLeftInternal, None).unwrap();

        runtime.push_register(3).unwrap();

        runtime.access_left_internal().unwrap();

        assert_eq!(runtime.get_register().get(0).unwrap(), &1);
    }

    #[test]
    fn access_left_internal_incompatible_type_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();

        runtime.push_instruction(Instruction::AccessLeftInternal, None).unwrap();

        runtime.push_register(1).unwrap();

        runtime.access_left_internal().unwrap();

        assert_eq!(runtime.get_data_type(2).unwrap(), ExpressionDataType::Unit);
        assert_eq!(runtime.get_register().get(0).unwrap(), &2);
    }

    #[test]
    fn access_pair_right() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();

        runtime.push_instruction(Instruction::AccessRightInternal, None).unwrap();

        runtime.push_register(3).unwrap();

        runtime.access_right_internal().unwrap();

        assert_eq!(runtime.get_register().get(0).unwrap(), &2);
    }

    #[test]
    fn access_right_internal_incompatible_type_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();

        runtime.push_instruction(Instruction::AccessRightInternal, None).unwrap();

        runtime.push_register(1).unwrap();

        runtime.access_right_internal().unwrap();

        assert_eq!(runtime.get_data_type(2).unwrap(), ExpressionDataType::Unit);
        assert_eq!(runtime.get_register().get(0).unwrap(), &2);
    }

    #[test]
    fn access_list_length() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();
        runtime.add_data(ExpressionData::list(vec![3], vec![3])).unwrap();

        runtime.push_instruction(Instruction::AccessLengthInternal, None).unwrap();

        runtime.push_register(4).unwrap();

        runtime.access_length_internal().unwrap();

        assert_eq!(runtime.get_integer(5).unwrap(), 1);
        assert_eq!(runtime.get_register().get(0).unwrap(), &5);
    }

    #[test]
    fn access_length_internal_incompatible_type_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();

        runtime.push_instruction(Instruction::AccessLengthInternal, None).unwrap();

        runtime.push_register(1).unwrap();

        runtime.access_length_internal().unwrap();

        assert_eq!(runtime.get_data_type(2).unwrap(), ExpressionDataType::Unit);
        assert_eq!(runtime.get_register().get(0).unwrap(), &2);
    }

    #[test]
    fn access_non_list_on_left_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(1).unwrap();
        runtime.push_register(2).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_data_type(3).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn access_non_symbol_on_right_is_unit() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();
        runtime.add_data(ExpressionData::list(vec![3], vec![3])).unwrap();
        runtime.add_data(ExpressionData::expression(10)).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(4).unwrap();
        runtime.push_register(5).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_data_type(6).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn access_no_refs_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();
        runtime.add_data(ExpressionData::list(vec![3], vec![3])).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        let result = runtime.access();

        assert!(result.is_err());
    }

    #[test]
    fn access_with_non_existent_key() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();
        runtime.add_data(ExpressionData::list(vec![3], vec![3])).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"two".to_string())).unwrap();

        runtime.push_instruction(Instruction::Access, None).unwrap();

        runtime.push_register(4).unwrap();
        runtime.push_register(5).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.get_register().len(), 1);
        assert_eq!(runtime.get_data_type(6).unwrap(), ExpressionDataType::Unit);
    }
}
