use log::trace;

use crate::{error, GarnishLangRuntime, GarnishLangRuntimeResult};

use super::data::GarnishLangRuntimeDataPool;

impl<Data> GarnishLangRuntime<Data>
where
    Data: GarnishLangRuntimeDataPool,
{
    pub fn put(&mut self, i: usize) -> GarnishLangRuntimeResult {
        trace!("Instruction - Put | Data - {:?}", i);
        match i >= self.end_of_constant_data {
            true => Err(error(format!(
                "Attempting to put reference to {:?} which is out of bounds of constant data that ends at {:?}.",
                i, self.end_of_constant_data
            ))),
            false => {
                self.reference_stack.push(i);
                Ok(())
            }
        }
    }

    pub fn put_input(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Put Input");

        self.reference_stack.push(match self.inputs.last() {
            None => Err(error(format!("No inputs available to put reference.")))?,
            Some(r) => *r,
        });

        Ok(())
    }

    pub fn push_input(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Push Input");
        let r = self.addr_of_raw_data(match self.data.len() > 0 {
            true => self.data.len() - 1,
            false => Err(error(format!("No data available to push as input.")))?,
        })?;

        self.inputs.push(r);
        self.current_result = Some(r);

        Ok(())
    }

    pub fn put_result(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Put Result");

        match self.current_result {
            None => Err(error(format!("No result available to put reference.")))?,
            Some(i) => {
                self.reference_stack.push(i);
            }
        }

        Ok(())
    }

    pub fn push_result(&mut self) -> GarnishLangRuntimeResult {
        trace!("Instruction - Output Result");
        match self.data.len() {
            0 => Err(error(format!("Not enough data to perform output result operation."))),
            n => {
                self.current_result = Some(self.addr_of_raw_data(n - 1)?);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ExpressionData, GarnishLangRuntime, Instruction};

    #[test]
    fn put() {
        let mut runtime = GarnishLangRuntime::simple();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.end_constant_data().unwrap();

        runtime.put(1).unwrap();

        assert_eq!(*runtime.reference_stack.get(0).unwrap(), 1);
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

        runtime.add_input_reference(2).unwrap();

        runtime.put_input().unwrap();

        assert_eq!(*runtime.reference_stack.get(0).unwrap(), 2);
    }

    #[test]
    fn push_input() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.push_input().unwrap();

        assert_eq!(runtime.inputs.get(0).unwrap(), &2);
        assert_eq!(runtime.current_result.unwrap(), 2usize);
    }

    #[test]
    fn push_result() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_instruction(Instruction::PushResult, None).unwrap();
        runtime.push_result().unwrap();

        assert_eq!(runtime.get_result().unwrap().bytes, 10i64.to_le_bytes());
    }

    #[test]
    fn put_result() {
        let mut runtime = GarnishLangRuntime::simple();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.current_result = Some(2);

        runtime.put_result().unwrap();

        assert_eq!(*runtime.reference_stack.get(0).unwrap(), 2);
    }
}
