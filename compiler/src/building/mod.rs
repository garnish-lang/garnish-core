use crate::error::{implementation_error, implementation_error_with_token, CompilerError};
use garnish_lang_runtime::*;
use garnish_traits::Instruction;
use log::trace;

use crate::parsing::parser::*;

#[derive(Debug, PartialOrd, Eq, PartialEq, Clone)]
pub struct InstructionMetadata {
    parse_node_index: Option<usize>,
}

impl InstructionMetadata {
    fn new(parse_node_index: Option<usize>) -> Self {
        InstructionMetadata { parse_node_index }
    }

    pub fn get_parse_node_index(&self) -> Option<usize> {
        self.parse_node_index
    }
}

struct ResolveNodeInfo {
    node_index: Option<usize>,
    initialized: bool,
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
            initialized: false,
            first_resolved: false,
            can_ignore_first: false,
            second_resolved: false,
            resolved: false,
            parent_definition,
        }
    }
}

type DefinitionResolveInfo = (bool, Option<usize>);

fn get_resolve_info(node: &ParseNode, nodes: &Vec<ParseNode>) -> (DefinitionResolveInfo, DefinitionResolveInfo) {
    match node.get_definition() {
        Definition::Number
        | Definition::CharList
        | Definition::ByteList
        | Definition::Identifier
        | Definition::Property
        | Definition::Symbol
        | Definition::Value
        | Definition::Unit
        | Definition::False // Shares logic with subexpression, TODO look into refactoring this logic to specify all three visits instead of just two
        | Definition::True => {
            ((false, node.get_left()), (false, node.get_right()))
        }
        Definition::Opposite => {
            // if right value is a number, don't resolve it
            // will be handled during opposite's resolve
            // missing node errors will be handled in core loop, just returning true here to make sure it gets checked
            match node.get_right() {
                None => ((true, node.get_right()), (false, None)),
                Some(right) => match nodes.get(right) {
                    None => ((true, node.get_right()), (false, None)),
                    Some(n) => match n.get_definition() {
                        Definition::Number => ((false, None), (false, None)),
                        _ => ((true, node.get_right()), (false, None)),
                    }
                }
            }
        }
        Definition::AccessLeftInternal | Definition::AbsoluteValue | Definition::BitwiseNot | Definition::Not | Definition::TypeOf => ((true, node.get_right()), (false, None)),
        Definition::EmptyApply | Definition::AccessLengthInternal | Definition::AccessRightInternal => ((true, node.get_left()), (false, None)),
        Definition::Addition
        | Definition::Subtraction
        | Definition::MultiplicationSign
        | Definition::Division
        | Definition::IntegerDivision
        | Definition::ExponentialSign
        | Definition::Remainder
        | Definition::BitwiseAnd
        | Definition::BitwiseOr
        | Definition::BitwiseXor
        | Definition::BitwiseLeftShift
        | Definition::BitwiseRightShift
        | Definition::And
        | Definition::Or
        | Definition::Xor
        | Definition::TypeCast
        | Definition::TypeEqual
        | Definition::Equality
        | Definition::Inequality
        | Definition::LessThan
        | Definition::LessThanOrEqual
        | Definition::GreaterThan
        | Definition::GreaterThanOrEqual
        | Definition::Pair
        | Definition::Access
        | Definition::Subexpression // Same order for child resolution but has special check, might need to move out of here eventually
        | Definition::Reapply
        | Definition::Apply
        | Definition::Range
        | Definition::EndExclusiveRange
        | Definition::StartExclusiveRange
        | Definition::ExclusiveRange
        | Definition::AppendLink
        | Definition::PrependLink
        | Definition::Concatenation => {
            ((true, node.get_left()), (true, node.get_right()))
        }
        Definition::ApplyTo => ((true, node.get_right()), (true, node.get_left())),
        Definition::List => ((true, node.get_left()), (true, node.get_right())),
        Definition::CommaList => ((false, node.get_left()), (false, node.get_right())),
        Definition::SideEffect => ((true, node.get_right()), (false, None)),
        Definition::Group => ((true, node.get_right()), (false, None)),
        Definition::NestedExpression => ((false, None), (false, None)),
        Definition::JumpIfTrue => ((true, node.get_left()), (false, None)),
        Definition::JumpIfFalse => ((true, node.get_left()), (false, None)),
        Definition::ElseJump => ((true, node.get_left()), (true, node.get_right())),
        Definition::Drop => ((false, None), (false, None)),
    }
}

