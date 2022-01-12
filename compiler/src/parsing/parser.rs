use crate::error::{
    append_token_details, composition_error, implementation_error, implementation_error_with_token, unclosed_grouping_error,
    unmatched_grouping_error, CompilerError,
};
use log::trace;
use std::{collections::HashMap, hash::Hash, vec};

use crate::lexing::lexer::*;

#[derive(Debug, PartialOrd, Eq, PartialEq, Clone, Copy, Hash)]
pub enum Definition {
    Number,
    Identifier,
    Property,
    AbsoluteValue,
    EmptyApply,
    Addition,
    Equality,
    Pair,
    Access,
    AccessLeftInternal,
    AccessRightInternal,
    AccessLengthInternal,
    List,
    CommaList,
    Drop,
    Symbol,
    Value,
    Unit,
    Subexpression,
    Group,
    NestedExpression,
    SideEffect,
    Apply,
    ApplyTo,
    Reapply,
    JumpIfTrue,
    JumpIfFalse,
    ElseJump,
    True,
    False,
}

impl Definition {
    pub fn is_value_like(self) -> bool {
        self == Definition::Number
            || self == Definition::Identifier
            || self == Definition::Property
            || self == Definition::Symbol
            || self == Definition::Unit
            || self == Definition::Value
            || self == Definition::True
            || self == Definition::False
    }

    pub fn is_group_like(self) -> bool {
        self == Definition::Group || self == Definition::NestedExpression || self == Definition::SideEffect
    }

    pub fn is_conditional(self) -> bool {
        self == Definition::JumpIfFalse || self == Definition::JumpIfTrue
    }

    pub fn is_optional(self) -> bool {
        self == Definition::CommaList
    }
}

#[derive(Debug, PartialOrd, Eq, PartialEq, Clone, Copy, Hash)]
pub enum SecondaryDefinition {
    None,
    Annotation,
    Value,
    OptionalBinaryLeftToRight,
    BinaryLeftToRight,
    UnaryPrefix,
    UnarySuffix,
    StartSideEffect,
    EndSideEffect,
    StartGrouping,
    EndGrouping,
    Subexpression,
    Whitespace,
    Identifier,
}

fn get_definition(token_type: TokenType) -> (Definition, SecondaryDefinition) {
    match token_type {
        // Values
        TokenType::Unknown => (Definition::Drop, SecondaryDefinition::Value),
        TokenType::UnitLiteral => (Definition::Unit, SecondaryDefinition::Value),
        TokenType::Symbol => (Definition::Symbol, SecondaryDefinition::Value),
        TokenType::Integer => (Definition::Number, SecondaryDefinition::Value),
        TokenType::Float => todo!(),
        TokenType::Value => (Definition::Value, SecondaryDefinition::Value),
        TokenType::True => (Definition::True, SecondaryDefinition::Value),
        TokenType::False => (Definition::False, SecondaryDefinition::Value),
        TokenType::Identifier => (Definition::Identifier, SecondaryDefinition::Identifier),

        // Groupings
        TokenType::StartExpression => (Definition::NestedExpression, SecondaryDefinition::StartGrouping),
        TokenType::EndExpression => (Definition::Drop, SecondaryDefinition::EndGrouping),
        TokenType::StartGroup => (Definition::Group, SecondaryDefinition::StartGrouping),
        TokenType::EndGroup => (Definition::Drop, SecondaryDefinition::EndGrouping),
        TokenType::StartSideEffect => (Definition::SideEffect, SecondaryDefinition::StartSideEffect),
        TokenType::EndSideEffect => (Definition::Drop, SecondaryDefinition::EndSideEffect),

        // Specialty
        TokenType::Annotation => (Definition::Drop, SecondaryDefinition::Annotation),
        TokenType::LineAnnotation => (Definition::Drop, SecondaryDefinition::Annotation),
        TokenType::Whitespace => (Definition::Drop, SecondaryDefinition::Whitespace),
        TokenType::Subexpression => (Definition::Subexpression, SecondaryDefinition::Subexpression),

        // Operations
        TokenType::EmptyApply => (Definition::EmptyApply, SecondaryDefinition::UnarySuffix),
        TokenType::AbsoluteValue => (Definition::AbsoluteValue, SecondaryDefinition::UnaryPrefix),
        TokenType::PlusSign => (Definition::Addition, SecondaryDefinition::BinaryLeftToRight),
        TokenType::Equality => (Definition::Equality, SecondaryDefinition::BinaryLeftToRight),
        TokenType::Period => (Definition::Access, SecondaryDefinition::BinaryLeftToRight),
        TokenType::Pair => (Definition::Pair, SecondaryDefinition::BinaryLeftToRight),
        TokenType::MultiplicationSign => todo!(),
        TokenType::ExponentialSign => todo!(),
        TokenType::Apply => (Definition::Apply, SecondaryDefinition::BinaryLeftToRight),
        TokenType::ApplyTo => (Definition::ApplyTo, SecondaryDefinition::BinaryLeftToRight),
        TokenType::LeftInternal => (Definition::AccessLeftInternal, SecondaryDefinition::UnaryPrefix),
        TokenType::RightInternal => (Definition::AccessRightInternal, SecondaryDefinition::UnarySuffix),
        TokenType::LengthInternal => (Definition::AccessLengthInternal, SecondaryDefinition::UnarySuffix),
        TokenType::Comma => (Definition::CommaList, SecondaryDefinition::OptionalBinaryLeftToRight),

        TokenType::Reapply => (Definition::Reapply, SecondaryDefinition::BinaryLeftToRight),

        // Conditionals
        TokenType::JumpIfFalse => (Definition::JumpIfFalse, SecondaryDefinition::BinaryLeftToRight),
        TokenType::JumpIfTrue => (Definition::JumpIfTrue, SecondaryDefinition::BinaryLeftToRight),
        TokenType::ElseJump => (Definition::ElseJump, SecondaryDefinition::BinaryLeftToRight),
    }
}

#[derive(Debug, PartialOrd, Eq, PartialEq, Clone)]
pub struct ParseNode {
    definition: Definition,
    secondary_definition: SecondaryDefinition,
    parent: Option<usize>,
    left: Option<usize>,
    right: Option<usize>,
    lex_token: LexerToken,
}

impl ParseNode {
    pub fn new(
        definition: Definition,
        secondary_definition: SecondaryDefinition,
        parent: Option<usize>,
        left: Option<usize>,
        right: Option<usize>,
        lex_token: LexerToken,
    ) -> ParseNode {
        ParseNode {
            definition,
            secondary_definition,
            parent,
            left,
            right,
            lex_token,
        }
    }

    pub fn get_definition(&self) -> Definition {
        self.definition
    }

    pub fn get_secondary_definition(&self) -> SecondaryDefinition {
        self.secondary_definition
    }

    pub fn get_parent(&self) -> Option<usize> {
        self.parent
    }

    pub fn get_left(&self) -> Option<usize> {
        self.left
    }

    pub fn get_right(&self) -> Option<usize> {
        self.right
    }

    pub fn get_lex_token(&self) -> LexerToken {
        self.lex_token.clone()
    }
}

#[derive(Debug, PartialOrd, Eq, PartialEq, Clone)]
pub struct ParseResult {
    root: usize,
    nodes: Vec<ParseNode>,
}

impl ParseResult {
    pub fn get_root(&self) -> usize {
        self.root
    }

    pub fn get_nodes(&self) -> &Vec<ParseNode> {
        &self.nodes
    }

    pub fn get_node(&self, index: usize) -> Option<&ParseNode> {
        self.nodes.get(index)
    }
}

fn make_priority_map() -> HashMap<Definition, usize> {
    let mut map = HashMap::new();

    map.insert(Definition::SideEffect, 5);

    map.insert(Definition::Number, 10);
    map.insert(Definition::Identifier, 10);
    map.insert(Definition::Property, 10);
    map.insert(Definition::Symbol, 10);
    map.insert(Definition::Unit, 10);
    map.insert(Definition::Value, 10);
    map.insert(Definition::True, 10);
    map.insert(Definition::False, 10);

    map.insert(Definition::Group, 20);
    map.insert(Definition::NestedExpression, 20);

    map.insert(Definition::Access, 30);

    map.insert(Definition::EmptyApply, 40);

    map.insert(Definition::AbsoluteValue, 50);
    map.insert(Definition::AccessLeftInternal, 50);

    map.insert(Definition::AccessRightInternal, 60);
    map.insert(Definition::AccessLengthInternal, 60);

    map.insert(Definition::Addition, 100);

    map.insert(Definition::Equality, 140);

    map.insert(Definition::Pair, 210);

    map.insert(Definition::List, 229);
    map.insert(Definition::CommaList, 230);

    map.insert(Definition::Apply, 250);
    map.insert(Definition::ApplyTo, 250);

    map.insert(Definition::Reapply, 260);

    map.insert(Definition::JumpIfTrue, 270);
    map.insert(Definition::JumpIfFalse, 270);

    map.insert(Definition::ElseJump, 280);

    map.insert(Definition::Subexpression, 1000);

    map
}

