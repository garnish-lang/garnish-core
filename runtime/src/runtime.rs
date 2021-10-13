use log::{trace};
use crate::instruction::*;
use crate::expression_data::*;

#[derive(Debug)]
pub struct GarnishLangRuntime {
    data: Vec<ExpressionData>,
    instructions: Vec<InstructionData>,
    instruction_cursor: usize,
    results: Vec<usize>,
    jump_path: Vec<usize>,
    inputs: Vec<usize>,
}

impl GarnishLangRuntime {
    pub fn new() -> Self {
        GarnishLangRuntime {
            data: vec![],
            instructions: vec![InstructionData { instruction: Instruction::EndExecution, data: None }],
            instruction_cursor: 1,
            results: vec![],
            jump_path: vec![],
            inputs: vec![]
        }
    }

    pub fn add_data(&mut self, data: ExpressionData) -> Result<(), String> {
        // Check if give a reference of reference
        // flatten reference to point to non-Reference data
        let data = match data.get_type() {
            ExpressionDataType::Reference => match self.data.get(data.as_reference().unwrap()) {
                None => Result::Err(format!("Reference given doesn't not exist in data."))?,
                Some(d) => match d.get_type() {
                    ExpressionDataType::Reference => d.clone(),
                    _ => data
                }
            }
            _ => data
        };

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

    pub fn add_input_reference(&mut self, reference: usize) -> Result<(), String> {
        match reference < self.data.len() {
            false => Result::Err("Input reference beyond bounds of data.".to_owned()),
            true => {
                self.inputs.push(reference);
                Ok(())
            }
        }
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

    pub fn end_execution(&mut self) -> Result<(), String> {
        trace!("Instruction - End Execution");
        self.instruction_cursor = self.instructions.len();

        Ok(())
    }

    pub fn put(&mut self, i: usize) -> Result<(), String> {
        trace!("Instruction - Put | Data - {:?}", i);
        self.add_reference_data(i)
    }

    pub fn put_input(&mut self) -> Result<(), String> {
        trace!("Instruction - Put Input");

        self.add_reference_data(match self.inputs.last() {
            None => Result::Err(format!("No inputs available to put reference."))?,
            Some(r) => *r
        })
    }

    pub fn push_input(&mut self) -> Result<(), String> {
        trace!("Instruction - Push Input");

        self.inputs.push(match self.data.len() > 0 {
            true => self.data.len() - 1,
            false => Result::Err(format!("No data available to push as input."))?
        });

        Ok(())
    }

    pub fn put_result(&mut self) -> Result<(), String> {
        trace!("Instruction - Put Result");

        self.add_reference_data(match self.results.last() {
            None => Result::Err(format!("No inputs available to put reference."))?,
            Some(r) => *r
        })
    }

    pub fn perform_addition(&mut self) -> Result<(), String> {
        trace!("Instruction - Addition");
        match self.data.len() {
            0 | 1 => Result::Err("Not enough data to perform addition operation.".to_string()),
            // 2 and greater
            _ => {
                let right_data = &self.data.pop().unwrap();
                let left_data = &self.data.pop().unwrap();

                let right_data = match right_data.get_type() {
                    ExpressionDataType::Reference => match self.data.get(right_data.as_reference()?) {
                        None => Result::Err(format!("Reference value doesn't reference existing value."))?,
                        Some(data) => data
                    },
                    _ => right_data
                };

                let left_data = match left_data.get_type() {
                    ExpressionDataType::Reference => match self.data.get(left_data.as_reference()?) {
                        None => Result::Err(format!("Reference value doesn't reference existing value."))?,
                        Some(data) => data
                    },
                    _ => left_data
                };

                let left = left_data.as_integer()?;
                let right = right_data.as_integer()?;

                trace!("Performing {:?} + {:?}", right, left);

                self.add_data(ExpressionData::integer(left + right))?;

                Ok(())
            }
        }
    }

    pub fn execute_expression(&mut self, index: usize) -> Result<(), String> {
        trace!("Instruction - Execute Expression | Data - {:?}", index);
        match index > 0 && index <= self.instructions.len() {
            false => Result::Err("Given index is out of bounds.".to_string()),
            true => {
                self.jump_path.push(self.instruction_cursor);
                self.instruction_cursor = index - 1;
                Ok(())
            }
        }
    }

    pub fn end_expression(&mut self) -> Result<(), String> {
        trace!("Instruction - End Expression");
        match self.jump_path.pop() {
            None => {
                self.instruction_cursor += 1;
                self.results.push(self.data.len() - 1);
            }
            Some(jump_point) => {
                self.instruction_cursor = jump_point;
            }
        }

        Ok(())
    }

    pub fn output_result(&mut self) -> Result<(), String> {
        trace!("Instruction - Output Result");
        match self.data.len() {
            0 => Result::Err("Not enough data to perform output result operation.".to_string()),
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
                    Instruction::EndExpression => self.end_expression(),
                    Instruction::ExecuteExpression => match instruction_data.data {
                        None => Result::Err(format!("No address given with execute expression instruction.")),
                        Some(i ) => self.execute_expression(i)
                    }
                    Instruction::Put => match instruction_data.data {
                        None => Result::Err(format!("No address given with put instruction.")),
                        Some(i ) => self.put(i)
                    }
                    Instruction::EndExecution => self.end_execution()
                    // _ => Result::Err("Not Implemented".to_string()),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ExpressionData, GarnishLangRuntime, Instruction};

    #[test]
    fn create_runtime() {
        GarnishLangRuntime::new();
    }

    #[test]
    fn end_execution_inserted_for_new() {
        let runtime = GarnishLangRuntime::new();

        assert_eq!(runtime.get_instruction(0).unwrap().instruction, Instruction::EndExecution);
    }

    #[test]
    fn add_data() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(100)).unwrap();

        assert_eq!(runtime.data.len(), 1);
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
    fn add_instruction() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_instruction(Instruction::Put, Some(0)).unwrap();

        assert_eq!(runtime.instructions.len(), 2);
    }

    #[test]
    fn add_input_reference() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_input_reference(0).unwrap();

        assert_eq!(runtime.inputs.get(0).unwrap().to_owned(), 0);
    }

    #[test]
    fn get_instruction() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_instruction(Instruction::Put, None).unwrap();

        assert_eq!(runtime.get_instruction(1).unwrap().get_instruction(), Instruction::Put);
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

        runtime.set_instruction_cursor(3).unwrap();

        assert_eq!(runtime.get_current_instruction().unwrap().get_instruction(), Instruction::PerformAddition);
    }

