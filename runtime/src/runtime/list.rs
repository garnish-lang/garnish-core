use log::trace;

use crate::{error, ExpressionData, ExpressionDataType, GarnishLangRuntime, GarnishLangRuntimeResult};

impl GarnishLangRuntime {
    pub fn make_list(&mut self, len: usize) -> GarnishLangRuntimeResult {
        trace!("Instruction - Make List | Length - {:?}", len);
        match self.reference_stack.len() >= len {
            false => Err(error(format!("Not enough references to make list of size {:?}", len))),
            true => {
                let mut list = vec![];
                let mut associative_list = vec![];

                for _ in 0..len {
                    let r = match self.reference_stack.pop() {
                        Some(i) => self.addr_of_raw_data(i)?,
                        None => Err(error(format!("Not enough references for list of len {:?}", len)))?,
                    };

                    let data = self.get_data_internal(r)?;

                    match data.get_type() {
                        ExpressionDataType::Pair => {
                            let pair = self.get_data_internal(r)?;
                            let left = self.addr_of_raw_data(match pair.as_pair() {
                                Err(e) => Err(error(e))?,
                                Ok((left, _)) => left,
                            })?;

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

                self.reference_stack.push(self.data.len());
                self.add_data(ExpressionData::list(list, ordered))?;

                Ok(())
            }
        }
    }

    pub fn access(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Access");
        match self.reference_stack.len() {
            0 | 1 => Err(error(format!("Not enough references to perform access operation."))),
            _ => {
                let right_ref = self.reference_stack.pop().unwrap();
                let left_ref = self.reference_stack.pop().unwrap();

                match self.get_access_addr(right_ref, left_ref)? {
                    None => {
                        self.reference_stack.push(self.data.len());
                        self.add_data(ExpressionData::unit())?;
                    }
                    Some(i) => {
                        self.add_reference_data(i)?;
                    }
                }
                Ok(())
            }
        }
    }

    pub(crate) fn get_access_addr(&self, sym: usize, list: usize) -> GarnishLangRuntimeResult<Option<usize>> {
        let sym_addr = self.addr_of_raw_data(sym)?;
        let list_addr = self.addr_of_raw_data(list)?;

        let sym_data = self.get_data_internal(sym_addr)?;
        let list_data = self.get_data_internal(list_addr)?;

        match (list_data.get_type(), sym_data.get_type()) {
            (ExpressionDataType::List, ExpressionDataType::Symbol) => {
                let sym_val = match sym_data.as_symbol_value() {
                    Err(e) => Err(error(e))?,
                    Ok(v) => v,
                };

                let (_, assocations) = match list_data.as_list() {
                    Err(e) => Err(error(e))?,
                    Ok(v) => v,
                };

                let mut i = sym_val as usize % assocations.len();
                let mut count = 0;

                loop {
                    // check to make sure item has same symbol
                    let r = self.addr_of_raw_data(assocations[i])?;
                    let data = self.get_data_internal(r)?; // this should be a pair

                    // should have symbol on left
                    match data.get_type() {
                        ExpressionDataType::Pair => {
                            match data.as_pair() {
                                Err(e) => Err(error(e))?,
                                Ok((left, right)) => {
                                    let left_r = self.addr_of_raw_data(left)?;
                                    let left_data = self.get_data_internal(left_r)?;

                                    match left_data.get_type() {
                                        ExpressionDataType::Symbol => {
                                            let v = match left_data.as_symbol_value() {
                                                Err(e) => Err(error(e))?,
                                                Ok(v) => v,
                                            };

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
        let mut runtime = GarnishLangRuntime::new();

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
    fn make_list_with_associations() {
        let mut runtime = GarnishLangRuntime::new();

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
        let mut runtime = GarnishLangRuntime::new();

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
    fn access_with_non_existent_key() {
        let mut runtime = GarnishLangRuntime::new();

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
