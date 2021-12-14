use log::trace;

use crate::{ExpressionData, ExpressionDataType, GarnishLangRuntime, GarnishLangRuntimeResult, RuntimeResult};

impl GarnishLangRuntime {
    pub fn perform_addition(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Addition");
        let (right_data, left_data) = self.next_two_ref_data()?;

        match (left_data.get_type(), right_data.get_type()) {
            (ExpressionDataType::Integer, ExpressionDataType::Integer) => {
                let left = left_data.as_integer().as_runtime_result()?;
                let right = right_data.as_integer().as_runtime_result()?;

                trace!("Performing {:?} + {:?}", left, right);

                self.add_data_ref(ExpressionData::integer(left + right))?;

                Ok(())
            }
            _ => {
                self.add_data_ref(ExpressionData::unit())?;

                Ok(())
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

        runtime.reference_stack.push(1);
        runtime.reference_stack.push(2);

        runtime.perform_addition().unwrap();

        assert_eq!(runtime.reference_stack, vec![3]);
        assert_eq!(runtime.data.get(3).unwrap().bytes, 30i64.to_le_bytes());
    }

    #[test]
    fn perform_addition_no_refs_is_err() {
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
        runtime.add_reference_data(1).unwrap();
        runtime.add_reference_data(2).unwrap();

        runtime.reference_stack.push(1);
        runtime.reference_stack.push(2);

        runtime.perform_addition().unwrap();

        assert_eq!(runtime.reference_stack, vec![5]);
        assert_eq!(runtime.data.get(5).unwrap().bytes, 30i64.to_le_bytes());
    }

    #[test]
    fn perform_addition_with_non_integers() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol(&"sym1".to_string(), 1)).unwrap();
        runtime.add_data(ExpressionData::symbol(&"sym2".to_string(), 2)).unwrap();

        runtime.reference_stack.push(1);
        runtime.reference_stack.push(2);

        runtime.perform_addition().unwrap();

        assert_eq!(runtime.reference_stack, vec![3]);
        assert_eq!(runtime.data.get(3).unwrap().get_type(), ExpressionDataType::Unit);
    }
}
