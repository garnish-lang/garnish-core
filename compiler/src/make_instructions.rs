use std::usize;

use garnish_lang_runtime::InstructionData;
use garnish_lang_runtime::*;
use log::trace;

use crate::{parser::*};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct InstructionSet {
    instructions: Vec<InstructionData>,
    data: Vec<ExpressionData>,
    jump_table: Vec<usize>
}

impl InstructionSet {
    pub fn new() -> InstructionSet {
        let mut data = vec![];
        data.push(ExpressionData::unit());

        InstructionSet { instructions: vec![], data, jump_table: vec![] }
    }

    pub fn get_instructions(&self) -> &Vec<InstructionData> {
        &self.instructions
    }

    pub fn get_data(&self) -> &Vec<ExpressionData> {
        &self.data
    }

    pub fn get_jump_table(&self) -> &Vec<usize> {
        &self.jump_table
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
        Definition::NestedExpression => ((false, None), (false, None)),
        Definition::ApplyIfTrue => todo!(),
        Definition::ApplyIfFalse => todo!(),
        Definition::ConditionalBranch => todo!(),
        Definition::Drop => todo!(),
    }
}

fn resolve_node(node: &ParseNode, instructions: &mut Vec<InstructionData>, data: &mut Vec<ExpressionData>, list_count: Option<&usize>, current_jump_index: usize) -> Result<(), String> {
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
        Definition::NestedExpression => {
            instructions.push(InstructionData::new(Instruction::Put, Some(data.len())));

            data.push(ExpressionData::expression(current_jump_index));
        },
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

    let mut root_stack = vec![root];

    // since we will be popping and pushing values from root_stack
    // need to keep separate count of total so expression values put in data are accurate
    let mut root_count = 1;

    // arbitrary max iterations for roots
    let max_roots = 100;
    let mut root_iter_count = 0;

    while let Some(root_index) = root_stack.pop() {
        trace!("Makeing instructions for tree starting at index {:?}", root_index);

        let mut stack = vec![ResolveNodeInfo::new(Some(root_index), Definition::Drop)];
        
        // push start of this expression to jump table
        instruction_set.jump_table.push(instruction_set.instructions.len());

        // limit, maximum times a node is visited is 3
        // so limit to 3 times node count should allow for more than enough
        let max_iterations = nodes.len() * 3;
        let mut iter_count = 0;

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
    
                                    resolve_node(node, &mut instruction_set.instructions, &mut instruction_set.data, None, 0)?;
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
    
                                    resolve_node(node, &mut instruction_set.instructions, &mut instruction_set.data, list_counts.last(), root_count)?;
                                }
    
                                // after resolving, if a nested expression
                                // and add nested expressions child to deferred list
                                if node.get_definition() == Definition::NestedExpression {
                                    match node.get_right() {
                                        None => Err(format!("No child value on {:?} node at {:?}", node.get_definition(), node_index))?,
                                        Some(node_index) => {
                                            trace!("Adding index {:?} to root stack", node_index);
                                            root_stack.push(node_index)
                                        }
                                    }

                                    // up root count as well
                                    root_count += 1;
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

            iter_count += 1;
            if iter_count > max_iterations {
                return Err(format!("Max iterations reached in resolving tree at root {:?}", root_index));
            }
        }
    
        instruction_set.instructions.push(InstructionData::new(Instruction::EndExpression, None));

        trace!("Finished instructions for tree starting at index {:?}", root_index);

        root_iter_count += 1;

        if root_iter_count > max_roots {
            return Err(format!("Max iterations for roots reached."));
        }
    }
    

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
        expected_instructions: Vec<(Instruction, Option<usize>)>,
        expected_data: Vec<ExpressionData>,
    ) {
        assert_instruction_data_jumps(root, nodes, expected_instructions, expected_data, vec![0]);
    }

    pub fn assert_instruction_data_jumps(
        root: usize,
        nodes: Vec<(Definition, Option<usize>, Option<usize>, Option<usize>, &str, TokenType)>,
        expected_instructions: Vec<(Instruction, Option<usize>)>,
        expected_data: Vec<ExpressionData>,
        expected_jumps: Vec<usize>
    ) {
        let expected_instructions: Vec<InstructionData> = expected_instructions.iter().map(|i| InstructionData::new(i.0, i.1)).collect();

        let expected_data: Vec<ExpressionData> = iter::once(ExpressionData::unit()).chain(expected_data.into_iter()).collect();

        let result = get_instruction_data(root, nodes).unwrap();

        assert_eq!(result.get_instructions().clone(), expected_instructions);
        assert_eq!(result.get_data(), &expected_data);
        assert_eq!(result.get_jump_table(), &expected_jumps);
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

    #[test]
    fn list_in_list() {
        assert_instruction_data(
            10,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::List, Some(3), Some(0), Some(2), ",", TokenType::Comma), // 1
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
                (Definition::List, Some(8), Some(1), Some(4), ",", TokenType::Comma), // 3
                (Definition::Group, None, None, Some(6), "(", TokenType::StartGroup),
                (Definition::Number, Some(6), None, None, "15", TokenType::Number), 
                (Definition::List, Some(4), Some(5), Some(7), ",", TokenType::Comma), // 6
                (Definition::Number, Some(6), None, None, "20", TokenType::Number), 
                (Definition::List, Some(10), Some(3), Some(9), ",", TokenType::Comma), // 8
                (Definition::Number, Some(8), None, None, "25", TokenType::Number),
                (Definition::List, None, Some(8), Some(11), ",", TokenType::Comma), // 10
                (Definition::Number, Some(10), None, None, "30", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::Put, Some(2)),
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::MakeList, Some(2)),
                (Instruction::Put, Some(5)),
                (Instruction::Put, Some(6)),
                (Instruction::MakeList, Some(5)),
                (Instruction::EndExpression, None),
            ],
            vec![
                ExpressionData::integer(5),
                ExpressionData::integer(10),
                ExpressionData::integer(15),
                ExpressionData::integer(20),
                ExpressionData::integer(25),
                ExpressionData::integer(30),
            ],
        );
    }
}