// returns true/false on whether or not an instruction was added
fn resolve_node<Data: GarnishLangRuntimeData>(
    node: &ParseNode,
    nodes: &Vec<ParseNode>,
    data: &mut Data,
    list_count: Option<&Data::Size>,
    current_jump_index: Data::Size,
    nearest_expression_point: Data::Size,
) -> Result<bool, CompilerError<Data::Error>> {
    match node.get_definition() {
        Definition::Unit => {
            // all unit literals will use unit used in the zero element slot of data
            let addr = data.add_unit()?;
            data.push_instruction(Instruction::Put, Some(addr))?;
        }
        Definition::True => {
            let addr = data.add_true()?;
            data.push_instruction(Instruction::Put, Some(addr))?;
        }
        Definition::False => {
            let addr = data.add_false()?;
            data.push_instruction(Instruction::Put, Some(addr))?;
        }
        Definition::Number => {
            let addr = data.parse_add_number(node.get_lex_token().get_text())?;
            data.push_instruction(Instruction::Put, Some(addr))?;
        }
        Definition::CharList => {
            let addr = data.parse_add_char_list(node.get_lex_token().get_text())?;
            data.push_instruction(Instruction::Put, Some(addr))?;
        }
        Definition::ByteList => {
            let addr = data.parse_add_byte_list(node.get_lex_token().get_text())?;
            data.push_instruction(Instruction::Put, Some(addr))?;
        }
        Definition::Identifier => {
            let addr = data.parse_add_symbol(node.get_lex_token().get_text())?;

            data.push_instruction(Instruction::Resolve, Some(addr))?;
        }
        Definition::Property => {
            let addr = data.parse_add_symbol(node.get_lex_token().get_text())?;
            data.push_instruction(Instruction::Put, Some(addr))?;
        }
        Definition::Symbol => {
            let addr = data.parse_add_symbol(&node.get_lex_token().get_text()[1..])?;
            data.push_instruction(Instruction::Put, Some(addr))?;
        }
        Definition::Value => {
            // all unit literals will use unit used in the zero element slot of data
            data.push_instruction(Instruction::PutValue, None)?;
        }
        Definition::EmptyApply => {
            data.push_instruction(Instruction::EmptyApply, None)?;
        }
        Definition::Addition => {
            data.push_instruction(Instruction::Add, None)?;
        }
        Definition::Subtraction => {
            data.push_instruction(Instruction::Subtract, None)?;
        }
        Definition::MultiplicationSign => {
            data.push_instruction(Instruction::Multiply, None)?;
        }
        Definition::Division => {
            data.push_instruction(Instruction::Divide, None)?;
        }
        Definition::IntegerDivision => {
            data.push_instruction(Instruction::IntegerDivide, None)?;
        }
        Definition::ExponentialSign => {
            data.push_instruction(Instruction::Power, None)?;
        }
        Definition::Remainder => {
            data.push_instruction(Instruction::Remainder, None)?;
        }
        Definition::Opposite => {
            // if number push number with negative sign prepended
            let (num, num_str) = match node.get_right() {
                None => (false, String::new()),
                Some(right) => match nodes.get(right) {
                    None => (false, String::new()),
                    Some(n) => match n.get_definition() {
                        Definition::Number => (true, format!("-{}", n.get_lex_token().get_text())),
                        _ => (false, String::new()),
                    }
                }
            };

            if num {
                let addr = data.parse_add_number(num_str.as_str())?;
                data.push_instruction(Instruction::Put, Some(addr))?;
            } else {
                data.push_instruction(Instruction::Opposite, None)?;
            }
        }
        Definition::AbsoluteValue => {
            data.push_instruction(Instruction::AbsoluteValue, None)?;
        }
        Definition::BitwiseNot => {
            data.push_instruction(Instruction::BitwiseNot, None)?;
        }
        Definition::BitwiseAnd => {
            data.push_instruction(Instruction::BitwiseAnd, None)?;
        }
        Definition::BitwiseOr => {
            data.push_instruction(Instruction::BitwiseOr, None)?;
        }
        Definition::BitwiseXor => {
            data.push_instruction(Instruction::BitwiseXor, None)?;
        }
        Definition::BitwiseLeftShift => {
            data.push_instruction(Instruction::BitwiseShiftLeft, None)?;
        }
        Definition::BitwiseRightShift => {
            data.push_instruction(Instruction::BitwiseShiftRight, None)?;
        }
        Definition::And => {
            data.push_instruction(Instruction::And, None)?;
        }
        Definition::Or => {
            data.push_instruction(Instruction::Or, None)?;
        }
        Definition::Xor => {
            data.push_instruction(Instruction::Xor, None)?;
        }
        Definition::Not => {
            data.push_instruction(Instruction::Not, None)?;
        }
        Definition::TypeOf => {
            data.push_instruction(Instruction::TypeOf, None)?;
        }
        Definition::TypeCast => {
            data.push_instruction(Instruction::ApplyType, None)?;
        }
        Definition::TypeEqual => {
            data.push_instruction(Instruction::TypeEqual, None)?;
        }
        Definition::Equality => {
            data.push_instruction(Instruction::Equal, None)?;
        }
        Definition::Inequality => {
            data.push_instruction(Instruction::NotEqual, None)?;
        }
        Definition::LessThan => {
            data.push_instruction(Instruction::LessThan, None)?;
        }
        Definition::LessThanOrEqual => {
            data.push_instruction(Instruction::LessThanOrEqual, None)?;
        }
        Definition::GreaterThan => {
            data.push_instruction(Instruction::GreaterThan, None)?;
        }
        Definition::GreaterThanOrEqual => {
            data.push_instruction(Instruction::GreaterThanOrEqual, None)?;
        }
        Definition::Pair => {
            data.push_instruction(Instruction::MakePair, None)?;
        }
        Definition::Range => {
            data.push_instruction(Instruction::MakeRange, None)?;
        }
        Definition::EndExclusiveRange => {
            data.push_instruction(Instruction::MakeEndExclusiveRange, None)?;
        }
        Definition::StartExclusiveRange => {
            data.push_instruction(Instruction::MakeStartExclusiveRange, None)?;
        }
        Definition::ExclusiveRange => {
            data.push_instruction(Instruction::MakeExclusiveRange, None)?;
        }
        Definition::Concatenation => {
            data.push_instruction(Instruction::Concat, None)?;
        }
        Definition::AppendLink => {
            data.push_instruction(Instruction::AppendLink, None)?;
        }
        Definition::PrependLink => {
            data.push_instruction(Instruction::PrependLink, None)?;
        }
        Definition::Access => {
            data.push_instruction(Instruction::Access, None)?;
        }
        Definition::AccessLeftInternal => {
            data.push_instruction(Instruction::AccessLeftInternal, None)?;
        }
        Definition::AccessRightInternal => {
            data.push_instruction(Instruction::AccessRightInternal, None)?;
        }
        Definition::AccessLengthInternal => {
            data.push_instruction(Instruction::AccessLengthInternal, None)?;
        }
        Definition::List | Definition::CommaList => match list_count {
            None => implementation_error_with_token(format!("No list count passed to list node resolve."), &node.get_lex_token())?,
            Some(count) => {
                data.push_instruction(Instruction::MakeList, Some(*count))?;
            }
        },
        Definition::Subexpression => {
            data.push_instruction(Instruction::UpdateValue, None)?;
        }
        Definition::NestedExpression => {
            data.push_instruction(Instruction::Put, Some(data.get_data_len()))?;

            data.add_expression(current_jump_index)?;
        }
        Definition::Apply => {
            data.push_instruction(Instruction::Apply, None)?;
        }
        Definition::ApplyTo => {
            data.push_instruction(Instruction::Apply, None)?;
        }
        Definition::Reapply => {
            data.push_instruction(Instruction::Reapply, Some(nearest_expression_point))?;
        }
        Definition::JumpIfTrue => {
            data.push_instruction(Instruction::JumpIfTrue, Some(current_jump_index))?;
        }
        Definition::JumpIfFalse => {
            data.push_instruction(Instruction::JumpIfFalse, Some(current_jump_index))?;
        }
        Definition::SideEffect => {
            data.push_instruction(Instruction::EndSideEffect, None)?;
        }
        Definition::Group => return Ok(false),    // no additional instructions for groups
        Definition::ElseJump => return Ok(false), // no additional instructions
        // no runtime meaning, parser only utility
        Definition::Drop => return Err(CompilerError::new_message("Drop definition is not allowed during build.".to_string())),
    }

    Ok(true)
}