    #[test]
    fn end_execution() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_instruction(Instruction::Put, None).unwrap();
        runtime.add_instruction(Instruction::Put, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.set_instruction_cursor(3).unwrap();

        runtime.end_execution().unwrap();

        assert_eq!(runtime.instruction_cursor, 4);
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
    fn perform_addition_with_references() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::reference(0)).unwrap();
        runtime.add_data(ExpressionData::reference(1)).unwrap();
        runtime.perform_addition().unwrap();

        assert_eq!(runtime.data.get(2).unwrap().bytes, 30i64.to_le_bytes());
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

        assert_eq!(runtime.instruction_cursor, 1);
        assert_eq!(runtime.jump_path.get(0).unwrap().to_owned(), 1)
    }

    #[test]
    fn end_expression() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_instruction(Instruction::Put, Some(0)).unwrap();

        runtime.end_expression().unwrap();

        assert_eq!(runtime.instruction_cursor, 2);
        assert_eq!(runtime.get_result(0).unwrap().bytes, 10i64.to_le_bytes());
    }

    #[test]
    fn end_expression_with_path() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_instruction(Instruction::Put, Some(0)).unwrap();
        runtime.add_instruction(Instruction::Put, Some(0)).unwrap();
        runtime.add_instruction(Instruction::EndExpression, Some(0)).unwrap();
        runtime.add_instruction(Instruction::Put, Some(0)).unwrap();
        runtime.add_instruction(Instruction::ExecuteExpression, Some(0)).unwrap();

        runtime.jump_path.push(4);
        runtime.set_instruction_cursor(2).unwrap();
        runtime.end_expression().unwrap();

        assert_eq!(runtime.instruction_cursor, 4);
    }

    #[test]
    fn put() {
        let mut runtime = GarnishLangRuntime::new();
        runtime.add_data(ExpressionData::integer(10)).unwrap();

        runtime.put(0).unwrap();

        assert_eq!(runtime.data.get(1).unwrap().as_reference().unwrap(), 0);
    }

    #[test]
    fn put_input() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.add_input_reference(1).unwrap();

        runtime.put_input().unwrap();

        assert_eq!(runtime.data.get(2).unwrap(), &ExpressionData::reference(1));
    }

    #[test]
    fn push_input() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.push_input().unwrap();

        assert_eq!(runtime.inputs.get(0).unwrap(), &1);
    }

    #[test]
    fn put_result() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.results.push(1);

        runtime.put_result().unwrap();

        assert_eq!(runtime.data.get(2).unwrap(), &ExpressionData::reference(1));
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