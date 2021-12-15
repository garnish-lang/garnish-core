// use log::trace;

use crate::{ExpressionData, ExpressionDataType, GarnishLangRuntime, GarnishLangRuntimeData, GarnishLangRuntimeResult, RuntimeResult};

impl<Data> GarnishLangRuntime<Data>
where
    Data: GarnishLangRuntimeData,
{
    pub fn get_data_pool(&self) -> &Data {
        &self.data
    }

    pub fn add_data(&mut self, data: ExpressionData) -> GarnishLangRuntimeResult<usize> {
        // Check if give a reference of reference
        // flatten reference to point to non-Reference data
        let data = match data.get_type() {
            ExpressionDataType::Reference => {
                let ref_addr = data.as_reference().as_runtime_result()?;
                match self.data.get_data_type(ref_addr)? {
                    ExpressionDataType::Reference => ExpressionData::reference(self.data.get_reference(ref_addr)?),
                    _ => data,
                }
            }
            ExpressionDataType::Symbol => {
                // self.symbols.extend(data.symbols.clone());
                data
            }
            _ => data,
        };

        let addr = self.data.get_data_len();
        self.data.add_data(data.clone())?;
        Ok(addr)
    }

    pub fn end_constant_data(&mut self) -> GarnishLangRuntimeResult {
        self.data.set_end_of_constant(self.data.get_data_len())
    }

    pub fn get_end_of_constant_data(&self) -> usize {
        self.data.get_end_of_constant_data()
    }

    pub fn add_data_ref(&mut self, data: ExpressionData) -> GarnishLangRuntimeResult<usize> {
        let addr = self.add_data(data)?;
        self.data.push_register(addr).unwrap();
        Ok(addr)
    }

    pub fn get_data_len(&self) -> usize {
        self.data.get_data_len()
    }

    pub fn add_reference_data(&mut self, reference: usize) -> GarnishLangRuntimeResult<usize> {
        self.add_data(ExpressionData::reference(reference))
    }

    pub fn remove_non_constant_data(&mut self) -> GarnishLangRuntimeResult {
        self.data.remove_non_constant_data()
    }

    pub(crate) fn next_ref(&mut self) -> GarnishLangRuntimeResult<usize> {
        self.data.pop_register()
    }

    pub(crate) fn next_two_raw_ref(&mut self) -> GarnishLangRuntimeResult<(usize, usize)> {
        let first_ref = self.next_ref()?;
        let second_ref = self.next_ref()?;

        Ok((self.addr_of_raw_data(first_ref)?, self.addr_of_raw_data(second_ref)?))
    }

    pub(crate) fn addr_of_raw_data(&self, addr: usize) -> GarnishLangRuntimeResult<usize> {
        Ok(match self.data.get_data_type(addr)? {
            ExpressionDataType::Reference => self.data.get_reference(addr)?,
            _ => addr,
        })
    }

    // push utilities

    pub fn push_unit(&mut self) -> GarnishLangRuntimeResult {
        self.data.add_unit().and_then(|v| self.data.push_register(v))
    }

    pub fn push_integer(&mut self, value: i64) -> GarnishLangRuntimeResult {
        self.data.add_integer(value).and_then(|v| self.data.push_register(v))
    }

    pub fn push_boolean(&mut self, value: bool) -> GarnishLangRuntimeResult {
        match value {
            true => self.data.add_true(),
            false => self.data.add_false(),
        }
        .and_then(|v| self.data.push_register(v))
    }

    pub fn push_list(&mut self, list: Vec<usize>, associations: Vec<usize>) -> GarnishLangRuntimeResult {
        self.data.add_list(list, associations).and_then(|v| self.data.push_register(v))
    }

    pub fn push_reference(&mut self, value: usize) -> GarnishLangRuntimeResult {
        self.data.add_reference(value).and_then(|v| self.data.push_register(v))
    }

    pub fn push_pair(&mut self, left: usize, right: usize) -> GarnishLangRuntimeResult {
        self.data.add_pair((left, right)).and_then(|v| self.data.push_register(v))
    }
}

#[cfg(test)]
mod tests {
    use crate::{runtime::data::GarnishLangRuntimeData, ExpressionData, GarnishLangRuntime, Instruction};

    #[test]
    fn add_data() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(100)).unwrap();

        assert_eq!(runtime.get_data_len(), 2);
    }

    #[test]
    fn add_data_ref() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data_ref(ExpressionData::integer(100)).unwrap();

        assert_eq!(runtime.data.get_register(), &vec![1]);
        assert_eq!(runtime.get_data_len(), 2);
    }

    #[test]
    fn get_data() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(100)).unwrap();
        runtime.add_data(ExpressionData::integer(200)).unwrap();

        assert_eq!(runtime.data.get_integer(2).unwrap(), 200);
    }

    #[test]
    fn end_constant_data() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(100)).unwrap();
        runtime.add_data(ExpressionData::integer(200)).unwrap();
        runtime.end_constant_data().unwrap();

        assert_eq!(runtime.get_end_of_constant_data(), 3);
    }

    #[test]
    fn add_data_returns_addr() {
        let mut runtime = GarnishLangRuntime::simple();

        assert_eq!(runtime.add_data(ExpressionData::integer(100)).unwrap(), 1);
        assert_eq!(runtime.add_data(ExpressionData::integer(100)).unwrap(), 2);
        assert_eq!(runtime.add_data(ExpressionData::integer(100)).unwrap(), 3);
        assert_eq!(runtime.add_data(ExpressionData::integer(100)).unwrap(), 4);
    }

    #[test]
    fn add_reference_of_reference_falls_through() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(100)).unwrap();
        runtime.add_data(ExpressionData::reference(1)).unwrap();
        runtime.add_data(ExpressionData::reference(2)).unwrap();

        assert_eq!(runtime.data.get_reference(3).unwrap(), 1);
    }

    #[test]
    fn push_top_reference() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(100)).unwrap();
        runtime.add_reference_data(0).unwrap();

        assert_eq!(runtime.get_data_len(), 3);
    }

    #[test]
    fn remove_data() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::symbol(&"false".to_string(), 0)).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.end_constant_data().unwrap();

        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.remove_non_constant_data().unwrap();

        assert_eq!(runtime.get_data_len(), 5);
    }
}

#[cfg(test)]
mod internal {
    use crate::{ExpressionData, GarnishLangRuntime, GarnishLangRuntimeData};

    #[test]
    fn next_ref() {
        let mut runtime = GarnishLangRuntime::simple();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.data.push_register(2).unwrap();

        let result = runtime.next_ref().unwrap();

        assert_eq!(result, 2);
    }

    #[test]
    fn next_ref_data_no_ref_is_error() {
        let mut runtime = GarnishLangRuntime::simple();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        let result = runtime.next_ref();

        assert!(result.is_err());
    }

    #[test]
    fn next_two_ref_data() {
        let mut runtime = GarnishLangRuntime::simple();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.data.push_register(1).unwrap();
        runtime.data.push_register(2).unwrap();

        let (first, second) = runtime.next_two_raw_ref().unwrap();

        assert_eq!(first, 2);
        assert_eq!(second, 1);
    }

    #[test]
    fn next_two_ref_data_one_ref_is_error() {
        let mut runtime = GarnishLangRuntime::simple();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.data.push_register(1).unwrap();

        let result = runtime.next_two_raw_ref();

        assert!(result.is_err());
    }

    #[test]
    fn next_two_ref_data_zero_refs_is_error() {
        let mut runtime = GarnishLangRuntime::simple();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        let result = runtime.next_two_raw_ref();

        assert!(result.is_err());
    }
}