pub fn build_with_data<Data: GarnishLangRuntimeData>(
    root: usize,
    nodes: Vec<ParseNode>,
    data: &mut Data,
) -> Result<Vec<InstructionMetadata>, CompilerError<Data::Error>> {
    // emtpy yields empty
    if nodes.is_empty() {
        return Ok(vec![]);
    }

    let mut metadata = vec![];

    // since we will be popping and pushing values from root_stack
    // need to keep separate count of total so expression values put in data are accurate
    let mut jump_count = data.get_jump_table_len() + Data::Size::one();

    // tuple of (root node, last instruction of this node, nearest expression jump point)
    let mut root_stack = vec![(root, Instruction::EndExpression, None, data.get_jump_table_len())];

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
        let jump_index = data.get_jump_table_len();
        data.push_jump_point(Data::Size::zero())?; // updated down below

        // limit, maximum times a node is visited is 3
        // so limit to 3 times node count should allow for more than enough
        let max_iterations = nodes.len() * 3;
        let mut iter_count = 0;

        loop {
            let pop = match stack.last_mut() {
                None => break, // no more nodes
                Some(resolve_node_info) => match resolve_node_info.node_index {
                    None => implementation_error(format!(
                        "None value for node index. All nodes should resolve properly if starting from root node."
                    ))?,
                    Some(node_index) => match nodes.get(node_index) {
                        // all nodes should exist if starting from root
                        None => implementation_error(format!(
                            "Node at index {:?} does not exist. All nodes should resolve properly if starting from root node.",
                            node_index
                        ))?,
                        Some(node) => {
                            trace!("---------------------------------------------------------");
                            trace!("Visiting node with definition {:?} at {:?}", node.get_definition(), node_index);

                            let ((first_expected, first_index), (second_expected, second_index)) = get_resolve_info(node, &nodes);

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
                            let we_are_list_item = (!we_are_list_type && parent_is_list_type
                                || we_are_list && resolve_node_info.parent_definition == Definition::CommaList
                                || we_are_comma_list && resolve_node_info.parent_definition == Definition::List)
                                && node.get_definition() != Definition::SideEffect;

                            if !resolve_node_info.initialized {
                                // initialization for specific nodes

                                // on first visit to a root list node
                                // may not pass through first or second branches, due to allowing single item lists and empty lists
                                // start a new list count
                                if we_are_list_root {
                                    trace!("Starting new list count");
                                    list_counts.push(Data::Size::zero());
                                }

                                // side effect inserts two instruction
                                // first on initialization and second on resolution
                                if node.get_definition() == Definition::SideEffect {
                                    data.push_instruction(Instruction::StartSideEffect, None)?;

                                    // use same node for now
                                    metadata.push(InstructionMetadata::new(resolve_node_info.node_index));
                                }

                                resolve_node_info.initialized = true;
                            }

                            let pop = if (first_expected || first_index.is_some()) && !resolve_node_info.first_resolved {
                                // on first visit to a conditional branch node
                                // add to jump table and store the position in conditional stack
                                if we_are_parent_conditional_branch || we_are_non_chained_conditional {
                                    trace!(
                                        "Starting new conditional branch. Return slot in jump table {:?}",
                                        data.get_jump_table_len()
                                    );
                                    let i = data.get_jump_table_len();
                                    data.push_jump_point(Data::Size::zero())?; // this will be updated when conditional branch nod resolves
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
                            } else if (second_expected || second_index.is_some()) && !resolve_node_info.second_resolved {
                                // special check for subexpression, so far is only operations that isn't fully depth first
                                // gets resolved before second child
                                if node.get_definition() == Definition::Subexpression || node.get_definition().is_value_like() {
                                    trace!("Resolving {:?} at {:?} (Subexpression)", node.get_definition(), node_index);

                                    let instruction_created = resolve_node(node, &nodes, data, None, Data::Size::zero(), nearest_expression_point)?;

                                    // should always create, but check for posterity
                                    if instruction_created {
                                        metadata.push(InstructionMetadata::new(resolve_node_info.node_index))
                                    }

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
                                let we_are_resolved_value = node.get_definition().is_value_like() && resolve_node_info.resolved;
                                let we_are_subexpression = node.get_definition() == Definition::Subexpression;

                                let resolve = !we_are_subexpression && !we_are_sub_list && !we_are_resolved_value;

                                // subexpression already resolved before second child
                                if resolve {
                                    trace!("Resolving {:?} at {:?}", node.get_definition(), node_index);

                                    let instruction_created = resolve_node(node, &nodes, data, list_counts.last(), jump_count, nearest_expression_point)?;

                                    if instruction_created {
                                        metadata.push(InstructionMetadata::new(resolve_node_info.node_index))
                                    }

                                    if we_are_list_root {
                                        list_counts.pop();
                                    }
                                }

                                // after resolving, if a nested expression
                                // and add nested expressions child to deferred list
                                let mut return_instruction = (Instruction::EndExpression, None);
                                if we_are_conditional {
                                    return_instruction = match conditional_stack.last() {
                                        None => implementation_error(format!("Conditional stack empty when attempting to resolve conditional."))?,
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
                                        None => implementation_error(format!("End of conditional branch and conditional stack is empty."))?,
                                        Some(jump_index) => match data.get_jump_point_mut(jump_index) {
                                            None => implementation_error(format!(
                                                "No value in jump table at index {:?} to update for conditional branch.",
                                                jump_index
                                            ))?,
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
                                        None => {
                                            implementation_error(format!("No child value on {:?} node at {:?}", node.get_definition(), node_index))?
                                        }
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
                                        None => implementation_error(format!("Child of list node has no count add to."))?,
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
                return implementation_error(format!("Max iterations reached in resolving tree at root {:?}", root_index));
            }
        }

        trace!("Adding return instruction {:?} with data {:?}", return_instruction, instruction_data);

        data.push_instruction(return_instruction, instruction_data)?;
        metadata.push(InstructionMetadata::new(None));

        trace!("Finished instructions for tree starting at index {:?}", root_index);

        root_iter_count += 1;

        if root_iter_count > max_roots {
            return implementation_error(format!("Max iterations for roots reached."));
        }

        match data.get_jump_point_mut(jump_index) {
            None => implementation_error(format!(
                "Failed to update jump point at index {:?} because None was returned.",
                jump_index
            ))?,
            Some(i) => *i = jump_point,
        }
    }

    Ok(metadata)
}

#[cfg(test)]
mod test_utils {
    use crate::error::CompilerError;
    use crate::*;
    use garnish_data::data::SimpleDataList;
    use garnish_data::InstructionData;
    use garnish_data::*;
    use garnish_traits::Instruction;

    pub fn assert_instruction_data(
        root: usize,
        nodes: Vec<(Definition, Option<usize>, Option<usize>, Option<usize>, &str, TokenType)>,
        expected_instructions: Vec<(Instruction, Option<usize>)>,
        expected_data: SimpleDataList,
    ) {
        assert_instruction_data_jumps(root, nodes, expected_instructions, expected_data, vec![0]);
    }

    pub fn assert_instruction_data_jumps(
        root: usize,
        nodes: Vec<(Definition, Option<usize>, Option<usize>, Option<usize>, &str, TokenType)>,
        expected_instructions: Vec<(Instruction, Option<usize>)>,
        expected_data: SimpleDataList,
        expected_jumps: Vec<usize>,
    ) {
        let expected_instructions: Vec<InstructionData> = expected_instructions.iter().map(|i| InstructionData::new(i.0, i.1)).collect();

        let (result, _) = get_instruction_data(root, nodes).unwrap();

        assert_eq!(result.get_instructions().clone(), expected_instructions);
        assert_eq!(result.get_data(), &expected_data);
        assert_eq!(result.get_jump_points(), &expected_jumps);
    }

    pub fn get_instruction_data(
        root: usize,
        nodes: Vec<(Definition, Option<usize>, Option<usize>, Option<usize>, &str, TokenType)>,
    ) -> Result<(SimpleRuntimeData, Vec<InstructionMetadata>), CompilerError<DataError>> {
        let nodes: Vec<ParseNode> = nodes
            .iter()
            .map(|v| ParseNode::new(v.0, SecondaryDefinition::None, v.1, v.2, v.3, LexerToken::new(v.4.to_string(), v.5, 0, 0)))
            .collect();

        let mut data = SimpleRuntimeData::new();

        let metadata = build_with_data(root, nodes, &mut data)?;

        Ok((data, metadata))
    }
}

#[cfg(test)]
mod general {
    use super::test_utils::*;
    use crate::*;

    #[test]
    fn drop_definition_is_err() {
        let result = get_instruction_data(0, vec![(Definition::Drop, None, None, None, "5", TokenType::Number)]);

        assert!(result.is_err())
    }

    #[test]
    fn build_empty_node_list() {
        let result = get_instruction_data(0, vec![]);

        assert!(result.is_ok())
    }
}

#[cfg(test)]
mod values {
    use std::vec;

    use super::test_utils::*;
    use crate::*;
    use garnish_data::data::{SimpleData, SimpleDataList};
    use garnish_data::*;
    use garnish_traits::Instruction;

    #[test]
    fn put_integer() {
        assert_instruction_data(
            0,
            vec![(Definition::Number, None, None, None, "5", TokenType::Number)],
            vec![(Instruction::Put, Some(3)), (Instruction::EndExpression, None)],
            SimpleDataList::default().append(SimpleData::Number(5.into())),
        );
    }

    #[test]
    fn boolean_true() {
        assert_instruction_data(
            0,
            vec![(Definition::True, None, None, None, "$?", TokenType::True)],
            vec![(Instruction::Put, Some(2)), (Instruction::EndExpression, None)],
            SimpleDataList::default(),
        );
    }

    #[test]
    fn boolean_false() {
        assert_instruction_data(
            0,
            vec![(Definition::False, None, None, None, "$!", TokenType::False)],
            vec![(Instruction::Put, Some(1)), (Instruction::EndExpression, None)],
            SimpleDataList::default(),
        );
    }

    #[test]
    fn resolve_identifier() {
        assert_instruction_data(
            0,
            vec![(Definition::Identifier, None, None, None, "value", TokenType::Identifier)],
            vec![(Instruction::Resolve, Some(3)), (Instruction::EndExpression, None)],
            SimpleDataList::default().append(SimpleData::Symbol(symbol_value("value"))),
        );
    }

    #[test]
    fn put_unit() {
        assert_instruction_data(
            0,
            vec![(Definition::Unit, None, None, None, "()", TokenType::UnitLiteral)],
            vec![(Instruction::Put, Some(0)), (Instruction::EndExpression, None)],
            SimpleDataList::default(),
        );
    }

    #[test]
    fn put_symbol() {
        assert_instruction_data(
            0,
            vec![(Definition::Symbol, None, None, None, ":symbol", TokenType::Symbol)],
            vec![(Instruction::Put, Some(3)), (Instruction::EndExpression, None)],
            SimpleDataList::default().append(SimpleData::Symbol(symbol_value("symbol"))),
        );
    }

    #[test]
    fn put_char_list() {
        assert_instruction_data(
            0,
            vec![(Definition::CharList, None, None, None, "\"characters\"", TokenType::CharList)],
            vec![(Instruction::Put, Some(3)), (Instruction::EndExpression, None)],
            SimpleDataList::default().append(SimpleData::CharList("characters".to_string())),
        );
    }

    #[test]
    fn put_byte_list() {
        assert_instruction_data(
            0,
            vec![(Definition::ByteList, None, None, None, "'abc'", TokenType::ByteList)],
            vec![(Instruction::Put, Some(3)), (Instruction::EndExpression, None)],
            SimpleDataList::default().append(SimpleData::ByteList(vec!['a' as u8, 'b' as u8, 'c' as u8])),
        );
    }

    #[test]
    fn put_empty_symbol() {
        assert_instruction_data(
            0,
            vec![(Definition::Symbol, None, None, None, ":", TokenType::Symbol)],
            vec![(Instruction::Put, Some(3)), (Instruction::EndExpression, None)],
            SimpleDataList::default().append(SimpleData::Symbol(symbol_value(""))),
        );
    }

    #[test]
    fn put_input() {
        assert_instruction_data(
            0,
            vec![(Definition::Value, None, None, None, "$", TokenType::Value)],
            vec![(Instruction::PutValue, None), (Instruction::EndExpression, None)],
            SimpleDataList::default(),
        );
    }
}

#[cfg(test)]
mod metadata {
    use super::test_utils::*;
    use crate::*;

    #[test]
    fn created() {
        let (_, metadata) = get_instruction_data(0, vec![(Definition::Number, None, None, None, "5", TokenType::Number)]).unwrap();

        assert_eq!(metadata, vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)])
    }

    #[test]
    fn group_is_ignored() {
        let (_, metadata) = get_instruction_data(
            0,
            vec![
                (Definition::Group, None, None, Some(1), "(", TokenType::StartGroup),
                (Definition::Number, Some(0), None, None, "5", TokenType::Number),
            ],
        )
        .unwrap();

        assert_eq!(metadata, vec![InstructionMetadata::new(Some(1)), InstructionMetadata::new(None)])
    }

    #[test]
    fn conditional_chain_ignored() {
        let (_, metadata) = get_instruction_data(
            3,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::JumpIfTrue, Some(3), Some(0), Some(2), "?>", TokenType::JumpIfTrue),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
                (Definition::ElseJump, None, Some(1), Some(4), "|>", TokenType::ElseJump),
                (Definition::Number, Some(3), None, None, "15", TokenType::Number),
            ],
        )
        .unwrap();

        assert_eq!(
            metadata,
            vec![
                InstructionMetadata::new(Some(0)),
                InstructionMetadata::new(Some(1)),
                InstructionMetadata::new(Some(4)),
                InstructionMetadata::new(None),
                InstructionMetadata::new(Some(2)),
                InstructionMetadata::new(None)
            ]
        );
    }

    #[test]
    fn subexpression() {
        let (_, metadata) = get_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::Subexpression, None, Some(0), Some(2), "\n\n", TokenType::Subexpression),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
        )
        .unwrap();

        assert_eq!(
            metadata,
            vec![
                InstructionMetadata::new(Some(0)),
                InstructionMetadata::new(Some(1)),
                InstructionMetadata::new(Some(2)),
                InstructionMetadata::new(None),
            ]
        );
    }
}

#[cfg(test)]
mod operations {
    use std::vec;

    use super::test_utils::*;
    use crate::*;
    use garnish_data::data::{SimpleData, SimpleDataList};
    use garnish_data::*;
    use garnish_traits::Instruction;

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
    fn same_integer_twice() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::Addition, None, Some(0), Some(2), "+", TokenType::PlusSign),
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(3)),
                (Instruction::Add, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default().append(SimpleData::Number(5.into())),
        );
    }

    #[test]
    fn same_symbol_twice() {
        let sym_val = symbol_value("sym");
        assert_instruction_data(
            1,
            vec![
                (Definition::Symbol, Some(1), None, None, ";sym", TokenType::Symbol),
                (Definition::Pair, None, Some(0), Some(2), "=", TokenType::Pair),
                (Definition::Symbol, Some(1), None, None, ";sym", TokenType::Symbol),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(3)),
                (Instruction::MakePair, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default().append(SimpleData::Symbol(sym_val)),
        );
    }

    #[test]
    fn same_identifier_twice() {
        let sym_val = symbol_value("sym");
        assert_instruction_data(
            1,
            vec![
                (Definition::Identifier, Some(1), None, None, "sym", TokenType::Identifier),
                (Definition::Pair, None, Some(0), Some(2), "=", TokenType::Pair),
                (Definition::Identifier, Some(1), None, None, "sym", TokenType::Identifier),
            ],
            vec![
                (Instruction::Resolve, Some(3)),
                (Instruction::Resolve, Some(3)),
                (Instruction::MakePair, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default().append(SimpleData::Symbol(sym_val)),
        );
    }

    #[test]
    fn same_property_twice() {
        let sym_val = symbol_value("sym");
        assert_instruction_data(
            3,
            vec![
                (Definition::Identifier, Some(1), None, None, "sym", TokenType::Identifier),
                (Definition::Access, Some(3), Some(0), Some(2), ".", TokenType::Period),
                (Definition::Property, Some(1), None, None, "sym", TokenType::Identifier),
                (Definition::Access, None, Some(1), Some(4), ".", TokenType::Period),
                (Definition::Property, Some(3), None, None, "sym", TokenType::Identifier),
            ],
            vec![
                (Instruction::Resolve, Some(3)),
                (Instruction::Put, Some(3)),
                (Instruction::Access, None),
                (Instruction::Put, Some(3)),
                (Instruction::Access, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default().append(SimpleData::Symbol(sym_val)),
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
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::Add, None),
                (Instruction::Put, Some(5)),
                (Instruction::Add, None),
                (Instruction::Put, Some(6)),
                (Instruction::Add, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into()))
                .append(SimpleData::Number(15.into()))
                .append(SimpleData::Number(20.into())),
        );
    }

    #[test]
    fn empty_apply() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Identifier, Some(1), None, None, "value", TokenType::Identifier),
                (Definition::EmptyApply, None, Some(0), None, "~~", TokenType::EmptyApply),
            ],
            vec![
                (Instruction::Resolve, Some(3)),
                (Instruction::EmptyApply, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default().append(SimpleData::Symbol(symbol_value("value"))),
        );
    }

    #[test]
    fn addition() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::Addition, None, Some(0), Some(2), "+", TokenType::PlusSign),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::Add, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn subtraction() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::Subtraction, None, Some(0), Some(2), "-", TokenType::Subtraction),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::Subtract, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn multiplication() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::MultiplicationSign, None, Some(0), Some(2), "*", TokenType::MultiplicationSign),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::Multiply, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn division() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::Division, None, Some(0), Some(2), "/", TokenType::Division),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::Divide, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn integer_division() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::IntegerDivision, None, Some(0), Some(2), "//", TokenType::IntegerDivision),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::IntegerDivide, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn exponential() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::ExponentialSign, None, Some(0), Some(2), "**", TokenType::ExponentialSign),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::Power, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn remainder() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::Remainder, None, Some(0), Some(2), "%", TokenType::Remainder),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::Remainder, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn absolute_value() {
        assert_instruction_data(
            0,
            vec![
                (Definition::AbsoluteValue, None, None, Some(1), "--", TokenType::AbsoluteValue),
                (Definition::Identifier, Some(0), None, None, "value", TokenType::Identifier),
            ],
            vec![
                (Instruction::Resolve, Some(3)),
                (Instruction::AbsoluteValue, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default().append(SimpleData::Symbol(symbol_value("value"))),
        );
    }

    #[test]
    fn negate_constant_number() {
        assert_instruction_data(
            0,
            vec![
                (Definition::Opposite, None, None, Some(1), "--", TokenType::Opposite),
                (Definition::Number, Some(0), None, None, "5", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default().append(SimpleData::Number((-5).into())),
        );
    }

    #[test]
    fn opposite() {
        assert_instruction_data(
            0,
            vec![
                (Definition::Opposite, None, None, Some(1), "--", TokenType::Opposite),
                (Definition::Identifier, Some(0), None, None, "value", TokenType::Identifier),
            ],
            vec![
                (Instruction::Resolve, Some(3)),
                (Instruction::Opposite, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default().append(SimpleData::Symbol(symbol_value("value"))),
        );
    }

    #[test]
    fn bitwise_not() {
        assert_instruction_data(
            0,
            vec![
                (Definition::BitwiseNot, None, None, Some(1), "!", TokenType::BitwiseNot),
                (Definition::Identifier, Some(0), None, None, "value", TokenType::Identifier),
            ],
            vec![
                (Instruction::Resolve, Some(3)),
                (Instruction::BitwiseNot, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default().append(SimpleData::Symbol(symbol_value("value"))),
        );
    }

    #[test]
    fn bitwise_and() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::BitwiseAnd, None, Some(0), Some(2), "&", TokenType::BitwiseAnd),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::BitwiseAnd, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn bitwise_or() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::BitwiseOr, None, Some(0), Some(2), "|", TokenType::BitwiseOr),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::BitwiseOr, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn bitwise_xor() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::BitwiseXor, None, Some(0), Some(2), "^", TokenType::BitwiseXor),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::BitwiseXor, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn bitwise_left_shift() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::BitwiseLeftShift, None, Some(0), Some(2), "<<", TokenType::BitwiseLeftShift),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::BitwiseShiftLeft, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn bitwise_right_shift() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::BitwiseRightShift, None, Some(0), Some(2), ">>", TokenType::BitwiseRightShift),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::BitwiseShiftRight, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn type_of() {
        assert_instruction_data(
            0,
            vec![
                (Definition::TypeOf, None, None, Some(1), "#", TokenType::TypeOf),
                (Definition::Identifier, Some(0), None, None, "value", TokenType::Identifier),
            ],
            vec![
                (Instruction::Resolve, Some(3)),
                (Instruction::TypeOf, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default().append(SimpleData::Symbol(symbol_value("value"))),
        );
    }

    #[test]
    fn apply_type() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::TypeCast, None, Some(0), Some(2), "~#", TokenType::TypeCast),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::ApplyType, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn type_equal() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::TypeEqual, None, Some(0), Some(2), "#=", TokenType::TypeEqual),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::TypeEqual, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
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
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::Equal, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn inequality() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::Inequality, None, Some(0), Some(2), "!=", TokenType::Inequality),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::NotEqual, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn less_than() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::LessThan, None, Some(0), Some(2), "<", TokenType::LessThan),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::LessThan, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn less_than_or_equal() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::LessThanOrEqual, None, Some(0), Some(2), "<=", TokenType::LessThanOrEqual),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::LessThanOrEqual, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn greater_than() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::GreaterThan, None, Some(0), Some(2), ">", TokenType::GreaterThan),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::GreaterThan, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn greater_than_or_equal() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (
                    Definition::GreaterThanOrEqual,
                    None,
                    Some(0),
                    Some(2),
                    ">=",
                    TokenType::GreaterThanOrEqual,
                ),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::GreaterThanOrEqual, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn not() {
        assert_instruction_data(
            0,
            vec![
                (Definition::Not, None, None, Some(1), "!!", TokenType::Not),
                (Definition::Identifier, Some(0), None, None, "value", TokenType::Identifier),
            ],
            vec![
                (Instruction::Resolve, Some(3)),
                (Instruction::Not, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default().append(SimpleData::Symbol(symbol_value("value"))),
        );
    }

    #[test]
    fn and() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::And, None, Some(0), Some(2), "&&", TokenType::And),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::And, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn or() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::Or, None, Some(0), Some(2), ">=", TokenType::Or),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::Or, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn xor() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::Xor, None, Some(0), Some(2), ">=", TokenType::Xor),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::Xor, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
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
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::MakePair, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
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
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::Access, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
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
                (Instruction::Put, Some(3)),
                (Instruction::AccessLeftInternal, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default().append(SimpleData::Number(10.into())),
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
                (Instruction::Put, Some(3)),
                (Instruction::AccessRightInternal, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default().append(SimpleData::Number(10.into())),
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
                (Instruction::Put, Some(3)),
                (Instruction::AccessLengthInternal, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default().append(SimpleData::Number(10.into())),
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
                (Instruction::Put, Some(3)),
                (Instruction::UpdateValue, None),
                (Instruction::Put, Some(4)),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn concatenation() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::Concatenation, None, Some(0), Some(2), "<-", TokenType::Concatenation),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::Concat, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn prepend_link() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::PrependLink, None, Some(0), Some(2), "<-", TokenType::PrependLink),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::PrependLink, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn append_link() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::AppendLink, None, Some(0), Some(2), "->", TokenType::AppendLink),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::AppendLink, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn range() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::Range, None, Some(0), Some(2), "..", TokenType::Range),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::MakeRange, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn start_exclusive_range() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (
                    Definition::StartExclusiveRange,
                    None,
                    Some(0),
                    Some(2),
                    ">..",
                    TokenType::StartExclusiveRange,
                ),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::MakeStartExclusiveRange, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn end_exclusive_range() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::EndExclusiveRange, None, Some(0), Some(2), "..<", TokenType::EndExclusiveRange),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::MakeEndExclusiveRange, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn exclusive_range() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::ExclusiveRange, None, Some(0), Some(2), ">..<", TokenType::ExclusiveRange),
                (Definition::Number, Some(1), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::MakeExclusiveRange, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
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
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::Apply, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
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
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::Apply, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(10.into()))
                .append(SimpleData::Number(5.into())),
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
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::Reapply, Some(0)),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }
}

