use log::trace;

use crate::{error, ExpressionData, ExpressionDataType, GarnishLangRuntime, GarnishLangRuntimeResult};

impl GarnishLangRuntime {
    pub fn perform_addition(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Addition");
        match self.reference_stack.len() {
            0 | 1 => Err(error(format!("Not enough data to perform addition operation."))),
            // 2 and greater
            _ => {
                let right_ref = self.reference_stack.pop().unwrap();
                let left_ref = self.reference_stack.pop().unwrap();
                let right_addr = self.addr_of_raw_data(right_ref)?;
                let left_addr = self.addr_of_raw_data(left_ref)?;
                let right_data = self.get_data_internal(right_addr)?;
                let left_data = self.get_data_internal(left_addr)?;

                match (left_data.get_type(), right_data.get_type()) {
                    (ExpressionDataType::Integer, ExpressionDataType::Integer) => {
                        let left = match left_data.as_integer() {
                            Ok(v) => v,
                            Err(e) => Err(error(e))?,
                        };
                        let right = match right_data.as_integer() {
                            Ok(v) => v,
                            Err(e) => Err(error(e))?,
                        };

                        trace!("Performing {:?} + {:?}", left, right);

                        self.data.pop();
                        self.data.pop();

                        self.reference_stack.push(self.data.len());
                        self.add_data(ExpressionData::integer(left + right))?;

                        Ok(())
                    }
                    _ => {
                        self.reference_stack.push(self.data.len());
                        self.add_data(ExpressionData::unit())?;

                        Ok(())
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ExpressionData, ExpressionDataType, GarnishLangRuntime};

    #[test]
    fn perform_addition() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        let result = runtime.perform_addition();

        assert!(result.is_err());
    }

    #[test]
    fn perform_addition_through_references() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_reference_data(0).unwrap();
        runtime.add_reference_data(1).unwrap();

        runtime.reference_stack.push(0);
        runtime.reference_stack.push(1);

        runtime.perform_addition().unwrap();

        assert_eq!(runtime.reference_stack, vec![2]);
        assert_eq!(runtime.data.get(2).unwrap().bytes, 30i64.to_le_bytes());
    }

    #[test]
    fn perform_addition_with_non_integers() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol(&"sym1".to_string(), 1)).unwrap();
        runtime.add_data(ExpressionData::symbol(&"sym2".to_string(), 2)).unwrap();

        runtime.reference_stack.push(0);
        runtime.reference_stack.push(1);

        runtime.perform_addition().unwrap();

        assert_eq!(runtime.reference_stack, vec![2]);
        assert_eq!(runtime.data.get(2).unwrap().get_type(), ExpressionDataType::Unit);
    }
}
