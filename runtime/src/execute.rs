use std::convert::TryFrom;

use garnish_common::{Error, ExpressionResult, Instruction, Result};

use crate::context::{DefaultContext, ExpressionContext};
use crate::runtime::CallType;
use crate::ExpressionRuntime;

pub(crate) enum LoopInstruction {
    Break,
    None,
}

impl ExpressionRuntime {
    pub fn execute_with_context<T>(
        &mut self,
        start: String,
        context: &T,
    ) -> Result<ExpressionResult>
    where
        T: ExpressionContext,
    {
        self.execute_instructions(start, context)
    }

    pub fn execute<T>(&mut self, start: T) -> Result<ExpressionResult>
    where
        T: ToString,
    {
        self.execute_instructions(start.to_string(), &DefaultContext {})
    }

    fn execute_instructions<T>(&mut self, start: String, context: &T) -> Result<ExpressionResult>
    where
        T: ExpressionContext,
    {
        let expression_index = match self.expression_map.get(&start.to_string()) {
            Some(i) => *i,
            None => Result::Err(
                format!("Expression '{}' does not exist in instructions.", start).into(),
            )?,
        };

        let instruction_start = self.get_expression_start(expression_index)?;

        self.ref_cursor = 0;

        // push frame subtracts one from start
        // to work with main loop better
        // so add one here to end up in proper spot
        self.push_frame(instruction_start);

        while !self.call_stack.is_empty() {
            match self.advance_instruction(context)? {
                LoopInstruction::Break => break,
                LoopInstruction::None => (),
            }
        }

        if self.ref_cursor == 0 {
            self.insert_unit()?;
        }

        let value_start = self.get_value_ref(self.ref_cursor - 1)?;

        Result::Ok(ExpressionResult::new_with_start(
            &self.data[..],
            Some(&self.symbol_table),
            value_start,
        )?)
    }

    pub fn next_instruction(&self) -> Result<Instruction> {
        let last_index = self.call_stack.len() - 1;
        let cursor = self.call_stack[last_index].cursor + 1;

        match Instruction::try_from(self.instructions[cursor]) {
            Ok(i) => Ok(i),
            Err(e) => Err(Error::new_from_string(e)),
        }
    }

    pub(crate) fn advance_instruction<T>(&mut self, context: &T) -> Result<LoopInstruction>
    where
        T: ExpressionContext,
    {
        let last_index = self.call_stack.len() - 1;

        // every frame starts one behind its actual start
        // so we increment first to get next instruction
        self.call_stack[last_index].cursor += 1;
        let cursor = self.call_stack[last_index].cursor;
        let instruction = Instruction::try_from(self.instructions[cursor])?;

        match instruction {
            Instruction::Put => self.execute_put()?,
            Instruction::MakePair => self.make_pair()?,
            Instruction::MakeInclusiveRange => self.make_inclusive_range()?,
            Instruction::MakeExclusiveRange => self.make_exclusive_range()?,
            Instruction::MakeStartExclusiveRange => self.make_start_exclusive_range()?,
            Instruction::MakeEndExclusiveRange => self.make_end_exclusive_range()?,
            Instruction::StartList => self.start_list(),
            Instruction::MakeList => self.make_list()?,
            Instruction::MakeLink => self.make_link()?,
            Instruction::PerformAddition => self.perform_addition()?,
            Instruction::PerformSubtraction => self.perform_subtraction()?,
            Instruction::PerformMultiplication => self.perform_multiplication()?,
            Instruction::PerformDivision => self.perform_division()?,
            Instruction::PerformIntegerDivision => self.perform_integer_division()?,
            Instruction::PerformRemainder => self.perform_remainder()?,
            Instruction::PerformNegation => self.perform_negation()?,
            Instruction::PerformAbsoluteValue => self.perform_absolute_value()?,
            Instruction::PerformExponential => self.perform_exponential()?,
            Instruction::PerformBitwiseAnd => self.perform_bitwise_and()?,
            Instruction::PerformBitwiseOr => self.perform_bitwise_or()?,
            Instruction::PerformBitwiseXor => self.perform_bitwise_xor()?,
            Instruction::PerformBitwiseNot => self.perform_bitwise_not()?,
            Instruction::PerformBitwiseLeftShift => self.perform_bitwise_left_shift()?,
            Instruction::PerformBitwiseRightShift => self.perform_bitwise_right_shift()?,
            Instruction::PerformLogicalAND => self.perform_logical_and()?,
            Instruction::PerformLogicalOR => self.perform_logical_or()?,
            Instruction::PerformLogicalXOR => self.perform_logical_xor()?,
            Instruction::PerformLogicalNOT => self.perform_logical_not()?,
            Instruction::PerformTypeCast => self.perform_type_cast()?,
            Instruction::PerformEqualityComparison => self.perform_equality_comparison()?,
            Instruction::PerformInequalityComparison => self.perform_inequality_comparison()?,
            Instruction::PerformLessThanComparison => self.perform_less_than_comparison()?,
            Instruction::PerformLessThanOrEqualComparison => {
                self.perform_less_than_or_equal_comparison()?
            }
            Instruction::PerformGreaterThanComparison => self.perform_greater_than_comparison()?,
            Instruction::PerformGreaterThanOrEqualComparison => {
                self.perform_greater_than_or_equal_comparison()?
            }
            Instruction::PerformTypeComparison => self.perform_type_comparison()?,
            Instruction::PerformAccess => self.perform_access()?,
            Instruction::PutInput => self.put_input()?,
            Instruction::PushInput => self.push_input()?,
            Instruction::PushUnitInput => self.push_unit_input()?,
            Instruction::PutResult => self.put_result()?,
            Instruction::OutputResult => self.output_result()?,
            Instruction::Resolve => self.resolve(context)?,
            Instruction::Apply => self.apply(context)?,
            Instruction::PartiallyApply => self.partially_apply()?,
            Instruction::EndExpression => self.perform_end_expression()?,
            Instruction::ExecuteExpression => self.perform_execute_expression()?,
            Instruction::Invoke => self.invoke(context)?,
            Instruction::ConditionalExecute => self.conditional_execute()?,
            Instruction::ResultConditionalExecute => self.result_conditional_execute()?,
            Instruction::Iterate => self.perform_iterate()?,
            Instruction::IterationOutput => self.perform_iteration_output()?,
            Instruction::IterationContinue => self.perform_iteration_continue()?,
            Instruction::IterationSkip => self.perform_iteration_skip()?,
            Instruction::IterationComplete => self.perform_iteration_complete()?,
            Instruction::IterateToSingleResult => self.perform_iterate_to_single_result()?,
            Instruction::ReverseIterate => self.perform_reverse_iterate()?,
            Instruction::ReverseIterateToSingleResult => self.perform_reverse_iterate_to_single_result()?,
            Instruction::MultiIterate => self.perform_multi_iterate()?,
        }

        Ok(if self.call_stack.is_empty() {
            LoopInstruction::Break
        } else {
            LoopInstruction::None
        })
    }

