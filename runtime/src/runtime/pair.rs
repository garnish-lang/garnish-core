use log::trace;

use crate::{ExpressionData, GarnishLangRuntime, GarnishLangRuntimeResult};

impl GarnishLangRuntime {
    pub fn make_pair(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Make Pair");
        let right_addr = self.next_raw_ref()?;
        let left_addr = self.next_raw_ref()?;

        self.reference_stack.push(self.data.len());
        self.add_data(ExpressionData::pair(left_addr, right_addr))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{ExpressionData, ExpressionDataType, GarnishLangRuntime, Instruction};

    #[test]
    fn make_pair() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"my_symbol".to_string())).unwrap();

        runtime.reference_stack.push(1);
        runtime.reference_stack.push(2);

        runtime.add_instruction(Instruction::MakePair, None).unwrap();

        runtime.make_pair().unwrap();

        assert_eq!(runtime.data.get(3).unwrap().get_type(), ExpressionDataType::Pair);
        assert_eq!(runtime.data.get(3).unwrap().as_pair().unwrap(), (1, 2));

        assert_eq!(runtime.reference_stack.len(), 1);
        assert_eq!(*runtime.reference_stack.get(0).unwrap(), 3);
    }

    #[test]
    fn make_pair_no_refs_is_err() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::symbol_from_string(&"my_symbol".to_string())).unwrap();

        runtime.add_instruction(Instruction::MakePair, None).unwrap();

        let result = runtime.make_pair();

        assert!(result.is_err());
    }
}
