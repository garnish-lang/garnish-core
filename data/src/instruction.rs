pub use garnish_traits::Instruction;

#[derive(Debug, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub struct InstructionData {
    pub instruction: Instruction,
    pub data: Option<usize>,
}

impl InstructionData {
    pub fn new(instruction: Instruction, data: Option<usize>) -> InstructionData {
        InstructionData { instruction, data }
    }

    pub fn get_instruction(&self) -> Instruction {
        self.instruction
    }

    pub fn get_data(&self) -> Option<usize> {
        self.data
    }
}

#[cfg(test)]
mod tests {}