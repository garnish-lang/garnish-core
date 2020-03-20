use crate::context::ExpressionContext;
use crate::ExpressionRuntime;
use expr_lang_common::{Error, Instruction};

impl ExpressionRuntime {
    pub fn instruction_iter<'a, T>(
        &'a mut self,
        start: String,
        context: &'a T,
    ) -> Result<InstructionIter<'a, T>, String>
    where
        T: ExpressionContext,
    {
        InstructionIter::new(self, start, context)
    }
}

pub struct InstructionData {
    instruction: Instruction,
    #[allow(dead_code)]
    instruction_index: usize,
    expression_runtime: ExpressionRuntime,
    #[allow(dead_code)]
    error: Error,
}

impl InstructionData {
    pub fn get_instruction(&self) -> Instruction {
        self.instruction.clone()
    }

    pub fn get_expression_runtime(&self) -> &ExpressionRuntime {
        &self.expression_runtime
    }
}

pub struct InstructionIter<'a, T> {
    expression_runtime: &'a mut ExpressionRuntime,
    context: &'a T,
}

impl<'a, T> InstructionIter<'a, T>
where
    T: ExpressionContext,
{
    fn new(
        expression_runtime: &'a mut ExpressionRuntime,
        start: String,
        context: &'a T,
    ) -> Result<Self, String> {
        let expression_index = match expression_runtime.expression_map.get(&start.to_string()) {
            Some(i) => *i - 1, // expression indices start at 1
            None => Result::Err(format!(
                "Expression '{}' does not exist in instructions.",
                start.to_string()
            ))?,
        };

        let instruction_start = *expression_runtime
            .expression_table
            .get(expression_index)
            .unwrap();

        expression_runtime.ref_cursor = 0;

        expression_runtime.push_frame(instruction_start);

        Ok(InstructionIter {
            expression_runtime,
            context,
        })
    }
}

impl<'a, T> Iterator for InstructionIter<'a, T>
where
    T: ExpressionContext,
{
    type Item = InstructionData;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.expression_runtime.call_stack.is_empty() {
            let index = self.expression_runtime.current_instruction_cursor();
            match self.expression_runtime.next_instruction() {
                Ok(instruction) => {
                    let error = match self.expression_runtime.advance_instruction(self.context) {
                        Ok(_x) => "".into(),
                        Err(e) => e,
                    };

                    return Some(InstructionData {
                        instruction,
                        instruction_index: index,
                        expression_runtime: self.expression_runtime.clone(),
                        error,
                    });
                }
                Err(error) => {
                    return Some(InstructionData {
                        instruction: Instruction::Put,
                        instruction_index: index,
                        expression_runtime: self.expression_runtime.clone(),
                        error,
                    })
                }
            };
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::context::DefaultContext;
    use crate::runtime::ExpressionRuntime;
    use expr_lang_common::{ExpressionValue, Instruction};
    use expr_lang_instruction_set_builder::InstructionSetBuilder;

    #[test]
    fn instruction_iterator() {
        let mut instructions = InstructionSetBuilder::new();
        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.perform_addition();

        instructions.end_expression();
        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let context = DefaultContext {};

        let mut instructions: Vec<Instruction> = vec![];
        for data in expression_runtime
            .instruction_iter("main".to_string(), &context)
            .unwrap()
        {
            instructions.push(data.instruction);
        }

        let expected = vec![
            Instruction::Put,
            Instruction::Put,
            Instruction::PerformAddition,
            Instruction::EndExpression,
        ];

        assert_eq!(instructions, expected);
    }
}
