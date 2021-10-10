
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

        runtime.add_instruction(Instruction::Put, Some(ExpressionData::reference(0)));

        assert_eq!(runtime.instructions.len(), 1);
    }
}