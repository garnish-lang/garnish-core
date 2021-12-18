use log::trace;

use crate::{ExpressionDataType, GarnishLangRuntime, GarnishLangRuntimeResult, NestInto};

use super::data::GarnishLangRuntimeData;

impl<Data> GarnishLangRuntime<Data>
where
    Data: GarnishLangRuntimeData,
{
    pub fn make_list(&mut self, len: usize) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - Make List | Length - {:?}", len);

        self.data.start_list(len).nest_into()?;

        for _ in 0..len {
            self.next_ref().and_then(|r| {
                self.data
                    .get_data_type(r)
                    .and_then(|t| match t {
                        ExpressionDataType::Pair => self.data.get_pair(r).and_then(|(left, _)| {
                            self.data.get_data_type(left).and_then(|t| match t {
                                ExpressionDataType::Symbol => Ok(true),
                                _ => Ok(false),
                            })
                        }),
                        _ => Ok(false), // Other values are just simple items
                    })
                    .and_then(|is_associative| self.data.add_to_list(r, is_associative))
                    .nest_into()
            })?
        }

        self.data.end_list().and_then(|r| self.data.push_register(r)).nest_into()
    }

    pub fn access(&mut self) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - Access");

        let right_ref = self.next_ref()?;
        let left_ref = self.next_ref()?;

        match self.get_access_addr(right_ref, left_ref)? {
            None => self.push_unit(),
            Some(i) => self.data.push_register(i).nest_into(),
        }
    }

    pub fn access_left_internal(&mut self) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - Access Left Internal");
        self.next_ref().and_then(|r| match self.data.get_data_type(r).nest_into()? {
            ExpressionDataType::Pair => self.data.get_pair(r).and_then(|(left, _)| self.data.push_register(left)).nest_into(),
            _ => self.push_unit(),
        })
    }

    pub fn access_right_internal(&mut self) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - Access Right Internal");
        self.next_ref().and_then(|r| match self.data.get_data_type(r).nest_into()? {
            ExpressionDataType::Pair => self.data.get_pair(r).and_then(|(_, right)| self.data.push_register(right)).nest_into(),
            _ => self.push_unit(),
        })
    }

    pub fn access_length_internal(&mut self) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - Access Length Internal");
        self.next_ref().and_then(|r| match self.data.get_data_type(r).nest_into()? {
            ExpressionDataType::List => self
                .data
                .get_list_len(r)
                .and_then(|len| self.data.add_integer(len as i64).and_then(|r| self.data.push_register(r)))
                .nest_into(),
            _ => self.push_unit(),
        })
    }

    pub(crate) fn get_access_addr(&self, sym: usize, list: usize) -> GarnishLangRuntimeResult<Data::Error, Option<usize>> {
        let sym_ref = self.addr_of_raw_data(sym)?;
        let list_ref = self.addr_of_raw_data(list)?;

        match (
            self.data.get_data_type(list_ref).nest_into()?,
            self.data.get_data_type(sym_ref).nest_into()?,
        ) {
            (ExpressionDataType::List, ExpressionDataType::Symbol) => {
                let sym_val = self.data.get_symbol(sym_ref).nest_into()?;

                Ok(self.data.get_list_item_with_symbol(list_ref, sym_val).nest_into()?)
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{runtime::data::GarnishLangRuntimeData, ExpressionData, ExpressionDataType, GarnishLangRuntime, Instruction};

    #[test]
    fn make_list() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.data.push_register(1).unwrap();
        runtime.data.push_register(2).unwrap();
        runtime.data.push_register(3).unwrap();

        runtime.add_instruction(Instruction::MakeList, Some(3)).unwrap();

        runtime.make_list(3).unwrap();

        assert_eq!(runtime.data.get_list_len(4).unwrap(), 3);
        assert_eq!(runtime.data.get_list_item(4, 0).unwrap(), 1);
        assert_eq!(runtime.data.get_list_item(4, 1).unwrap(), 2);
        assert_eq!(runtime.data.get_list_item(4, 2).unwrap(), 3);
    }

    #[test]
    fn make_list_no_refs_is_err() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.add_instruction(Instruction::MakeList, Some(3)).unwrap();

        let result = runtime.make_list(3);

        assert!(result.is_err());
    }

    #[test]
    fn make_list_with_associations() {
        let mut runtime = GarnishLangRuntime::simple();

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

        runtime.data.push_register(7).unwrap();
        runtime.data.push_register(8).unwrap();
        runtime.data.push_register(9).unwrap();

        runtime.add_instruction(Instruction::MakeList, Some(3)).unwrap();

        runtime.make_list(3).unwrap();

        assert_eq!(runtime.data.get_list_len(10).unwrap(), 3);
        assert_eq!(runtime.data.get_list_item(10, 0).unwrap(), 7);
        assert_eq!(runtime.data.get_list_item(10, 1).unwrap(), 8);
        assert_eq!(runtime.data.get_list_item(10, 2).unwrap(), 9);

        assert_eq!(runtime.data.get_list_associations_len(10).unwrap(), 3);
        assert_eq!(runtime.data.get_list_association(10, 0).unwrap(), 9);
        assert_eq!(runtime.data.get_list_association(10, 1).unwrap(), 7);
        assert_eq!(runtime.data.get_list_association(10, 2).unwrap(), 8);
    }

    #[test]
    fn access() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();
        runtime.add_data(ExpressionData::list(vec![3], vec![3])).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();

        runtime.add_instruction(Instruction::Access, None).unwrap();

        runtime.data.push_register(4).unwrap();
        runtime.data.push_register(5).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.data.get_register().get(0).unwrap(), &2);
    }

    #[test]
    fn access_pair_left() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();

        runtime.add_instruction(Instruction::AccessLeftInternal, None).unwrap();

        runtime.data.push_register(3).unwrap();

        runtime.access_left_internal().unwrap();

        assert_eq!(runtime.data.get_register().get(0).unwrap(), &1);
    }

    #[test]
    fn access_left_internal_incompatible_type_is_unit() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();

        runtime.add_instruction(Instruction::AccessLeftInternal, None).unwrap();

        runtime.data.push_register(1).unwrap();

        runtime.access_left_internal().unwrap();

        assert_eq!(runtime.data.get_data_type(2).unwrap(), ExpressionDataType::Unit);
        assert_eq!(runtime.data.get_register().get(0).unwrap(), &2);
    }

    #[test]
    fn access_pair_right() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();

        runtime.add_instruction(Instruction::AccessRightInternal, None).unwrap();

        runtime.data.push_register(3).unwrap();

        runtime.access_right_internal().unwrap();

        assert_eq!(runtime.data.get_register().get(0).unwrap(), &2);
    }

    #[test]
    fn access_right_internal_incompatible_type_is_unit() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();

        runtime.add_instruction(Instruction::AccessRightInternal, None).unwrap();

        runtime.data.push_register(1).unwrap();

        runtime.access_right_internal().unwrap();

        assert_eq!(runtime.data.get_data_type(2).unwrap(), ExpressionDataType::Unit);
        assert_eq!(runtime.data.get_register().get(0).unwrap(), &2);
    }

    #[test]
    fn access_list_length() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();
        runtime.add_data(ExpressionData::list(vec![3], vec![3])).unwrap();

        runtime.add_instruction(Instruction::AccessLengthInternal, None).unwrap();

        runtime.data.push_register(4).unwrap();

        runtime.access_length_internal().unwrap();

        assert_eq!(runtime.data.get_integer(5).unwrap(), 1);
        assert_eq!(runtime.data.get_register().get(0).unwrap(), &5);
    }

    #[test]
    fn access_length_internal_incompatible_type_is_unit() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();

        runtime.add_instruction(Instruction::AccessLengthInternal, None).unwrap();

        runtime.data.push_register(1).unwrap();

        runtime.access_length_internal().unwrap();

        assert_eq!(runtime.data.get_data_type(2).unwrap(), ExpressionDataType::Unit);
        assert_eq!(runtime.data.get_register().get(0).unwrap(), &2);
    }

    #[test]
    fn access_non_list_on_left_is_unit() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();

        runtime.add_instruction(Instruction::Access, None).unwrap();

        runtime.data.push_register(1).unwrap();
        runtime.data.push_register(2).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.data.get_data_type(3).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn access_non_symbol_on_right_is_unit() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();
        runtime.add_data(ExpressionData::list(vec![3], vec![3])).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();

        runtime.add_instruction(Instruction::Access, None).unwrap();

        runtime.data.push_register(4).unwrap();
        runtime.data.push_register(5).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.data.get_data_type(6).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn access_no_refs_is_err() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();
        runtime.add_data(ExpressionData::list(vec![3], vec![3])).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();

        runtime.add_instruction(Instruction::Access, None).unwrap();

        let result = runtime.access();

        assert!(result.is_err());
    }

    #[test]
    fn access_with_non_existent_key() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::symbol_from_string(&"one".to_string())).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::pair(1, 2)).unwrap();
        runtime.add_data(ExpressionData::list(vec![3], vec![3])).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"two".to_string())).unwrap();

        runtime.add_instruction(Instruction::Access, None).unwrap();

        runtime.data.push_register(4).unwrap();
        runtime.data.push_register(5).unwrap();

        runtime.access().unwrap();

        assert_eq!(runtime.data.get_register().len(), 1);
        assert_eq!(runtime.data.get_data_type(6).unwrap(), ExpressionDataType::Unit);
    }
}
