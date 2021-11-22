use log::trace;
use std::{collections::HashMap, hash::Hash, vec};

use crate::lexer::*;

#[derive(Debug, PartialOrd, Eq, PartialEq, Clone, Copy, Hash)]
pub enum Definition {
    Number,
    Identifier,
    AbsoluteValue,
    EmptyApply,
    Addition,
    Equality,
    Pair,
    Access,
    List,
    Drop,
    Symbol,
    Input,
    Result,
    Unit,
    Subexpression,
    Group,
    NestedExpression,
    ApplyIfTrue,
    ApplyIfFalse,
    ConditionalBranch,
}

#[derive(Debug, PartialOrd, Eq, PartialEq, Clone, Copy, Hash)]
pub enum Associativity {
    LeftToRight,
    RightToLeft,
}

impl Definition {
    pub fn is_value_like(self) -> bool {
        self == Definition::Number
            || self == Definition::Identifier
            || self == Definition::Symbol
            || self == Definition::Unit
            || self == Definition::Input
            || self == Definition::Result
    }

    pub fn is_group_like(self) -> bool {
        self == Definition::Group || self == Definition::NestedExpression
    }

    pub fn is_conditional(self) -> bool {
        self == Definition::ApplyIfFalse || self == Definition::ApplyIfTrue
    }

    pub fn associativity(self) -> Associativity {
        match self {
            Definition::AbsoluteValue => Associativity::RightToLeft,
            _ => Associativity::LeftToRight,
        }
    }
}

enum SecondaryDefinition {
    Value,
    BinaryLeftToRight,
    UnaryPrefix,
    UnarySuffix,
    StartGrouping,
    EndGrouping,
    Subexpression,
    Whitespace,
    Conditional,
    Comma,
}

fn get_definition(token_type: TokenType) -> (Definition, SecondaryDefinition) {
    match token_type {
        // Values
        TokenType::Unknown => (Definition::Drop, SecondaryDefinition::Value),
        TokenType::UnitLiteral => (Definition::Unit, SecondaryDefinition::Value),
        TokenType::Symbol => (Definition::Symbol, SecondaryDefinition::Value),
        TokenType::Number => (Definition::Number, SecondaryDefinition::Value),
        TokenType::Result => (Definition::Result, SecondaryDefinition::Value),
        TokenType::Input => (Definition::Input, SecondaryDefinition::Value),
        TokenType::Identifier => (Definition::Identifier, SecondaryDefinition::Value),

        // Groupings
        TokenType::StartExpression => (Definition::NestedExpression, SecondaryDefinition::StartGrouping),
        TokenType::EndExpression => (Definition::Drop, SecondaryDefinition::EndGrouping),
        TokenType::StartGroup => (Definition::Group, SecondaryDefinition::StartGrouping),
        TokenType::EndGroup => (Definition::Drop, SecondaryDefinition::EndGrouping),

        // Specialty
        // TokenType::Annotation => (Definition::Addition, SecondaryDefinition::BinaryLeftToRight),
        TokenType::Whitespace => (Definition::Drop, SecondaryDefinition::Whitespace),
        TokenType::Subexpression => (Definition::Subexpression, SecondaryDefinition::Subexpression),
        TokenType::Comma => (Definition::List, SecondaryDefinition::Comma),

        // Operations
        TokenType::EmptyApply => (Definition::EmptyApply, SecondaryDefinition::UnarySuffix),
        TokenType::AbsoluteValue => (Definition::AbsoluteValue, SecondaryDefinition::UnaryPrefix),
        TokenType::PlusSign => (Definition::Addition, SecondaryDefinition::BinaryLeftToRight),
        TokenType::Equality => (Definition::Equality, SecondaryDefinition::BinaryLeftToRight),
        TokenType::Period => (Definition::Access, SecondaryDefinition::BinaryLeftToRight),
        TokenType::Pair => (Definition::Pair, SecondaryDefinition::BinaryLeftToRight),
        // TokenType::MultiplicationSign => (Definition::Addition, SecondaryDefinition::BinaryLeftToRight),
        // TokenType::ExponentialSign => (Definition::Addition, SecondaryDefinition::BinaryLeftToRight),
        // TokenType::Apply => (Definition::Addition, SecondaryDefinition::BinaryLeftToRight),
        // TokenType::ApplyTo => (Definition::Addition, SecondaryDefinition::BinaryLeftToRight),
        // TokenType::Reapply => (Definition::Addition, SecondaryDefinition::BinaryLeftToRight),

        // Conditionals
        TokenType::ApplyIfFalse => (Definition::ApplyIfFalse, SecondaryDefinition::Conditional),
        TokenType::ApplyIfTrue => (Definition::ApplyIfTrue, SecondaryDefinition::Conditional),
        _ => todo!(),
    }
}

