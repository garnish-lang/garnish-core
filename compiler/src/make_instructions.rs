use garnish_lang_runtime::InstructionData;
use garnish_lang_runtime::*;
use log::trace;

use crate::parser::*;

pub struct InstructionSet {
    instructions: Vec<InstructionData>,
    data: Vec<ExpressionData>,
}

impl InstructionSet {
    pub fn new() -> InstructionSet {
        let mut data = vec![];
        data.push(ExpressionData::unit());

        InstructionSet { instructions: vec![], data }
    }

    pub fn get_instructions(&self) -> &Vec<InstructionData> {
        &self.instructions
    }

    pub fn get_data(&self) -> &Vec<ExpressionData> {
        &self.data
    }
}

fn add_instructions_for_node(
    node_index: Option<usize>,
    nodes: &Vec<ParseNode>,
    instructions: &mut Vec<InstructionData>,
    data: &mut Vec<ExpressionData>,
) -> Result<(), String> {
    Ok(match node_index {
        None => Err(format!(
            "None value for input index. All nodes should resolve properly if starting from root node."
        ))?,
        Some(node_index) => match nodes.get(node_index) {
            // all nodes should exist if starting from root
            None => Err(format!(
                "Node at index {:?} does not exist. All nodes should resolve properly if starting from root node.",
                node_index
            ))?,
            Some(node) => match node.get_definition() {
                Definition::Number => {
                    instructions.push(InstructionData::new(Instruction::Put, Some(data.len())));

                    data.push(ExpressionData::integer(match node.get_lex_token().get_text().parse::<i64>() {
                        Err(e) => Err(e.to_string())?,
                        Ok(i) => i,
                    }));
                }
                Definition::Identifier => {
                    instructions.push(InstructionData::new(Instruction::Put, Some(data.len())));
                    instructions.push(InstructionData::new(Instruction::Resolve, None));

                    data.push(ExpressionData::symbol_from_string(node.get_lex_token().get_text()));
                }
                Definition::Unit => {
                    // all unit literals will use unit used in the zero element slot of data
                    instructions.push(InstructionData::new(Instruction::Put, Some(0)));
                }
                Definition::Symbol => {
                    instructions.push(InstructionData::new(Instruction::Put, Some(data.len())));

                    data.push(ExpressionData::symbol_from_string(&node.get_lex_token().get_text()[1..].to_string()));
                }
                Definition::Input => {
                    // all unit literals will use unit used in the zero element slot of data
                    instructions.push(InstructionData::new(Instruction::PutInput, None));
                }
                Definition::Result => {
                    // all unit literals will use unit used in the zero element slot of data
                    instructions.push(InstructionData::new(Instruction::PutResult, None));
                }
                Definition::AbsoluteValue => todo!(), // not currently in runtime
                Definition::EmptyApply => {
                    add_instructions_for_node(node.get_left(), nodes, instructions, data)?;

                    instructions.push(InstructionData::new(Instruction::Put, Some(0)));
                    instructions.push(InstructionData::new(Instruction::Apply, None));
                }
                Definition::Addition => todo!(),
                Definition::Equality => todo!(),
                Definition::Pair => todo!(),
                Definition::Access => todo!(),
                Definition::List => todo!(),
                Definition::Subexpression => todo!(),
                Definition::Group => todo!(),
                Definition::NestedExpression => todo!(),
                Definition::Apply => todo!(),
                Definition::ApplyTo => todo!(),
                Definition::Reapply => todo!(),
                Definition::ApplyIfTrue => todo!(),
                Definition::ApplyIfFalse => todo!(),
                Definition::ConditionalBranch => todo!(),
                // no runtime meaning, parser only utility
                Definition::Drop => (),
            },
        },
    })
}

pub fn instructions_from_ast(root: usize, nodes: Vec<ParseNode>) -> Result<InstructionSet, String> {
    let mut instruction_set = InstructionSet::new();

    add_instructions_for_node(Some(root), &nodes, &mut instruction_set.instructions, &mut instruction_set.data)?;

    instruction_set.instructions.push(InstructionData::new(Instruction::EndExpression, None));

    Ok(instruction_set)
}

#[cfg(test)]
mod test_utils {
    use crate::*;
    use garnish_lang_runtime::*;
    use std::iter;

