#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]

pub enum Instruction {
    Put = 1,
    PutInput,
    PutResult,
    PushInput,
    PushResult,
    Jump,
    EndExpression,
    ExecuteExpression,
    PerformAddition,
    EndExecution,
    JumpIfTrue,
    JumpIfFalse,
    EqualityComparison,
    MakePair,
    MakeList,
    Apply,
    Reapply,
    Access,
    Resolve,
    Return,
    ReturnTo,
}

#[derive(Debug, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub struct InstructionData {
    pub(crate) instruction: Instruction,
    pub(crate) data: Option<usize>,
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
