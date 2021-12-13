use log::trace;

use crate::{error, ExpressionData, ExpressionDataType, GarnishLangRuntime, GarnishLangRuntimeResult};

impl GarnishLangRuntime {
    pub fn equality_comparison(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Equality Comparison");
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

                let result = match (left_data.get_type(), right_data.get_type()) {
                    (ExpressionDataType::Integer, ExpressionDataType::Integer) => {
                        let left = match left_data.as_integer() {
                            Ok(v) => v,
                            Err(e) => Err(error(e))?,
                        };
                        let right = match right_data.as_integer() {
                            Ok(v) => v,
                            Err(e) => Err(error(e))?,
                        };

                        trace!("Comparing {:?} == {:?}", left, right);

                        left == right
                    }
                    (l, r) => Err(error(format!("Comparison between types not implemented {:?} and {:?}", l, r)))?,
                };

                self.data.pop();
                self.data.pop();

                self.add_data(match result {
                    true => ExpressionData::boolean_true(),
                    false => ExpressionData::boolean_false(),
                })?;

                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ExpressionData, GarnishLangRuntime, Instruction};

    #[test]
    fn equality_true() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();

        runtime.reference_stack.push(0);
        runtime.reference_stack.push(1);

        runtime.add_instruction(Instruction::EqualityComparison, None).unwrap();

        runtime.equality_comparison().unwrap();

        assert!(runtime.data.get(0).unwrap().as_boolean().unwrap());
    }

    #[test]
    fn equality_false() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.reference_stack.push(0);
        runtime.reference_stack.push(1);

        runtime.add_instruction(Instruction::EqualityComparison, None).unwrap();

        runtime.equality_comparison().unwrap();

        assert!(!runtime.data.get(0).unwrap().as_boolean().unwrap());
    }
}
