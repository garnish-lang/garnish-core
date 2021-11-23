use std::usize;

use garnish_lang_runtime::InstructionData;
use garnish_lang_runtime::*;
use log::trace;

use crate::{parser::*};

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

struct ResolveNodeInfo {
    node_index: Option<usize>,
    first_resolved: bool,
    second_resolved: bool,
    resolved: bool,
    parent_definition: Definition
}

impl ResolveNodeInfo {
    fn new(node_index: Option<usize>, parent_definition: Definition) -> ResolveNodeInfo {
        ResolveNodeInfo {
            node_index,
            first_resolved: false,
            second_resolved: false,
            resolved: false,
            parent_definition
        }
    }
}

type DefinitionResolveInfo = (bool, Option<usize>);

fn get_resolve_info(node: &ParseNode) -> (DefinitionResolveInfo, DefinitionResolveInfo) {
    match node.get_definition() {
        Definition::Number | Definition::Identifier | Definition::Symbol | Definition::Input | Definition::Result | Definition::Unit => {
            ((false, None), (false, None))
        }
        Definition::Reapply => ((true, node.get_right()), (false, None)),
        Definition::AbsoluteValue => todo!(),
        Definition::EmptyApply => ((true, node.get_left()), (false, None)),
        Definition::Addition 
        | Definition::Equality 
        | Definition::Pair 
        | Definition::Access 
        | Definition::Subexpression // Same order for child resolution but has special check, might need to move out of here eventually
        | Definition::Apply => {
            ((true, node.get_left()), (true, node.get_right()))
        }
        Definition::ApplyTo => ((true, node.get_right()), (true, node.get_left())),
        Definition::List => ((true, node.get_left()), (true, node.get_right())),
        Definition::Group => ((true, node.get_right()), (false, None)),
        Definition::NestedExpression => todo!(),
        Definition::ApplyIfTrue => todo!(),
        Definition::ApplyIfFalse => todo!(),
        Definition::ConditionalBranch => todo!(),
        Definition::Drop => todo!(),
    }
}

fn resolve_node(node: &ParseNode, instructions: &mut Vec<InstructionData>, data: &mut Vec<ExpressionData>, list_count: Option<&usize>) -> Result<(), String> {
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
            instructions.push(InstructionData::new(Instruction::Put, Some(0)));
            instructions.push(InstructionData::new(Instruction::Apply, None));
        }
        Definition::Addition => {
            instructions.push(InstructionData::new(Instruction::PerformAddition, None));
        }
        Definition::Equality => {
            instructions.push(InstructionData::new(Instruction::EqualityComparison, None));
        }
        Definition::Pair => {
            instructions.push(InstructionData::new(Instruction::MakePair, None));
        }
        Definition::Access => {
            instructions.push(InstructionData::new(Instruction::Access, None));
        }
        Definition::List => {
            match list_count {
                None => Err(format!("No list count passed to list node resolve."))?,
                Some(count) => {
                    instructions.push(InstructionData::new(
                        Instruction::MakeList,
                        Some(*count),
                    ));
                }
            }
        }
        Definition::Subexpression => {
            instructions.push(InstructionData::new(Instruction::PushResult, None));
        }
        Definition::Group => (), // no additional instructions for groups
        Definition::NestedExpression => todo!(),
        Definition::Apply => {
            instructions.push(InstructionData::new(Instruction::Apply, None));
        }
        Definition::ApplyTo => {
            instructions.push(InstructionData::new(Instruction::Apply, None));
        }
        Definition::Reapply => {
            instructions.push(InstructionData::new(Instruction::Reapply, None));
        }
        Definition::ApplyIfTrue => todo!(),
        Definition::ApplyIfFalse => todo!(),
        Definition::ConditionalBranch => todo!(),
        // no runtime meaning, parser only utility
        Definition::Drop => (),
    }

    Ok(())
}

