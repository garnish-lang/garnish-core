use log::trace;

use crate::{ExpressionDataType, GarnishLangRuntime, GarnishLangRuntimeResult};

use super::data::GarnishLangRuntimeDataPool;

impl<Data> GarnishLangRuntime<Data>
where
    Data: GarnishLangRuntimeDataPool,
{
    pub fn perform_addition(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Addition");

        let (right_addr, left_addr) = self.next_two_raw_ref()?;

        match (self.data.get_data_type(left_addr)?, self.data.get_data_type(right_addr)?) {
            (ExpressionDataType::Integer, ExpressionDataType::Integer) => {
                let left = self.data.get_integer(left_addr)?;
                let right = self.data.get_integer(right_addr)?;

                trace!("Performing {:?} + {:?}", left, right);

                self.push_integer(left + right)
            }
            _ => self.push_unit(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{runtime::data::GarnishLangRuntimeDataPool, ExpressionData, ExpressionDataType, GarnishLangRuntime};

    #[test]
    fn perform_addition() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.data.push_register(1).unwrap();
        runtime.data.push_register(2).unwrap();

        runtime.perform_addition().unwrap();

        assert_eq!(runtime.data.get_register(), &vec![3]);
        assert_eq!(runtime.data.get_integer(3).unwrap(), 30);
    }

    #[test]
    fn perform_addition_no_refs_is_err() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        let result = runtime.perform_addition();

        assert!(result.is_err());
    }

    #[test]
    fn perform_addition_through_references() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_reference_data(1).unwrap();
        runtime.add_reference_data(2).unwrap();

        runtime.data.push_register(1).unwrap();
        runtime.data.push_register(2).unwrap();

        runtime.perform_addition().unwrap();

        assert_eq!(runtime.data.get_register(), &vec![5]);
        assert_eq!(runtime.data.get_integer(5).unwrap(), 30);
    }

    #[test]
    fn perform_addition_with_non_integers() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::symbol(&"sym1".to_string(), 1)).unwrap();
        runtime.add_data(ExpressionData::symbol(&"sym2".to_string(), 2)).unwrap();

        runtime.data.push_register(1).unwrap();
        runtime.data.push_register(2).unwrap();

        runtime.perform_addition().unwrap();

        assert_eq!(runtime.data.get_register(), &vec![3]);
        assert_eq!(runtime.data.get_data_type(3).unwrap(), ExpressionDataType::Unit);
    }
}
