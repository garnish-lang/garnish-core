// use log::trace;

use crate::{error, ExpressionData, ExpressionDataType, GarnishLangRuntime, GarnishLangRuntimeResult, RuntimeResult};

pub trait GarnishLangRuntimeDataPool {
    fn new() -> Self;

    fn get_data_type(&self, index: usize) -> GarnishLangRuntimeResult<ExpressionDataType>;
    fn get_integer(&self, index: usize) -> GarnishLangRuntimeResult<i64>;
    fn get_reference(&self, index: usize) -> GarnishLangRuntimeResult<usize>;
    fn get_symbol(&self, index: usize) -> GarnishLangRuntimeResult<u64>;
    fn get_expression(&self, index: usize) -> GarnishLangRuntimeResult<usize>;
    fn get_external(&self, index: usize) -> GarnishLangRuntimeResult<usize>;
    fn get_pair(&self, index: usize) -> GarnishLangRuntimeResult<(usize, usize)>;
    fn get_list_len(&self, index: usize) -> GarnishLangRuntimeResult<usize>;
    fn get_list_item(&self, list_index: usize, item_index: usize) -> GarnishLangRuntimeResult<usize>;

    fn add_integer(&mut self, value: i64) -> GarnishLangRuntimeResult<usize>;
    fn add_reference(&mut self, value: usize) -> GarnishLangRuntimeResult<usize>;
    fn add_symbol(&mut self, value: u64) -> GarnishLangRuntimeResult<usize>;
    fn add_expression(&mut self, value: usize) -> GarnishLangRuntimeResult<usize>;
    fn add_external(&mut self, value: usize) -> GarnishLangRuntimeResult<usize>;
    fn add_pair(&mut self, value: (usize, usize)) -> GarnishLangRuntimeResult<usize>;
    fn add_list(&mut self, value: Vec<usize>) -> GarnishLangRuntimeResult<usize>;
}

pub struct SimpleRuntimeData {
    data: Vec<ExpressionData>,
}

impl SimpleRuntimeData {
    fn get(&self, index: usize) -> GarnishLangRuntimeResult<&ExpressionData> {
        match self.data.get(index) {
            None => Err(error(format!("No data at addr {:?}", index))),
            Some(d) => Ok(d),
        }
    }
}

impl GarnishLangRuntimeDataPool for SimpleRuntimeData {
    fn new() -> Self {
        SimpleRuntimeData {
            data: vec![ExpressionData::unit()],
        }
    }

    fn get_data_type(&self, index: usize) -> GarnishLangRuntimeResult<ExpressionDataType> {
        Ok(self.get(index)?.get_type())
    }

    fn get_integer(&self, index: usize) -> GarnishLangRuntimeResult<i64> {
        self.get(index)?.as_integer().as_runtime_result()
    }

    fn get_reference(&self, index: usize) -> GarnishLangRuntimeResult<usize> {
        self.get(index)?.as_reference().as_runtime_result()
    }

    fn get_symbol(&self, index: usize) -> GarnishLangRuntimeResult<u64> {
        self.get(index)?.as_symbol_value().as_runtime_result()
    }

    fn get_expression(&self, index: usize) -> GarnishLangRuntimeResult<usize> {
        self.get(index)?.as_expression().as_runtime_result()
    }

    fn get_external(&self, index: usize) -> GarnishLangRuntimeResult<usize> {
        self.get(index)?.as_external().as_runtime_result()
    }

    fn get_pair(&self, index: usize) -> GarnishLangRuntimeResult<(usize, usize)> {
        self.get(index)?.as_pair().as_runtime_result()
    }

    fn get_list_len(&self, index: usize) -> GarnishLangRuntimeResult<usize> {
        Ok(self.get(index)?.as_list().as_runtime_result()?.0.len())
    }

    fn get_list_item(&self, list_index: usize, item_index: usize) -> GarnishLangRuntimeResult<usize> {
        match self.get(list_index)?.as_list().as_runtime_result()?.0.get(item_index) {
            None => Err(error(format!("No list item at index {:?} for list at addr {:?}", item_index, list_index))),
            Some(v) => Ok(*v),
        }
    }

    fn add_integer(&mut self, value: i64) -> GarnishLangRuntimeResult<usize> {
        self.data.push(ExpressionData::integer(value));
        Ok(self.data.len() - 1)
    }

    fn add_reference(&mut self, value: usize) -> GarnishLangRuntimeResult<usize> {
        self.data.push(ExpressionData::reference(value));
        Ok(self.data.len() - 1)
    }

    fn add_symbol(&mut self, value: u64) -> GarnishLangRuntimeResult<usize> {
        self.data.push(ExpressionData::symbol(&"".to_string(), value));
        Ok(self.data.len() - 1)
    }

    fn add_expression(&mut self, value: usize) -> GarnishLangRuntimeResult<usize> {
        self.data.push(ExpressionData::expression(value));
        Ok(self.data.len() - 1)
    }

    fn add_external(&mut self, value: usize) -> GarnishLangRuntimeResult<usize> {
        self.data.push(ExpressionData::external(value));
        Ok(self.data.len() - 1)
    }

    fn add_pair(&mut self, value: (usize, usize)) -> GarnishLangRuntimeResult<usize> {
        self.data.push(ExpressionData::pair(value.0, value.1));
        Ok(self.data.len() - 1)
    }

