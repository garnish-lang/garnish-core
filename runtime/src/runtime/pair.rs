use log::trace;

use crate::{GarnishLangRuntime, GarnishLangRuntimeResult};

use super::data::GarnishLangRuntimeData;

impl<Data> GarnishLangRuntime<Data>
where
    Data: GarnishLangRuntimeData,
{
    pub fn make_pair(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Make Pair");

        let (right_addr, left_addr) = self.next_two_raw_ref()?;

        self.push_pair(left_addr, right_addr)
    }
}

#[cfg(test)]
mod tests {
    use crate::{runtime::data::GarnishLangRuntimeData, ExpressionData, ExpressionDataType, GarnishLangRuntime, Instruction};

    #[test]
    fn make_pair() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"my_symbol".to_string())).unwrap();

        runtime.data.push_register(1).unwrap();
        runtime.data.push_register(2).unwrap();

        runtime.add_instruction(Instruction::MakePair, None).unwrap();

        runtime.make_pair().unwrap();

        assert_eq!(runtime.data.get_data_type(3).unwrap(), ExpressionDataType::Pair);
        assert_eq!(runtime.data.get_pair(3).unwrap(), (1, 2));

        assert_eq!(runtime.data.get_register().len(), 1);
        assert_eq!(*runtime.data.get_register().get(0).unwrap(), 3);
    }

    #[test]
    fn make_pair_no_refs_is_err() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"my_symbol".to_string())).unwrap();

        runtime.add_instruction(Instruction::MakePair, None).unwrap();

        let result = runtime.make_pair();

        assert!(result.is_err());
    }
}
