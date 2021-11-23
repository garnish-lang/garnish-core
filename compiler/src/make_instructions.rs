use garnish_lang_runtime::InstructionData;
use garnish_lang_runtime::*;
use log::trace;

use crate::parser::*;

#[derive(Debug, Eq, PartialEq, Clone)]
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
    // making_list: &mut bool,
    list_counts: &mut Vec<usize>,
    parent_definition: Definition,
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
            Some(node) => {
                trace!(
                    "Visiting node with definition {:?} at {:?} and parent {:?}",
                    node.get_definition(),
                    node_index,
                    parent_definition
                );

                match list_counts.last_mut() {
                    None => (),
                    Some(count) => {
                        if node.get_definition() != Definition::List {
                            *count += 1;
                            trace!("Adding to list, current count {:?}", count);
                        }
                    }
                }

                match node.get_definition() {
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
                        add_instructions_for_node(node.get_left(), nodes, instructions, data, list_counts, node.get_definition())?;

                        instructions.push(InstructionData::new(Instruction::Put, Some(0)));
                        instructions.push(InstructionData::new(Instruction::Apply, None));
                    }
                    Definition::Addition => {
                        add_instructions_for_node(node.get_left(), nodes, instructions, data, list_counts, node.get_definition())?;
                        add_instructions_for_node(node.get_right(), nodes, instructions, data, list_counts, node.get_definition())?;

                        instructions.push(InstructionData::new(Instruction::PerformAddition, None));
                    }
                    Definition::Equality => {
                        add_instructions_for_node(node.get_left(), nodes, instructions, data, list_counts, node.get_definition())?;
                        add_instructions_for_node(node.get_right(), nodes, instructions, data, list_counts, node.get_definition())?;

                        instructions.push(InstructionData::new(Instruction::EqualityComparison, None));
                    }
                    Definition::Pair => {
                        add_instructions_for_node(node.get_left(), nodes, instructions, data, list_counts, node.get_definition())?;
                        add_instructions_for_node(node.get_right(), nodes, instructions, data, list_counts, node.get_definition())?;

                        instructions.push(InstructionData::new(Instruction::MakePair, None));
                    }
                    Definition::Access => {
                        add_instructions_for_node(node.get_left(), nodes, instructions, data, list_counts, node.get_definition())?;
                        add_instructions_for_node(node.get_right(), nodes, instructions, data, list_counts, node.get_definition())?;

                        instructions.push(InstructionData::new(Instruction::Access, None));
                    }
                    Definition::List => {
                        list_counts.push(0);

                        add_instructions_for_node(node.get_left(), nodes, instructions, data, list_counts, node.get_definition())?;
                        add_instructions_for_node(node.get_right(), nodes, instructions, data, list_counts, node.get_definition())?;

                        if parent_definition != Definition::List {
                            instructions.push(InstructionData::new(
                                Instruction::MakeList,
                                Some(match list_counts.pop() {
                                    None => Err(format!("Count not get list count out of array."))?,
                                    Some(i) => i,
                                }),
                            ));
                        }
                    }
                    Definition::Subexpression => {
                        add_instructions_for_node(node.get_left(), nodes, instructions, data, list_counts, node.get_definition())?;

                        instructions.push(InstructionData::new(Instruction::PushResult, None));

                        add_instructions_for_node(node.get_right(), nodes, instructions, data, list_counts, node.get_definition())?;
                    }
                    Definition::Group => todo!(),
                    Definition::NestedExpression => todo!(),
                    Definition::Apply => {
                        add_instructions_for_node(node.get_left(), nodes, instructions, data, list_counts, node.get_definition())?;
                        add_instructions_for_node(node.get_right(), nodes, instructions, data, list_counts, node.get_definition())?;

                        instructions.push(InstructionData::new(Instruction::Apply, None));
                    }
                    Definition::ApplyTo => {
                        add_instructions_for_node(node.get_right(), nodes, instructions, data, list_counts, node.get_definition())?;
                        add_instructions_for_node(node.get_left(), nodes, instructions, data, list_counts, node.get_definition())?;

                        instructions.push(InstructionData::new(Instruction::Apply, None));
                    }
                    Definition::Reapply => {
                        add_instructions_for_node(node.get_right(), nodes, instructions, data, list_counts, node.get_definition())?;

                        instructions.push(InstructionData::new(Instruction::Reapply, None));
                    }
                    Definition::ApplyIfTrue => todo!(),
                    Definition::ApplyIfFalse => todo!(),
                    Definition::ConditionalBranch => todo!(),
                    // no runtime meaning, parser only utility
                    Definition::Drop => (),
                }

                trace!("{:?} at {:?} resolved", node.get_definition(), node_index);
            }
        },
    })
}

