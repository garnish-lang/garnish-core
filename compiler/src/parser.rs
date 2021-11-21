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

    pub fn associativity(self) -> Associativity {
        match self {
            Definition::AbsoluteValue => Associativity::RightToLeft,
            _ => Associativity::LeftToRight,
        }
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

fn make_priority_map() -> (HashMap<Definition, usize>, Vec<Vec<usize>>) {
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
    map.insert(Definition::Subexpression, 100);

    let mut priority_table = vec![];
    for _ in 0..=26 {
        priority_table.push(vec![]);
    }

    (map, priority_table)
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
                    "Comparing priorities (current - {:?} {:?}) (left - {:?} {:?}",
                    definition,
                    my_priority,
                    n.definition,
                    their_priority
                );

                let is_our_group = n.definition.is_group_like()
                    && match under_group {
                        None => false,
                        Some(group_index) => group_index == left_index,
                    };

                // need to find node with higher priority and stop before it
                if my_priority < their_priority || is_our_group {
                    trace!("Stopping walk with true left {:?}", true_left);
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

    let new_left = true_left;
    let mut parent = None;

    match true_left {
        None => (), // allowed
        Some(left) => {
            let new_left_parent = match nodes.get_mut(left) {
                None => Err(format!("Index assigned to node has no value in node list. {:?}", left))?,
                Some(left_node) => {
                    trace!("Checking true left with index {:?}", left);
                    let n: &mut ParseNode = left_node;
                    let new_parent = n.parent;

                    match new_parent {
                        None => Some(id), // nothing additional
                        Some(parent_index) => match nodes.get_mut(parent_index) {
                            None => Err(format!("Index assigned to node has no value in node list. {:?}", parent_index))?,
                            Some(parent_node) => {
                                trace!("Checking true left parent with index {:?}", parent_index);

                                let pn: &mut ParseNode = parent_node;
                                let their_priority = match priority_map.get(&pn.definition) {
                                    None => Err(format!("Definition '{:?}' not registered in priority map.", pn.definition))?,
                                    Some(priority) => *priority,
                                };

                                if my_priority < their_priority || pn.definition.is_group_like() {
                                    trace!("Priority is less than true left's parent");
                                    // make their parent, my parent
                                    parent = Some(parent_index);

                                    // update their parent to point at us
                                    pn.right = Some(id);

                                    Some(id)
                                } else {
                                    unreachable!() // waiting to be proven wrong

                                    // trace!("Priority is greater than or equal to true left's parent");
                                    // pn.parent = Some(i);
                                    // new_left = Some(parent_index);
                                    // parent = None;

                                    // Some(parent_index)
                                }
                            }
                        },
                    }
                }
            };

            trace!("Setting true left parent to {:?}", new_left_parent);
            match nodes.get_mut(left) {
                None => Err(format!("Index assigned to node has no value in node list. {:?}", left))?,
                Some(left_node) => left_node.parent = new_left_parent,
            }
        }
    }

    *check_for_list = false;

    Ok((definition, parent, new_left, right))
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

pub fn parse(lex_tokens: Vec<LexerToken>) -> Result<ParseResult, String> {
    trace!("Starting parse");
    let (priority_map, _) = make_priority_map();

    let mut nodes = vec![];

    let mut next_parent = None;
    let mut last_left = None;
    let mut check_for_list = false;
    let mut last_token = LexerToken::empty();
    let mut next_last_left = None;
    let mut group_stack = vec![];
    let mut current_group = None;

    for (_, token) in lex_tokens.iter().enumerate() {
        trace!("Token {:?}", token.get_token_type());

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

        let (definition, parent, left, right) = match token.get_token_type() {
            TokenType::Number => {
                trace!("Parsing Number token");

                parse_value_like(
                    id,
                    Definition::Number,
                    &mut check_for_list,
                    next_parent,
                    &mut next_last_left,
                    &mut nodes,
                    last_left,
                    &priority_map,
                    last_token,
                    under_group,
                )?
            }
            TokenType::Symbol => {
                trace!("Parsing Symbol token");

                parse_value_like(
                    id,
                    Definition::Symbol,
                    &mut check_for_list,
                    next_parent,
                    &mut next_last_left,
                    &mut nodes,
                    last_left,
                    &priority_map,
                    last_token,
                    under_group,
                )?
            }
            TokenType::Identifier => {
                trace!("Parsing Identifier token");

                parse_value_like(
                    id,
                    Definition::Identifier,
                    &mut check_for_list,
                    next_parent,
                    &mut next_last_left,
                    &mut nodes,
                    last_left,
                    &priority_map,
                    last_token,
                    under_group,
                )?
            }
            TokenType::UnitLiteral => {
                trace!("Parsing Symbol token");

                parse_value_like(
                    id,
                    Definition::Unit,
                    &mut check_for_list,
                    next_parent,
                    &mut next_last_left,
                    &mut nodes,
                    last_left,
                    &priority_map,
                    last_token,
                    under_group,
                )?
            }
            TokenType::Input => {
                trace!("Parsing Symbol token");

                parse_value_like(
                    id,
                    Definition::Input,
                    &mut check_for_list,
                    next_parent,
                    &mut next_last_left,
                    &mut nodes,
                    last_left,
                    &priority_map,
                    last_token,
                    under_group,
                )?
            }
            TokenType::Result => {
                trace!("Parsing Symbol token");

                parse_value_like(
                    id,
                    Definition::Result,
                    &mut check_for_list,
                    next_parent,
                    &mut next_last_left,
                    &mut nodes,
                    last_left,
                    &priority_map,
                    last_token,
                    under_group,
                )?
            }
            TokenType::EmptyApply => {
                trace!("Parsing EmptyApply token");

                next_parent = Some(id);

                parse_token(
                    id,
                    Definition::EmptyApply,
                    last_left,
                    None,
                    &mut nodes,
                    &priority_map,
                    &mut check_for_list,
                    under_group,
                )?
            }
            TokenType::AbsoluteValue => {
                trace!("Parsing AbsoluteValue token");

                let parent = next_parent;
                next_parent = Some(id);

                (Definition::AbsoluteValue, parent, None, assumed_right)
            }
            TokenType::Whitespace => {
                trace!("Parsing HorizontalSpace token");

                // need to check for list
                // will be list if previous token and next token are value-like
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
                                check_for_list = true;
                            }
                        }
                    },
                }

                // retain last left instead of below code setting it to token that isn't being created
                next_last_left = last_left;

                (Definition::Drop, None, None, None)
            }
            TokenType::Subexpression => {
                trace!("Parsing Subexpression token");

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
            TokenType::Comma => {
                trace!("Parsing Comma token");

                next_parent = Some(id);
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
            TokenType::Pair => {
                trace!("Parsing Pair token");

                next_parent = Some(id);
                parse_token(
                    id,
                    Definition::Pair,
                    last_left,
                    assumed_right,
                    &mut nodes,
                    &priority_map,
                    &mut check_for_list,
                    under_group,
                )?
            }
            TokenType::Period => {
                trace!("Parsing Period token");

                next_parent = Some(id);
                parse_token(
                    id,
                    Definition::Access,
                    last_left,
                    assumed_right,
                    &mut nodes,
                    &priority_map,
                    &mut check_for_list,
                    under_group,
                )?
            }
            TokenType::PlusSign => {
                trace!("Parsing PlusSign token");

                next_parent = Some(id);
                parse_token(
                    id,
                    Definition::Addition,
                    last_left,
                    assumed_right,
                    &mut nodes,
                    &priority_map,
                    &mut check_for_list,
                    under_group,
                )?
            }
            TokenType::Equality => {
                trace!("Parsing Equality token");

                next_parent = Some(id);
                parse_token(
                    id,
                    Definition::Equality,
                    last_left,
                    assumed_right,
                    &mut nodes,
                    &priority_map,
                    &mut check_for_list,
                    under_group,
                )?
            }
            TokenType::StartGroup => {
                current_group = Some(group_stack.len());
                group_stack.push(id);

                let parent = next_parent;
                next_parent = Some(id);
                (Definition::Group, parent, None, assumed_right)
            }
            TokenType::EndGroup => {
                next_last_left = group_stack.pop();
                current_group = match group_stack.len() == 0 {
                    true => None,
                    false => Some(group_stack.len() - 1),
                };

                (Definition::Drop, None, None, None)
            }
            TokenType::StartExpression => {
                current_group = Some(group_stack.len());
                group_stack.push(id);

                let parent = next_parent;
                next_parent = Some(id);
                (Definition::NestedExpression, parent, None, assumed_right)
            }
            TokenType::EndExpression => {
                next_last_left = group_stack.pop();
                current_group = match group_stack.len() == 0 {
                    true => None,
                    false => Some(group_stack.len() - 1),
                };

                (Definition::Drop, None, None, None)
            }
            t => Err(format!("Definition from token type {:?} not defined.", t))?,
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

        // match priority_map.get(&definition) {
        //     None => Err(format!("Definition '{:?}' not registered in priority map.", definition))?,
        //     Some(priority) => match priority_table.get_mut(*priority) {
        //         None => Err(format!("No priority table regisitered at level {:?}", priority))?,
        //         Some(table) => table.push(id),
        //     },
        // }
    }

    // for table in priority_table.iter() {
    //     for addr in table {
    //         match nodes.get_mut(*addr) {
    //             None => Err(format!("Node with address {:?} regisitered in priority table does not exist", addr))?,
    //             Some(node) => match (node.get_left(), node.get_right()) {
    //                 (None, None) => (), //
    //                 (Some(left), None) => (),
    //                 (None, Some(right)) => (),
    //                 (Some(left), Some(right)) => (),
    //             },
    //         }
    //     }
    // }

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
}