#[cfg(test)]
mod lists {
    use std::vec;

    use super::test_utils::*;
    use crate::*;
    use garnish_data::data::{SimpleData, SimpleDataList};
    use garnish_data::*;
    use garnish_traits::Instruction;

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
                (Instruction::Resolve, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::Access, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Symbol(symbol_value("list")))
                .append(SimpleData::Symbol(symbol_value("property"))),
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
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::MakeList, Some(2)),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn empty_list() {
        assert_instruction_data(
            0,
            vec![(Definition::CommaList, None, None, None, ",", TokenType::Comma)],
            vec![(Instruction::MakeList, Some(0)), (Instruction::EndExpression, None)],
            SimpleDataList::default(),
        );
    }

    #[test]
    fn single_item_left() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::CommaList, None, Some(0), None, ",", TokenType::Comma),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::MakeList, Some(1)),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default().append(SimpleData::Number(5.into())),
        );
    }

    #[test]
    fn single_item_right() {
        assert_instruction_data(
            0,
            vec![
                (Definition::CommaList, None, None, Some(1), ",", TokenType::Comma),
                (Definition::Number, Some(2), None, None, "5", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::MakeList, Some(1)),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default().append(SimpleData::Number(5.into())),
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
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::Put, Some(5)),
                (Instruction::Put, Some(6)),
                (Instruction::MakeList, Some(4)),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into()))
                .append(SimpleData::Number(15.into()))
                .append(SimpleData::Number(20.into())),
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
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::Put, Some(5)),
                (Instruction::Put, Some(6)),
                (Instruction::MakeList, Some(2)),
                (Instruction::Put, Some(7)),
                (Instruction::Put, Some(8)),
                (Instruction::MakeList, Some(5)),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into()))
                .append(SimpleData::Number(15.into()))
                .append(SimpleData::Number(20.into()))
                .append(SimpleData::Number(25.into()))
                .append(SimpleData::Number(30.into())),
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
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::Put, Some(5)),
                (Instruction::Put, Some(6)),
                (Instruction::Put, Some(7)),
                (Instruction::MakeList, Some(4)),
                (Instruction::Put, Some(8)),
                (Instruction::MakeList, Some(3)),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into()))
                .append(SimpleData::Number(15.into()))
                .append(SimpleData::Number(20.into()))
                .append(SimpleData::Number(25.into()))
                .append(SimpleData::Number(30.into())),
        );
    }
}

