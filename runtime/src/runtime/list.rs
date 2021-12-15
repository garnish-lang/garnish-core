use log::trace;

use crate::{error, ExpressionDataType, GarnishLangRuntime, GarnishLangRuntimeResult};

use super::data::GarnishLangRuntimeData;

impl<Data> GarnishLangRuntime<Data>
where
    Data: GarnishLangRuntimeData,
{
    pub fn make_list(&mut self, len: usize) -> GarnishLangRuntimeResult {
        trace!("Instruction - Make List | Length - {:?}", len);
        let mut list = vec![];
        let mut associative_list = vec![];

        for _ in 0..len {
            let r = self.next_ref()?;

            match self.data.get_data_type(r)? {
                ExpressionDataType::Pair => {
                    let (left, _) = self.data.get_pair(r)?;

                    match self.data.get_data_type(left)? {
                        ExpressionDataType::Symbol => associative_list.push(r),
                        _ => (),
                    }
                }
                _ => (), // Other values are just simple items
            }

            list.push(r);
        }

        list.reverse();

        // reorder associative values by modulo value
        let mut ordered = vec![0usize; associative_list.len()];
        for index in 0..associative_list.len() {
            let item = associative_list[index];
            let mut i = item % associative_list.len();
            let mut count = 0;
            while ordered[i] != 0 {
                i += 1;
                if i >= associative_list.len() {
                    i = 0;
                }

                count += 1;
                if count > associative_list.len() {
                    return Err(error(format!("Could not place associative value")));
                }
            }

            ordered[i] = item;
        }

        self.push_list(list, ordered)
    }

    pub fn access(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Access");

        let right_ref = self.next_ref()?;
        let left_ref = self.next_ref()?;

        match self.get_access_addr(right_ref, left_ref)? {
            None => self.push_unit(),
            Some(i) => self.push_reference(i),
        }
    }

    pub(crate) fn get_access_addr(&self, sym: usize, list: usize) -> GarnishLangRuntimeResult<Option<usize>> {
        let sym_ref = self.addr_of_raw_data(sym)?;
        let list_ref = self.addr_of_raw_data(list)?;

        match (self.data.get_data_type(list_ref)?, self.data.get_data_type(sym_ref)?) {
            (ExpressionDataType::List, ExpressionDataType::Symbol) => {
                let sym_val = self.data.get_symbol(sym_ref)?;

                let assocations_len = self.data.get_list_associations_len(list_ref)?;

                let mut i = sym_val as usize % assocations_len;
                let mut count = 0;

                loop {
                    // check to make sure item has same symbol
                    let association_ref = self.data.get_list_association(list_ref, i)?;
                    let pair_ref = self.addr_of_raw_data(association_ref)?; // this should be a pair

                    // should have symbol on left
                    match self.data.get_data_type(pair_ref)? {
                        ExpressionDataType::Pair => {
                            let (left, right) = self.data.get_pair(pair_ref)?;

                            let left_ref = self.addr_of_raw_data(left)?;

                            match self.data.get_data_type(left_ref)? {
                                ExpressionDataType::Symbol => {
                                    let v = self.data.get_symbol(left_ref)?;

                                    if v == sym_val {
                                        // found match
                                        // insert pair right as value
                                        return Ok(Some(right));
                                    }
                                }
                                t => Err(error(format!("Association created with non-symbol type {:?} on pair left.", t)))?,
                            }
                        }
                        t => Err(error(format!("Association created with non-pair type {:?}.", t)))?,
                    }

                    i += 1;
                    if i >= assocations_len {
                        i = 0;
                    }

                    count += 1;
                    if count > assocations_len {
                        return Ok(None);
                    }
                }
            }
            (l, r) => Err(error(format!(
                "Access operation with {:?} on the left and {:?} on the right is not supported.",
                l, r
            )))?,
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

        assert_eq!(runtime.data.get_reference(6).unwrap(), 2);
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
