use garnish_lang_runtime::*;
use log::trace;

use crate::parser::*;

pub type GarnishLangCompilerResult<T, N> = Result<T, GarnishLangCompilerError<N>>;
pub type GarnishLangCompilerError<T> = NestedError<T>;

struct ResolveNodeInfo {
    node_index: Option<usize>,
    first_resolved: bool,
    can_ignore_first: bool,
    second_resolved: bool,
    resolved: bool,
    parent_definition: Definition,
}

impl ResolveNodeInfo {
    fn new(node_index: Option<usize>, parent_definition: Definition) -> ResolveNodeInfo {
        ResolveNodeInfo {
            node_index,
            first_resolved: false,
            can_ignore_first: false,
            second_resolved: false,
            resolved: false,
            parent_definition,
        }
    }
}

type DefinitionResolveInfo = (bool, Option<usize>);

fn get_resolve_info(node: &ParseNode) -> (DefinitionResolveInfo, DefinitionResolveInfo) {
    match node.get_definition() {
        Definition::Number
        | Definition::Identifier
        | Definition::Property
        | Definition::Symbol
        | Definition::Value
        | Definition::Unit
        | Definition::False
        | Definition::True => {
            ((false, None), (false, None))
        }
        Definition::AccessLeftInternal => ((true, node.get_right()), (false, None)),
        Definition::AbsoluteValue => todo!(),
        Definition::EmptyApply | Definition::AccessLengthInternal | Definition::AccessRightInternal => ((true, node.get_left()), (false, None)),
        Definition::Addition
        | Definition::Equality
        | Definition::Pair
        | Definition::Access
        | Definition::Subexpression // Same order for child resolution but has special check, might need to move out of here eventually
        | Definition::Reapply
        | Definition::Apply => {
            ((true, node.get_left()), (true, node.get_right()))
        }
        Definition::ApplyTo => ((true, node.get_right()), (true, node.get_left())),
        Definition::List | Definition::CommaList => ((true, node.get_left()), (true, node.get_right())),
        Definition::Group => ((true, node.get_right()), (false, None)),
        Definition::NestedExpression => ((false, None), (false, None)),
        Definition::JumpIfTrue => ((true, node.get_left()), (false, None)),
        Definition::JumpIfFalse => ((true, node.get_left()), (false, None)),
        Definition::ElseJump => ((true, node.get_left()), (true, node.get_right())),
        Definition::Drop => todo!(),
    }
}

