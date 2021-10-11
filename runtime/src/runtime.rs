use std::convert::TryInto;


pub enum Instruction {
    Put = 1,
}

pub struct InstructionData {
    instruction: Instruction,
    data: Option<ExpressionData>
}

pub struct ExpressionData {
    bytes: Vec<u8>
}

impl ExpressionData {
    pub fn integer(i: i64) -> ExpressionData {
        ExpressionData { bytes: i.to_le_bytes()[..].to_vec() }
    }

    pub fn reference(r: usize) -> ExpressionData {
        ExpressionData { bytes: r.to_le_bytes()[..].to_vec() }
    }

    pub fn as_integer(&self) -> Result<i64, String> {
        let (bytes, _) = self.bytes.split_at(std::mem::size_of::<i64>());
        Ok(i64::from_le_bytes(match bytes.try_into() {
            Ok(v) => v,
            Err(e) => Result::Err(e.to_string())?
        }))
    }
}

pub struct GarnishLangRuntime {
    data: Vec<ExpressionData>,
    instructions: Vec<InstructionData>
}

impl GarnishLangRuntime {
    pub fn new() -> Self {
        GarnishLangRuntime {
            data: vec![],
            instructions: vec![]
        }
    }

    pub fn add_data(&mut self, data: ExpressionData) -> Result<(), String> {
        self.data.push(data);
        Ok(())
    }

    pub fn add_reference(&mut self, reference: usize) -> Result<(), String> {
        self.data.push(ExpressionData::reference(reference));
        Ok(())
    }

    pub fn add_instruction(&mut self, instruction: Instruction, data: Option<ExpressionData>) -> Result<(), String> {
        self.instructions.push(InstructionData { instruction, data });
        Ok(())
    }

    pub fn perform_addition(&mut self) -> Result<(), String> {
        match self.data.len() {
            0 | 1 => Result::Err("Not enough data to perform addition operation.".to_string()),
            // 2 and greater
            _ => {
                let right_data = self.data.pop().unwrap();
                let left_data = self.data.pop().unwrap();

                let left = left_data.as_integer()?;
                let right = right_data.as_integer()?;

                self.add_data(ExpressionData::integer(left + right))?;

                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{GarnishLangRuntime, ExpressionData, Instruction};

    #[test]
    fn create_runtime() {
        GarnishLangRuntime::new();
    }

    #[test]
    fn integer_expression_data() {
        let d = ExpressionData::integer(1234567890);
        assert_eq!(d.bytes, 1234567890i64.to_le_bytes());
    }

    #[test]
    fn reference_expression_data() {
        let d = ExpressionData::reference(1234567890);
        assert_eq!(d.bytes, 1234567890usize.to_le_bytes());
    }

    #[test]
    fn expression_data_as_integer() {
        assert_eq!(ExpressionData::reference(1234567890).as_integer().unwrap(), 1234567890)
    }

    #[test]
    fn add_data() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(100)).unwrap();

        assert_eq!(runtime.data.len(), 1);
    }

    #[test]
    fn push_top_reference() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(100)).unwrap();
        runtime.add_reference(0).unwrap();

        assert_eq!(runtime.data.len(), 2);
    }

    #[test]
    fn add_instruction() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_instruction(Instruction::Put, Some(ExpressionData::reference(0))).unwrap();

        assert_eq!(runtime.instructions.len(), 1);
    }

    #[test]
    fn perform_addition() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.perform_addition().unwrap();

        assert_eq!(runtime.data.get(0).unwrap().bytes, 30i64.to_le_bytes());
    }
}