fn parse_token(
    id: usize,
    definition: Definition,
    left: Option<usize>,
    right: Option<usize>,
    nodes: &mut Vec<ParseNode>,
    priority_map: &HashMap<Definition, usize>,
    check_for_list: &mut bool,
    under_group: Option<usize>,
) -> Result<(Definition, Option<usize>, Option<usize>, Option<usize>), CompilerError> {
    let my_priority = match priority_map.get(&definition) {
        None => implementation_error(format!("Definition '{:?}' not registered in priority map.", definition))?,
        Some(priority) => *priority,
    };

    let mut true_left = left;
    let mut current_left = left;

    let mut count = 0;

    let mut parent = None;

    // // go up tree until no parent
    trace!("Searching parent chain for true left starting at {:?}", current_left);
    while let Some(left_index) = current_left {
        trace!("Walking: {:?}", left_index);
        match nodes.get(left_index) {
            None => implementation_error(format!("Index assigned to node has no value in node list. {:?}", left_index))?,
            Some(node) => {
                let n: &ParseNode = node;

                let their_priority = match priority_map.get(&n.definition) {
                    None => implementation_error(format!("Definition '{:?}' not registered in priority map.", n.definition))?,
                    Some(priority) => *priority,
                };

                trace!(
                    "Comparing priorities (current - {:?} {:?}) (left - {:?} {:?})",
                    definition,
                    my_priority,
                    n.definition,
                    their_priority
                );

                trace!("Check group. Current group is {:?}. Current index is {:?}", under_group, left_index);
                let is_our_group = n.definition.is_group_like()
                    && match under_group {
                        None => false,
                        Some(group_index) => group_index == left_index,
                    };

                // need to find node with higher priority and stop before it
                if my_priority < their_priority || is_our_group {
                    trace!("Stopping walk with true left {:?}", true_left);

                    parent = Some(left_index);

                    // stop
                    break;
                } else {
                    // continue

                    true_left = Some(left_index);
                    current_left = n.parent
                }
            }
        };

        // safty net, max iterations to len of nodes
        count += 1;
        if count > nodes.len() {
            implementation_error(format!("Max iterations reached when searching for last parent."))?;
        }
    }

    if parent == true_left {
        trace!("Parent and true left are same, Unsetting true left.");
        // can happen with optional binary operators that are next to start of groups
        // causes cyclic relationship
        // the parent is the correct value
        // unset true left

        true_left = None;
    }

    match true_left {
        None => (),
        Some(index) => match nodes.get_mut(index) {
            None => implementation_error(format!("Index assigned to node has no value in node list. {:?}", index))?,
            Some(node) => {
                trace!("Updating true left's ({:?}) parent to {:?}", index, id);
                node.parent = Some(id)
            }
        },
    }

    match parent {
        None => (),
        Some(index) => match nodes.get_mut(index) {
            None => implementation_error(format!("Index assigned to node has no value in node list. {:?}", index))?,
            Some(node) => {
                trace!("Updating parent's ({:?}) right to {:?}", index, id);
                let right = node.right;

                node.right = Some(id);

                // in case of side effect
                // parent's right might not have been reassigned by becoming true left
                // do it again here shouldn't change anything for other cases
                // pre assigned above to avoid mutability borrow issue
                match right {
                    None => (),
                    Some(r) => match nodes.get_mut(r) {
                        // None fine here since, right is initially assumed, may not have been created yet
                        // but should exist for side effect case
                        None => (),
                        Some(node) => {
                            node.parent = Some(id);
                            // and assign it this node as our new left
                            true_left = Some(r);
                        }
                    },
                }
            }
        },
    }

    *check_for_list = false;

    Ok((definition, parent, true_left, right))
}

fn parse_value_like(
    id: usize,
    definition: Definition,
    check_for_list: &mut bool,
    next_last_left: &mut Option<usize>,
    nodes: &mut Vec<ParseNode>,
    last_left: Option<usize>,
    priority_map: &HashMap<Definition, usize>,
    last_token: LexerToken,
    under_group: Option<usize>,
) -> Result<(Definition, Option<usize>, Option<usize>, Option<usize>), CompilerError> {
    let mut our_left = last_left;
    let mut our_id = id;

    if *check_for_list {
        trace!("List flag is set, creating list node before current node.");
        our_id = id + 1;
        // use current id for list token
        let list_info = parse_token(
            id,
            Definition::List,
            last_left,
            Some(our_id),
            nodes,
            &priority_map,
            check_for_list,
            under_group,
        )?;
        nodes.push(ParseNode::new(
            list_info.0,
            SecondaryDefinition::StartGrouping,
            list_info.1,
            list_info.2,
            list_info.3,
            last_token.clone(),
        ));

        // update parent to point to list node just created
        our_left = Some(id);

        // last left is the node we are about to create
        *next_last_left = Some(nodes.len());
    }

    parse_token(our_id, definition, our_left, None, nodes, priority_map, check_for_list, under_group)
}

fn setup_space_list_check(
    last_left: Option<usize>,
    current_group: Option<usize>,
    nodes: &mut Vec<ParseNode>,
    check_for_list: &mut bool,
    next_last_left: &mut Option<usize>,
) -> Result<(Definition, Option<usize>, Option<usize>, Option<usize>), CompilerError> {
    match last_left {
        None => (), // ignore, spaces at begining of input can't create a list
        Some(left) => match nodes.get(left) {
            None => implementation_error(format!("Index assigned to node has no value in node list. {:?}", left))?,
            Some(left_node) => {
                trace!(
                    "Checking for space list. Left {:?} {:?}. Current group {:?}.",
                    left,
                    left_node.get_definition(),
                    current_group
                );

                let is_value = left_node.definition.is_value_like();
                let is_group_value = left_node.definition.is_group_like() && last_left != current_group;
                if is_value || is_group_value {
                    trace!(
                        "Value-like definition {:?} found. Will check next token for value-like to make list",
                        left_node.definition
                    );
                    *check_for_list = true;
                }
            }
        },
    }

    // retain last left instead of below code setting it to token that isn't being created
    *next_last_left = last_left;

    Ok((Definition::Drop, None, None, None))
}

// composition validation
// value definitions must be preceded and succeded by non-value
// binary ops must be preceded by a value or unary suffix and succeded by value or unary prefix
// unary prefix must be preceded by unary prefix or binary and succeded by value or unary prefix
// unary suffix must be preceded by value or unary suffix, and succeded by unary suffix or binary
fn check_composition(previous: SecondaryDefinition, current: SecondaryDefinition, check_for_list: bool, token: &LexerToken) -> Result<(), CompilerError> {
    trace!("Composition check between previous {:?} and current {:?}", previous, current);
    match (previous, current) {
        (SecondaryDefinition::Value, SecondaryDefinition::Value) if !check_for_list => composition_error(previous, current, &token),
        (SecondaryDefinition::None, SecondaryDefinition::EndGrouping)
        | (SecondaryDefinition::None, SecondaryDefinition::BinaryLeftToRight)
        | (SecondaryDefinition::None, SecondaryDefinition::UnarySuffix)
        | (SecondaryDefinition::Subexpression, SecondaryDefinition::BinaryLeftToRight)
        | (SecondaryDefinition::Subexpression, SecondaryDefinition::UnarySuffix)
        | (SecondaryDefinition::Value, SecondaryDefinition::Identifier)
        | (SecondaryDefinition::Value, SecondaryDefinition::StartGrouping)
        | (SecondaryDefinition::Value, SecondaryDefinition::UnaryPrefix)
        | (SecondaryDefinition::Identifier, SecondaryDefinition::Value)
        | (SecondaryDefinition::Identifier, SecondaryDefinition::Identifier)
        | (SecondaryDefinition::Identifier, SecondaryDefinition::StartGrouping)
        | (SecondaryDefinition::Identifier, SecondaryDefinition::UnaryPrefix)
        | (SecondaryDefinition::StartGrouping, SecondaryDefinition::None)
        | (SecondaryDefinition::StartGrouping, SecondaryDefinition::EndGrouping)
        | (SecondaryDefinition::StartGrouping, SecondaryDefinition::BinaryLeftToRight)
        | (SecondaryDefinition::StartGrouping, SecondaryDefinition::UnarySuffix)
        | (SecondaryDefinition::EndGrouping, SecondaryDefinition::Value)
        | (SecondaryDefinition::EndGrouping, SecondaryDefinition::Identifier)
        | (SecondaryDefinition::EndGrouping, SecondaryDefinition::StartGrouping)
        | (SecondaryDefinition::EndGrouping, SecondaryDefinition::UnaryPrefix)
        | (SecondaryDefinition::StartSideEffect, SecondaryDefinition::EndSideEffect)
        | (SecondaryDefinition::StartSideEffect, SecondaryDefinition::BinaryLeftToRight)
        | (SecondaryDefinition::StartSideEffect, SecondaryDefinition::UnarySuffix)
        | (SecondaryDefinition::BinaryLeftToRight, SecondaryDefinition::None)
        | (SecondaryDefinition::BinaryLeftToRight, SecondaryDefinition::Subexpression)
        | (SecondaryDefinition::BinaryLeftToRight, SecondaryDefinition::EndGrouping)
        | (SecondaryDefinition::BinaryLeftToRight, SecondaryDefinition::EndSideEffect)
        | (SecondaryDefinition::BinaryLeftToRight, SecondaryDefinition::BinaryLeftToRight)
        | (SecondaryDefinition::BinaryLeftToRight, SecondaryDefinition::OptionalBinaryLeftToRight)
        | (SecondaryDefinition::OptionalBinaryLeftToRight, SecondaryDefinition::BinaryLeftToRight)
        | (SecondaryDefinition::OptionalBinaryLeftToRight, SecondaryDefinition::OptionalBinaryLeftToRight)
        | (SecondaryDefinition::UnaryPrefix, SecondaryDefinition::None)
        | (SecondaryDefinition::UnaryPrefix, SecondaryDefinition::Subexpression)
        | (SecondaryDefinition::UnaryPrefix, SecondaryDefinition::EndGrouping)
        | (SecondaryDefinition::UnaryPrefix, SecondaryDefinition::EndSideEffect)
        | (SecondaryDefinition::UnaryPrefix, SecondaryDefinition::BinaryLeftToRight)
        | (SecondaryDefinition::UnarySuffix, SecondaryDefinition::Value)
        | (SecondaryDefinition::UnarySuffix, SecondaryDefinition::Identifier)
        | (SecondaryDefinition::UnarySuffix, SecondaryDefinition::StartGrouping)
        | (SecondaryDefinition::UnarySuffix, SecondaryDefinition::UnaryPrefix) => composition_error(previous, current, &token),
        _ => Ok(()),
    }
}