#[cfg(test)]
mod groups {
    use super::test_utils::*;
    use crate::*;

    use garnish_data::data::{SimpleData, SimpleDataList};
    use garnish_traits::Instruction;

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
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::Add, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }
}

#[cfg(test)]
mod side_effects {
    use super::test_utils::*;
    use crate::*;

    use garnish_data::data::{SimpleData, SimpleDataList};
    use garnish_traits::Instruction;

    #[test]
    fn single_operation() {
        assert_instruction_data(
            0,
            vec![
                (Definition::SideEffect, None, None, Some(2), "[", TokenType::StartSideEffect),
                (Definition::Number, Some(2), None, None, "5", TokenType::Number),
                (Definition::Addition, Some(0), Some(1), Some(3), "+", TokenType::PlusSign),
                (Definition::Number, Some(2), None, None, "10", TokenType::Number),
            ],
            vec![
                (Instruction::StartSideEffect, None),
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::Add, None),
                (Instruction::EndSideEffect, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
        );
    }

    #[test]
    fn after_value() {
        assert_instruction_data(
            0,
            vec![
                (Definition::Number, None, None, Some(1), "5", TokenType::Number),
                (Definition::SideEffect, Some(0), None, Some(3), "[", TokenType::StartSideEffect),
                (Definition::Number, Some(3), None, None, "10", TokenType::Number),
                (Definition::Addition, Some(1), Some(2), Some(4), "+", TokenType::PlusSign),
                (Definition::Number, Some(3), None, None, "15", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::StartSideEffect, None),
                (Instruction::Put, Some(4)),
                (Instruction::Put, Some(5)),
                (Instruction::Add, None),
                (Instruction::EndSideEffect, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into()))
                .append(SimpleData::Number(15.into())),
        );
    }

    #[test]
    fn before_value() {
        assert_instruction_data(
            4,
            vec![
                (Definition::SideEffect, Some(4), None, Some(2), "[", TokenType::StartSideEffect),
                (Definition::Number, Some(2), None, None, "5", TokenType::Number),
                (Definition::Addition, Some(0), Some(1), Some(3), "+", TokenType::PlusSign),
                (Definition::Number, Some(2), None, None, "10", TokenType::Number),
                (Definition::Number, None, Some(0), None, "15", TokenType::Number),
            ],
            vec![
                (Instruction::StartSideEffect, None),
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::Add, None),
                (Instruction::EndSideEffect, None),
                (Instruction::Put, Some(5)),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into()))
                .append(SimpleData::Number(15.into())),
        );
    }

    #[test]
    fn doesnt_add_to_list() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::List, None, Some(0), Some(2), " ", TokenType::Whitespace),
                (Definition::Number, Some(1), None, Some(3), "10", TokenType::Number),
                (Definition::SideEffect, Some(2), None, Some(5), "[", TokenType::StartSideEffect),
                (Definition::Number, Some(5), None, None, "15", TokenType::Number),
                (Definition::Addition, Some(3), Some(4), Some(6), "+", TokenType::PlusSign),
                (Definition::Number, Some(5), None, None, "20", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::StartSideEffect, None),
                (Instruction::Put, Some(5)),
                (Instruction::Put, Some(6)),
                (Instruction::Add, None),
                (Instruction::EndSideEffect, None),
                (Instruction::MakeList, Some(2)),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into()))
                .append(SimpleData::Number(15.into()))
                .append(SimpleData::Number(20.into())),
        );
    }

    #[test]
    fn doesnt_add_to_comma_list() {
        assert_instruction_data(
            1,
            vec![
                (Definition::Number, Some(1), None, None, "5", TokenType::Number),
                (Definition::CommaList, None, Some(0), Some(2), ",", TokenType::Comma),
                (Definition::SideEffect, Some(1), None, Some(4), "[", TokenType::StartSideEffect),
                (Definition::Number, Some(4), None, None, "10", TokenType::Number),
                (Definition::Addition, Some(2), Some(3), Some(5), "+", TokenType::PlusSign),
                (Definition::Number, Some(4), None, None, "15", TokenType::Number),
            ],
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::StartSideEffect, None),
                (Instruction::Put, Some(4)),
                (Instruction::Put, Some(5)),
                (Instruction::Add, None),
                (Instruction::EndSideEffect, None),
                (Instruction::MakeList, Some(1)),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into()))
                .append(SimpleData::Number(15.into())),
        );
    }
}