#[cfg(test)]
mod groups {
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

#[cfg(test)]
mod nested_expressions {
    use super::test_utils::*;
    use crate::*;
    use garnish_lang_runtime::*;

    #[test]
    fn single_nested() {
        assert_instruction_data_jumps(
            0,
            vec![
                (Definition::NestedExpression, None, None, Some(2), "{", TokenType::StartExpression),
                (Definition::Number, Some(2), None, None, "5", TokenType::Number),
                (Definition::Addition, Some(0), Some(1), Some(3), "+", TokenType::PlusSign),
                (Definition::Number, Some(2), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::EndExpression, None),
                (Instruction::Put, Some(2)),
                (Instruction::Put, Some(3)),
                (Instruction::PerformAddition, None),
                (Instruction::EndExpression, None),
            ],
            vec![
                ExpressionData::expression(1),
                ExpressionData::integer(5),
                ExpressionData::integer(10),
            ],
            vec![0, 2]
        );
    }

    #[test]
    fn multiple_nested() {
        assert_instruction_data_jumps(
            0,
            vec![
                (Definition::NestedExpression, None, None, Some(2), "{", TokenType::StartExpression),
                (Definition::Number, Some(2), None, None, "5", TokenType::Number),
                (Definition::Apply, Some(0), Some(1), Some(3), "~", TokenType::Apply), // 2
                (Definition::NestedExpression, Some(2), None, Some(5), "{", TokenType::StartExpression),
                (Definition::Number, Some(5), None, None, "10", TokenType::Number),
                (Definition::Apply, Some(3), Some(4), Some(6), "~", TokenType::Apply), // 5
                (Definition::NestedExpression, Some(5), None, Some(7), "{", TokenType::StartExpression),
                (Definition::Number, Some(6), None, None, "15", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::EndExpression, None), // 1
                (Instruction::Put, Some(2)),
                (Instruction::Put, Some(3)),
                (Instruction::Apply, None),
                (Instruction::EndExpression, None), // 5
                (Instruction::Put, Some(4)),
                (Instruction::Put, Some(5)),
                (Instruction::Apply, None),
                (Instruction::EndExpression, None), // 9
                (Instruction::Put, Some(6)),
                (Instruction::EndExpression, None),
            ],
            vec![
                ExpressionData::expression(1),
                ExpressionData::integer(5),
                ExpressionData::expression(2),
                ExpressionData::integer(10),
                ExpressionData::expression(3),
                ExpressionData::integer(15),
            ],
            vec![0, 2, 6, 10]
        );
    }
}