use log::trace;

use crate::{error, ExpressionDataType, GarnishLangRuntime, GarnishLangRuntimeResult};

use super::data::GarnishLangRuntimeData;

impl<Data> GarnishLangRuntime<Data>
where
    Data: GarnishLangRuntimeData,
{
    pub fn equality_comparison(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Equality Comparison");

        let (right_addr, left_addr) = self.next_two_raw_ref()?;

        let result = match (self.data.get_data_type(left_addr)?, self.data.get_data_type(right_addr)?) {
            (ExpressionDataType::Integer, ExpressionDataType::Integer) => {
                let left = self.data.get_integer(left_addr)?;
                let right = self.data.get_integer(right_addr)?;

                trace!("Comparing {:?} == {:?}", left, right);

                left == right
            }
            (l, r) => Err(error(format!("Comparison between types not implemented {:?} and {:?}", l, r)))?,
        };

        self.push_boolean(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::{runtime::data::GarnishLangRuntimeData, ExpressionData, ExpressionDataType, GarnishLangRuntime, Instruction};

    #[test]
    fn equality_true() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();

        runtime.data.push_register(1).unwrap();
        runtime.data.push_register(2).unwrap();

        runtime.add_instruction(Instruction::EqualityComparison, None).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.data.get_register(), &vec![3]);
        assert_eq!(runtime.data.get_data_type(3).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_false() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.data.push_register(1).unwrap();
        runtime.data.push_register(2).unwrap();

        runtime.add_instruction(Instruction::EqualityComparison, None).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.data.get_register(), &vec![3]);
        assert_eq!(runtime.data.get_data_type(3).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn equality_no_references_is_err() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();

        runtime.add_instruction(Instruction::EqualityComparison, None).unwrap();

        let result = runtime.equality_comparison();

        assert!(result.is_err());
    }
}