const EMPTY_TOKENS: &[LexerToken] = &[];
fn trim_tokens(tokens: &Vec<LexerToken>) -> &[LexerToken] {
    let mut start = 0;
    let mut end = tokens.len();

    for token in tokens.iter() {
        if token.get_token_type() == TokenType::Whitespace || token.get_token_type() == TokenType::Subexpression {
            start += 1;
        } else {
            break;
        }
    }

    for token in tokens.iter().rev() {
        if token.get_token_type() == TokenType::Whitespace || token.get_token_type() == TokenType::Subexpression {
            end -= 1;
        } else {
            break;
        }
    }

    if start > end {
        return EMPTY_TOKENS;
    }

    &tokens[start..end]
}

pub fn parse(lex_tokens: Vec<LexerToken>) -> Result<ParseResult, CompilerError> {
    trace!("Starting parse");
    let priority_map = make_priority_map();

    let mut nodes = vec![];

    let mut next_parent = None;
    let mut last_left = None;
    let mut check_for_list = false;
    let mut last_token = LexerToken::empty();
    let mut next_last_left = None;
    let mut group_stack: Vec<(usize, bool)> = vec![];
    let mut current_group = None;
    let mut previous_second_def = SecondaryDefinition::None;

    let trimmed = trim_tokens(&lex_tokens);

    if trimmed.is_empty() {
        return Ok(ParseResult { root: 0, nodes });
    }

    for token in trimmed.iter() {
        trace!(
            "------ Start Token {:?} -------------------------------------------------------------------------",
            token.get_token_type()
        );
        trace!("");

        let id = nodes.len();
        trace!("Id: {}", id);

        let under_group = match current_group {
            None => None,
            Some(current) => match group_stack.get(current) {
                None => implementation_error_with_token(format!("Current group set to non-existant group in stack."), token)?,
                Some((group, _)) => Some(*group),
            },
        };

        // can't do this update with next_last_left on previous iteration because it could be None
        // if last left is side effect, we're not inside the side effect and side effect has a parent
        // change last left to the side effect's parent
        // current group is gotten above as under_group
        match last_left {
            None => (),
            Some(i) => match nodes.get(i) {
                None => implementation_error_with_token(format!("Index assigned to node has no value in node list. {:?}", i), token)?,
                Some(node) => {
                    let node: &ParseNode = node;
                    trace!("Checking if last left ({:?}) needs to be changed due to end of side effect.", last_left);
                    if node.get_definition() == Definition::SideEffect && last_left != under_group && node.parent.is_some() {
                        trace!("Changing last left to side effect's parent {:?}", node.parent);
                        last_left = node.parent;

                        last_left.and_then(|p| nodes.get(p)).and_then(|node| {
                            // need to update prev def as well for composition check
                            previous_second_def = node.secondary_definition;
                            Some(())
                        });
                    }
                }
            },
        }

        // most operations will have their right be the next element
        // corrects are made after the fact
        let assumed_right = match id + 1 >= lex_tokens.len() {
            true => None,
            false => Some(id + 1),
        };

        let (definition, secondary_definition) = get_definition(token.get_token_type());

        trace!(
            "Given preliminary definition of {:?} and secondary definition of {:?}",
            definition,
            secondary_definition
        );

        check_composition(previous_second_def, secondary_definition, check_for_list, token)?;

        // done with previous, can update now
        previous_second_def = secondary_definition;

        let (definition, parent, left, right) = match secondary_definition {
            SecondaryDefinition::None => implementation_error("Secondary definition of none shouldn't reach check.".to_string())?,
            SecondaryDefinition::Whitespace => setup_space_list_check(
                last_left,
                under_group,
                &mut nodes,
                &mut check_for_list,
                &mut &mut next_last_left,
            )?,
            SecondaryDefinition::Annotation => {
                next_last_left = last_left;
                (definition, None, None, None)
            }
            SecondaryDefinition::Value => append_token_details(
                parse_value_like(
                    id,
                    definition,
                    &mut check_for_list,
                    &mut next_last_left,
                    &mut nodes,
                    last_left,
                    &priority_map,
                    last_token,
                    under_group,
                ),
                token,
            )?,
            SecondaryDefinition::Identifier => {
                // having a parent of access means its on the left
                // and this identifier a property in the access operation
                // otherwise its a normal identifier that needs to be resolved
                let definition = match next_parent {
                    None => Definition::Identifier,
                    Some(parent) => match nodes.get(parent) {
                        None => implementation_error_with_token(format!("No node found at next parent index {:?}", parent), token)?,
                        Some(p) => {
                            let p: &ParseNode = p;
                            if p.definition == Definition::Access {
                                Definition::Property
                            } else {
                                Definition::Identifier
                            }
                        }
                    },
                };

                parse_value_like(
                    id,
                    definition,
                    &mut check_for_list,
                    &mut next_last_left,
                    &mut nodes,
                    last_left,
                    &priority_map,
                    last_token,
                    under_group,
                )?
            }
            SecondaryDefinition::BinaryLeftToRight | SecondaryDefinition::OptionalBinaryLeftToRight => {
                next_parent = Some(id);
                parse_token(
                    id,
                    definition,
                    last_left,
                    assumed_right,
                    &mut nodes,
                    &priority_map,
                    &mut check_for_list,
                    under_group,
                )?
            }
            SecondaryDefinition::UnaryPrefix => {
                let parent = next_parent;
                next_parent = Some(id);

                (definition, parent, None, assumed_right)
            }
            SecondaryDefinition::UnarySuffix => {
                next_parent = Some(id);
                parse_token(
                    id,
                    definition,
                    last_left,
                    None, // preventing sharing with BinaryLeftToRight
                    &mut nodes,
                    &priority_map,
                    &mut check_for_list,
                    under_group,
                )?
            }
            SecondaryDefinition::StartGrouping => {
                let mut parent = next_parent;
                let mut right = assumed_right;

                let mut our_id = id;

                current_group = Some(group_stack.len());

                // both groupings are considered to be value like
                // groups will result in a single value after evaluation
                // nested expressions are replaced with an expression value placeholder
                if check_for_list {
                    trace!("List flag is set, creating list node before current node.");

                    // our new id will be +1, since list node will use current id
                    our_id += 1;

                    // use current id for list token
                    let list_info = parse_token(
                        id,
                        Definition::List,
                        last_left,
                        Some(our_id),
                        &mut nodes,
                        &priority_map,
                        &mut check_for_list,
                        under_group,
                    )?;
                    nodes.push(ParseNode::new(
                        list_info.0,
                        SecondaryDefinition::StartGrouping,
                        list_info.1,
                        list_info.2,
                        list_info.3,
                        last_token.clone(),
                    ));

                    // update parent to point to list node just created
                    parent = Some(id);

                    // update right since we just added to node list
                    right = Some(our_id + 1);

                    // last left  is the node we are about to create
                    next_last_left = Some(our_id);
                }

                group_stack.push((our_id, check_for_list));
                next_parent = Some(our_id);

                (definition, parent, None, right)
            }
            SecondaryDefinition::StartSideEffect => {
                next_parent = Some(id);

                // gets set to false in parse token
                // but group stack should be pushed to after, store data now
                let group_info = (id, check_for_list);

                let info = parse_token(
                    id,
                    definition,
                    last_left,
                    assumed_right,
                    &mut nodes,
                    &priority_map,
                    &mut check_for_list,
                    under_group,
                )?;

                trace!("Starting side effect. Will check for list at end {:?}", group_info.1);
                current_group = Some(group_stack.len());
                group_stack.push(group_info);

                info
            }
            SecondaryDefinition::EndGrouping | SecondaryDefinition::EndSideEffect => {
                // should always have a value after popping hear
                // if not it means we didn't pass through start grouping and equal amount of times
                match group_stack.pop() {
                    None => unmatched_grouping_error(token)?,
                    Some((left, need_list_check)) => match nodes.get(left) {
                        None => implementation_error_with_token(format!("Index assigned to node has no value in node list. {:?}", left), token)?,
                        Some(start_group_node) => {
                            trace!("Ending grouping starting with {:?}. Will check for list {:?}", start_group_node.get_definition(), need_list_check);

                            next_last_left = Some(left);
                            check_for_list = need_list_check;

                            let expected_token = match start_group_node.definition {
                                Definition::Group => TokenType::EndGroup,
                                Definition::NestedExpression => TokenType::EndExpression,
                                Definition::SideEffect => TokenType::EndSideEffect,
                                d => implementation_error_with_token(format!("Unknown group definition added to group stack {:?}", d), token)?,
                            };

                            trace!("Expecting matching group token {:?}", expected_token);

                            // Check that end group matches start group
                            if token.get_token_type() != expected_token {
                                Err(
                                    CompilerError::new_message(format!("Syntax Error: Expected {:?} token, found {:?}", expected_token, token.get_token_type()))
                                        .append_token_details(&token),
                                )?;
                            }
                        }
                    },
                }

                current_group = match group_stack.is_empty() {
                    true => None,
                    false => Some(group_stack.len() - 1),
                };

                // check last left for optional
                // unset its right if so
                match last_left {
                    None => (), // unreachable?
                    Some(left) => match nodes.get_mut(left) {
                        None => implementation_error_with_token(format!("Index assigned to node has no value in node list. {:?}", left), token)?,
                        Some(left_node) => {
                            if left_node.definition.is_optional() {
                                left_node.right = None;
                            }
                        }
                    },
                }

                (Definition::Drop, None, None, None)
            }
            SecondaryDefinition::Subexpression => {
                let in_group = match current_group {
                    None => false,
                    Some(group) => match group_stack.get(group) {
                        None => implementation_error_with_token(format!("Current group set to non-existant group in stack."), token)?,
                        Some((group_index, _check_for_list)) => match nodes.get(*group_index) {
                            None => implementation_error_with_token(format!("Index assigned to node has no value in node list. {:?}", group), token)?,
                            Some(group_node) => {
                                trace!("Current group node definition is {:?}", group_node.definition);
                                group_node.definition == Definition::Group
                            }
                        },
                    },
                };

                if in_group {
                    trace!("Inside of a group, treating as white space");
                    setup_space_list_check(
                        last_left,
                        under_group,
                        &mut nodes,
                        &mut check_for_list,
                        &mut &mut next_last_left,
                    )?
                } else {
                    // only one in a row
                    // check if last parser node is a subexpression
                    // drop this token if so
                    let drop = match last_left {
                        None => false, // not a subexpression node
                        Some(left) => match nodes.get_mut(left) {
                            None => implementation_error_with_token(format!("Index assigned to node has no value in node list. {:?}", left), token)?,
                            Some(left_node) => {
                                // check last left for optional
                                // unset its right if so
                                if left_node.definition.is_optional() {
                                    left_node.right = None;
                                }

                                left_node.definition == Definition::Subexpression
                            }
                        },
                    };

                    if drop {
                        trace!("Previous parser node was a subexpression, dropping this one.");
                        // retain last left instead of below code setting it to token that isn't being created
                        next_last_left = last_left;
                        (Definition::Drop, None, None, None)
                    } else {
                        next_parent = Some(id);
                        parse_token(
                            id,
                            Definition::Subexpression,
                            last_left,
                            assumed_right,
                            &mut nodes,
                            &priority_map,
                            &mut check_for_list,
                            under_group,
                        )?
                    }
                }
            }
        };

        trace!(
            "Defined as {:?} with relationships parent {:?} left {:?} right {:?}",
            definition,
            parent,
            left,
            right
        );

        if definition != Definition::Drop {
            nodes.push(ParseNode::new(definition, secondary_definition, parent, left, right, token.clone()));
        }

        last_left = match next_last_left {
            Some(i) => {
                next_last_left = None;
                Some(i)
            }
            None => match nodes.len() == 0 {
                true => None,
                false => Some(id),
            },
        };

        trace!("Last left set to {:?}", last_left);

        last_token = token.clone();

        trace!("");
        trace!(
            "------ End Token {:?} -------------------------------------------------------------------------",
            token.get_token_type()
        );
    }

    // final composition check
    // previous is def of last node
    check_composition(previous_second_def, SecondaryDefinition::None, check_for_list, &last_token)?;

    // also make sure all groups have been closed
    if !group_stack.is_empty() {
        unclosed_grouping_error(&last_token)?;
    }

    // walk up tree to find root
    trace!("Finding root node");
    let mut root = 0;
    let mut node = match nodes.get(0) {
        None => implementation_error(format!("No node regisistered in first slot."))?,
        Some(n) => n,
    };

    let mut count = 0;

    trace!("Starting with node {:?} with definition {:?}", 0, node.get_definition());
    while !node.parent.is_none() {
        match node.get_parent() {
            None => unreachable!(),
            Some(i) => match nodes.get(i) {
                None => implementation_error(format!("No node regisistered in slot {:?} of node in slot {:?}", i, root))?,
                Some(parent) => {
                    trace!("Moving up to node {:?} with definition {:?}", i, parent.definition);
                    root = i;
                    node = parent;
                }
            },
        }

        // safty net, max iterations to len of nodes
        count += 1;
        if count > nodes.len() {
            implementation_error(format!("Max iterations reached when searching for root node."))?;
        }
    }

    Ok(ParseResult { root, nodes })
}