fn resolve_node<Data: GarnishLangRuntimeData>(
    node: &ParseNode,
    data: &mut Data,
    list_count: Option<&Data::Size>,
    current_jump_index: Data::Size,
    nearest_expression_point: Data::Size,
) -> GarnishLangCompilerResult<(), Data::Error> {
    match node.get_definition() {
        Definition::Number => {
            data.push_instruction(Instruction::Put, Some(data.get_data_len())).nest_into()?;

            data.add_integer(match node.get_lex_token().get_text().parse::<Data::Integer>() {
                Err(_) => Err(GarnishLangCompilerError::new(format!(
                    "Could not parse value from integer string {:?}",
                    node.get_lex_token().get_text()
                )))?,
                Ok(i) => i,
            })
            .nest_into()?;
        }
        Definition::Identifier => {
            data.push_instruction(Instruction::Put, Some(data.get_data_len())).nest_into()?;
            data.push_instruction(Instruction::Resolve, None).nest_into()?;

            data.add_symbol(node.get_lex_token().get_text()).nest_into()?;
        }
        Definition::Property => {
            data.push_instruction(Instruction::Put, Some(data.get_data_len())).nest_into()?;

            data.add_symbol(node.get_lex_token().get_text()).nest_into()?;
        }
        Definition::Unit => {
            // all unit literals will use unit used in the zero element slot of data
            data.push_instruction(Instruction::Put, Some(Data::Size::zero())).nest_into()?;
        }
        Definition::Symbol => {
            data.push_instruction(Instruction::Put, Some(data.get_data_len())).nest_into()?;

            data.add_symbol(&node.get_lex_token().get_text()[1..]).nest_into()?;
        }
        Definition::Value => {
            // all unit literals will use unit used in the zero element slot of data
            data.push_instruction(Instruction::PutValue, None).nest_into()?;
        }
        Definition::True => {
            data.push_instruction(Instruction::Put, Some(data.get_data_len())).nest_into()?;
            data.add_true().nest_into()?;
        }
        Definition::False => {
            data.push_instruction(Instruction::Put, Some(data.get_data_len())).nest_into()?;
            data.add_false().nest_into()?;
        }
        Definition::AbsoluteValue => todo!(), // not currently in runtime
        Definition::EmptyApply => {
            data.push_instruction(Instruction::EmptyApply, None).nest_into()?;
        }
        Definition::Addition => {
            data.push_instruction(Instruction::PerformAddition, None).nest_into()?;
        }
        Definition::Equality => {
            data.push_instruction(Instruction::EqualityComparison, None).nest_into()?;
        }
        Definition::Pair => {
            data.push_instruction(Instruction::MakePair, None).nest_into()?;
        }
        Definition::Access => {
            data.push_instruction(Instruction::Access, None).nest_into()?;
        }
        Definition::AccessLeftInternal => {
            data.push_instruction(Instruction::AccessLeftInternal, None).nest_into()?;
        }
        Definition::AccessRightInternal => {
            data.push_instruction(Instruction::AccessRightInternal, None).nest_into()?;
        }
        Definition::AccessLengthInternal => {
            data.push_instruction(Instruction::AccessLengthInternal, None).nest_into()?;
        }
        Definition::List | Definition::CommaList => match list_count {
            None => Err(GarnishLangCompilerError::new(format!("No list count passed to list node resolve.")))?,
            Some(count) => {
                data.push_instruction(Instruction::MakeList, Some(*count)).nest_into()?;
            }
        },
        Definition::Subexpression => {
            data.push_instruction(Instruction::UpdateValue, None).nest_into()?;
        }
        Definition::Group => (), // no additional instructions for groups
        Definition::NestedExpression => {
            data.push_instruction(Instruction::Put, Some(data.get_data_len())).nest_into()?;

            data.add_expression(current_jump_index).nest_into()?;
        }
        Definition::Apply => {
            data.push_instruction(Instruction::Apply, None).nest_into()?;
        }
        Definition::ApplyTo => {
            data.push_instruction(Instruction::Apply, None).nest_into()?;
        }
        Definition::Reapply => {
            data.push_instruction(Instruction::Reapply, Some(nearest_expression_point)).nest_into()?;
        }
        Definition::JumpIfTrue => {
            data.push_instruction(Instruction::JumpIfTrue, Some(current_jump_index)).nest_into()?;
        }
        Definition::JumpIfFalse => {
            data.push_instruction(Instruction::JumpIfFalse, Some(current_jump_index)).nest_into()?;
        }
        Definition::ElseJump => (), // no additional instructions
        // no runtime meaning, parser only utility
        Definition::Drop => (),
    }

    Ok(())
}

