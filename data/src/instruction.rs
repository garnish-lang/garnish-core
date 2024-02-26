pub use garnish_lang_traits::Instruction;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Container for [`Instruction`] and any data need by those instructions during execution.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub struct SimpleInstruction {
    pub instruction: Instruction,
    pub data: Option<usize>,
}

impl SimpleInstruction {
    pub fn new(instruction: Instruction, data: Option<usize>) -> SimpleInstruction {
        SimpleInstruction { instruction, data }
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