#[cfg(test)]
mod nested_expressions {
    use super::test_utils::*;
    use crate::*;

    use garnish_data::data::{SimpleData, SimpleDataList};
    use garnish_traits::Instruction;

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
                (Instruction::Put, Some(3)),
                (Instruction::EndExpression, None),
                (Instruction::Put, Some(4)),
                (Instruction::Put, Some(5)),
                (Instruction::Add, None),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Expression(1))
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
            vec![0, 2],
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
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::MakePair, None),
                (Instruction::EndExpression, None), // 3
                (Instruction::Put, Some(5)),
                (Instruction::EndExpression, None), //5
                (Instruction::Put, Some(6)),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Expression(1))
                .append(SimpleData::Expression(2))
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
            vec![0, 4, 6],
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
                (Instruction::Put, Some(3)),
                (Instruction::EndExpression, None), // 1
                (Instruction::Put, Some(4)),
                (Instruction::Put, Some(5)),
                (Instruction::Apply, None),
                (Instruction::EndExpression, None), // 5
                (Instruction::Put, Some(6)),
                (Instruction::Put, Some(7)),
                (Instruction::Apply, None),
                (Instruction::EndExpression, None), // 9
                (Instruction::Put, Some(8)),
                (Instruction::EndExpression, None),
            ],
            SimpleDataList::default()
                .append(SimpleData::Expression(1))
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Expression(2))
                .append(SimpleData::Number(10.into()))
                .append(SimpleData::Expression(3))
                .append(SimpleData::Number(15.into())),
            vec![0, 2, 6, 10],
        );
    }
}