pub fn instructions_from_ast<Data: GarnishLangRuntimeData>(root: usize, nodes: Vec<ParseNode>, data: &mut Data) -> GarnishLangCompilerResult<(), Data::Error> {
    // since we will be popping and pushing values from root_stack
    // need to keep separate count of total so expression values put in data are accurate
    let mut jump_count = data.get_jump_point_count() + Data::Size::one();

    // tuple of (root node, last instruction of this node, nearest expression jump point)
    let mut root_stack = vec![(root, Instruction::EndExpression, None, data.get_jump_point_count())];

    // arbitrary max iterations for roots
    let max_roots = 100;
    let mut root_iter_count = 0;

    while let Some((root_index, return_instruction, instruction_data, nearest_expression_point)) = root_stack.pop() {
        trace!("Makeing instructions for tree starting at index {:?}", root_index);

        let mut conditional_stack = vec![];
        let mut list_counts: Vec<Data::Size> = vec![];
        let mut stack = vec![ResolveNodeInfo::new(Some(root_index), Definition::Drop)];

        // push start of this expression to jump table
        let jump_point = data.get_instruction_len();
        let jump_index = data.get_jump_point_count();
        data.push_jump_point(Data::Size::zero()).nest_into()?; // updated down below

        // limit, maximum times a node is visited is 3
        // so limit to 3 times node count should allow for more than enough
        let max_iterations = nodes.len() * 3;
        let mut iter_count = 0;

        loop {
            let pop = match stack.last_mut() {
                None => break, // no more nodes
                Some(resolve_node_info) => match resolve_node_info.node_index {
                    None => Err(GarnishLangCompilerError::new(format!(
                        "None value for node index. All nodes should resolve properly if starting from root node."
                    )))?,
                    Some(node_index) => match nodes.get(node_index) {
                        // all nodes should exist if starting from root
                        None => Err(GarnishLangCompilerError::new(format!(
                            "Node at index {:?} does not exist. All nodes should resolve properly if starting from root node.",
                            node_index
                        )))?,
                        Some(node) => {
                            trace!("---------------------------------------------------------");
                            trace!("Visiting node with definition {:?} at {:?}", node.get_definition(), node_index);

                            let ((first_expected, first_index), (second_expected, second_index)) = get_resolve_info(node);

                            let we_are_conditional =
                                node.get_definition() == Definition::JumpIfFalse || node.get_definition() == Definition::JumpIfTrue;

                            let we_are_parent_conditional_branch =
                                node.get_definition() == Definition::ElseJump && resolve_node_info.parent_definition != Definition::ElseJump;

                            let we_are_non_chained_conditional = we_are_conditional && resolve_node_info.parent_definition != Definition::ElseJump;

                            let we_are_list = node.get_definition() == Definition::List;
                            let we_are_comma_list = node.get_definition() == Definition::CommaList;
                            let we_are_list_type = we_are_list || we_are_comma_list;

                            let parent_is_list_type = resolve_node_info.parent_definition == Definition::List
                                || resolve_node_info.parent_definition == Definition::CommaList;

                            let we_are_sub_list =
                                we_are_list_type && parent_is_list_type && node.get_definition() == resolve_node_info.parent_definition;

                            let we_are_list_root = we_are_list_type && node.get_definition() != resolve_node_info.parent_definition;

                            // we are list item if either
                            // we are not a list and our parent is
                            // or we are a list and our parent is the other type of list (i.e. we are List and parent is CommaList or we are CommaList and our parent is List)
                            let we_are_list_item = !we_are_list_type && parent_is_list_type
                                || we_are_list && resolve_node_info.parent_definition == Definition::CommaList
                                || we_are_comma_list && resolve_node_info.parent_definition == Definition::List;

                            let pop = if first_expected && !resolve_node_info.first_resolved {
                                // on first visit to a list node
                                // if parent isn't a list, we start a new list count
                                if we_are_list_root {
                                    trace!("Starting new list count");
                                    list_counts.push(Data::Size::zero());
                                }

                                // on first visit to a conditional branch node
                                // add to jump table and store the position in conditional stack
                                if we_are_parent_conditional_branch || we_are_non_chained_conditional {
                                    trace!(
                                        "Starting new conditional branch. Return slot in jump table {:?}",
                                        data.get_jump_point_count()
                                    );
                                    let i = data.get_jump_point_count();
                                    data.push_jump_point(Data::Size::zero()).nest_into()?; // this will be updated when conditional branch nod resolves
                                    conditional_stack.push(i);
                                    jump_count += Data::Size::one();
                                }

                                trace!("Pushing first child {:?}", first_index);

                                resolve_node_info.first_resolved = true;

                                // ignore only if first doesn't exist
                                if !(resolve_node_info.can_ignore_first && first_index.is_none()) {
                                    stack.push(ResolveNodeInfo::new(first_index, node.get_definition()));
                                }

                                false
                            } else if second_expected && !resolve_node_info.second_resolved {
                                // special check for subexpression, so far is only operations that isn't fully depth first
                                // gets resolved before second child
                                if node.get_definition() == Definition::Subexpression {
                                    trace!("Resolving {:?} at {:?} (Subexpression)", node.get_definition(), node_index);

                                    resolve_node(node, data, None, Data::Size::zero(), nearest_expression_point)?;
                                    resolve_node_info.resolved = true;
                                }

                                // check next child
                                trace!("Pushing second child {:?}", second_index);

                                let mut new = ResolveNodeInfo::new(second_index, node.get_definition());

                                // if we are the root of a conditional branch (parent isn't also a conditional branch)
                                // our second child (right) is allowed to ignore their first child
                                // need to pass this information
                                if node.get_definition() == Definition::ElseJump && resolve_node_info.parent_definition != Definition::ElseJump {
                                    trace!("Allowing second child of root conditional branch to ignore its first child.");
                                    new.can_ignore_first = true;
                                }

                                resolve_node_info.second_resolved = true;
                                stack.push(new);

                                false
                            } else {
                                // all children resolved, now resolve this node
                                let we_are_subexpression = node.get_definition() == Definition::Subexpression;

                                let resolve = !we_are_subexpression && !we_are_sub_list;

                                // subexpression already resolved before second child
                                if resolve {
                                    trace!("Resolving {:?} at {:?}", node.get_definition(), node_index);

                                    resolve_node(node, data, list_counts.last(), jump_count, nearest_expression_point)?;

                                    if we_are_list_root {
                                        list_counts.pop();
                                    }
                                }

                                // after resolving, if a nested expression
                                // and add nested expressions child to deferred list
                                let mut return_instruction = (Instruction::EndExpression, None);
                                if we_are_conditional {
                                    return_instruction = match conditional_stack.last() {
                                        None => Err(GarnishLangCompilerError::new(format!(
                                            "Conditional stack empty when attempting to resolve conditional."
                                        )))?,
                                        // apply if true/false resolve fully before the parent conditional branch
                                        // second look up to get accurate value instruction data will happen when this instruction is placed
                                        // add 1 to account for current base jump
                                        Some(return_index) => (Instruction::JumpTo, Some(*return_index)),
                                    };
                                    trace!(
                                        "Return instruction for conditional is {:?} with data {:?}",
                                        return_instruction.0,
                                        return_instruction.1
                                    );
                                }

                                // if conditional branch, need to update jump table and pop from conditional stack
                                if we_are_parent_conditional_branch || we_are_non_chained_conditional {
                                    let count = data.get_instruction_len();
                                    match conditional_stack.pop() {
                                        None => Err(GarnishLangCompilerError::new(format!(
                                            "End of conditional branch and conditional stack is empty."
                                        )))?,
                                        Some(jump_index) => match data.get_jump_point_mut(jump_index) {
                                            None => Err(GarnishLangCompilerError::new(format!(
                                                "No value in jump table at index {:?} to update for conditional branch.",
                                                jump_index
                                            )))?,
                                            Some(jump_value) => {
                                                trace!(
                                                    "Updating jump table at index {:?} to {:?} for conditional branch.",
                                                    jump_index,
                                                    *jump_value
                                                );
                                                *jump_value = count;
                                            }
                                        },
                                    }
                                }

                                if node.get_definition() == Definition::NestedExpression || we_are_conditional {
                                    let nearest_expression = if node.get_definition() == Definition::NestedExpression {
                                        jump_count
                                    } else {
                                        nearest_expression_point
                                    };
                                    match node.get_right() {
                                        None => Err(GarnishLangCompilerError::new(format!(
                                            "No child value on {:?} node at {:?}",
                                            node.get_definition(),
                                            node_index
                                        )))?,
                                        Some(node_index) => {
                                            trace!("Adding index {:?} to root stack", node_index);
                                            root_stack.insert(0, (node_index, return_instruction.0, return_instruction.1, nearest_expression))
                                        }
                                    }

                                    // up root count as well
                                    jump_count += Data::Size::one();
                                }

                                // If this node's parent is a list
                                // add to its count, unless we are a list
                                if we_are_list_item {
                                    match list_counts.last_mut() {
                                        None => Err(GarnishLangCompilerError::new(format!("Child of list node has no count add to.")))?,
                                        Some(count) => {
                                            *count += Data::Size::one();
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
                return Err(GarnishLangCompilerError::new(format!(
                    "Max iterations reached in resolving tree at root {:?}",
                    root_index
                )));
            }
        }

        trace!("Adding return instruction {:?} with data {:?}", return_instruction, instruction_data);

        data.push_instruction(return_instruction, instruction_data).nest_into()?;

        trace!("Finished instructions for tree starting at index {:?}", root_index);

        root_iter_count += 1;

        if root_iter_count > max_roots {
            return Err(GarnishLangCompilerError::new(format!("Max iterations for roots reached.")));
        }

        data.get_jump_point_mut(jump_index)
            .and_then(|i| Some(*i = jump_point))
            .ok_or(GarnishLangCompilerError::new(format!(
                "Failed to update jump point at index {:?} because None was returned.",
                jump_index
            )))?;
    }

    Ok(())
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
        assert_instruction_data_jumps(root, nodes, expected_instructions, expected_data, vec![1]);
    }

    pub fn assert_instruction_data_jumps(
        root: usize,
        nodes: Vec<(Definition, Option<usize>, Option<usize>, Option<usize>, &str, TokenType)>,
        expected_instructions: Vec<(Instruction, Option<usize>)>,
        expected_data: Vec<ExpressionData>,
        expected_jumps: Vec<usize>,
    ) {
        let expected_instructions: Vec<InstructionData> = expected_instructions.iter().map(|i| InstructionData::new(i.0, i.1)).collect();
        let expected_instructions: Vec<InstructionData> = iter::once(InstructionData::new(Instruction::EndExecution, None))
            .chain(expected_instructions.into_iter())
            .collect();

        let expected_data: Vec<ExpressionData> = iter::once(ExpressionData::unit()).chain(expected_data.into_iter()).collect();

        let result = get_instruction_data(root, nodes).unwrap();

        assert_eq!(result.get_instructions().clone(), expected_instructions);
        assert_eq!(result.get_data(), &expected_data);
        assert_eq!(result.get_jump_points(), &expected_jumps);
    }

    pub fn get_instruction_data(
        root: usize,
        nodes: Vec<(Definition, Option<usize>, Option<usize>, Option<usize>, &str, TokenType)>,
    ) -> GarnishLangCompilerResult<SimpleRuntimeData, String> {
        let nodes: Vec<ParseNode> = nodes
            .iter()
            .map(|v| ParseNode::new(v.0, v.1, v.2, v.3, LexerToken::new(v.4.to_string(), v.5, 0, 0)))
            .collect();

        let mut data = SimpleRuntimeData::new();

        instructions_from_ast(root, nodes, &mut data)?;

        Ok(data)
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
    fn boolean_true() {
        assert_instruction_data(
            0,
            vec![(Definition::True, None, None, None, "$?", TokenType::True)],
            vec![(Instruction::Put, Some(1)), (Instruction::EndExpression, None)],
            vec![ExpressionData::boolean_true()],
        );
    }

    #[test]
    fn boolean_false() {
        assert_instruction_data(
            0,
            vec![(Definition::False, None, None, None, "$!", TokenType::False)],
            vec![(Instruction::Put, Some(1)), (Instruction::EndExpression, None)],
            vec![ExpressionData::boolean_false()],
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
            vec![(Definition::Value, None, None, None, "$", TokenType::Value)],
            vec![(Instruction::PutValue, None), (Instruction::EndExpression, None)],
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
                (Instruction::EmptyApply, None),
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
    fn access_left_internal() {
        assert_instruction_data(
            0,
            vec![
                (Definition::AccessLeftInternal, None, None, Some(1), "_.", TokenType::LeftInternal),
                (Definition::Number, Some(0), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::AccessLeftInternal, None),
                (Instruction::EndExpression, None),
            ],
            vec![ExpressionData::integer(10)],
        );
    }

    #[test]
    fn access_right_internal() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
                (Definition::AccessRightInternal, None, Some(0), None, "._", TokenType::RightInternal),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::AccessRightInternal, None),
                (Instruction::EndExpression, None),
            ],
            vec![ExpressionData::integer(10)],
        );
    }

    #[test]
    fn access_length_internal() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
                (Definition::AccessLengthInternal, None, Some(0), None, ".|", TokenType::LengthInternal),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::AccessLengthInternal, None),
                (Instruction::EndExpression, None),
            ],
            vec![ExpressionData::integer(10)],
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
                (Instruction::UpdateValue, None),
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
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::Reapply, None, Some(0), Some(2), "^~", TokenType::Reapply),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::Put, Some(2)),
                (Instruction::Reapply, Some(0)),
                (Instruction::EndExpression, None),
            ],
            vec![ExpressionData::integer(5), ExpressionData::integer(10)],
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
    fn access() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Identifier, Some(1), None, None, "list", TokenType::Identifier),
                (Definition::Access, None, Some(0), Some(2), ".", TokenType::Period),
                (Definition::Property, Some(1), None, None, "property", TokenType::Identifier),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::Resolve, None),
                (Instruction::Put, Some(2)),
                (Instruction::Access, None),
                (Instruction::EndExpression, None),
            ],
            vec![
                ExpressionData::symbol_from_string(&"list".to_string()),
                ExpressionData::symbol_from_string(&"property".to_string()),
            ],
        );
    }

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

    #[test]
    fn space_list_in_comma_list() {
        assert_instruction_data(
            9,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::CommaList, Some(9), Some(0), Some(7), ",", TokenType::Comma),
                //
                (Definition::Number, Some(3), None, None, "10", TokenType::Number),
                (Definition::List, Some(5), Some(2), Some(4), " ", TokenType::Whitespace),
                (Definition::Number, Some(3), None, None, "15", TokenType::Number),
                (Definition::List, Some(7), Some(3), Some(6), " ", TokenType::Whitespace),
                (Definition::Number, Some(5), None, None, "20", TokenType::Number),
                (Definition::List, Some(1), Some(5), Some(8), " ", TokenType::Whitespace),
                (Definition::Number, Some(7), None, None, "25", TokenType::Number),
                //
                (Definition::CommaList, None, Some(1), Some(10), ",", TokenType::Comma),
                (Definition::Number, Some(9), None, None, "30", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::Put, Some(2)),
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::Put, Some(5)),
                (Instruction::MakeList, Some(4)),
                (Instruction::Put, Some(6)),
                (Instruction::MakeList, Some(3)),
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
            vec![ExpressionData::integer(5), ExpressionData::integer(10)],
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
            vec![ExpressionData::expression(1), ExpressionData::integer(5), ExpressionData::integer(10)],
            vec![1, 3],
        );
    }

    #[test]
    fn two_on_same_level() {
        assert_instruction_data_jumps(
            2,
            vec![
                (Definition::NestedExpression, Some(2), None, Some(1), "{", TokenType::StartExpression),
                (Definition::Number, Some(0), None, None, "5", TokenType::Number),
                (Definition::Pair, None, Some(0), Some(3), "=", TokenType::Pair),
                (Definition::NestedExpression, Some(2), None, Some(4), "{", TokenType::StartExpression),
                (Definition::Number, Some(3), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::Put, Some(2)),
                (Instruction::MakePair, None),
                (Instruction::EndExpression, None), // 3
                (Instruction::Put, Some(3)),
                (Instruction::EndExpression, None), //5
                (Instruction::Put, Some(4)),
                (Instruction::EndExpression, None),
            ],
            vec![
                ExpressionData::expression(1),
                ExpressionData::expression(2),
                ExpressionData::integer(5),
                ExpressionData::integer(10),
            ],
            vec![1, 5, 7],
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
            vec![1, 3, 7, 11],
        );
    }
}

