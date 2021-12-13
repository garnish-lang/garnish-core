// use log::trace;

use crate::{error, ExpressionData, ExpressionDataType, GarnishLangRuntime, GarnishLangRuntimeResult};

impl GarnishLangRuntime {
    pub fn add_data(&mut self, data: ExpressionData) -> GarnishLangRuntimeResult<usize> {
        // Check if give a reference of reference
        // flatten reference to point to non-Reference data
        let data = match data.get_type() {
            ExpressionDataType::Reference => match self.data.get(data.as_reference().unwrap()) {
                None => Err(error(format!("Reference given doesn't not exist in data.")))?,
                Some(d) => match d.get_type() {
                    ExpressionDataType::Reference => d.clone(),
                    _ => data,
                },
            },
            ExpressionDataType::Symbol => {
                self.symbols.extend(data.symbols.clone());
                data
            }
            _ => data,
        };

        let addr = self.data.len();
        self.data.push(data);
        Ok(addr)
    }

    pub fn end_constant_data(&mut self) -> GarnishLangRuntimeResult {
        self.end_of_constant_data = self.data.len();

        Ok(())
    }

    pub fn get_end_of_constant_data(&self) -> usize {
        self.end_of_constant_data
    }

    pub fn get_data(&self, index: usize) -> Option<&ExpressionData> {
        self.data.get(index)
    }

    pub fn get_data_len(&self) -> usize {
        self.data.len()
    }

    pub fn add_reference_data(&mut self, reference: usize) -> GarnishLangRuntimeResult {
        self.data.push(ExpressionData::reference(reference));
        Ok(())
    }

    pub fn remove_data(&mut self, from: usize) -> GarnishLangRuntimeResult {
        match from < self.data.len() {
            true => {
                self.data = Vec::from(&self.data[..from]);
                Ok(())
            }
            false => Err(error(format!("Given address is beyond data size."))),
        }
    }

    pub(crate) fn get_data_internal(&self, index: usize) -> GarnishLangRuntimeResult<&ExpressionData> {
        match self.data.get(index) {
            None => Err(error(format!("No data at addr {:?}", index))),
            Some(d) => Ok(d),
        }
    }

    pub(crate) fn addr_of_raw_data(&self, addr: usize) -> GarnishLangRuntimeResult<usize> {
        Ok(match self.data.get(addr) {
            None => Err(error(format!("No data at addr {:?}", addr)))?,
            Some(d) => match d.get_type() {
                ExpressionDataType::Reference => match d.as_reference() {
                    Err(e) => Err(error(e))?,
                    Ok(i) => i,
                },
                _ => addr,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{ExpressionData, GarnishLangRuntime, Instruction};

    #[test]
    fn add_data() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(100)).unwrap();

        assert_eq!(runtime.data.len(), 1);
    }

    #[test]
    fn get_data() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(100)).unwrap();
        runtime.add_data(ExpressionData::integer(200)).unwrap();

        assert_eq!(runtime.get_data(1).unwrap().as_integer().unwrap(), 200);
    }

    #[test]
    fn end_constant_data() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(100)).unwrap();
        runtime.add_data(ExpressionData::integer(200)).unwrap();
        runtime.end_constant_data().unwrap();

        assert_eq!(runtime.get_end_of_constant_data(), 2);
    }

    #[test]
    fn add_data_returns_addr() {
        let mut runtime = GarnishLangRuntime::new();

        assert_eq!(runtime.add_data(ExpressionData::integer(100)).unwrap(), 0);
        assert_eq!(runtime.add_data(ExpressionData::integer(100)).unwrap(), 1);
        assert_eq!(runtime.add_data(ExpressionData::integer(100)).unwrap(), 2);
        assert_eq!(runtime.add_data(ExpressionData::integer(100)).unwrap(), 3);
    }

    #[test]
    fn add_reference_of_reference_falls_through() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(100)).unwrap();
        runtime.add_data(ExpressionData::reference(0)).unwrap();
        runtime.add_data(ExpressionData::reference(1)).unwrap();

        assert_eq!(runtime.data.get(2).unwrap().as_reference().unwrap(), 0);
    }

    #[test]
    fn push_top_reference() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(100)).unwrap();
        runtime.add_reference_data(0).unwrap();

        assert_eq!(runtime.data.len(), 2);
    }

    #[test]
    fn add_symbol() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol(&"false".to_string(), 0)).unwrap();

        let false_sym = runtime.symbols.get("false").unwrap();
        let false_data = runtime.data.get(0).unwrap();

        assert_eq!(false_sym, &0);
        assert_eq!(false_data.as_symbol_name().unwrap(), "false".to_string());
        assert_eq!(false_data.as_symbol_value().unwrap(), 0u64);
    }

    #[test]
    fn remove_data() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol(&"false".to_string(), 0)).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        let addr = runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.remove_data(addr).unwrap();

        assert_eq!(runtime.data.len(), 4);
    }

    #[test]
    fn remove_data_out_of_bounds() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::symbol(&"false".to_string(), 0)).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        let result = runtime.remove_data(10);

        assert!(result.is_err());
    }
}