#[cfg(test)]
mod conditionals {
    use super::test_utils::*;
    use crate::*;

    use garnish_data::data::{SimpleData, SimpleDataList};
    use garnish_traits::Instruction;

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
                (Instruction::Put, Some(3)),
                (Instruction::JumpIfTrue, Some(2)),
                (Instruction::EndExpression, None),
                (Instruction::Put, Some(4)),
                (Instruction::JumpTo, Some(1)),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
            vec![0, 2, 3],
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
                (Instruction::Put, Some(3)),
                (Instruction::JumpIfFalse, Some(2)),
                (Instruction::EndExpression, None),
                (Instruction::Put, Some(4)),
                (Instruction::JumpTo, Some(1)),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into())),
            vec![0, 2, 3],
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
                (Instruction::Put, Some(3)),
                (Instruction::JumpIfTrue, Some(2)),
                (Instruction::Put, Some(4)),
                (Instruction::JumpIfTrue, Some(3)),
                (Instruction::EndExpression, None),
                (Instruction::Put, Some(5)), // 6
                (Instruction::JumpTo, Some(1)),
                (Instruction::Put, Some(6)), // 8
                (Instruction::JumpTo, Some(1)),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(15.into()))
                .append(SimpleData::Number(10.into()))
                .append(SimpleData::Number(20.into())),
            vec![0, 4, 5, 7],
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
                (Instruction::Put, Some(3)),
                (Instruction::JumpIfTrue, Some(2)),
                (Instruction::Put, Some(4)),
                (Instruction::EndExpression, None), // 4
                (Instruction::Put, Some(5)),
                (Instruction::JumpTo, Some(1)),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(15.into()))
                .append(SimpleData::Number(10.into())),
            vec![0, 3, 4],
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
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::Reapply, Some(0)),
                (Instruction::Put, Some(5)),
                (Instruction::EndExpression, None), // 4
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into()))
                .append(SimpleData::Number(15.into())),
            vec![0, 4], // TODO: Either more tests to verify if extra jump is needed or find way to remove it
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
                (Instruction::Put, Some(3)),
                (Instruction::JumpIfTrue, Some(2)), // 2
                (Instruction::Put, Some(4)),
                (Instruction::JumpIfTrue, Some(3)), // 4
                (Instruction::Put, Some(5)),
                (Instruction::JumpIfTrue, Some(4)), // 6
                (Instruction::EndExpression, None),
                (Instruction::Put, Some(6)), // 8
                (Instruction::JumpTo, Some(1)),
                (Instruction::Put, Some(7)), // 10
                (Instruction::JumpTo, Some(1)),
                (Instruction::Put, Some(8)), // 12
                (Instruction::JumpTo, Some(1)),
            ],
            SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(15.into()))
                .append(SimpleData::Number(25.into()))
                .append(SimpleData::Number(10.into()))
                .append(SimpleData::Number(20.into()))
                .append(SimpleData::Number(30.into())),
            vec![0, 6, 7, 9, 11],
        );
    }
}
