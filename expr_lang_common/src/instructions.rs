use std::collections::HashMap;
use std::convert::{Infallible, TryFrom, TryInto};
use std::mem;

#[derive(Debug, Eq, PartialEq, Clone)]
#[repr(u8)]
pub enum Instruction {
    Put = 1,
    StartList,
    MakeList,
    MakePair,
    MakeLink,
    MakeInclusiveRange,
    MakeExclusiveRange,
    MakeStartExclusiveRange,
    MakeEndExclusiveRange,
    PerformAccess,
    PerformAddition,
    PerformSubtraction,
    PerformMultiplication,
    PerformDivision,
    PerformIntegerDivision,
    PerformRemainder,
    PerformNegation,
    PerformAbsoluteValue,
    PerformExponential,
    PerformBitwiseAnd,
    PerformBitwiseOr,
    PerformBitwiseXor,
    PerformBitwiseNot,
    PerformBitwiseLeftShift,
    PerformBitwiseRightShift,
    PerformLogicalAND,
    PerformLogicalOR,
    PerformLogicalXOR,
    PerformLogicalNOT,
    PerformTypeCast,
    ExecuteExpression,
    EndExpression,
    PutInput,
    PushInput,
    PushUnitInput,
    PutResult,
    OutputResult,
    Resolve,
    Invoke,
    Apply,
    PartiallyApply,
    ConditionalExecute,
    ResultConditionalExecute,
    PerformEqualityComparison,
    PerformInequalityComparison,
    PerformLessThanComparison,
    PerformLessThanOrEqualComparison,
    PerformGreaterThanComparison,
    PerformGreaterThanOrEqualComparison,
    PerformTypeComparison,
    Iterate,
    IterateToSingleResult,
    IterationOutput,
    IterationContinue,
    IterationSkip,
    IterationComplete,
}

const LAST_INSTRUCTION_VALUE: u8 = Instruction::IterationComplete as u8;

impl TryInto<u8> for Instruction {
    type Error = Infallible;

    fn try_into(self) -> Result<u8, Self::Error> {
        Ok(self as u8)
    }
}

impl TryFrom<u8> for Instruction {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Err(format!("Invalid Instruction code: {}", value)),
            x if x > 0 && x <= LAST_INSTRUCTION_VALUE => {
                Ok(unsafe { mem::transmute::<u8, Instruction>(value) })
            }
            _ => Err(format!("Invalid Instruction code: {}", value)),
        }
    }
}

pub trait InstructionSet {
    fn get_data(&self) -> &Vec<u8>;
    fn get_instructions(&self) -> &Vec<u8>;
    fn get_expression_table(&self) -> &Vec<usize>;
    fn get_expression_map(&self) -> &HashMap<String, usize>;
    fn get_symbol_table(&self) -> &HashMap<String, usize>;
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use crate::Instruction;

    use super::LAST_INSTRUCTION_VALUE;

    #[test]
    fn instruction_from_valid_u8() {
        let instruction = Instruction::try_from(1).unwrap();
        assert_eq!(instruction, Instruction::Put);
    }

    #[test]
    fn instruction_from_zero_results_in_error() {
        let instruction = Instruction::try_from(0);
        assert!(instruction.is_err());
    }

    #[test]
    fn instruction_from_iteration_complete() {
        let instruction = Instruction::try_from(LAST_INSTRUCTION_VALUE).unwrap();
        assert_eq!(instruction, Instruction::IterationComplete);
    }

    #[test]
    fn instruction_from_value_larger_than_iteration_complete() {
        let instruction = Instruction::try_from(LAST_INSTRUCTION_VALUE + 1);
        assert!(instruction.is_err());
    }
}
