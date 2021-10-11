use std::{convert::TryInto};


#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Instruction {
    Put = 1,
    EndExpression,
    ExecuteExpression,
    PerformAddition,
}

#[derive(Clone)]
pub struct InstructionData {
    instruction: Instruction,
    data: Option<usize>
}

impl InstructionData {
    pub fn get_instruction(&self) -> Instruction {
        self.instruction
    }

    pub fn get_data(&self) -> Option<usize> {
        self.data
    }
}

#[derive(Clone)]
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
    instructions: Vec<InstructionData>,
    instruction_cursor: usize,
    results: Vec<usize>,
    jump_path: Vec<usize>,
}

impl GarnishLangRuntime {
    pub fn new() -> Self {
        GarnishLangRuntime {
            data: vec![],
            instructions: vec![],
            instruction_cursor: 0,
            results: vec![],
            jump_path: vec![],
        }
    }

    pub fn add_data(&mut self, data: ExpressionData) -> Result<(), String> {
        self.data.push(data);
        Ok(())
    }

    pub fn add_reference_data(&mut self, reference: usize) -> Result<(), String> {
        self.data.push(ExpressionData::reference(reference));
        Ok(())
    }

    pub fn add_instruction(&mut self, instruction: Instruction, data: Option<usize>) -> Result<(), String> {
        self.instructions.push(InstructionData { instruction, data });
        Ok(())
    }

    pub fn get_instruction(&self, i: usize) -> Option<&InstructionData> {
        self.instructions.get(i)
    }

    pub fn get_current_instruction(&self) -> Option<&InstructionData> {
        self.instructions.get(self.instruction_cursor)
    }

    pub fn get_result(&self, i: usize) -> Option<&ExpressionData> {
        match self.results.get(i) {
            None => None,
            Some(index) => self.data.get(*index)
        }
    }

    pub fn advance_instruction(&mut self) -> Result<usize, String> {
        match self.instruction_cursor + 1 >= self.instructions.len() {
            true => Result::Err("No instructions left.".to_string()),
            false => {
                self.instruction_cursor += 1;
                Ok(self.instruction_cursor)
            }
        }
    }

    pub fn set_instruction_cursor(&mut self, i: usize) -> Result<usize, String> {
        match i >= self.instructions.len() {
            true => Result::Err("Instruction doesn't exist.".to_string()),
            false => {
                self.instruction_cursor = i;
                Ok(self.instruction_cursor)
            }
        }
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

    pub fn execute_expression(&mut self, index: usize) -> Result<(), String> {
        match index < self.instructions.len() {
            false => Result::Err("Given index is out of bounds.".to_string()),
            true => {
                self.jump_path.push(self.instruction_cursor);
                self.instruction_cursor = index;
                Ok(())
            }
        }
    }

    pub fn output_result(&mut self) -> Result<(), String> {
        match self.data.len() {
            0 => Result::Err("No enough data to perform output result peration.".to_string()),
            n => {
                self.results.push(n - 1);
                Ok(())
            }
        }
    }

    pub fn execute_current_instruction(&mut self) -> Result<(), String> {
        match self.instructions.get(self.instruction_cursor) {
            None => Result::Err("No instructions left.".to_string()),
            Some(instruction_data) => {
                match instruction_data.instruction {
                    Instruction::PerformAddition => self.perform_addition(),
                    _ => Result::Err("Not Implemented".to_string()),
                }
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
        runtime.add_reference_data(0).unwrap();

        assert_eq!(runtime.data.len(), 2);
    }

    #[test]
    fn add_instruction() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_instruction(Instruction::Put, Some(0)).unwrap();

        assert_eq!(runtime.instructions.len(), 1);
    }

    #[test]
    fn get_instruction() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_instruction(Instruction::Put, None).unwrap();

        assert_eq!(runtime.get_instruction(0).unwrap().get_instruction(), Instruction::Put);
    }

    #[test]
    fn get_current_instruction() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_instruction(Instruction::Put, None).unwrap();

        assert_eq!(runtime.get_current_instruction().unwrap().get_instruction(), Instruction::Put);
    }

    #[test]
    fn advance_instruction() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_instruction(Instruction::Put, None).unwrap();
        runtime.add_instruction(Instruction::EndExpression, None).unwrap();

        runtime.advance_instruction().unwrap();

        assert_eq!(runtime.get_current_instruction().unwrap().get_instruction(), Instruction::EndExpression);
    }

    #[test]
    fn set_instruction_cursor() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_instruction(Instruction::Put, None).unwrap();
        runtime.add_instruction(Instruction::Put, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.set_instruction_cursor(2).unwrap();

        assert_eq!(runtime.get_current_instruction().unwrap().get_instruction(), Instruction::PerformAddition);
    }

    #[test]
    fn perform_addition() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.perform_addition().unwrap();

        assert_eq!(runtime.data.get(0).unwrap().bytes, 30i64.to_le_bytes());
    }

    #[test]
    fn output_result() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.output_result().unwrap();

        assert_eq!(runtime.get_result(0).unwrap().bytes, 10i64.to_le_bytes()); 
    }

    #[test]
    fn execute_expression() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_instruction(Instruction::ExecuteExpression, Some(0)).unwrap();
        runtime.add_instruction(Instruction::Put, Some(0)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.instruction_cursor = 1;
        runtime.execute_expression(2).unwrap();

        assert_eq!(runtime.instruction_cursor, 2);
        assert_eq!(runtime.jump_path.get(0).unwrap().to_owned(), 1)
    }

    #[test]
    fn execute_current_instruction() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.execute_current_instruction().unwrap();

        assert_eq!(runtime.data.get(0).unwrap().bytes, 30i64.to_le_bytes());
    }
}