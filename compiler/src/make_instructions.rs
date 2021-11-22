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
    node: Option<&ParseNode>,
    instructions: &mut Vec<InstructionData>,
    data: &mut Vec<ExpressionData>,
) -> Result<(), String> {
    match node {
        None => Ok(()),
        Some(node) => match node.get_definition() {
            Definition::Number => {
                instructions.push(InstructionData::new(Instruction::Put, Some(data.len())));

                data.push(ExpressionData::integer(match node.get_lex_token().get_text().parse::<i64>() {
                    Err(e) => Err(e.to_string())?,
                    Ok(i) => i,
                }));

                Ok(())
            }
            Definition::Identifier => {
                instructions.push(InstructionData::new(Instruction::Put, Some(data.len())));
                instructions.push(InstructionData::new(Instruction::Resolve, None));

                data.push(ExpressionData::symbol_from_string(node.get_lex_token().get_text()));

                Ok(())
            }
            Definition::Unit => {
                // all unit literals will use unit used in the zero element slot of data
                instructions.push(InstructionData::new(Instruction::Put, Some(0)));

                Ok(())
            }
            Definition::Symbol => {
                instructions.push(InstructionData::new(Instruction::Put, Some(data.len())));

                data.push(ExpressionData::symbol_from_string(&node.get_lex_token().get_text()[1..].to_string()));

                Ok(())
            }
            Definition::Input => {
                // all unit literals will use unit used in the zero element slot of data
                instructions.push(InstructionData::new(Instruction::PutInput, None));

                Ok(())
            }
            Definition::Result => {
                // all unit literals will use unit used in the zero element slot of data
                instructions.push(InstructionData::new(Instruction::PutResult, None));

                Ok(())
            }
            Definition::AbsoluteValue => todo!(), // not currently in runtime
            Definition::EmptyApply => todo!(),
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
            Definition::Drop => Ok(()),
        },
    }
}

pub fn instructions_from_ast(root: usize, nodes: Vec<ParseNode>) -> Result<InstructionSet, String> {
    let mut instruction_set = InstructionSet::new();

    add_instructions_for_node(nodes.get(root), &mut instruction_set.instructions, &mut instruction_set.data)?;

    instruction_set.instructions.push(InstructionData::new(Instruction::EndExpression, None));

    Ok(instruction_set)
}

#[cfg(test)]
mod values {
    use crate::*;
    use garnish_lang_runtime::*;

    #[test]
    fn put_number() {
        let nodes = vec![ParseNode::new(
            Definition::Number,
            None,
            None,
            None,
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
        )];

        let result = instructions_from_ast(0, nodes).unwrap();

        let instructions = result.get_instructions();
        let data = result.get_data();

        assert_eq!(
            instructions.clone(),
            vec![
                InstructionData::new(Instruction::Put, Some(1)),
                InstructionData::new(Instruction::EndExpression, None)
            ]
        );

        assert_eq!(data, &[ExpressionData::unit(), ExpressionData::integer(5)])
    }

    #[test]
    fn resolve_identifier() {
        let nodes = vec![ParseNode::new(
            Definition::Identifier,
            None,
            None,
            None,
            LexerToken::new("value".to_string(), TokenType::Identifier, 0, 0),
        )];

        let result = instructions_from_ast(0, nodes).unwrap();

        let instructions = result.get_instructions();
        let data = result.get_data();

        assert_eq!(
            instructions.clone(),
            vec![
                InstructionData::new(Instruction::Put, Some(1)),
                InstructionData::new(Instruction::Resolve, None),
                InstructionData::new(Instruction::EndExpression, None)
            ]
        );

        assert_eq!(data, &[ExpressionData::unit(), ExpressionData::symbol_from_string(&"value".to_string())])
    }

    #[test]
    fn put_unit() {
        let nodes = vec![ParseNode::new(
            Definition::Unit,
            None,
            None,
            None,
            LexerToken::new("()".to_string(), TokenType::UnitLiteral, 0, 0),
        )];

        let result = instructions_from_ast(0, nodes).unwrap();

        let instructions = result.get_instructions();
        let data = result.get_data();

        assert_eq!(
            instructions.clone(),
            vec![
                InstructionData::new(Instruction::Put, Some(0)),
                InstructionData::new(Instruction::EndExpression, None)
            ]
        );

        assert_eq!(data, &[ExpressionData::unit()])
    }

    #[test]
    fn put_symbol() {
        let nodes = vec![ParseNode::new(
            Definition::Symbol,
            None,
            None,
            None,
            LexerToken::new(":symbol".to_string(), TokenType::Symbol, 0, 0),
        )];

        let result = instructions_from_ast(0, nodes).unwrap();

        let instructions = result.get_instructions();
        let data = result.get_data();

        assert_eq!(
            instructions.clone(),
            vec![
                InstructionData::new(Instruction::Put, Some(1)),
                InstructionData::new(Instruction::EndExpression, None)
            ]
        );

        assert_eq!(data, &[ExpressionData::unit(), ExpressionData::symbol_from_string(&"symbol".to_string())])
    }

    #[test]
    fn put_empty_symbol() {
        let nodes = vec![ParseNode::new(
            Definition::Symbol,
            None,
            None,
            None,
            LexerToken::new(":".to_string(), TokenType::Symbol, 0, 0),
        )];

        let result = instructions_from_ast(0, nodes).unwrap();

        let instructions = result.get_instructions();
        let data = result.get_data();

        assert_eq!(
            instructions.clone(),
            vec![
                InstructionData::new(Instruction::Put, Some(1)),
                InstructionData::new(Instruction::EndExpression, None)
            ]
        );

        assert_eq!(data, &[ExpressionData::unit(), ExpressionData::symbol_from_string(&"".to_string())])
    }

    #[test]
    fn put_input() {
        let nodes = vec![ParseNode::new(
            Definition::Input,
            None,
            None,
            None,
            LexerToken::new("$".to_string(), TokenType::Input, 0, 0),
        )];

        let result = instructions_from_ast(0, nodes).unwrap();

        let instructions = result.get_instructions();
        let data = result.get_data();

        assert_eq!(
            instructions.clone(),
            vec![
                InstructionData::new(Instruction::PutInput, None),
                InstructionData::new(Instruction::EndExpression, None)
            ]
        );

        assert_eq!(data, &[ExpressionData::unit()])
    }

    #[test]
    fn put_result() {
        let nodes = vec![ParseNode::new(
            Definition::Result,
            None,
            None,
            None,
            LexerToken::new("$?".to_string(), TokenType::Result, 0, 0),
        )];

        let result = instructions_from_ast(0, nodes).unwrap();

        let instructions = result.get_instructions();
        let data = result.get_data();

        assert_eq!(
            instructions.clone(),
            vec![
                InstructionData::new(Instruction::PutResult, None),
                InstructionData::new(Instruction::EndExpression, None)
            ]
        );

        assert_eq!(data, &[ExpressionData::unit()])
    }
}