#[cfg(test)]
mod composition_errors {
    use crate::*;

    #[test]
    fn double_value_token() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn double_identifier_token() {
        let tokens = vec![
            LexerToken::new("value".to_string(), TokenType::Identifier, 0, 0),
            LexerToken::new("value".to_string(), TokenType::Identifier, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn value_identifier_token() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("value".to_string(), TokenType::Identifier, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn value_start_group() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn identifier_start_group() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Identifier, 0, 0),
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn end_group_value() {
        let tokens = vec![
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn end_group_identifier() {
        let tokens = vec![
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Identifier, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn identifier_value_token() {
        let tokens = vec![
            LexerToken::new("value".to_string(), TokenType::Identifier, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn double_binary() {
        let tokens = vec![
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn start_group_binary() {
        let tokens = vec![
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn binary_end_group() {
        let tokens = vec![
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn value_unary_prefix() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
            LexerToken::new("++".to_string(), TokenType::AbsoluteValue, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn identifier_unary_prefix() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Identifier, 0, 0),
            LexerToken::new("++".to_string(), TokenType::AbsoluteValue, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn end_group_unary_prefix() {
        let tokens = vec![
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
            LexerToken::new("++".to_string(), TokenType::AbsoluteValue, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn unary_prefix_end_group() {
        let tokens = vec![
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("++".to_string(), TokenType::AbsoluteValue, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn unary_prefix_binary() {
        let tokens = vec![
            LexerToken::new("++".to_string(), TokenType::AbsoluteValue, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn unary_suffix_value() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
            LexerToken::new("~~".to_string(), TokenType::EmptyApply, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn unary_suffix_identifier() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
            LexerToken::new("~~".to_string(), TokenType::EmptyApply, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Identifier, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn unary_suffix_unary_prefix() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
            LexerToken::new("~~".to_string(), TokenType::EmptyApply, 0, 0),
            LexerToken::new("++".to_string(), TokenType::AbsoluteValue, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Identifier, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn unary_suffix_start_group() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
            LexerToken::new("~~".to_string(), TokenType::EmptyApply, 0, 0),
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn start_group_unary_suffix() {
        let tokens = vec![
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("~~".to_string(), TokenType::EmptyApply, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn empty_group() {
        let tokens = vec![
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn mismatched_group() {
        let tokens = vec![
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
            LexerToken::new("}".to_string(), TokenType::EndExpression, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn empty_expression() {
        let tokens = vec![
            LexerToken::new("{".to_string(), TokenType::StartExpression, 0, 0),
            LexerToken::new("}".to_string(), TokenType::EndExpression, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn mismatched_expression() {
        let tokens = vec![
            LexerToken::new("{".to_string(), TokenType::StartExpression, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
            LexerToken::new("]".to_string(), TokenType::EndSideEffect, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn empty_side_effect() {
        let tokens = vec![
            LexerToken::new("[".to_string(), TokenType::StartSideEffect, 0, 0),
            LexerToken::new("]".to_string(), TokenType::EndSideEffect, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn mismatched_side_effect() {
        let tokens = vec![
            LexerToken::new("[".to_string(), TokenType::StartSideEffect, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn end_group_start_group() {
        let tokens = vec![
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn start_with_binary() {
        let tokens = vec![
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn end_with_binary() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn start_with_unary_suffix() {
        let tokens = vec![
            LexerToken::new("~~".to_string(), TokenType::EmptyApply, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn end_with_unary_prefix() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("++".to_string(), TokenType::AbsoluteValue, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn start_with_end_group() {
        let tokens = vec![LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0)];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn start_with_end_side_effect() {
        let tokens = vec![LexerToken::new("]".to_string(), TokenType::EndSideEffect, 0, 0)];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn start_with_end_expression() {
        let tokens = vec![LexerToken::new("}".to_string(), TokenType::EndExpression, 0, 0)];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn end_with_start_group() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn end_with_start_side_effect() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("[".to_string(), TokenType::StartSideEffect, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn end_with_start_expression() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("{".to_string(), TokenType::StartExpression, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn subexpression_binary() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn binary_subexpression() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn subexpression_unary_suffix() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new("~~".to_string(), TokenType::EmptyApply, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn unary_prefix_subexpression() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("++".to_string(), TokenType::AbsoluteValue, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn unclosed_group() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn end_group_without_start() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn optional_optional() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
            LexerToken::new(",".to_string(), TokenType::Comma, 0, 0),
            LexerToken::new(",".to_string(), TokenType::Comma, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn binary_optional() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new(",".to_string(), TokenType::Comma, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn optional_binary() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
            LexerToken::new(",".to_string(), TokenType::Comma, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Value, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn start_side_effect_binary() {
        let tokens = vec![
            LexerToken::new("[".to_string(), TokenType::StartSideEffect, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("]".to_string(), TokenType::EndSideEffect, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn start_side_effect_unary_suffix() {
        let tokens = vec![
            LexerToken::new("[".to_string(), TokenType::StartSideEffect, 0, 0),
            LexerToken::new("~~".to_string(), TokenType::EmptyApply, 0, 0),
            LexerToken::new("]".to_string(), TokenType::EndSideEffect, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn binary_end_side_effect() {
        let tokens = vec![
            LexerToken::new("[".to_string(), TokenType::StartSideEffect, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("]".to_string(), TokenType::EndSideEffect, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn unary_prefix_end_side_effect() {
        let tokens = vec![
            LexerToken::new("[".to_string(), TokenType::StartSideEffect, 0, 0),
            LexerToken::new("++".to_string(), TokenType::AbsoluteValue, 0, 0),
            LexerToken::new("]".to_string(), TokenType::EndSideEffect, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn side_effect_surrounded_by_value() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("[".to_string(), TokenType::StartSideEffect, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("]".to_string(), TokenType::EndSideEffect, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }

    #[test]
    fn side_effect_surrounded_by_binary() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("[".to_string(), TokenType::StartSideEffect, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("]".to_string(), TokenType::EndSideEffect, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens);

        assert!(result.is_err());
    }
}

#[cfg(test)]
mod tests {
    use crate::lexing::lexer::*;
    use crate::*;

    type DefAssertionInfo = (usize, Definition, Option<usize>, Option<usize>, Option<usize>);

    pub fn assert_result(result: &ParseResult, root: usize, assertions: &[DefAssertionInfo]) {
        assert_eq!(result.root, root, "Expected root to be {:?} was {:?}", root, result.root);

        for (index, definition, parent, left, right) in assertions {
            let node = result.get_node(*index).unwrap();

            assert_eq!(
                node.get_definition(),
                *definition,
                "{:?}: Expected definition to be {:?} was {:?}",
                index,
                *definition,
                node.get_definition()
            );

            assert_eq!(
                node.get_parent(),
                *parent,
                "{:?}: Expected parent to be {:?} was {:?}",
                index,
                *parent,
                node.get_parent()
            );

            assert_eq!(
                node.get_left(),
                *left,
                "{:?}: Expected left to be {:?} was {:?}",
                index,
                *left,
                node.get_left()
            );

            assert_eq!(
                node.get_right(),
                *right,
                "{:?}: Expected right to be {:?} was {:?}",
                index,
                right,
                node.get_right()
            );
        }
    }

    #[test]
    fn value_like_definitions() {
        let value_like = [
            Definition::Number,
            Definition::Identifier,
            Definition::Property,
            Definition::Symbol,
            Definition::Unit,
            Definition::Value,
            Definition::True,
            Definition::False,
        ];

        for def in value_like {
            assert!(def.is_value_like());
        }
    }

    #[test]
    fn optional_binary_definition() {
        let value_like = [Definition::CommaList];

        for def in value_like {
            assert!(def.is_optional());
        }
    }

    #[test]
    fn group_like_definitions() {
        let value_like = [Definition::Group, Definition::NestedExpression, Definition::SideEffect];

        for def in value_like {
            assert!(def.is_group_like());
        }
    }

    #[test]
    fn conditional_definitions() {
        let value_like = [Definition::JumpIfTrue, Definition::JumpIfFalse];

        for def in value_like {
            assert!(def.is_conditional());
        }
    }

    #[test]
    fn white_space_and_sub_expressions_trimed() {
        let tokens = vec![
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(&result, 0, &[(0, Definition::Number, None, None, None)]);
    }

    #[test]
    fn all_whitespace_or_subexpressions_is_empty() {
        let tokens = vec![
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(&result, 0, &[]);
    }

    #[test]
    fn single_number() {
        let tokens = vec![LexerToken::new("5".to_string(), TokenType::Integer, 0, 0)];

        let result = parse(tokens).unwrap();

        assert_result(&result, 0, &[(0, Definition::Number, None, None, None)]);
    }

    #[test]
    fn single_identifier() {
        let tokens = vec![LexerToken::new("value".to_string(), TokenType::Identifier, 0, 0)];

        let result = parse(tokens).unwrap();

        assert_result(&result, 0, &[(0, Definition::Identifier, None, None, None)]);
    }

    #[test]
    fn single_symbol() {
        let tokens = vec![LexerToken::new(":symbol".to_string(), TokenType::Symbol, 0, 0)];

        let result = parse(tokens).unwrap();

        assert_result(&result, 0, &[(0, Definition::Symbol, None, None, None)]);
    }

    #[test]
    fn single_input() {
        let tokens = vec![LexerToken::new("$".to_string(), TokenType::Value, 0, 0)];

        let result = parse(tokens).unwrap();

        assert_result(&result, 0, &[(0, Definition::Value, None, None, None)]);
    }

    #[test]
    fn single_true() {
        let tokens = vec![LexerToken::new("$?".to_string(), TokenType::True, 0, 0)];

        let result = parse(tokens).unwrap();

        assert_result(&result, 0, &[(0, Definition::True, None, None, None)]);
    }

    #[test]
    fn single_false() {
        let tokens = vec![LexerToken::new("$!".to_string(), TokenType::False, 0, 0)];

        let result = parse(tokens).unwrap();

        assert_result(&result, 0, &[(0, Definition::False, None, None, None)]);
    }

    #[test]
    fn single_unit() {
        let tokens = vec![LexerToken::new("()".to_string(), TokenType::UnitLiteral, 0, 0)];

        let result = parse(tokens).unwrap();

        assert_result(&result, 0, &[(0, Definition::Unit, None, None, None)]);
    }

    #[test]
    fn addition() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::Addition, None, Some(0), Some(2)),
                (2, Definition::Number, Some(1), None, None),
            ],
        );
    }

    #[test]
    fn equality() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("==".to_string(), TokenType::Equality, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::Equality, None, Some(0), Some(2)),
                (2, Definition::Number, Some(1), None, None),
            ],
        );
    }

    #[test]
    fn apply() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("~".to_string(), TokenType::Apply, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::Apply, None, Some(0), Some(2)),
                (2, Definition::Number, Some(1), None, None),
            ],
        );
    }

    #[test]
    fn apply_to() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("~>".to_string(), TokenType::ApplyTo, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::ApplyTo, None, Some(0), Some(2)),
                (2, Definition::Number, Some(1), None, None),
            ],
        );
    }

    #[test]
    fn reapply() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("^~".to_string(), TokenType::Reapply, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::Reapply, None, Some(0), Some(2)),
                (2, Definition::Number, Some(1), None, None),
            ],
        );
    }

    #[test]
    fn pair() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("=".to_string(), TokenType::Pair, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::Pair, None, Some(0), Some(2)),
                (2, Definition::Number, Some(1), None, None),
            ],
        );
    }

    #[test]
    fn access() {
        let tokens = vec![
            LexerToken::new("value".to_string(), TokenType::Identifier, 0, 0),
            LexerToken::new(".".to_string(), TokenType::Period, 0, 0),
            LexerToken::new("property".to_string(), TokenType::Identifier, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Identifier, Some(1), None, None),
                (1, Definition::Access, None, Some(0), Some(2)),
                (2, Definition::Property, Some(1), None, None),
            ],
        );
    }

    #[test]
    fn access_left_internal() {
        let tokens = vec![
            LexerToken::new("_.".to_string(), TokenType::LeftInternal, 0, 0),
            LexerToken::new("property".to_string(), TokenType::Identifier, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            0,
            &[
                (0, Definition::AccessLeftInternal, None, None, Some(1)),
                (1, Definition::Identifier, Some(0), None, None),
            ],
        );
    }

    #[test]
    fn access_right_internal() {
        let tokens = vec![
            LexerToken::new("property".to_string(), TokenType::Identifier, 0, 0),
            LexerToken::new("._".to_string(), TokenType::RightInternal, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Identifier, Some(1), None, None),
                (1, Definition::AccessRightInternal, None, Some(0), None),
            ],
        );
    }

    #[test]
    fn access_length_internal() {
        let tokens = vec![
            LexerToken::new("property".to_string(), TokenType::Identifier, 0, 0),
            LexerToken::new(".|".to_string(), TokenType::LengthInternal, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Identifier, Some(1), None, None),
                (1, Definition::AccessLengthInternal, None, Some(0), None),
            ],
        );
    }

    #[test]
    fn subexpression() {
        let tokens = vec![
            LexerToken::new("value".to_string(), TokenType::Identifier, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new("property".to_string(), TokenType::Identifier, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Identifier, Some(1), None, None),
                (1, Definition::Subexpression, None, Some(0), Some(2)),
                (2, Definition::Identifier, Some(1), None, None),
            ],
        );
    }

    #[test]
    fn subexpression_drop_multiple_in_a_row() {
        let tokens = vec![
            LexerToken::new("value".to_string(), TokenType::Identifier, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new("property".to_string(), TokenType::Identifier, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Identifier, Some(1), None, None),
                (1, Definition::Subexpression, None, Some(0), Some(2)),
                (2, Definition::Identifier, Some(1), None, None),
            ],
        );
    }

    #[test]
    fn absolute_value() {
        let tokens = vec![
            LexerToken::new("++".to_string(), TokenType::AbsoluteValue, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            0,
            &[
                (0, Definition::AbsoluteValue, None, None, Some(1)),
                (1, Definition::Number, Some(0), None, None),
            ],
        );
    }

    #[test]
    fn empty_apply() {
        let tokens = vec![
            LexerToken::new("value".to_string(), TokenType::Identifier, 0, 0),
            LexerToken::new("~~".to_string(), TokenType::EmptyApply, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Identifier, Some(1), None, None),
                (1, Definition::EmptyApply, None, Some(0), None),
            ],
        );
    }

    #[test]
    fn absolute_value_then_addition() {
        let tokens = vec![
            LexerToken::new("++".to_string(), TokenType::AbsoluteValue, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            2,
            &[
                (0, Definition::AbsoluteValue, Some(2), None, Some(1)),
                (1, Definition::Number, Some(0), None, None),
                (2, Definition::Addition, None, Some(0), Some(3)),
                (3, Definition::Number, Some(2), None, None),
            ],
        );
    }

    #[test]
    fn addition_then_absolute_value() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("++".to_string(), TokenType::AbsoluteValue, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::Addition, None, Some(0), Some(2)),
                (2, Definition::AbsoluteValue, Some(1), None, Some(3)),
                (3, Definition::Number, Some(2), None, None),
            ],
        );
    }

    #[test]
    fn three_absolute_value_operations() {
        let tokens = vec![
            LexerToken::new("++".to_string(), TokenType::AbsoluteValue, 0, 0),
            LexerToken::new("++".to_string(), TokenType::AbsoluteValue, 0, 0),
            LexerToken::new("++".to_string(), TokenType::AbsoluteValue, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            0,
            &[
                (0, Definition::AbsoluteValue, None, None, Some(1)),
                (1, Definition::AbsoluteValue, Some(0), None, Some(2)),
                (2, Definition::AbsoluteValue, Some(1), None, Some(3)),
                (3, Definition::Number, Some(2), None, None),
            ],
        );
    }

    #[test]
    fn unary_different_associativity_and_priority() {
        let tokens = vec![
            LexerToken::new("++".to_string(), TokenType::AbsoluteValue, 0, 0),
            LexerToken::new("++".to_string(), TokenType::AbsoluteValue, 0, 0),
            LexerToken::new("++".to_string(), TokenType::AbsoluteValue, 0, 0),
            LexerToken::new("value".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("~~".to_string(), TokenType::EmptyApply, 0, 0),
            LexerToken::new("~~".to_string(), TokenType::EmptyApply, 0, 0),
            LexerToken::new("~~".to_string(), TokenType::EmptyApply, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            0,
            &[
                (0, Definition::AbsoluteValue, None, None, Some(1)),
                (1, Definition::AbsoluteValue, Some(0), None, Some(2)),
                (2, Definition::AbsoluteValue, Some(1), None, Some(6)),
                (3, Definition::Number, Some(4), None, None),
                (4, Definition::EmptyApply, Some(5), Some(3), None),
                (5, Definition::EmptyApply, Some(6), Some(4), None),
                (6, Definition::EmptyApply, Some(2), Some(5), None),
            ],
        );
    }

    #[test]
    fn three_addition_operations() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            5,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::Addition, Some(3), Some(0), Some(2)),
                (2, Definition::Number, Some(1), None, None),
                (3, Definition::Addition, Some(5), Some(1), Some(4)),
                (4, Definition::Number, Some(3), None, None),
                (5, Definition::Addition, None, Some(3), Some(6)),
                (6, Definition::Number, Some(5), None, None),
            ],
        );
    }

    #[test]
    fn binary_operations_different_priority() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("==".to_string(), TokenType::Equality, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0), // 4
        ];

        // 5 == 5 + 5

        //         ==
        //     5      +
        //          5   5

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::Equality, None, Some(0), Some(3)),
                (2, Definition::Number, Some(3), None, None),
                (3, Definition::Addition, Some(1), Some(2), Some(4)),
                (4, Definition::Number, Some(3), None, None),
            ],
        );
    }

    #[test]
    fn multiple_binary_operations() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("==".to_string(), TokenType::Equality, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0), // 4
            LexerToken::new("==".to_string(), TokenType::Equality, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0), // 8
            LexerToken::new("==".to_string(), TokenType::Equality, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0), // 12
            LexerToken::new("==".to_string(), TokenType::Equality, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        // 5 == 5 + 5 == 5 + 5 == 5 + 5 == 5

        //                              ==
        //                       ==         5
        //               ==          +
        //         ==         +     5  5
        //     5      +     5   5
        //          5   5

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            13,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::Equality, Some(5), Some(0), Some(3)),
                (2, Definition::Number, Some(3), None, None),
                (3, Definition::Addition, Some(1), Some(2), Some(4)),
                (4, Definition::Number, Some(3), None, None),
                (5, Definition::Equality, Some(9), Some(1), Some(7)),
                (6, Definition::Number, Some(7), None, None),
                (7, Definition::Addition, Some(5), Some(6), Some(8)),
                (8, Definition::Number, Some(7), None, None),
                (9, Definition::Equality, Some(13), Some(5), Some(11)),
                (10, Definition::Number, Some(11), None, None),
                (11, Definition::Addition, Some(9), Some(10), Some(12)),
                (12, Definition::Number, Some(11), None, None),
                (13, Definition::Equality, None, Some(9), Some(14)),
                (14, Definition::Number, Some(13), None, None),
            ],
        );
    }

    #[test]
    fn multiple_binary_operations_with_spaces() {
        let tokens = vec![
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("==".to_string(), TokenType::Equality, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0), // 4
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("==".to_string(), TokenType::Equality, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0), // 8
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("==".to_string(), TokenType::Equality, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0), // 12
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("==".to_string(), TokenType::Equality, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
        ];

        // 5 == 5 + 5 == 5 + 5 == 5 + 5 == 5

        //                              ==
        //                       ==         5
        //               ==          +
        //         ==         +     5  5
        //     5      +     5   5
        //          5   5

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            13,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::Equality, Some(5), Some(0), Some(3)),
                (2, Definition::Number, Some(3), None, None),
                (3, Definition::Addition, Some(1), Some(2), Some(4)),
                (4, Definition::Number, Some(3), None, None),
                (5, Definition::Equality, Some(9), Some(1), Some(7)),
                (6, Definition::Number, Some(7), None, None),
                (7, Definition::Addition, Some(5), Some(6), Some(8)),
                (8, Definition::Number, Some(7), None, None),
                (9, Definition::Equality, Some(13), Some(5), Some(11)),
                (10, Definition::Number, Some(11), None, None),
                (11, Definition::Addition, Some(9), Some(10), Some(12)),
                (12, Definition::Number, Some(11), None, None),
                (13, Definition::Equality, None, Some(9), Some(14)),
                (14, Definition::Number, Some(13), None, None),
            ],
        );
    }

    #[test]
    fn prefix_unary_with_access() {
        let tokens = vec![
            LexerToken::new("++".to_string(), TokenType::AbsoluteValue, 0, 0),
            LexerToken::new("value".to_string(), TokenType::Identifier, 0, 0),
            LexerToken::new(".".to_string(), TokenType::Period, 0, 0),
            LexerToken::new("property".to_string(), TokenType::Identifier, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            0,
            &[
                (0, Definition::AbsoluteValue, None, None, Some(2)),
                (1, Definition::Identifier, Some(2), None, None),
                (2, Definition::Access, Some(0), Some(1), Some(3)),
                (3, Definition::Property, Some(2), None, None),
            ],
        );
    }

    #[test]
    fn suffix_unary_with_access() {
        let tokens = vec![
            LexerToken::new("value".to_string(), TokenType::Identifier, 0, 0),
            LexerToken::new(".".to_string(), TokenType::Period, 0, 0),
            LexerToken::new("property".to_string(), TokenType::Identifier, 0, 0),
            LexerToken::new("~~".to_string(), TokenType::EmptyApply, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            3,
            &[
                (0, Definition::Identifier, Some(1), None, None),
                (1, Definition::Access, Some(3), Some(0), Some(2)),
                (2, Definition::Property, Some(1), None, None),
                (3, Definition::EmptyApply, None, Some(1), None),
            ],
        );
    }

    #[test]
    fn suffix_unary_with_binary_operation() {
        let tokens = vec![
            LexerToken::new("value".to_string(), TokenType::Identifier, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("property".to_string(), TokenType::Identifier, 0, 0),
            LexerToken::new("~~".to_string(), TokenType::EmptyApply, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Identifier, Some(1), None, None),
                (1, Definition::Addition, None, Some(0), Some(3)),
                (2, Definition::Identifier, Some(3), None, None),
                (3, Definition::EmptyApply, Some(1), Some(2), None),
            ],
        );
    }

    #[test]
    fn suffix_unary_with_binary_operation_and_access() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("value".to_string(), TokenType::Identifier, 0, 0),
            LexerToken::new(".".to_string(), TokenType::Period, 0, 0),
            LexerToken::new("property".to_string(), TokenType::Identifier, 0, 0),
            LexerToken::new("~~".to_string(), TokenType::EmptyApply, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::Addition, None, Some(0), Some(5)),
                (2, Definition::Identifier, Some(3), None, None),
                (3, Definition::Access, Some(5), Some(2), Some(4)),
                (4, Definition::Property, Some(3), None, None),
                (5, Definition::EmptyApply, Some(1), Some(3), None),
            ],
        );
    }
}

#[cfg(test)]
mod lists {
    use super::tests::*;
    use crate::lexing::lexer::*;
    use crate::*;

    #[test]
    fn two_item_comma_list() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(",".to_string(), TokenType::Comma, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::CommaList, None, Some(0), Some(2)),
                (2, Definition::Number, Some(1), None, None),
            ],
        );
    }

    #[test]
    fn empty_list() {
        let tokens = vec![LexerToken::new(",".to_string(), TokenType::Comma, 0, 0)];

        let result = parse(tokens).unwrap();

        assert_result(&result, 0, &[(0, Definition::CommaList, None, None, None)]);
    }

    #[test]
    fn empty_list_in_group() {
        let tokens = vec![
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new(",".to_string(), TokenType::Comma, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            0,
            &[
                (0, Definition::Group, None, None, Some(1)),
                (1, Definition::CommaList, Some(0), None, None),
            ],
        );
    }

    #[test]
    fn single_item_left() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(",".to_string(), TokenType::Comma, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::CommaList, None, Some(0), None),
            ],
        );
    }

    #[test]
    fn single_item_left_subexpression() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(",".to_string(), TokenType::Comma, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            2,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::CommaList, Some(2), Some(0), None),
                (2, Definition::Subexpression, None, Some(1), Some(3)),
                (3, Definition::Number, Some(2), None, None),
            ],
        );
    }

    #[test]
    fn single_item_left_in_group() {
        let tokens = vec![
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(",".to_string(), TokenType::Comma, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            0,
            &[
                (0, Definition::Group, None, None, Some(2)),
                (1, Definition::Number, Some(2), None, None),
                (2, Definition::CommaList, Some(0), Some(1), None),
            ],
        );
    }

    #[test]
    fn single_item_right() {
        let tokens = vec![
            LexerToken::new(",".to_string(), TokenType::Comma, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            0,
            &[
                (0, Definition::CommaList, None, None, Some(1)),
                (1, Definition::Number, Some(0), None, None),
            ],
        );
    }

    #[test]
    fn subexpression_single_item_right() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new(",".to_string(), TokenType::Comma, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::Subexpression, None, Some(0), Some(2)),
                (2, Definition::CommaList, Some(1), None, Some(3)),
                (3, Definition::Number, Some(2), None, None),
            ],
        );
    }

    #[test]
    fn single_item_right_in_group() {
        let tokens = vec![
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new(",".to_string(), TokenType::Comma, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            0,
            &[
                (0, Definition::Group, None, None, Some(1)),
                (1, Definition::CommaList, Some(0), None, Some(2)),
                (2, Definition::Number, Some(1), None, None),
            ],
        );
    }

    #[test]
    fn comma_list_nested_in_space_list() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(",".to_string(), TokenType::Comma, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(",".to_string(), TokenType::Comma, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            9,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::CommaList, Some(9), Some(0), Some(7)),
                //
                (2, Definition::Number, Some(3), None, None),
                (3, Definition::List, Some(5), Some(2), Some(4)),
                (4, Definition::Number, Some(3), None, None),
                (5, Definition::List, Some(7), Some(3), Some(6)),
                (6, Definition::Number, Some(5), None, None),
                (7, Definition::List, Some(1), Some(5), Some(8)),
                (8, Definition::Number, Some(7), None, None),
                //
                (9, Definition::CommaList, None, Some(1), Some(10)),
                (10, Definition::Number, Some(9), None, None),
            ],
        );
    }

    #[test]
    fn two_item_space_list() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::List, None, Some(0), Some(2)),
                (2, Definition::Number, Some(1), None, None),
            ],
        );
    }

    #[test]
    fn space_list_all_value_like() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("value".to_string(), TokenType::Identifier, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new(":symbol".to_string(), TokenType::Symbol, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("()".to_string(), TokenType::UnitLiteral, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("$".to_string(), TokenType::Value, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("$?".to_string(), TokenType::True, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            9,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::List, Some(3), Some(0), Some(2)),
                (2, Definition::Identifier, Some(1), None, None),
                (3, Definition::List, Some(5), Some(1), Some(4)),
                (4, Definition::Symbol, Some(3), None, None),
                (5, Definition::List, Some(7), Some(3), Some(6)),
                (6, Definition::Unit, Some(5), None, None),
                (7, Definition::List, Some(9), Some(5), Some(8)),
                (8, Definition::Value, Some(7), None, None),
                (9, Definition::List, None, Some(7), Some(10)),
                (10, Definition::True, Some(9), None, None),
            ],
        );
    }

    #[test]
    fn space_list_with_operations() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("20".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            3,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::Addition, Some(3), Some(0), Some(2)),
                (2, Definition::Number, Some(1), None, None),
                (3, Definition::List, None, Some(1), Some(5)),
                (4, Definition::Number, Some(5), None, None),
                (5, Definition::Addition, Some(3), Some(4), Some(6)),
                (6, Definition::Number, Some(5), None, None),
            ],
        );
    }
}

#[cfg(test)]
mod side_effects {
    use super::tests::*;
    use crate::lexing::lexer::*;
    use crate::*;

    #[test]
    fn alone() {
        let tokens = vec![
            LexerToken::new("[".to_string(), TokenType::StartSideEffect, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("]".to_string(), TokenType::EndSideEffect, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            0,
            &[
                (0, Definition::SideEffect, None, None, Some(1)),
                (1, Definition::Number, Some(0), None, None),
            ],
        );
    }

    #[test]
    fn after_value() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("[".to_string(), TokenType::StartSideEffect, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("]".to_string(), TokenType::EndSideEffect, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            0,
            &[
                (0, Definition::Number, None, None, Some(1)),
                (1, Definition::SideEffect, Some(0), None, Some(2)),
                (2, Definition::Number, Some(1), None, None),
            ],
        );
    }

    #[test]
    fn before_value() {
        let tokens = vec![
            LexerToken::new("[".to_string(), TokenType::StartSideEffect, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("]".to_string(), TokenType::EndSideEffect, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            2,
            &[
                (0, Definition::SideEffect, Some(2), None, Some(1)),
                (1, Definition::Number, Some(0), None, None),
                (2, Definition::Number, None, Some(0), None),
            ],
        );
    }

    #[test]
    fn between_binary() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("[".to_string(), TokenType::StartSideEffect, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("]".to_string(), TokenType::EndSideEffect, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::Addition, None, Some(0), Some(4)),
                (2, Definition::SideEffect, Some(4), None, Some(3)),
                (3, Definition::Number, Some(2), None, None),
                (4, Definition::Number, Some(1), Some(2), None),
            ],
        );
    }

    #[test]
    fn between_space_list() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("[".to_string(), TokenType::StartSideEffect, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("]".to_string(), TokenType::EndSideEffect, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            3,
            &[
                (0, Definition::Number, Some(3), None, Some(1)),
                (1, Definition::SideEffect, Some(0), None, Some(2)),
                (2, Definition::Number, Some(1), None, None),
                (3, Definition::List, None, Some(0), Some(4)),
                (4, Definition::Number, Some(3), None, None),
            ],
        );
    }

    #[test]
    fn after_value_space_is_not_list() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("[".to_string(), TokenType::StartSideEffect, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("]".to_string(), TokenType::EndSideEffect, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            0,
            &[
                (0, Definition::Number, None, None, Some(1)),
                (1, Definition::SideEffect, Some(0), None, Some(2)),
                (2, Definition::Number, Some(1), None, None),
            ],
        );
    }

    #[test]
    fn after_binary() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("[".to_string(), TokenType::StartSideEffect, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("]".to_string(), TokenType::EndSideEffect, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::Addition, None, Some(0), Some(2)),
                (2, Definition::Number, Some(1), None, Some(3)),
                (3, Definition::SideEffect, Some(2), None, Some(4)),
                (4, Definition::Number, Some(3), None, None),
            ],
        );
    }

    #[test]
    fn before_binary() {
        let tokens = vec![
            LexerToken::new("[".to_string(), TokenType::StartSideEffect, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("]".to_string(), TokenType::EndSideEffect, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            3,
            &[
                (0, Definition::SideEffect, Some(2), None, Some(1)),
                (1, Definition::Number, Some(0), None, None),
                (2, Definition::Number, Some(3), Some(0), None),
                (3, Definition::Addition, None, Some(2), Some(4)),
                (4, Definition::Number, Some(3), None, None),
            ],
        );
    }

    #[test]
    fn between_unary() {
        let tokens = vec![
            LexerToken::new("++".to_string(), TokenType::AbsoluteValue, 0, 0),
            LexerToken::new("[".to_string(), TokenType::StartSideEffect, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("]".to_string(), TokenType::EndSideEffect, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            0,
            &[
                (0, Definition::AbsoluteValue, None, None, Some(3)),
                (1, Definition::SideEffect, Some(3), None, Some(2)),
                (2, Definition::Number, Some(1), None, None),
                (3, Definition::Number, Some(0), Some(1), None),
            ],
        );
    }
}

#[cfg(test)]
mod groups {
    use super::tests::*;
    use crate::lexing::lexer::*;
    use crate::*;

    fn assert_group_nested_results(
        root: usize,
        tokens: Vec<LexerToken>,
        assertions: &[(usize, Definition, Option<usize>, Option<usize>, Option<usize>)],
    ) {
        // test groups
        assert_result(&parse(tokens.clone()).unwrap(), root, assertions);

        let exp_tokens: Vec<LexerToken> = tokens
            .iter()
            .map(|t| match t.get_token_type() {
                TokenType::StartGroup => LexerToken::new("{".to_string(), TokenType::StartExpression, 0, 0),
                TokenType::EndGroup => LexerToken::new("}".to_string(), TokenType::EndExpression, 0, 0),
                _ => t.clone(),
            })
            .collect();

        let exp_assertions: Vec<(usize, Definition, Option<usize>, Option<usize>, Option<usize>)> = assertions
            .iter()
            .map(|(i, def, p, l, r)| match def {
                Definition::Group => (*i, Definition::NestedExpression, *p, *l, *r),
                _ => (*i, *def, *p, *l, *r),
            })
            .collect();

        assert_result(&parse(exp_tokens.clone()).unwrap(), root, &exp_assertions);
    }

    #[test]
    fn single_value() {
        let tokens = vec![
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
        ];

        assert_group_nested_results(
            0,
            tokens,
            &[(0, Definition::Group, None, None, Some(1)), (1, Definition::Number, Some(0), None, None)],
        )
    }

    #[test]
    fn single_value_with_spaces() {
        let tokens = vec![
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
        ];

        assert_group_nested_results(
            0,
            tokens,
            &[(0, Definition::Group, None, None, Some(1)), (1, Definition::Number, Some(0), None, None)],
        )
    }

    #[test]
    fn in_list() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
        ];

        assert_group_nested_results(
            1,
            tokens,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::List, None, Some(0), Some(2)),
                (2, Definition::Group, Some(1), None, Some(3)),
                (3, Definition::Number, Some(2), None, None),
            ],
        )
    }

    #[test]
    fn in_list_with_spaces() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
        ];

        assert_group_nested_results(
            1,
            tokens,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::List, None, Some(0), Some(2)),
                (2, Definition::Group, Some(1), None, Some(3)),
                (3, Definition::Number, Some(2), None, None),
            ],
        )
    }

    #[test]
    fn in_list_with_list() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("15".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
        ];

        assert_group_nested_results(
            1,
            tokens,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::List, None, Some(0), Some(2)),
                (2, Definition::Group, Some(1), None, Some(4)),
                (3, Definition::Number, Some(4), None, None),
                (4, Definition::List, Some(2), Some(3), Some(5)),
                (5, Definition::Number, Some(4), None, None),
            ],
        )
    }

    #[test]
    fn list_of_groups() {
        let tokens = vec![
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("15".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("15".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
        ];

        assert_group_nested_results(
            4,
            tokens,
            &[
                (0, Definition::Group, Some(4), None, Some(2)),
                (1, Definition::Number, Some(2), None, None),
                (2, Definition::List, Some(0), Some(1), Some(3)),
                (3, Definition::Number, Some(2), None, None),
                (4, Definition::List, None, Some(0), Some(5)),
                (5, Definition::Group, Some(4), None, Some(7)),
                (6, Definition::Number, Some(7), None, None),
                (7, Definition::List, Some(5), Some(6), Some(8)),
                (8, Definition::Number, Some(7), None, None),
            ],
        )
    }

    #[test]
    fn single_operation() {
        let tokens = vec![
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
        ];

        assert_group_nested_results(
            0,
            tokens,
            &[
                (0, Definition::Group, None, None, Some(2)),
                (1, Definition::Number, Some(2), None, None),
                (2, Definition::Addition, Some(0), Some(1), Some(3)),
                (3, Definition::Number, Some(2), None, None),
            ],
        );
    }

    #[test]
    fn single_operation_with_operations_outside() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        assert_group_nested_results(
            6,
            tokens,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::Addition, Some(6), Some(0), Some(2)),
                (2, Definition::Group, Some(1), None, Some(4)),
                (3, Definition::Number, Some(4), None, None),
                (4, Definition::Addition, Some(2), Some(3), Some(5)),
                (5, Definition::Number, Some(4), None, None),
                (6, Definition::Addition, None, Some(1), Some(7)),
                (7, Definition::Number, Some(6), None, None),
            ],
        );
    }

    #[test]
    fn single_operation_with_unary_suffix_operations_outside() {
        let tokens = vec![
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
            LexerToken::new("~~".to_string(), TokenType::EmptyApply, 0, 0),
        ];

        assert_group_nested_results(
            4,
            tokens,
            &[
                (0, Definition::Group, Some(4), None, Some(2)),
                (1, Definition::Number, Some(2), None, None),
                (2, Definition::Addition, Some(0), Some(1), Some(3)),
                (3, Definition::Number, Some(2), None, None),
                (4, Definition::EmptyApply, None, Some(0), None),
            ],
        );
    }

    #[test]
    fn single_operation_with_unary_prefix_operations_outside() {
        let tokens = vec![
            LexerToken::new("++".to_string(), TokenType::AbsoluteValue, 0, 0),
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
        ];

        assert_group_nested_results(
            0,
            tokens,
            &[
                (0, Definition::AbsoluteValue, None, None, Some(1)),
                (1, Definition::Group, Some(0), None, Some(3)),
                (2, Definition::Number, Some(3), None, None),
                (3, Definition::Addition, Some(1), Some(2), Some(4)),
                (4, Definition::Number, Some(3), None, None),
            ],
        );
    }

    #[test]
    fn multiple_nested_groups() {
        let tokens = vec![
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
        ];

        assert_group_nested_results(
            0,
            tokens,
            &[
                // group 1
                (0, Definition::Group, None, None, Some(12)),
                (1, Definition::Number, Some(2), None, None),
                (2, Definition::Addition, Some(12), Some(1), Some(3)),
                // group 2
                (3, Definition::Group, Some(2), None, Some(10)),
                (4, Definition::Number, Some(5), None, None),
                (5, Definition::Addition, Some(10), Some(4), Some(6)),
                // group 3
                (6, Definition::Group, Some(5), None, Some(8)),
                (7, Definition::Number, Some(8), None, None),
                (8, Definition::Addition, Some(6), Some(7), Some(9)),
                (9, Definition::Number, Some(8), None, None),
                // group 2
                (10, Definition::Addition, Some(3), Some(5), Some(11)),
                (11, Definition::Number, Some(10), None, None),
                // group 1
                (12, Definition::Addition, Some(0), Some(2), Some(13)),
                (13, Definition::Number, Some(12), None, None),
            ],
        );
    }

    #[test]
    fn subexpression_in_group_makes_list() {
        let tokens = vec![
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            0,
            &[
                (0, Definition::Group, None, None, Some(2)),
                (1, Definition::Number, Some(2), None, None),
                (2, Definition::List, Some(0), Some(1), Some(3)),
                (3, Definition::Number, Some(2), None, None),
            ],
        );
    }

    #[test]
    fn multiple_subexpression_in_group_makes_list() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::Addition, None, Some(0), Some(2)),
                (2, Definition::Group, Some(1), None, Some(4)),
                (3, Definition::Number, Some(4), None, None),
                (4, Definition::List, Some(2), Some(3), Some(5)),
                (5, Definition::Number, Some(4), None, None),
            ],
        );
    }

    #[test]
    fn subexpression_in_nested_expression_is_subexpression() {
        let tokens = vec![
            LexerToken::new("{".to_string(), TokenType::StartExpression, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("}".to_string(), TokenType::EndExpression, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            0,
            &[
                (0, Definition::NestedExpression, None, None, Some(2)),
                (1, Definition::Number, Some(2), None, None),
                (2, Definition::Subexpression, Some(0), Some(1), Some(3)),
                (3, Definition::Number, Some(2), None, None),
            ],
        );
    }

    #[test]
    fn multiple_subexpression_in_nested_expression_is_subexpression() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("{".to_string(), TokenType::StartExpression, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("}".to_string(), TokenType::EndExpression, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::Addition, None, Some(0), Some(2)),
                (2, Definition::NestedExpression, Some(1), None, Some(4)),
                (3, Definition::Number, Some(4), None, None),
                (4, Definition::Subexpression, Some(2), Some(3), Some(5)),
                (5, Definition::Number, Some(4), None, None),
            ],
        );
    }
}

#[cfg(test)]
mod conditionals {
    use super::tests::*;
    use crate::lexing::lexer::*;
    use crate::*;

    #[test]
    fn conditional_if() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("?>".to_string(), TokenType::JumpIfTrue, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::JumpIfTrue, None, Some(0), Some(2)),
                (2, Definition::Number, Some(1), None, None),
            ],
        );
    }

    #[test]
    fn conditional_else() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("!>".to_string(), TokenType::JumpIfFalse, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::JumpIfFalse, None, Some(0), Some(2)),
                (2, Definition::Number, Some(1), None, None),
            ],
        );
    }

    #[test]
    fn conditional_chain_of_two() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("?>".to_string(), TokenType::JumpIfTrue, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("|>".to_string(), TokenType::ElseJump, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("?>".to_string(), TokenType::JumpIfTrue, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            3,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::JumpIfTrue, Some(3), Some(0), Some(2)),
                (2, Definition::Number, Some(1), None, None),
                (3, Definition::ElseJump, None, Some(1), Some(5)),
                (4, Definition::Number, Some(5), None, None),
                (5, Definition::JumpIfTrue, Some(3), Some(4), Some(6)),
                (6, Definition::Number, Some(5), None, None),
            ],
        );
    }

    #[test]
    fn conditional_chain_of_three() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("?>".to_string(), TokenType::JumpIfTrue, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("|>".to_string(), TokenType::ElseJump, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("?>".to_string(), TokenType::JumpIfTrue, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("|>".to_string(), TokenType::ElseJump, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("?>".to_string(), TokenType::JumpIfTrue, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            7,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::JumpIfTrue, Some(3), Some(0), Some(2)),
                (2, Definition::Number, Some(1), None, None),
                // second
                (3, Definition::ElseJump, Some(7), Some(1), Some(5)),
                (4, Definition::Number, Some(5), None, None),
                (5, Definition::JumpIfTrue, Some(3), Some(4), Some(6)),
                (6, Definition::Number, Some(5), None, None),
                // third
                (7, Definition::ElseJump, None, Some(3), Some(9)),
                (8, Definition::Number, Some(9), None, None),
                (9, Definition::JumpIfTrue, Some(7), Some(8), Some(10)),
                (10, Definition::Number, Some(9), None, None),
            ],
        );
    }

    #[test]
    fn conditional_chain_with_both_conditional_definitions() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("!>".to_string(), TokenType::JumpIfFalse, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("|>".to_string(), TokenType::ElseJump, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("?>".to_string(), TokenType::JumpIfTrue, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("|>".to_string(), TokenType::ElseJump, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("!>".to_string(), TokenType::JumpIfFalse, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            7,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::JumpIfFalse, Some(3), Some(0), Some(2)),
                (2, Definition::Number, Some(1), None, None),
                // second
                (3, Definition::ElseJump, Some(7), Some(1), Some(5)),
                (4, Definition::Number, Some(5), None, None),
                (5, Definition::JumpIfTrue, Some(3), Some(4), Some(6)),
                (6, Definition::Number, Some(5), None, None),
                // third
                (7, Definition::ElseJump, None, Some(3), Some(9)),
                (8, Definition::Number, Some(9), None, None),
                (9, Definition::JumpIfFalse, Some(7), Some(8), Some(10)),
                (10, Definition::Number, Some(9), None, None),
            ],
        );
    }

    #[test]
    fn conditional_chain_last_conditional_having_no_condition() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("?>".to_string(), TokenType::JumpIfTrue, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("|>".to_string(), TokenType::ElseJump, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            3,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::JumpIfTrue, Some(3), Some(0), Some(2)),
                (2, Definition::Number, Some(1), None, None),
                (3, Definition::ElseJump, None, Some(1), Some(4)),
                (4, Definition::Number, Some(3), None, None),
            ],
        );
    }

    #[test]
    fn reapply_with_else() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("^~".to_string(), TokenType::Reapply, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("|>".to_string(), TokenType::ElseJump, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            3,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::Reapply, Some(3), Some(0), Some(2)),
                (2, Definition::Number, Some(1), None, None),
                (3, Definition::ElseJump, None, Some(1), Some(4)),
                (4, Definition::Number, Some(3), None, None),
            ],
        );
    }

    #[test]
    fn conditional_ends_with_group() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new("?>".to_string(), TokenType::JumpIfTrue, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Integer, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            6,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::Addition, Some(6), Some(0), Some(2)),
                (2, Definition::Group, Some(1), None, Some(4)),
                (3, Definition::Number, Some(4), None, None),
                (4, Definition::JumpIfTrue, Some(2), Some(3), Some(5)),
                (5, Definition::Number, Some(4), None, None),
                (6, Definition::Addition, None, Some(1), Some(7)),
                (7, Definition::Number, Some(6), None, None),
            ],
        );
    }
}

#[cfg(test)]
mod annotations {
    use super::tests::*;
    use crate::lexing::lexer::*;
    use crate::*;

    #[test]
    fn annotations_are_dropped() {
        let tokens = vec![
            LexerToken::new("@value".to_string(), TokenType::Annotation, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(&result, 0, &[(0, Definition::Number, None, None, None)]);
    }

    #[test]
    fn line_annotations_are_dropped() {
        let tokens = vec![
            LexerToken::new("@@ Some line annotation".to_string(), TokenType::Annotation, 0, 0),
            LexerToken::new("\n".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Integer, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(&result, 0, &[(0, Definition::Number, None, None, None)]);
    }
}