    fn perform_execute_expression(&mut self) -> Result {
        let expression_index = self.consume_constant_reference()?;
        let expression_start = self.get_expression_start(expression_index)?;
        self.push_frame(expression_start);

        return Ok(());
    }

    fn perform_end_expression(&mut self) -> Result {
        if self.call_stack.is_empty() {
            return Err("Attempt to end expression with empty call stack")?;
        }

        let frame = self.call_stack[self.call_stack.len() - 1].clone();

        if frame.call_type == CallType::Iteration {
            self.perform_reiterate()?;
            return Ok(());
        }

        self.call_stack.pop(); 

        if frame.call_type != CallType::Conditional {
            self.pop_input();
        }

        if self.call_stack.is_empty() {
            // primary expression has ended
            // push last ref onto result stack
            self.output_result()?;
        } else {
            // perform type specific functionality before clearing results
            if frame.call_type == CallType::ExpressionIteration {
                self.iteration_expression_end(frame.clone())?;
            }

            // just ended a sub expression
            // clear all results output by it
            while self.result_stack.len() > frame.result_start {
                self.result_stack.pop();
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use garnish_common::ExpressionValue;
    use garnish_instruction_set_builder::InstructionSetBuilder;

    use crate::ExpressionRuntime;

    #[test]
    fn executing_empty_expression_is_ok() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");
        instructions.end_expression();
        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        assert!(expression_runtime.execute("main").is_ok());
    }

    #[test]
    fn putting_result_during_expression_execution_uses_input() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();

        instructions.output_result();

        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.push_input();
        instructions.execute_expression("add_5");

        instructions.end_expression();

        instructions.start_expression("add_5");

        instructions.put_result();
        instructions.put(ExpressionValue::integer(5)).unwrap();
        instructions.perform_addition();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();
        let num = result.as_integer().unwrap();

        assert_eq!(num, 25);
    }

    #[test]
    fn putting_result_after_output_result_inside_expression_execution_uses_that_result() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();

        instructions.output_result();

        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.push_input();
        instructions.execute_expression("add_5_to_50");

        instructions.end_expression();

        instructions.start_expression("add_5_to_50");

        instructions.put(ExpressionValue::integer(50)).unwrap();

        instructions.output_result();

        instructions.put_result();
        instructions.put(ExpressionValue::integer(5)).unwrap();
        instructions.perform_addition();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();
        let num = result.as_integer().unwrap();

        assert_eq!(num, 55);
    }

    #[test]
    fn results_output_by_sub_expression_executions_are_not_apart_of_final_result_set() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();

        instructions.output_result();

        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.push_input();
        instructions.execute_expression("add_5_to_50");

        instructions.end_expression();

        instructions.start_expression("add_5_to_50");

        instructions.put(ExpressionValue::integer(50)).unwrap();

        instructions.output_result();

        instructions.put_result();
        instructions.put(ExpressionValue::integer(5)).unwrap();
        instructions.perform_addition();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        expression_runtime.execute("main").unwrap();

        let result_count = expression_runtime.result_stack.len();

        let first_result = expression_runtime.get_result(0).unwrap().unwrap();
        let second_result = expression_runtime.get_result(1).unwrap().unwrap();

        assert_eq!(result_count, 2);
        assert_eq!(first_result.as_integer().unwrap(), 10);
        assert_eq!(second_result.as_integer().unwrap(), 55);
    }
}