    fn add_list(&mut self, value: Vec<usize>) -> GarnishLangRuntimeResult<usize> {
        self.data.push(ExpressionData::list(value, vec![]));
        Ok(self.data.len() - 1)
    }
}

impl<Data> GarnishLangRuntime<Data>
where
    Data: GarnishLangRuntimeDataPool,
{
    pub fn add_data(&mut self, data: ExpressionData) -> GarnishLangRuntimeResult<usize> {
        // Check if give a reference of reference
        // flatten reference to point to non-Reference data
        let data = match data.get_type() {
            ExpressionDataType::Reference => match self.data.get(data.as_reference().as_runtime_result()?) {
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

    pub fn add_data_ref(&mut self, data: ExpressionData) -> GarnishLangRuntimeResult<usize> {
        let addr = self.add_data(data)?;
        self.reference_stack.push(addr);
        Ok(addr)
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
                ExpressionDataType::Reference => d.as_reference().as_runtime_result()?,
                _ => addr,
            },
        })
    }

    pub(crate) fn get_raw_data_internal(&self, addr: usize) -> GarnishLangRuntimeResult<&ExpressionData> {
        let addr = self.addr_of_raw_data(addr)?;
        self.get_data_internal(addr)
    }

    pub(crate) fn next_ref_data(&mut self) -> GarnishLangRuntimeResult<&ExpressionData> {
        let r = self.next_ref()?;
        let addr = self.addr_of_raw_data(r)?;
        self.get_data_internal(addr)
    }

    pub(crate) fn next_two_ref_data<'a>(&'a mut self) -> GarnishLangRuntimeResult<(&'a ExpressionData, &'a ExpressionData)> {
        let first_ref = self.next_ref()?;
        let second_ref = self.next_ref()?;
        let first = self.get_raw_data_internal(first_ref)?;
        let second = self.get_raw_data_internal(second_ref)?;

        Ok((first, second))
    }

    pub(crate) fn next_ref(&mut self) -> GarnishLangRuntimeResult<usize> {
        match self.reference_stack.pop() {
            None => Err(error(format!("No references left.")))?,
            Some(r) => Ok(r),
        }
    }

    pub(crate) fn next_raw_ref(&mut self) -> GarnishLangRuntimeResult<usize> {
        let r = self.next_ref()?;
        self.addr_of_raw_data(r)
    }
}

#[cfg(test)]
mod tests {
    use crate::{ExpressionData, GarnishLangRuntime, Instruction};

    #[test]
    fn add_data() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(100)).unwrap();

        assert_eq!(runtime.data.len(), 2);
    }

    #[test]
    fn add_data_ref() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data_ref(ExpressionData::integer(100)).unwrap();

        assert_eq!(runtime.reference_stack, vec![1]);
        assert_eq!(runtime.data.len(), 2);
    }

    #[test]
    fn get_data() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(100)).unwrap();
        runtime.add_data(ExpressionData::integer(200)).unwrap();

        assert_eq!(runtime.get_data(2).unwrap().as_integer().unwrap(), 200);
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

        assert_eq!(runtime.data.get(3).unwrap().as_reference().unwrap(), 1);
    }

    #[test]
    fn push_top_reference() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(100)).unwrap();
        runtime.add_reference_data(0).unwrap();

        assert_eq!(runtime.data.len(), 3);
    }

    #[test]
    fn add_symbol() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::symbol(&"false".to_string(), 0)).unwrap();

        let false_sym = runtime.symbols.get("false").unwrap();
        let false_data = runtime.data.get(1).unwrap();

        assert_eq!(false_sym, &0);
        assert_eq!(false_data.as_symbol_name().unwrap(), "false".to_string());
        assert_eq!(false_data.as_symbol_value().unwrap(), 0u64);
    }

    #[test]
    fn remove_data() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::symbol(&"false".to_string(), 0)).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        let addr = runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.remove_data(addr).unwrap();

        assert_eq!(runtime.data.len(), 5);
    }

    #[test]
    fn remove_data_out_of_bounds() {
        let mut runtime = GarnishLangRuntime::simple();

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

#[cfg(test)]
mod internal {
    use crate::{ExpressionData, GarnishLangRuntime};

    #[test]
    fn next_ref_data() {
        let mut runtime = GarnishLangRuntime::simple();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.reference_stack.push(2);

        let result = runtime.next_ref_data();

        assert_eq!(result.unwrap().as_integer().unwrap(), 20);
    }

    #[test]
    fn next_ref_data_no_ref_is_error() {
        let mut runtime = GarnishLangRuntime::simple();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        let result = runtime.next_ref_data();

        assert!(result.is_err());
    }

    #[test]
    fn next_two_ref_data() {
        let mut runtime = GarnishLangRuntime::simple();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.reference_stack.push(1);
        runtime.reference_stack.push(2);

        let result = runtime.next_two_ref_data();

        let (first, second) = result.unwrap();

        assert_eq!(first.as_integer().unwrap(), 20);
        assert_eq!(second.as_integer().unwrap(), 10);
    }

    #[test]
    fn next_two_ref_data_one_ref_is_error() {
        let mut runtime = GarnishLangRuntime::simple();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.reference_stack.push(1);

        let result = runtime.next_two_ref_data();

        assert!(result.is_err());
    }

    #[test]
    fn next_two_ref_data_zero_refs_is_error() {
        let mut runtime = GarnishLangRuntime::simple();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        let result = runtime.next_two_ref_data();

        assert!(result.is_err());
    }
}