pub fn instructions_from_ast(root: usize, nodes: Vec<ParseNode>) -> Result<InstructionSet, String> {
    let mut instruction_set = InstructionSet::new();

    let mut list_counts: Vec<usize> = vec![];
    // let mut making_list =

    let mut stack = vec![ResolveNodeInfo::new(Some(root), Definition::Drop)];

    loop {
        let pop = match stack.last_mut() {
            None => break, // no more nodes
            Some(resolve_node_info) => match resolve_node_info.node_index {
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
                        trace!("---------------------------------------------------------");
                        trace!("Visiting node with definition {:?} at {:?}", node.get_definition(), node_index);

                        let ((first_expected, first_index), (second_expected, second_index)) = get_resolve_info(node);
                        // check first child
                        // if child not resolved, return it to be added
                        let pop = if first_expected && !resolve_node_info.first_resolved {
                            // on first visit to a list node
                            // if parent isn't a list, we start a new list count
                            if node.get_definition() == Definition::List && resolve_node_info.parent_definition != Definition::List {
                                trace!("Starting new list count");
                                list_counts.push(0);
                            }

                            trace!("Pushing first child {:?}", first_index);

                            resolve_node_info.first_resolved = true;
                            stack.push(ResolveNodeInfo::new(first_index, node.get_definition()));

                            false
                        } else if second_expected && !resolve_node_info.second_resolved {
                            // special check for subexpression, so far is only operations that isn't fully depth first
                            // gets resolved before second child
                            if node.get_definition() == Definition::Subexpression {
                                trace!("Resolving {:?} at {:?} (Subexpression)", node.get_definition(), node_index);

                                resolve_node(node, &mut instruction_set.instructions, &mut instruction_set.data, None)?;
                                resolve_node_info.resolved = true;
                            }

                            // check next child
                            trace!("Pushing second child {:?}", second_index);

                            resolve_node_info.second_resolved = true;
                            stack.push(ResolveNodeInfo::new(second_index, node.get_definition()));

                            false
                        } else {
                            // all children resolved, now resolve this node
                            let we_are_subexpression = node.get_definition() == Definition::Subexpression;
                            let we_are_list = node.get_definition() == Definition::List;
                            let parent_is_list = resolve_node_info.parent_definition == Definition::List;
                            let we_are_sublist = parent_is_list && we_are_list;

                            let resolve = !we_are_subexpression && !we_are_sublist;

                            // subexpression already resolved before second child
                            if resolve {
                                trace!("Resolving {:?} at {:?}", node.get_definition(), node_index);

                                resolve_node(node, &mut instruction_set.instructions, &mut instruction_set.data, list_counts.last())?;
                            }

                            // If this node's parent is a list
                            // add to its count, unless we are a list
                            if parent_is_list && !we_are_list {
                                match list_counts.last_mut() {
                                    None => Err(format!("Child of list node has no count add to."))?,
                                    Some(count) => {
                                        *count += 1;
                                        trace!("Added to current list count. Current count is at {:?}", count);
                                    }
                                }
                            }

                            trace!("Node with definition {:?} at {:?} fully resolved", node.get_definition(), node_index);

                            resolve_node_info.resolved = true;

                            true
                        };

                        trace!("---------------------------------------------------------");

                        pop
                    }
                },
            },
        };

        if pop {
            stack.pop();
        }
    }

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
    fn multiple_addition() {
        assert_instruction_data(
            5,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::Addition, Some(3), Some(0), Some(2), "+", TokenType::EmptyApply),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
                (Definition::Addition, Some(5), Some(1), Some(4), "+", TokenType::EmptyApply),
                (Definition::Number, Some(3), None, None, "15", TokenType::Number),
                (Definition::Addition, None, Some(3), Some(6), "+", TokenType::EmptyApply),
                (Definition::Number, Some(5), None, None, "20", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::Put, Some(2)),
                (Instruction::PerformAddition, None),
                (Instruction::Put, Some(3)),
                (Instruction::PerformAddition, None),
                (Instruction::Put, Some(4)),
                (Instruction::PerformAddition, None),
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

#[cfg(test)]
mod groups {
    use std::vec;

    use super::test_utils::*;
    use crate::*;
    use garnish_lang_runtime::*;

    #[test]
    fn single_operation() {
        assert_instruction_data(
            0,
            vec![
                (Definition::Group, None, None, Some(2), "(", TokenType::StartGroup),
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::Addition, Some(0), Some(1), Some(3), "+", TokenType::PlusSign),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::Put, Some(2)),
                (Instruction::PerformAddition, None),
                (Instruction::EndExpression, None),
            ],
            vec![
                ExpressionData::integer(5),
                ExpressionData::integer(10),
            ],
        );
    }
}