#[cfg(test)]
mod conditionals {
    use super::test_utils::*;
    use crate::*;
    use garnish_lang_runtime::*;

    #[test]
    fn apply_if_true() {
        assert_instruction_data_jumps(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::JumpIfTrue, None, Some(0), Some(2), "?>", TokenType::JumpIfTrue),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::JumpIfTrue, Some(2)),
                (Instruction::EndExpression, None),
                (Instruction::Put, Some(2)),
                (Instruction::JumpTo, Some(1)),
            ],
            vec![ExpressionData::integer(5), ExpressionData::integer(10)],
            vec![1, 3, 4],
        );
    }

    #[test]
    fn apply_if_false() {
        assert_instruction_data_jumps(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::JumpIfFalse, None, Some(0), Some(2), "!>", TokenType::JumpIfFalse),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::JumpIfFalse, Some(2)),
                (Instruction::EndExpression, None),
                (Instruction::Put, Some(2)),
                (Instruction::JumpTo, Some(1)),
            ],
            vec![ExpressionData::integer(5), ExpressionData::integer(10)],
            vec![1, 3, 4],
        );
    }

    #[test]
    fn conditional_chain() {
        assert_instruction_data_jumps(
            3,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::JumpIfTrue, Some(3), Some(0), Some(2), "?>", TokenType::JumpIfTrue),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
                (Definition::ElseJump, None, Some(1), Some(5), "|>", TokenType::ElseJump),
                (Definition::Number, Some(5), None, None, "15", TokenType::Number),
                (Definition::JumpIfTrue, Some(3), Some(4), Some(6), "?>", TokenType::JumpIfTrue),
                (Definition::Number, Some(5), None, None, "20", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::JumpIfTrue, Some(2)),
                (Instruction::Put, Some(2)),
                (Instruction::JumpIfTrue, Some(3)),
                (Instruction::EndExpression, None),
                (Instruction::Put, Some(3)), // 6
                (Instruction::JumpTo, Some(1)),
                (Instruction::Put, Some(4)), // 8
                (Instruction::JumpTo, Some(1)),
            ],
            vec![
                ExpressionData::integer(5),
                ExpressionData::integer(15),
                ExpressionData::integer(10),
                ExpressionData::integer(20),
            ],
            vec![1, 5, 6, 8],
        );
    }

    #[test]
    fn conditional_chain_with_default_clause() {
        assert_instruction_data_jumps(
            3,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::JumpIfTrue, Some(3), Some(0), Some(2), "?>", TokenType::JumpIfTrue),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
                (Definition::ElseJump, None, Some(1), Some(4), "|>", TokenType::ElseJump),
                (Definition::Number, Some(3), None, None, "15", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::JumpIfTrue, Some(2)),
                (Instruction::Put, Some(2)),
                (Instruction::EndExpression, None), // 4
                (Instruction::Put, Some(3)),
                (Instruction::JumpTo, Some(1)),
            ],
            vec![ExpressionData::integer(5), ExpressionData::integer(15), ExpressionData::integer(10)],
            vec![1, 4, 5],
        );
    }

    #[test]
    fn reapply_with_else() {
        assert_instruction_data_jumps(
            3,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::Reapply, Some(3), Some(0), Some(2), "^~", TokenType::Reapply),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
                (Definition::ElseJump, None, Some(1), Some(4), "|>", TokenType::ElseJump),
                (Definition::Number, Some(3), None, None, "15", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::Put, Some(2)),
                (Instruction::Reapply, Some(0)),
                (Instruction::Put, Some(3)),
                (Instruction::EndExpression, None), // 5
            ],
            vec![ExpressionData::integer(5), ExpressionData::integer(10), ExpressionData::integer(15)],
            vec![1, 5], // TODO: Either more tests to verify if extra jump is needed or find way to remove it
        );
    }

    #[test]
    fn multiple_conditional_branches() {
        assert_instruction_data_jumps(
            7,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::JumpIfTrue, Some(3), Some(0), Some(2), "?>", TokenType::JumpIfTrue), // 1
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
                // 1
                (Definition::ElseJump, Some(7), Some(1), Some(5), "|>", TokenType::ElseJump), // 3
                // 1
                (Definition::Number, Some(5), None, None, "15", TokenType::Number),
                (Definition::JumpIfTrue, Some(3), Some(4), Some(6), "?>", TokenType::JumpIfTrue), // 5
                (Definition::Number, Some(5), None, None, "20", TokenType::Number),
                // 2
                (Definition::ElseJump, None, Some(3), Some(9), "|>", TokenType::ElseJump), // 7
                // 2
                (Definition::Number, Some(9), None, None, "25", TokenType::Number),
                (Definition::JumpIfTrue, Some(7), Some(8), Some(10), "?>", TokenType::JumpIfTrue), // 9
                (Definition::Number, Some(9), None, None, "30", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(1)),
                (Instruction::JumpIfTrue, Some(2)), // 2
                (Instruction::Put, Some(2)),
                (Instruction::JumpIfTrue, Some(3)), // 4
                (Instruction::Put, Some(3)),
                (Instruction::JumpIfTrue, Some(4)), // 6
                (Instruction::EndExpression, None),
                (Instruction::Put, Some(4)), // 8
                (Instruction::JumpTo, Some(1)),
                (Instruction::Put, Some(5)), // 10
                (Instruction::JumpTo, Some(1)),
                (Instruction::Put, Some(6)), // 12
                (Instruction::JumpTo, Some(1)),
            ],
            vec![
                ExpressionData::integer(5),
                ExpressionData::integer(15),
                ExpressionData::integer(25),
                ExpressionData::integer(10),
                ExpressionData::integer(20),
                ExpressionData::integer(30),
            ],
            vec![1, 7, 8, 10, 12],
        );
    }
}
