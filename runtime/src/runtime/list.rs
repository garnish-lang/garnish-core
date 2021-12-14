use log::trace;

use crate::{error, result::RuntimeResult, ExpressionData, ExpressionDataType, GarnishLangRuntime, GarnishLangRuntimeResult};

use super::data::GarnishLangRuntimeDataPool;

impl<Data> GarnishLangRuntime<Data>
where
    Data: GarnishLangRuntimeDataPool,
{
    pub fn make_list(&mut self, len: usize) -> GarnishLangRuntimeResult {
        trace!("Instruction - Make List | Length - {:?}", len);
        let mut list = vec![];
        let mut associative_list = vec![];

        for _ in 0..len {
            let r = self.next_ref()?;
            let data = self.get_raw_data_internal(r)?;

            match data.get_type() {
                ExpressionDataType::Pair => {
                    let pair = self.get_data_internal(r)?;
                    let left = self.addr_of_raw_data(pair.as_pair().as_runtime_result()?.0)?;

                    let left_data = self.get_data_internal(left)?;
                    match left_data.get_type() {
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

        self.add_data_ref(ExpressionData::list(list, ordered))?;

        Ok(())
    }

    pub fn access(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Access");

        let right_ref = self.next_ref()?;
        let left_ref = self.next_ref()?;

        match self.get_access_addr(right_ref, left_ref)? {
            None => {
                self.add_data_ref(ExpressionData::unit())?;
            }
            Some(i) => {
                self.add_reference_data(i)?;
            }
        }
        Ok(())
    }

    pub(crate) fn get_access_addr(&self, sym: usize, list: usize) -> GarnishLangRuntimeResult<Option<usize>> {
        let sym_data = self.get_raw_data_internal(sym)?;
        let list_data = self.get_raw_data_internal(list)?;

        match (list_data.get_type(), sym_data.get_type()) {
            (ExpressionDataType::List, ExpressionDataType::Symbol) => {
                let sym_val = sym_data.as_symbol_value().as_runtime_result()?;

                let (_, assocations) = list_data.as_list().as_runtime_result()?;

                let mut i = sym_val as usize % assocations.len();
                let mut count = 0;

                loop {
                    // check to make sure item has same symbol
                    let data = self.get_raw_data_internal(assocations[i])?; // this should be a pair

                    // should have symbol on left
                    match data.get_type() {
                        ExpressionDataType::Pair => {
                            match data.as_pair() {
                                Err(e) => Err(error(e))?,
                                Ok((left, right)) => {
                                    let left_data = self.get_raw_data_internal(left)?;

                                    match left_data.get_type() {
                                        ExpressionDataType::Symbol => {
                                            let v = left_data.as_symbol_value().as_runtime_result()?;

                                            if v == sym_val {
                                                // found match
                                                // insert pair right as value
                                                return Ok(Some(right));
                                            }
                                        }
                                        t => Err(error(format!("Association created with non-symbol type {:?} on pair left.", t)))?,
                                    }
                                }
                            };
                        }
                        t => Err(error(format!("Association created with non-pair type {:?}.", t)))?,
                    }

                    i += 1;
                    if i >= assocations.len() {
                        i = 0;
                    }

                    count += 1;
                    if count > assocations.len() {
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
    use crate::{ExpressionData, ExpressionDataType, GarnishLangRuntime, Instruction};

    #[test]
    fn make_list() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.reference_stack.push(1);
        runtime.reference_stack.push(2);
        runtime.reference_stack.push(3);

        runtime.add_instruction(Instruction::MakeList, Some(3)).unwrap();

        runtime.make_list(3).unwrap();

        let list = runtime.data.get(4).unwrap().as_list().unwrap();
        assert_eq!(list, (vec![1, 2, 3], vec![]));
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

        runtime.reference_stack.push(7);
        runtime.reference_stack.push(8);
        runtime.reference_stack.push(9);

        runtime.add_instruction(Instruction::MakeList, Some(3)).unwrap();

        runtime.make_list(3).unwrap();

        let list = runtime.data.get(10).unwrap().as_list().unwrap();
        assert_eq!(list, (vec![7, 8, 9], vec![9, 7, 8]));
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

        runtime.reference_stack.push(4);
        runtime.reference_stack.push(5);

        runtime.access().unwrap();

        assert_eq!(runtime.get_data(6).unwrap().as_reference().unwrap(), 2);
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

        runtime.reference_stack.push(4);
        runtime.reference_stack.push(5);

        runtime.access().unwrap();

        assert_eq!(runtime.reference_stack.len(), 1);
        assert_eq!(runtime.get_data(6).unwrap().get_type(), ExpressionDataType::Unit);
    }
}