pub fn instructions_from_ast(root: usize, nodes: Vec<ParseNode>) -> Result<InstructionSet, String> {
    let mut instruction_set = InstructionSet::new();
    let mut list_counts = vec![];
    // let mut making_list =

    add_instructions_for_node(
        Some(root),
        &nodes,
        &mut instruction_set.instructions,
        &mut instruction_set.data,
        &mut list_counts,
        Definition::Drop, // Initial value shouln't matter
    )?;

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

    #[test]
    fn addition() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::Addition, None, Some(0), Some(2), "+", TokenType::EmptyApply),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::Put, Some(2)),
                (Instruction::PerformAddition, None),
                (Instruction::EndExpression, None),
            ],
            vec![ExpressionData::integer(5), ExpressionData::integer(10)],
        );
    }

    #[test]
    fn equality() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::Equality, None, Some(0), Some(2), "==", TokenType::Equality),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::Put, Some(2)),
                (Instruction::EqualityComparison, None),
                (Instruction::EndExpression, None),
            ],
            vec![ExpressionData::integer(5), ExpressionData::integer(10)],
        );
    }

    #[test]
    fn pair() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::Pair, None, Some(0), Some(2), "=", TokenType::Pair),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::Put, Some(2)),
                (Instruction::MakePair, None),
                (Instruction::EndExpression, None),
            ],
            vec![ExpressionData::integer(5), ExpressionData::integer(10)],
        );
    }

    #[test]
    fn access() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::Access, None, Some(0), Some(2), ".", TokenType::Period),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::Put, Some(2)),
                (Instruction::Access, None),
                (Instruction::EndExpression, None),
            ],
            vec![ExpressionData::integer(5), ExpressionData::integer(10)],
        );
    }

    #[test]
    fn subexpression() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::Subexpression, None, Some(0), Some(2), "\n\n", TokenType::Subexpression),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::PushResult, None),
                (Instruction::Put, Some(2)),
                (Instruction::EndExpression, None),
            ],
            vec![ExpressionData::integer(5), ExpressionData::integer(10)],
        );
    }

    #[test]
    fn apply() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::Apply, None, Some(0), Some(2), "~", TokenType::Apply),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::Put, Some(2)),
                (Instruction::Apply, None),
                (Instruction::EndExpression, None),
            ],
            vec![ExpressionData::integer(5), ExpressionData::integer(10)],
        );
    }

    #[test]
    fn apply_to() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::ApplyTo, None, Some(0), Some(2), "~>", TokenType::ApplyTo),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::Put, Some(2)),
                (Instruction::Apply, None),
                (Instruction::EndExpression, None),
            ],
            vec![ExpressionData::integer(10), ExpressionData::integer(5)],
        );
    }

    #[test]
    fn reapply() {
        assert_instruction_data(
            0,
            vec![
                (Definition::Reapply, None, None, Some(1), "^~", TokenType::Reapply),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::Reapply, None),
                (Instruction::EndExpression, None),
            ],
            vec![ExpressionData::integer(10)],
        );
    }
}

#[cfg(test)]
mod lists {
    use std::vec;

    use super::test_utils::*;
    use crate::*;
    use garnish_lang_runtime::*;

    #[test]
    fn list() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::List, None, Some(0), Some(2), ",", TokenType::Comma),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::Put, Some(2)),
                (Instruction::MakeList, Some(2)),
                (Instruction::EndExpression, None),
            ],
            vec![ExpressionData::integer(5), ExpressionData::integer(10)],
        );
    }

    #[test]
    fn multiple_items_in_list() {
        assert_instruction_data(
            5,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::List, Some(3), Some(0), Some(2), ",", TokenType::Comma),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
                (Definition::List, Some(5), Some(1), Some(4), ",", TokenType::Comma),
                (Definition::Number, Some(3), None, None, "15", TokenType::Number),
                (Definition::List, None, Some(3), Some(6), ",", TokenType::Comma),
                (Definition::Number, Some(5), None, None, "20", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::Put, Some(2)),
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::MakeList, Some(4)),
                (Instruction::EndExpression, None),
            ],
            vec![
                ExpressionData::integer(5),
                ExpressionData::integer(10),
                ExpressionData::integer(15),
                ExpressionData::integer(20),
            ],
        );
    }
}