#[derive(Debug, PartialOrd, Eq, PartialEq, Clone)]
pub struct ParseNode {
    definition: Definition,
    parent: Option<usize>,
    left: Option<usize>,
    right: Option<usize>,
    lex_token: LexerToken,
}

impl ParseNode {
    pub fn new(definition: Definition, parent: Option<usize>, left: Option<usize>, right: Option<usize>, lex_token: LexerToken) -> ParseNode {
        ParseNode {
            definition,
            parent,
            left,
            right,
            lex_token,
        }
    }

    pub fn get_definition(&self) -> Definition {
        self.definition
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

    pub fn get_node(&self, index: usize) -> Option<&ParseNode> {
        self.nodes.get(index)
    }
}

fn make_priority_map() -> HashMap<Definition, usize> {
    let mut map = HashMap::new();

    map.insert(Definition::Number, 1);
    map.insert(Definition::Identifier, 1);
    map.insert(Definition::Symbol, 1);
    map.insert(Definition::Unit, 1);
    map.insert(Definition::Input, 1);
    map.insert(Definition::Result, 1);

    map.insert(Definition::Group, 2);
    map.insert(Definition::NestedExpression, 2);
    map.insert(Definition::Access, 3);
    map.insert(Definition::EmptyApply, 4);
    map.insert(Definition::AbsoluteValue, 5);
    map.insert(Definition::Addition, 10);
    map.insert(Definition::Equality, 14);
    map.insert(Definition::Pair, 21);
    map.insert(Definition::List, 23);
    map.insert(Definition::ApplyIfTrue, 27);
    map.insert(Definition::ApplyIfFalse, 27);
    map.insert(Definition::ConditionalBranch, 28);
    map.insert(Definition::Subexpression, 100);

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
) -> Result<(Definition, Option<usize>, Option<usize>, Option<usize>), String> {
    let my_priority = match priority_map.get(&definition) {
        None => Err(format!("Definition '{:?}' not registered in priority map.", definition))?,
        Some(priority) => *priority,
    };

    let mut true_left = left;
    let mut current_left = left;

    let mut count = 0;

    let mut parent = None;
    let mut update_parent = None;

    // // go up tree until no parent
    trace!("Searching parent chain for true left");
    while let Some(left_index) = current_left {
        trace!("Walking: {:?}", left_index);
        match nodes.get(left_index) {
            None => Err(format!("Index assigned to node has no value in node list. {:?}", left_index))?,
            Some(node) => {
                let n: &ParseNode = node;

                let their_priority = match priority_map.get(&n.definition) {
                    None => Err(format!("Definition '{:?}' not registered in priority map.", n.definition))?,
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

                    update_parent = Some(left_index);

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
            return Err(format!("Max iterations reached when searching for last parent."));
        }
    }

    match true_left {
        None => (),
        Some(index) => match nodes.get_mut(index) {
            None => Err(format!("Index assigned to node has no value in node list. {:?}", index))?,
            Some(node) => node.parent = Some(id),
        },
    }

    match update_parent {
        None => (),
        Some(index) => match nodes.get_mut(index) {
            None => Err(format!("Index assigned to node has no value in node list. {:?}", index))?,
            Some(node) => node.right = Some(id),
        },
    }

    *check_for_list = false;

    Ok((definition, parent, true_left, right))
}

fn parse_value_like(
    id: usize,
    definition: Definition,
    check_for_list: &mut bool,
    parent: Option<usize>,
    next_last_left: &mut Option<usize>,
    nodes: &mut Vec<ParseNode>,
    last_left: Option<usize>,
    priority_map: &HashMap<Definition, usize>,
    last_token: LexerToken,
    under_group: Option<usize>,
) -> Result<(Definition, Option<usize>, Option<usize>, Option<usize>), String> {
    let mut new_parent = parent;

    if *check_for_list {
        trace!("List flag is set, creating list node before current node.");
        // use current id for list token
        let list_info = parse_token(
            id,
            Definition::List,
            last_left,
            Some(id + 1),
            nodes,
            &priority_map,
            check_for_list,
            under_group,
        )?;
        nodes.push(ParseNode::new(list_info.0, list_info.1, list_info.2, list_info.3, last_token.clone()));

        // update parent to point to list node just created
        new_parent = Some(id);

        // last left is the node we are about to create
        *next_last_left = Some(nodes.len());
    }

    Ok((definition, new_parent, None, None))
}

fn setup_space_list_check(
    last_left: Option<usize>,
    nodes: &mut Vec<ParseNode>,
    check_for_list: &mut bool,
    next_last_left: &mut Option<usize>,
) -> Result<(Definition, Option<usize>, Option<usize>, Option<usize>), String> {
    match last_left {
        None => (), // ignore, spaces at begining of input can't create a list
        Some(left) => match nodes.get(left) {
            None => Err(format!("Index assigned to node has no value in node list. {:?}", left))?,
            Some(left_node) => {
                if left_node.definition.is_value_like() {
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

pub fn parse(lex_tokens: Vec<LexerToken>) -> Result<ParseResult, String> {
    trace!("Starting parse");
    let priority_map = make_priority_map();

    let mut nodes = vec![];

    let mut next_parent = None;
    let mut last_left = None;
    let mut check_for_list = false;
    let mut last_token = LexerToken::empty();
    let mut next_last_left = None;
    let mut group_stack = vec![];
    let mut current_group = None;

    for (_, token) in lex_tokens.iter().enumerate() {
        trace!(
            "------ Start Token {:?} -------------------------------------------------------------------------",
            token.get_token_type()
        );
        trace!("");

        let id = nodes.len();

        let under_group = match current_group {
            None => None,
            Some(current) => match group_stack.get(current) {
                None => Err(format!("Current group set to non-existant group in stack."))?,
                Some(group) => Some(*group),
            },
        };

        // most operations will have their right be the next element
        // corrects are made after the fact
        let assumed_right = match id + 1 >= lex_tokens.len() {
            true => None,
            false => Some(id + 1),
        };

        let (definition, secondary_definition) = get_definition(token.get_token_type());

        let (definition, parent, left, right) = match secondary_definition {
            SecondaryDefinition::Whitespace => setup_space_list_check(last_left, &mut nodes, &mut check_for_list, &mut &mut next_last_left)?,
            SecondaryDefinition::Value => parse_value_like(
                id,
                definition,
                &mut check_for_list,
                next_parent,
                &mut next_last_left,
                &mut nodes,
                last_left,
                &priority_map,
                last_token,
                under_group,
            )?,
            SecondaryDefinition::BinaryLeftToRight => {
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
                current_group = Some(group_stack.len());
                group_stack.push(id);

                let parent = next_parent;
                next_parent = Some(id);
                (definition, parent, None, assumed_right)
            }
            SecondaryDefinition::EndGrouping => {
                // if current grouping is a conditional group
                // end that group and our normal grouping
                let in_conditional = match current_group {
                    None => false,
                    Some(group) => match group_stack.get(group) {
                        None => Err(format!("Current group set to non-existant group in stack."))?,
                        Some(group_index) => match nodes.get(*group_index) {
                            None => Err(format!("Index assigned to node has no value in node list. {:?}", group))?,
                            Some(group_node) => {
                                trace!("Current group node definition is {:?}", group_node.definition);
                                group_node.definition.is_conditional()
                            }
                        },
                    },
                };

                if in_conditional {
                    // conditional node should have already been parented to contianing group
                    // current group will be corrected below
                    group_stack.pop();
                }

                next_last_left = group_stack.pop();
                current_group = match group_stack.len() == 0 {
                    true => None,
                    false => Some(group_stack.len() - 1),
                };

                (Definition::Drop, None, None, None)
            }
            SecondaryDefinition::Conditional => {
                next_parent = Some(id);

                trace!("Adding grouping with definition {:?}", definition);

                // if previous node was a conditional branch
                // we can create this conditional without a left
                let last_was_branch = match last_left {
                    None => false,
                    Some(left) => match nodes.get(left) {
                        None => Err(format!("Index assigned to node has no value in node list. {:?}", left))?,
                        Some(left_node) => left_node.definition == Definition::ConditionalBranch,
                    },
                };

                if last_was_branch {
                    (definition, last_left, None, assumed_right)
                } else {
                    current_group = Some(group_stack.len());
                    group_stack.push(id);

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
            }
            SecondaryDefinition::Comma => {
                // standard binary operation with List definition, unless in a conditional grouping

                trace!("Checking current group {:?}", current_group);
                let in_conditional = match current_group {
                    None => false,
                    Some(group) => match group_stack.get(group) {
                        None => Err(format!("Current group set to non-existant group in stack."))?,
                        Some(group_index) => match nodes.get(*group_index) {
                            None => Err(format!("Index assigned to node has no value in node list. {:?}", group))?,
                            Some(group_node) => {
                                trace!("Current group node definition is {:?}", group_node.definition);
                                group_node.definition.is_conditional()
                            }
                        },
                    },
                };

                next_parent = Some(id);

                if in_conditional {
                    trace!("In conditional grouping, will create conditional branch node.");
                    // end grouping and create new node with group as left
                    let left = group_stack.pop();
                    current_group = match group_stack.len() == 0 {
                        true => None,
                        false => Some(group_stack.len() - 1),
                    };

                    // redo under group, since we just moddified it
                    let under_group = match current_group {
                        None => None,
                        Some(current) => match group_stack.get(current) {
                            None => Err(format!("Current group set to non-existant group in stack."))?,
                            Some(group) => Some(*group),
                        },
                    };

                    parse_token(
                        id,
                        Definition::ConditionalBranch,
                        left,
                        assumed_right,
                        &mut nodes,
                        &priority_map,
                        &mut check_for_list,
                        under_group,
                    )?
                } else {
                    trace!("Not in conditional grouping, will create list node.");
                    parse_token(
                        id,
                        Definition::List,
                        last_left,
                        assumed_right,
                        &mut nodes,
                        &priority_map,
                        &mut check_for_list,
                        under_group,
                    )?
                }
            }
            SecondaryDefinition::Subexpression => {
                let (in_group, in_conditional) = match current_group {
                    None => (false, false),
                    Some(group) => match group_stack.get(group) {
                        None => Err(format!("Current group set to non-existant group in stack."))?,
                        Some(group_index) => match nodes.get(*group_index) {
                            None => Err(format!("Index assigned to node has no value in node list. {:?}", group))?,
                            Some(group_node) => {
                                trace!("Current group node definition is {:?}", group_node.definition);
                                (group_node.definition == Definition::Group, group_node.definition.is_conditional())
                            }
                        },
                    },
                };

                if in_group {
                    trace!("Inside of a group, treating as white space");
                    setup_space_list_check(last_left, &mut nodes, &mut check_for_list, &mut &mut next_last_left)?
                } else {
                    // only one in a row
                    // check if last parser node is a subexpression
                    // drop this token if so
                    let drop = match last_left {
                        None => false, // not a subexpression node
                        Some(left) => match nodes.get(left) {
                            None => Err(format!("Index assigned to node has no value in node list. {:?}", left))?,
                            Some(left_node) => left_node.definition == Definition::Subexpression,
                        },
                    };

                    // if in a conditional group, drop it now so it doesn't continue to next subexpression
                    if in_conditional {
                        // NEEDS A TEST

                        // conditional node should have already been parented to contianing group
                        // current group will be corrected below
                        group_stack.pop();
                    }

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
            nodes.push(ParseNode::new(definition, parent, left, right, token.clone()));
        }

        last_left = match next_last_left {
            Some(i) => {
                next_last_left = None;
                Some(i)
            }
            None => Some(id),
        };

        trace!("Last left set to {:?}", last_left);

        last_token = token.clone();

        trace!("");
        trace!(
            "------ End Token {:?} -------------------------------------------------------------------------",
            token.get_token_type()
        );
    }

    // walk up tree to find root
    trace!("Finding root node");
    let mut root = 0;
    let mut node = match nodes.get(0) {
        None => Err(format!("No node regisistered in first slot."))?,
        Some(n) => n,
    };

    let mut count = 0;

    trace!("Starting with node {:?} with definition {:?}", 0, node.get_definition());
    while !node.parent.is_none() {
        match node.get_parent() {
            None => unreachable!(),
            Some(i) => match nodes.get(i) {
                None => Err(format!("No node regisistered in slot {:?} of node in slot {:?}", i, root))?,
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
            return Err(format!("Max iterations reached when searching for root node."));
        }
    }

    Ok(ParseResult { root, nodes })
}

#[cfg(test)]
mod tests {
    use crate::lexer::*;
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
            Definition::Symbol,
            Definition::Unit,
            Definition::Input,
            Definition::Result,
        ];

        for def in value_like {
            assert!(def.is_value_like());
        }
    }

    #[test]
    fn group_like_definitions() {
        let value_like = [Definition::Group, Definition::NestedExpression];

        for def in value_like {
            assert!(def.is_group_like());
        }
    }

    #[test]
    fn conditional_definitions() {
        let value_like = [Definition::ApplyIfTrue, Definition::ApplyIfFalse];

        for def in value_like {
            assert!(def.is_conditional());
        }
    }

    #[test]
    fn single_number() {
        let tokens = vec![LexerToken::new("5".to_string(), TokenType::Number, 0, 0)];

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
        let tokens = vec![LexerToken::new("$".to_string(), TokenType::Input, 0, 0)];

        let result = parse(tokens).unwrap();

        assert_result(&result, 0, &[(0, Definition::Input, None, None, None)]);
    }

    #[test]
    fn single_result() {
        let tokens = vec![LexerToken::new("$?".to_string(), TokenType::Result, 0, 0)];

        let result = parse(tokens).unwrap();

        assert_result(&result, 0, &[(0, Definition::Result, None, None, None)]);
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
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
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
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("==".to_string(), TokenType::Equality, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
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
    fn pair() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("=".to_string(), TokenType::Pair, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
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
                (2, Definition::Identifier, Some(1), None, None),
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
    fn partial_addition_no_left() {
        let tokens = vec![
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            0,
            &[
                (0, Definition::Addition, None, None, Some(1)),
                (1, Definition::Number, Some(0), None, None),
            ],
        );
    }

    #[test]
    fn partial_addition_no_right() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::Addition, None, Some(0), None),
            ],
        );
    }

    #[test]
    fn absolute_value() {
        let tokens = vec![
            LexerToken::new("++".to_string(), TokenType::AbsoluteValue, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
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
    fn absolute_value_reversed() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("++".to_string(), TokenType::AbsoluteValue, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            0,
            &[
                (0, Definition::Number, None, None, None),
                (1, Definition::AbsoluteValue, None, None, None),
            ],
        );
    }

    #[test]
    fn absolute_value_then_addition() {
        let tokens = vec![
            LexerToken::new("++".to_string(), TokenType::AbsoluteValue, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
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
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("++".to_string(), TokenType::AbsoluteValue, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
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
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
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
            LexerToken::new("value".to_string(), TokenType::Number, 0, 0),
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
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
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
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("==".to_string(), TokenType::Equality, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0), // 4
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
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("==".to_string(), TokenType::Equality, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0), // 4
            LexerToken::new("==".to_string(), TokenType::Equality, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0), // 8
            LexerToken::new("==".to_string(), TokenType::Equality, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0), // 12
            LexerToken::new("==".to_string(), TokenType::Equality, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
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
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("==".to_string(), TokenType::Equality, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0), // 4
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("==".to_string(), TokenType::Equality, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0), // 8
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("==".to_string(), TokenType::Equality, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0), // 12
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("==".to_string(), TokenType::Equality, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
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

        println!("{:#?}", result);

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
                (3, Definition::Identifier, Some(2), None, None),
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
                (2, Definition::Identifier, Some(1), None, None),
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
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
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
                (4, Definition::Identifier, Some(3), None, None),
                (5, Definition::EmptyApply, Some(1), Some(3), None),
            ],
        );
    }
}

#[cfg(test)]
mod lists {
    use super::tests::*;
    use crate::lexer::*;
    use crate::*;

    #[test]
    fn two_item_comma_list() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new(",".to_string(), TokenType::Comma, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
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
    fn two_item_space_list() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
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
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("value".to_string(), TokenType::Identifier, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new(":symbol".to_string(), TokenType::Symbol, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("()".to_string(), TokenType::UnitLiteral, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("$".to_string(), TokenType::Input, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("$?".to_string(), TokenType::Result, 0, 0),
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
                (8, Definition::Input, Some(7), None, None),
                (9, Definition::List, None, Some(7), Some(10)),
                (10, Definition::Result, Some(9), None, None),
            ],
        );
    }

    #[test]
    fn space_list_with_operations() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Number, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new(" ".to_string(), TokenType::Whitespace, 0, 0),
            LexerToken::new("20".to_string(), TokenType::Number, 0, 0),
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
mod groups {
    use super::tests::*;
    use crate::lexer::*;
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
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
        ];

        assert_group_nested_results(
            0,
            tokens,
            &[(0, Definition::Group, None, None, Some(1)), (1, Definition::Number, Some(0), None, None)],
        )
    }

    #[test]
    fn single_operation() {
        let tokens = vec![
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
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
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
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
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
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
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
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
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
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
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
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
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
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
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
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
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("{".to_string(), TokenType::StartExpression, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new("\n\n".to_string(), TokenType::Subexpression, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
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
    use crate::lexer::*;
    use crate::*;

    #[test]
    fn conditional_if() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("?>".to_string(), TokenType::ApplyIfTrue, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Number, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::ApplyIfTrue, None, Some(0), Some(2)),
                (2, Definition::Number, Some(1), None, None),
            ],
        );
    }

    #[test]
    fn conditional_else() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("!>".to_string(), TokenType::ApplyIfFalse, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Number, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            1,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::ApplyIfFalse, None, Some(0), Some(2)),
                (2, Definition::Number, Some(1), None, None),
            ],
        );
    }

    #[test]
    fn conditional_chain_of_two() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("?>".to_string(), TokenType::ApplyIfTrue, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Number, 0, 0),
            LexerToken::new(",".to_string(), TokenType::Comma, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("?>".to_string(), TokenType::ApplyIfTrue, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Number, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            3,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::ApplyIfTrue, Some(3), Some(0), Some(2)),
                (2, Definition::Number, Some(1), None, None),
                (3, Definition::ConditionalBranch, None, Some(1), Some(5)),
                (4, Definition::Number, Some(5), None, None),
                (5, Definition::ApplyIfTrue, Some(3), Some(4), Some(6)),
                (6, Definition::Number, Some(5), None, None),
            ],
        );
    }

    #[test]
    fn conditional_chain_of_three() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("?>".to_string(), TokenType::ApplyIfTrue, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Number, 0, 0),
            LexerToken::new(",".to_string(), TokenType::Comma, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("?>".to_string(), TokenType::ApplyIfTrue, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Number, 0, 0),
            LexerToken::new(",".to_string(), TokenType::Comma, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("?>".to_string(), TokenType::ApplyIfTrue, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Number, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            7,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::ApplyIfTrue, Some(3), Some(0), Some(2)),
                (2, Definition::Number, Some(1), None, None),
                // second
                (3, Definition::ConditionalBranch, Some(7), Some(1), Some(5)),
                (4, Definition::Number, Some(5), None, None),
                (5, Definition::ApplyIfTrue, Some(3), Some(4), Some(6)),
                (6, Definition::Number, Some(5), None, None),
                // third
                (7, Definition::ConditionalBranch, None, Some(3), Some(9)),
                (8, Definition::Number, Some(9), None, None),
                (9, Definition::ApplyIfTrue, Some(7), Some(8), Some(10)),
                (10, Definition::Number, Some(9), None, None),
            ],
        );
    }

    #[test]
    fn conditional_chain_with_both_conditional_definitions() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("!>".to_string(), TokenType::ApplyIfFalse, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Number, 0, 0),
            LexerToken::new(",".to_string(), TokenType::Comma, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("?>".to_string(), TokenType::ApplyIfTrue, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Number, 0, 0),
            LexerToken::new(",".to_string(), TokenType::Comma, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("!>".to_string(), TokenType::ApplyIfFalse, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Number, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            7,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::ApplyIfFalse, Some(3), Some(0), Some(2)),
                (2, Definition::Number, Some(1), None, None),
                // second
                (3, Definition::ConditionalBranch, Some(7), Some(1), Some(5)),
                (4, Definition::Number, Some(5), None, None),
                (5, Definition::ApplyIfTrue, Some(3), Some(4), Some(6)),
                (6, Definition::Number, Some(5), None, None),
                // third
                (7, Definition::ConditionalBranch, None, Some(3), Some(9)),
                (8, Definition::Number, Some(9), None, None),
                (9, Definition::ApplyIfFalse, Some(7), Some(8), Some(10)),
                (10, Definition::Number, Some(9), None, None),
            ],
        );
    }

    #[test]
    fn conditional_chain_last_conditional_having_no_condition() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("?>".to_string(), TokenType::ApplyIfTrue, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Number, 0, 0),
            LexerToken::new(",".to_string(), TokenType::Comma, 0, 0),
            LexerToken::new("!>".to_string(), TokenType::ApplyIfFalse, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Number, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_result(
            &result,
            3,
            &[
                (0, Definition::Number, Some(1), None, None),
                (1, Definition::ApplyIfTrue, Some(3), Some(0), Some(2)),
                (2, Definition::Number, Some(1), None, None),
                (3, Definition::ConditionalBranch, None, Some(1), Some(4)),
                (4, Definition::ApplyIfFalse, Some(3), None, Some(5)),
                (5, Definition::Number, Some(4), None, None),
            ],
        );
    }

    #[test]
    fn conditional_ends_with_group() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("(".to_string(), TokenType::StartGroup, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("?>".to_string(), TokenType::ApplyIfTrue, 0, 0),
            LexerToken::new("10".to_string(), TokenType::Number, 0, 0),
            LexerToken::new(")".to_string(), TokenType::EndGroup, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
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
                (4, Definition::ApplyIfTrue, Some(2), Some(3), Some(5)),
                (5, Definition::Number, Some(4), None, None),
                (6, Definition::Addition, None, Some(1), Some(7)),
                (7, Definition::Number, Some(6), None, None),
            ],
        );
    }
}