    pub fn assert_instruction_data(
        root: usize,
        nodes: Vec<(Definition, Option<usize>, Option<usize>, Option<usize>, &str, TokenType)>,
        instructions: Vec<(Instruction, Option<usize>)>,
        data: Vec<ExpressionData>,
    ) {
        let expected_instructions: Vec<InstructionData> = instructions.iter().map(|i| InstructionData::new(i.0, i.1)).collect();

        let expected_data: Vec<ExpressionData> = iter::once(ExpressionData::unit()).chain(data.into_iter()).collect();

        let result = get_instruction_data(root, nodes).unwrap();

        assert_eq!(result.get_instructions().clone(), expected_instructions);
        assert_eq!(result.get_data(), &expected_data)
    }

    pub fn get_instruction_data(
        root: usize,
        nodes: Vec<(Definition, Option<usize>, Option<usize>, Option<usize>, &str, TokenType)>,
    ) -> Result<InstructionSet, String> {
        let nodes: Vec<ParseNode> = nodes
            .iter()
            .map(|v| ParseNode::new(v.0, v.1, v.2, v.3, LexerToken::new(v.4.to_string(), v.5, 0, 0)))
            .collect();

        instructions_from_ast(root, nodes)
    }
}

#[cfg(test)]
mod values {
    use std::vec;

    use super::test_utils::*;
    use crate::*;
    use garnish_lang_runtime::*;

    #[test]
    fn put_number() {
        assert_instruction_data(
            0,
            vec![(Definition::Number, None, None, None, "5", TokenType::Number)],
            vec![(Instruction::Put, Some(1)), (Instruction::EndExpression, None)],
            vec![ExpressionData::integer(5)],
        );
    }

    #[test]
    fn resolve_identifier() {
        assert_instruction_data(
            0,
            vec![(Definition::Identifier, None, None, None, "value", TokenType::Identifier)],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::Resolve, None),
                (Instruction::EndExpression, None),
            ],
            vec![ExpressionData::symbol_from_string(&"value".to_string())],
        );
    }

    #[test]
    fn put_unit() {
        assert_instruction_data(
            0,
            vec![(Definition::Unit, None, None, None, "()", TokenType::UnitLiteral)],
            vec![(Instruction::Put, Some(0)), (Instruction::EndExpression, None)],
            vec![],
        );
    }

    #[test]
    fn put_symbol() {
        assert_instruction_data(
            0,
            vec![(Definition::Symbol, None, None, None, ":symbol", TokenType::Symbol)],
            vec![(Instruction::Put, Some(1)), (Instruction::EndExpression, None)],
            vec![ExpressionData::symbol_from_string(&"symbol".to_string())],
        );
    }

    #[test]
    fn put_empty_symbol() {
        assert_instruction_data(
            0,
            vec![(Definition::Symbol, None, None, None, ":", TokenType::Symbol)],
            vec![(Instruction::Put, Some(1)), (Instruction::EndExpression, None)],
            vec![ExpressionData::symbol_from_string(&"".to_string())],
        );
    }

    #[test]
    fn put_input() {
        assert_instruction_data(
            0,
            vec![(Definition::Input, None, None, None, "$", TokenType::Input)],
            vec![(Instruction::PutInput, None), (Instruction::EndExpression, None)],
            vec![],
        );
    }

    #[test]
    fn put_result() {
        assert_instruction_data(
            0,
            vec![(Definition::Result, None, None, None, "$?", TokenType::Result)],
            vec![(Instruction::PutResult, None), (Instruction::EndExpression, None)],
            vec![],
        );
    }
}

#[cfg(test)]
mod operations {
    use std::vec;

    use super::test_utils::*;
    use crate::*;
    use garnish_lang_runtime::*;

    #[test]
    fn empty_apply() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Identifier, Some(1), None, None, "value", TokenType::Identifier),
                (Definition::EmptyApply, None, Some(0), None, "~~", TokenType::EmptyApply),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::Resolve, None),
                (Instruction::Put, Some(0)),
                (Instruction::Apply, None),
                (Instruction::EndExpression, None),
            ],
            vec![ExpressionData::symbol_from_string(&"value".to_string())],
        );
    }

    #[test]
    fn empty_apply_no_left_is_error() {
        let result = get_instruction_data(1, vec![(Definition::EmptyApply, None, Some(0), None, "~~", TokenType::EmptyApply)]);

        assert!(result.is_err());
    }

    #[test]
    fn empty_apply_invalid_child() {
        let result = get_instruction_data(
            1,
            vec![
                (Definition::Identifier, Some(1), None, None, "value", TokenType::Identifier),
                (Definition::EmptyApply, None, Some(10), None, "~~", TokenType::EmptyApply),
            ],
        );

        assert!(result.is_err());
    }
}
