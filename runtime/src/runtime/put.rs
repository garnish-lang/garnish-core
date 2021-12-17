use log::trace;

use crate::{error, GarnishLangRuntime, GarnishLangRuntimeResult, NestInto};

use super::data::GarnishLangRuntimeData;

impl<Data> GarnishLangRuntime<Data>
where
    Data: GarnishLangRuntimeData,
{
    pub fn put(&mut self, i: usize) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - Put | Data - {:?}", i);
        match i >= self.data.get_end_of_constant_data() {
            true => Err(error(format!(
                "Attempting to put reference to {:?} which is out of bounds of constant data that ends at {:?}.",
                i,
                self.data.get_end_of_constant_data()
            ))),
            false => self.data.push_register(i).nest_into(),
        }
    }

    pub fn put_input(&mut self) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - Put Input");

        match self.data.get_current_input() {
            None => self.push_unit(),
            Some(i) => self.data.push_register(i).nest_into(),
        }
    }

    pub fn push_input(&mut self) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - Push Input");
        let r = self.next_ref()?;

        self.data.push_input(r).nest_into()?;
        self.data.set_result(Some(r)).nest_into()
    }

    pub fn put_result(&mut self) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - Put Result");

        match self.data.get_result() {
            None => self.push_unit(),
            Some(i) => self.data.push_register(i).nest_into(),
        }
    }

    pub fn push_result(&mut self) -> GarnishLangRuntimeResult<Data::Error> {
        trace!("Instruction - Output Result");

        let r = self.next_ref()?;
        self.data.set_result(Some(r)).nest_into()
    }
}

#[cfg(test)]
mod tests {
    use crate::{runtime::data::GarnishLangRuntimeData, ExpressionData, ExpressionDataType, GarnishLangRuntime, Instruction};

    #[test]
    fn put() {
        let mut runtime = GarnishLangRuntime::simple();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.end_constant_data().unwrap();

        runtime.put(1).unwrap();

        assert_eq!(*runtime.data.get_register().get(0).unwrap(), 1);
    }

    #[test]
    fn put_outside_of_constant_data() {
        let mut runtime = GarnishLangRuntime::simple();
        runtime.add_data(ExpressionData::integer(10)).unwrap();

        let result = runtime.put(0);

        assert!(result.is_err());
    }

    #[test]
    fn put_input() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.data.push_input(2).unwrap();

        runtime.put_input().unwrap();

        assert_eq!(*runtime.data.get_register().get(0).unwrap(), 2);
    }

    #[test]
    fn put_input_is_unit_if_no_input() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.put_input().unwrap();

        assert_eq!(runtime.data.get_data_type(3).unwrap(), ExpressionDataType::Unit);
        assert_eq!(*runtime.data.get_register().get(0).unwrap(), 3);
    }

    #[test]
    fn push_input() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.data.push_register(2).unwrap();

        runtime.push_input().unwrap();

        assert_eq!(runtime.data.get_input(0).unwrap(), 2usize);
        assert_eq!(runtime.data.get_result().unwrap(), 2usize);
    }

    #[test]
    fn push_input_no_register_is_err() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        assert!(runtime.push_input().is_err());
    }

    #[test]
    fn push_result() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_instruction(Instruction::PushResult, None).unwrap();

        runtime.data.push_register(1).unwrap();

        runtime.push_result().unwrap();

        assert_eq!(runtime.data.get_result().unwrap(), 1usize);
        assert_eq!(runtime.data.get_integer(runtime.data.get_result().unwrap()).unwrap(), 10i64);
    }

    #[test]
    fn push_result_no_register_is_err() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_instruction(Instruction::PushResult, None).unwrap();

        assert!(runtime.push_result().is_err());
    }

    #[test]
    fn put_result() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.data.set_result(Some(2)).unwrap();

        runtime.put_result().unwrap();

        assert_eq!(*runtime.data.get_register().get(0).unwrap(), 2);
    }

    #[test]
    fn put_result_is_unit_if_no_result() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.put_result().unwrap();

        assert_eq!(runtime.data.get_data_type(3).unwrap(), ExpressionDataType::Unit);
        assert_eq!(*runtime.data.get_register().get(0).unwrap(), 3);
    }
}
