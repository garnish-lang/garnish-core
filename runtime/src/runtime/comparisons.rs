use log::trace;

use crate::{error, ExpressionData, ExpressionDataType, GarnishLangRuntime, GarnishLangRuntimeResult, RuntimeResult};

impl GarnishLangRuntime {
    pub fn equality_comparison(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Equality Comparison");

        let (right_data, left_data) = self.next_two_ref_data()?;

        let result = match (left_data.get_type(), right_data.get_type()) {
            (ExpressionDataType::Integer, ExpressionDataType::Integer) => {
                let left = left_data.as_integer().as_runtime_result()?;
                let right = right_data.as_integer().as_runtime_result()?;

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

#[cfg(test)]
mod tests {
    use crate::{ExpressionData, GarnishLangRuntime, Instruction};

    #[test]
    fn equality_true() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();

        runtime.reference_stack.push(1);
        runtime.reference_stack.push(2);

        runtime.add_instruction(Instruction::EqualityComparison, None).unwrap();

        runtime.equality_comparison().unwrap();

        assert!(runtime.data.get(1).unwrap().as_boolean().unwrap());
    }

    #[test]
    fn equality_false() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.reference_stack.push(1);
        runtime.reference_stack.push(2);

        runtime.add_instruction(Instruction::EqualityComparison, None).unwrap();

        runtime.equality_comparison().unwrap();

        assert!(!runtime.data.get(1).unwrap().as_boolean().unwrap());
    }
    #[test]
    fn equality_no_references_is_err() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();

        runtime.add_instruction(Instruction::EqualityComparison, None).unwrap();

        let result = runtime.equality_comparison();

        assert!(result.is_err());
    }
}
