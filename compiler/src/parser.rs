use log::trace;
use std::{collections::HashMap, hash::Hash};

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
}

#[derive(Debug, PartialOrd, Eq, PartialEq, Clone, Copy, Hash)]
pub enum Associativity {
    LeftToRight,
    RightToLeft,
}

impl Definition {
    pub fn is_value_like(self) -> bool {
        self == Definition::Number
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
    map.insert(Definition::Access, 3);
    map.insert(Definition::EmptyApply, 4);
    map.insert(Definition::AbsoluteValue, 5);
    map.insert(Definition::Addition, 10);
    map.insert(Definition::Equality, 14);
    map.insert(Definition::Pair, 21);

    let mut priority_table = vec![];
    for _ in 0..=26 {
        priority_table.push(vec![]);
    }

    (map, priority_table)
}

pub fn parse(lex_tokens: Vec<LexerToken>) -> Result<ParseResult, String> {
    trace!("Starting parse");
    let (priority_map, mut priority_table) = make_priority_map();

    let mut nodes = vec![];

    let mut next_parent = None;
    let mut last_left = None;

    for (i, token) in lex_tokens.iter().enumerate() {
        trace!("Token {:?}", token.get_token_type());

        // most operations will have their right be the next element
        // corrects are made after the fact
        let assumed_right = match i + 1 >= lex_tokens.len() {
            true => None,
            false => Some(i + 1),
        };

        let (definition, parent, left, right) = match token.get_token_type() {
            TokenType::Number => {
                trace!("Parsing Number token");
                (Definition::Number, next_parent, None, None)
            }
            TokenType::Identifier => {
                trace!("Parsing Identifier token");
                (Definition::Identifier, next_parent, None, None)
            }
            TokenType::EmptyApply => {
                trace!("Parsing EmptyApply token");

                let mut parent = next_parent;
                next_parent = Some(i);

                match last_left {
                    None => (), // allowed
                    Some(left) => match nodes.get_mut(left) {
                        None => Err(format!("Index assigned to node has no value in node list. {:?}", left))?,
                        Some(left_node) => {
                            let n: &mut ParseNode = left_node;
                            let new_parent = n.parent;
                            n.parent = Some(i);

                            match new_parent {
                                None => (), // nothing additional
                                Some(parent_index) => match nodes.get_mut(parent_index) {
                                    None => Err(format!("Index assigned to node has no value in node list. {:?}", parent_index))?,
                                    Some(parent_node) => {
                                        let pn: &mut ParseNode = parent_node;
                                        // make their parent, my parent
                                        parent = Some(parent_index);

                                        // update their parent to point at us
                                        pn.right = Some(i);
                                    }
                                },
                            }
                        }
                    },
                }

                (Definition::EmptyApply, parent, last_left, None)
            }
            TokenType::AbsoluteValue => {
                trace!("Parsing AbsoluteValue token");

                let parent = next_parent;
                next_parent = Some(i);

                (Definition::AbsoluteValue, parent, None, assumed_right)
            }
            TokenType::Pair => {
                trace!("Parsing Pair token");

                let my_definition = Definition::Pair;
                next_parent = Some(i);

                let mut true_left = last_left;
                let mut parent = last_left;

                let mut count = 0;

                // // go up tree until no parent
                trace!("Searching parent chain for true left");
                while let Some(parent_index) = parent {
                    trace!("Walking: {:?}", parent_index);
                    match nodes.get(parent_index) {
                        None => Err(format!("Index assigned to node has no value in node list. {:?}", parent_index))?,
                        Some(node) => {
                            let n: &ParseNode = node;
                            true_left = Some(parent_index);
                            parent = n.parent
                        }
                    };

                    // safty net, max iterations to len of nodes
                    count += 1;
                    if count > nodes.len() {
                        return Err(format!("Max iterations reached when searching for last parent."));
                    }
                }

                let mut new_parent = None;
                let mut new_left = true_left;

                // update last_left parent
                match true_left {
                    None => (), // allowed, likely the begining of input, group or sub-expression
                    Some(left) => match nodes.get_mut(left) {
                        None => Err(format!("Index assigned to last left has no value in nodes. {:?}", left))?,
                        Some(node) => {
                            let n: &mut ParseNode = node;
                            let my_priority = match priority_map.get(&my_definition) {
                                None => Err(format!("Definition '{:?}' not registered in priority map.", my_definition))?,
                                Some(priority) => *priority,
                            };

                            let their_priority = match priority_map.get(&n.definition) {
                                None => Err(format!("Definition '{:?}' not registered in priority map.", n.definition))?,
                                Some(priority) => *priority,
                            };

                            // lower priority means lower in tree
                            // lowest priority gets parent set
                            if my_priority < their_priority {
                                new_parent = true_left;
                                new_left = node.right;
                                node.right = Some(i);

                                // set true left's right's parent to us
                                match new_left {
                                    None => (), // allowed, unary prefix
                                    Some(right_index) => match nodes.get_mut(right_index) {
                                        None => Err(format!("Index assigned to node has no value in node list. {:?}", right_index))?,
                                        Some(right_node) => {
                                            let rn: &mut ParseNode = right_node;
                                            rn.parent = Some(i);
                                        }
                                    },
                                }
                            } else {
                                node.parent = Some(i);
                            }
                        }
                    },
                }
                (my_definition, new_parent, new_left, assumed_right)
            }
            TokenType::Period => {
                trace!("Parsing Period token");

                let my_definition = Definition::Access;
                next_parent = Some(i);

                let mut true_left = last_left;
                let mut parent = last_left;

                let mut count = 0;

                // // go up tree until no parent
                trace!("Searching parent chain for true left");
                while let Some(parent_index) = parent {
                    trace!("Walking: {:?}", parent_index);
                    match nodes.get(parent_index) {
                        None => Err(format!("Index assigned to node has no value in node list. {:?}", parent_index))?,
                        Some(node) => {
                            let n: &ParseNode = node;
                            true_left = Some(parent_index);
                            parent = n.parent
                        }
                    };

                    // safty net, max iterations to len of nodes
                    count += 1;
                    if count > nodes.len() {
                        return Err(format!("Max iterations reached when searching for last parent."));
                    }
                }

                let mut new_parent = None;
                let mut new_left = true_left;

                // update last_left parent
                match true_left {
                    None => (), // allowed, likely the begining of input, group or sub-expression
                    Some(left) => match nodes.get_mut(left) {
                        None => Err(format!("Index assigned to last left has no value in nodes. {:?}", left))?,
                        Some(node) => {
                            let n: &mut ParseNode = node;
                            let my_priority = match priority_map.get(&my_definition) {
                                None => Err(format!("Definition '{:?}' not registered in priority map.", my_definition))?,
                                Some(priority) => *priority,
                            };

                            let their_priority = match priority_map.get(&n.definition) {
                                None => Err(format!("Definition '{:?}' not registered in priority map.", n.definition))?,
                                Some(priority) => *priority,
                            };

                            // lower priority means lower in tree
                            // lowest priority gets parent set
                            if my_priority < their_priority {
                                new_parent = true_left;
                                new_left = node.right;
                                node.right = Some(i);

                                // set true left's right's parent to us
                                match new_left {
                                    None => (), // allowed, unary prefix
                                    Some(right_index) => match nodes.get_mut(right_index) {
                                        None => Err(format!("Index assigned to node has no value in node list. {:?}", right_index))?,
                                        Some(right_node) => {
                                            let rn: &mut ParseNode = right_node;
                                            rn.parent = Some(i);
                                        }
                                    },
                                }
                            } else {
                                node.parent = Some(i);
                            }
                        }
                    },
                }
                (my_definition, new_parent, new_left, assumed_right)
            }
            TokenType::PlusSign => {
                trace!("Parsing PlusSign token");

                let my_definition = Definition::Addition;
                next_parent = Some(i);

                let mut true_left = last_left;
                let mut parent = last_left;

                let mut count = 0;

                // // go up tree until no parent
                trace!("Searching parent chain for true left");
                while let Some(parent_index) = parent {
                    trace!("Walking: {:?}", parent_index);
                    match nodes.get(parent_index) {
                        None => Err(format!("Index assigned to node has no value in node list. {:?}", parent_index))?,
                        Some(node) => {
                            let n: &ParseNode = node;
                            true_left = Some(parent_index);
                            parent = n.parent
                        }
                    };

                    // safty net, max iterations to len of nodes
                    count += 1;
                    if count > nodes.len() {
                        return Err(format!("Max iterations reached when searching for last parent."));
                    }
                }

                let mut new_parent = None;
                let mut new_left = true_left;

                // update last_left parent
                match true_left {
                    None => (), // allowed, likely the begining of input, group or sub-expression
                    Some(left) => match nodes.get_mut(left) {
                        None => Err(format!("Index assigned to last left has no value in nodes. {:?}", left))?,
                        Some(node) => {
                            let n: &mut ParseNode = node;
                            let my_priority = match priority_map.get(&my_definition) {
                                None => Err(format!("Definition '{:?}' not registered in priority map.", my_definition))?,
                                Some(priority) => *priority,
                            };

                            let their_priority = match priority_map.get(&n.definition) {
                                None => Err(format!("Definition '{:?}' not registered in priority map.", n.definition))?,
                                Some(priority) => *priority,
                            };

                            // lower priority means lower in tree
                            // lowest priority gets parent set
                            if my_priority < their_priority {
                                new_parent = true_left;
                                new_left = node.right;
                                node.right = Some(i);

                                // set true left's right's parent to us
                                match new_left {
                                    None => (), // allowed, unary prefix
                                    Some(right_index) => match nodes.get_mut(right_index) {
                                        None => Err(format!("Index assigned to node has no value in node list. {:?}", right_index))?,
                                        Some(right_node) => {
                                            let rn: &mut ParseNode = right_node;
                                            rn.parent = Some(i);
                                        }
                                    },
                                }
                            } else {
                                node.parent = Some(i);
                            }
                        }
                    },
                }
                (my_definition, new_parent, new_left, assumed_right)
            }
            TokenType::Equality => {
                trace!("Parsing Equality token");

                let my_definition = Definition::Equality;
                next_parent = Some(i);

                let mut true_left = last_left;
                let mut parent = last_left;

                let mut count = 0;

                // // go up tree until no parent
                trace!("Searching parent chain for true left");
                while let Some(parent_index) = parent {
                    trace!("Walking: {:?}", parent_index);
                    match nodes.get(parent_index) {
                        None => Err(format!("Index assigned to node has no value in node list. {:?}", parent_index))?,
                        Some(node) => {
                            let n: &ParseNode = node;
                            true_left = Some(parent_index);
                            parent = n.parent
                        }
                    };

                    // safty net, max iterations to len of nodes
                    count += 1;
                    if count > nodes.len() {
                        return Err(format!("Max iterations reached when searching for last parent."));
                    }
                }

                let mut new_parent = None;
                let mut new_left = true_left;

                // update last_left parent
                match true_left {
                    None => (), // allowed, likely the begining of input, group or sub-expression
                    Some(left) => match nodes.get_mut(left) {
                        None => Err(format!("Index assigned to last left has no value in nodes. {:?}", left))?,
                        Some(node) => {
                            let my_priority = match priority_map.get(&my_definition) {
                                None => Err(format!("Definition '{:?}' not registered in priority map.", my_definition))?,
                                Some(priority) => *priority,
                            };

                            let their_priority = match priority_map.get(&node.definition) {
                                None => Err(format!("Definition '{:?}' not registered in priority map.", node.definition))?,
                                Some(priority) => *priority,
                            };

                            // lower priority means lower in tree
                            // lowest priority gets parent set
                            if my_priority < their_priority {
                                new_parent = true_left;
                                new_left = node.right;
                                node.right = Some(i);

                                // set true left's right's parent to us
                                match new_left {
                                    None => (), // allowed, unary prefix
                                    Some(right_index) => match nodes.get_mut(right_index) {
                                        None => Err(format!("Index assigned to node has no value in node list. {:?}", right_index))?,
                                        Some(right_node) => {
                                            let rn: &mut ParseNode = right_node;
                                            rn.parent = Some(i);
                                        }
                                    },
                                }
                            } else {
                                node.parent = Some(i);
                            }
                        }
                    },
                }
                (my_definition, new_parent, new_left, assumed_right)
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

        match priority_map.get(&definition) {
            None => Err(format!("Definition '{:?}' not registered in priority map.", definition))?,
            Some(priority) => match priority_table.get_mut(*priority) {
                None => Err(format!("No priority table regisitered at level {:?}", priority))?,
                Some(table) => table.push(i),
            },
        }

        nodes.push(ParseNode::new(definition, parent, left, right, token.clone()));
        last_left = Some(i);
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

    #[test]
    fn single_number() {
        let tokens = vec![LexerToken::new("5".to_string(), TokenType::Number, 0, 0)];

        let result = parse(tokens).unwrap();

        assert_eq!(result.get_root(), 0);

        let root = result.get_node(0).unwrap();

        assert_eq!(root.get_definition(), Definition::Number);
        assert!(root.get_left().is_none());
        assert!(root.get_right().is_none());
    }

    #[test]
    fn single_identifier() {
        let tokens = vec![LexerToken::new("value".to_string(), TokenType::Identifier, 0, 0)];

        let result = parse(tokens).unwrap();

        assert_eq!(result.get_root(), 0);

        let root = result.get_node(0).unwrap();

        assert_eq!(root.get_definition(), Definition::Identifier);
        assert!(root.get_left().is_none());
        assert!(root.get_right().is_none());
    }

    #[test]
    fn addition() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_eq!(result.get_root(), 1);

        let left = result.get_node(0).unwrap();
        let root = result.get_node(1).unwrap();
        let right = result.get_node(2).unwrap();

        assert_eq!(root.get_definition(), Definition::Addition);
        assert_eq!(root.get_left().unwrap(), 0);
        assert_eq!(root.get_right().unwrap(), 2);

        assert_eq!(left.get_definition(), Definition::Number);
        assert!(left.get_left().is_none());
        assert!(left.get_right().is_none());

        assert_eq!(right.get_definition(), Definition::Number);
        assert!(right.get_left().is_none());
        assert!(right.get_right().is_none());
    }

    #[test]
    fn equality() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("==".to_string(), TokenType::Equality, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_eq!(result.get_root(), 1);

        let left = result.get_node(0).unwrap();
        let root = result.get_node(1).unwrap();
        let right = result.get_node(2).unwrap();

        assert_eq!(root.get_definition(), Definition::Equality);
        assert_eq!(root.get_left().unwrap(), 0);
        assert_eq!(root.get_right().unwrap(), 2);

        assert_eq!(left.get_definition(), Definition::Number);
        assert!(left.get_left().is_none());
        assert!(left.get_right().is_none());

        assert_eq!(right.get_definition(), Definition::Number);
        assert!(right.get_left().is_none());
        assert!(right.get_right().is_none());
    }

    #[test]
    fn pair() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("=".to_string(), TokenType::Pair, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_eq!(result.get_root(), 1);

        let left = result.get_node(0).unwrap();
        let root = result.get_node(1).unwrap();
        let right = result.get_node(2).unwrap();

        assert_eq!(root.get_definition(), Definition::Pair);
        assert_eq!(root.get_left().unwrap(), 0);
        assert_eq!(root.get_right().unwrap(), 2);

        assert_eq!(left.get_definition(), Definition::Number);
        assert!(left.get_left().is_none());
        assert!(left.get_right().is_none());

        assert_eq!(right.get_definition(), Definition::Number);
        assert!(right.get_left().is_none());
        assert!(right.get_right().is_none());
    }

    #[test]
    fn access() {
        let tokens = vec![
            LexerToken::new("value".to_string(), TokenType::Identifier, 0, 0),
            LexerToken::new(".".to_string(), TokenType::Period, 0, 0),
            LexerToken::new("property".to_string(), TokenType::Identifier, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_eq!(result.get_root(), 1);

        let left = result.get_node(0).unwrap();
        let root = result.get_node(1).unwrap();
        let right = result.get_node(2).unwrap();

        assert_eq!(root.get_definition(), Definition::Access);
        assert_eq!(root.get_left().unwrap(), 0);
        assert_eq!(root.get_right().unwrap(), 2);

        assert_eq!(left.get_definition(), Definition::Identifier);
        assert!(left.get_left().is_none());
        assert!(left.get_right().is_none());

        assert_eq!(right.get_definition(), Definition::Identifier);
        assert!(right.get_left().is_none());
        assert!(right.get_right().is_none());
    }

    #[test]
    fn partial_addition_no_left() {
        let tokens = vec![
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_eq!(result.get_root(), 0);

        let root = result.get_node(0).unwrap();
        let right = result.get_node(1).unwrap();

        assert_eq!(root.get_definition(), Definition::Addition);
        assert!(root.get_left().is_none());
        assert_eq!(root.get_right().unwrap(), 1);

        assert_eq!(right.get_definition(), Definition::Number);
        assert!(right.get_left().is_none());
        assert!(right.get_right().is_none());
    }

    #[test]
    fn partial_addition_no_right() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("+".to_string(), TokenType::PlusSign, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_eq!(result.get_root(), 1);

        let left = result.get_node(0).unwrap();
        let root = result.get_node(1).unwrap();

        assert_eq!(root.get_definition(), Definition::Addition);
        assert_eq!(root.get_left().unwrap(), 0);
        assert!(root.get_right().is_none());

        assert_eq!(left.get_definition(), Definition::Number);
        assert!(left.get_left().is_none());
        assert!(left.get_right().is_none());
    }

    #[test]
    fn absolute_value() {
        let tokens = vec![
            LexerToken::new("++".to_string(), TokenType::AbsoluteValue, 0, 0),
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_eq!(result.get_root(), 0);

        let root = result.get_node(0).unwrap();
        let right = result.get_node(1).unwrap();

        assert_eq!(root.get_definition(), Definition::AbsoluteValue);
        assert_eq!(root.get_right().unwrap(), 1);
        assert!(root.get_left().is_none());

        assert_eq!(right.get_definition(), Definition::Number);
        assert_eq!(right.get_parent().unwrap(), 0);
        assert!(right.get_left().is_none());
        assert!(right.get_right().is_none());
    }

    #[test]
    fn empty_apply() {
        let tokens = vec![
            LexerToken::new("value".to_string(), TokenType::Identifier, 0, 0),
            LexerToken::new("~~".to_string(), TokenType::EmptyApply, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_eq!(result.get_root(), 1);

        let left = result.get_node(0).unwrap();
        let root = result.get_node(1).unwrap();

        assert_eq!(root.get_definition(), Definition::EmptyApply);
        assert_eq!(root.get_left().unwrap(), 0);
        assert!(root.get_right().is_none());

        assert_eq!(left.get_definition(), Definition::Identifier);
        assert_eq!(left.get_parent().unwrap(), 1);
        assert!(left.get_left().is_none());
        assert!(left.get_right().is_none());
    }

    #[test]
    fn absolute_value_reversed() {
        let tokens = vec![
            LexerToken::new("5".to_string(), TokenType::Number, 0, 0),
            LexerToken::new("++".to_string(), TokenType::AbsoluteValue, 0, 0),
        ];

        let result = parse(tokens).unwrap();

        assert_eq!(result.get_root(), 0);

        let left = result.get_node(0).unwrap();
        let root = result.get_node(1).unwrap();

        assert_eq!(root.get_definition(), Definition::AbsoluteValue);
        assert!(root.get_left().is_none());
        assert!(root.get_right().is_none());

        assert_eq!(left.get_definition(), Definition::Number);
        assert!(left.get_parent().is_none());
        assert!(left.get_left().is_none());
        assert!(left.get_right().is_none());
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

        assert_eq!(result.get_root(), 2);

        let root = result.get_node(2).unwrap();

        let abs_sign = result.get_node(0).unwrap();

        let first_five = result.get_node(1).unwrap();
        let second_five = result.get_node(3).unwrap();

        assert_eq!(root.get_definition(), Definition::Addition);
        assert_eq!(root.get_left().unwrap(), 0);
        assert_eq!(root.get_right().unwrap(), 3);

        assert_eq!(abs_sign.get_definition(), Definition::AbsoluteValue);
        assert_eq!(abs_sign.get_parent().unwrap(), 2);
        assert!(abs_sign.get_left().is_none());
        assert_eq!(abs_sign.get_right().unwrap(), 1);

        assert_eq!(first_five.get_definition(), Definition::Number);
        assert_eq!(first_five.get_parent().unwrap(), 0);
        assert!(first_five.get_left().is_none());
        assert!(first_five.get_right().is_none());

        assert_eq!(second_five.get_definition(), Definition::Number);
        assert_eq!(second_five.get_parent().unwrap(), 2);
        assert!(second_five.get_left().is_none());
        assert!(second_five.get_right().is_none());
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

        assert_eq!(result.get_root(), 1);

        let root = result.get_node(1).unwrap();

        let abs_sign = result.get_node(2).unwrap();

        let first_five = result.get_node(0).unwrap();
        let second_five = result.get_node(3).unwrap();

        assert_eq!(root.get_definition(), Definition::Addition);
        assert_eq!(root.get_left().unwrap(), 0);
        assert_eq!(root.get_right().unwrap(), 2);

        assert_eq!(abs_sign.get_definition(), Definition::AbsoluteValue);
        assert_eq!(abs_sign.get_parent().unwrap(), 1);
        assert!(abs_sign.get_left().is_none());
        assert_eq!(abs_sign.get_right().unwrap(), 3);

        assert_eq!(first_five.get_definition(), Definition::Number);
        assert_eq!(first_five.get_parent().unwrap(), 1);
        assert!(first_five.get_left().is_none());
        assert!(first_five.get_right().is_none());

        assert_eq!(second_five.get_definition(), Definition::Number);
        assert_eq!(second_five.get_parent().unwrap(), 2);
        assert!(second_five.get_left().is_none());
        assert!(second_five.get_right().is_none());
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

        assert_eq!(result.get_root(), 0);

        let first_abs = result.get_node(0).unwrap();
        let second_abs = result.get_node(1).unwrap();
        let third_abs = result.get_node(2).unwrap();

        let five = result.get_node(3).unwrap();

        assert_eq!(first_abs.get_definition(), Definition::AbsoluteValue);
        assert!(first_abs.get_parent().is_none());
        assert!(first_abs.get_left().is_none());
        assert_eq!(first_abs.get_right().unwrap(), 1);

        assert_eq!(second_abs.get_definition(), Definition::AbsoluteValue);
        assert_eq!(second_abs.get_parent().unwrap(), 0);
        assert!(second_abs.get_left().is_none());
        assert_eq!(second_abs.get_right().unwrap(), 2);

        assert_eq!(third_abs.get_definition(), Definition::AbsoluteValue);
        assert_eq!(third_abs.get_parent().unwrap(), 1);
        assert!(third_abs.get_left().is_none());
        assert_eq!(third_abs.get_right().unwrap(), 3);

        assert_eq!(five.get_definition(), Definition::Number);
        assert_eq!(five.get_parent().unwrap(), 2);
        assert!(five.get_left().is_none());
        assert!(five.get_right().is_none());
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

        assert_eq!(result.get_root(), 0);

        let first_abs = result.get_node(0).unwrap();
        let second_abs = result.get_node(1).unwrap();
        let third_abs = result.get_node(2).unwrap();

        let value = result.get_node(3).unwrap();

        let first_apply = result.get_node(4).unwrap();
        let second_apply = result.get_node(5).unwrap();
        let third_apply = result.get_node(6).unwrap();

        assert_eq!(first_abs.get_definition(), Definition::AbsoluteValue);
        assert!(first_abs.get_parent().is_none());
        assert!(first_abs.get_left().is_none());
        assert_eq!(first_abs.get_right().unwrap(), 1);

        assert_eq!(second_abs.get_definition(), Definition::AbsoluteValue);
        assert_eq!(second_abs.get_parent().unwrap(), 0);
        assert!(second_abs.get_left().is_none());
        assert_eq!(second_abs.get_right().unwrap(), 2);

        assert_eq!(third_abs.get_definition(), Definition::AbsoluteValue);
        assert_eq!(third_abs.get_parent().unwrap(), 1);
        assert!(third_abs.get_left().is_none());
        assert_eq!(third_abs.get_right().unwrap(), 6);

        assert_eq!(value.get_definition(), Definition::Number);
        assert_eq!(value.get_parent().unwrap(), 4);
        assert!(value.get_left().is_none());
        assert!(value.get_right().is_none());

        assert_eq!(first_apply.get_definition(), Definition::EmptyApply);
        assert_eq!(first_apply.get_parent().unwrap(), 5);
        assert_eq!(first_apply.get_left().unwrap(), 3);
        assert!(first_apply.get_right().is_none());

        assert_eq!(second_apply.get_definition(), Definition::EmptyApply);
        assert_eq!(second_apply.get_parent().unwrap(), 6);
        assert_eq!(second_apply.get_left().unwrap(), 4);
        assert!(second_apply.get_right().is_none());

        assert_eq!(third_apply.get_definition(), Definition::EmptyApply);
        assert_eq!(third_apply.get_parent().unwrap(), 2);
        assert_eq!(third_apply.get_left().unwrap(), 5);
        assert!(third_apply.get_right().is_none());
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

        assert_eq!(result.get_root(), 5);

        let first_plus = result.get_node(1).unwrap();
        let second_plus = result.get_node(3).unwrap();
        let third_plus = result.get_node(5).unwrap();

        let first_five = result.get_node(0).unwrap();
        let second_five = result.get_node(2).unwrap();
        let third_five = result.get_node(4).unwrap();
        let fourth_five = result.get_node(6).unwrap();

        assert_eq!(first_five.get_definition(), Definition::Number);
        assert_eq!(first_five.get_parent().unwrap(), 1);
        assert!(first_five.get_left().is_none());
        assert!(first_five.get_right().is_none());

        assert_eq!(second_five.get_definition(), Definition::Number);
        assert_eq!(second_five.get_parent().unwrap(), 1);
        assert!(second_five.get_left().is_none());
        assert!(second_five.get_right().is_none());

        assert_eq!(third_five.get_definition(), Definition::Number);
        assert_eq!(third_five.get_parent().unwrap(), 3);
        assert!(third_five.get_left().is_none());
        assert!(third_five.get_right().is_none());

        assert_eq!(fourth_five.get_definition(), Definition::Number);
        assert_eq!(fourth_five.get_parent().unwrap(), 5);
        assert!(fourth_five.get_left().is_none());
        assert!(fourth_five.get_right().is_none());

        assert_eq!(first_plus.get_definition(), Definition::Addition);
        assert_eq!(first_plus.get_parent().unwrap(), 3);
        assert_eq!(first_plus.get_left().unwrap(), 0);
        assert_eq!(first_plus.get_right().unwrap(), 2);

        assert_eq!(second_plus.get_definition(), Definition::Addition);
        assert_eq!(second_plus.get_parent().unwrap(), 5);
        assert_eq!(second_plus.get_left().unwrap(), 1);
        assert_eq!(second_plus.get_right().unwrap(), 4);

        assert_eq!(third_plus.get_definition(), Definition::Addition);
        assert!(third_plus.get_parent().is_none());
        assert_eq!(third_plus.get_left().unwrap(), 3);
        assert_eq!(third_plus.get_right().unwrap(), 6);
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

        assert_eq!(result.get_root(), 1);

        let first_plus = result.get_node(3).unwrap();

        let first_equality = result.get_node(1).unwrap();

        let first_five = result.get_node(0).unwrap();
        let second_five = result.get_node(2).unwrap();
        let third_five = result.get_node(4).unwrap();

        assert_eq!(first_five.get_definition(), Definition::Number);
        assert_eq!(first_five.get_parent().unwrap(), 1);
        assert!(first_five.get_left().is_none());
        assert!(first_five.get_right().is_none());

        assert_eq!(second_five.get_definition(), Definition::Number);
        assert_eq!(second_five.get_parent().unwrap(), 3);
        assert!(second_five.get_left().is_none());
        assert!(second_five.get_right().is_none());

        assert_eq!(third_five.get_definition(), Definition::Number);
        assert_eq!(third_five.get_parent().unwrap(), 3);
        assert!(third_five.get_left().is_none());
        assert!(third_five.get_right().is_none());

        assert_eq!(first_plus.get_definition(), Definition::Addition);
        assert_eq!(first_plus.get_parent().unwrap(), 1);
        assert_eq!(first_plus.get_left().unwrap(), 2);
        assert_eq!(first_plus.get_right().unwrap(), 4);

        assert_eq!(first_equality.get_definition(), Definition::Equality);
        assert!(first_equality.get_parent().is_none());
        assert_eq!(first_equality.get_left().unwrap(), 0);
        assert_eq!(first_equality.get_right().unwrap(), 3);
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

        assert_eq!(result.get_root(), 13);

        let first_plus = result.get_node(3).unwrap();
        let second_plus = result.get_node(7).unwrap();
        let third_plus = result.get_node(11).unwrap();

        let first_equality = result.get_node(1).unwrap();
        let second_equality = result.get_node(5).unwrap();
        let third_equality = result.get_node(9).unwrap();
        let fourth_equality = result.get_node(13).unwrap();

        let first_five = result.get_node(0).unwrap();
        let second_five = result.get_node(2).unwrap();
        let third_five = result.get_node(4).unwrap();
        let fourth_five = result.get_node(6).unwrap();
        let fifth_five = result.get_node(8).unwrap();
        let sixth_five = result.get_node(10).unwrap();
        let seventh_five = result.get_node(12).unwrap();
        let eigth_five = result.get_node(14).unwrap();

        assert_eq!(first_five.get_definition(), Definition::Number);
        assert_eq!(first_five.get_parent().unwrap(), 1);
        assert!(first_five.get_left().is_none());
        assert!(first_five.get_right().is_none());

        assert_eq!(second_five.get_definition(), Definition::Number);
        assert_eq!(second_five.get_parent().unwrap(), 3);
        assert!(second_five.get_left().is_none());
        assert!(second_five.get_right().is_none());

        assert_eq!(third_five.get_definition(), Definition::Number);
        assert_eq!(third_five.get_parent().unwrap(), 3);
        assert!(third_five.get_left().is_none());
        assert!(third_five.get_right().is_none());

        assert_eq!(fourth_five.get_definition(), Definition::Number);
        assert_eq!(fourth_five.get_parent().unwrap(), 7);
        assert!(fourth_five.get_left().is_none());
        assert!(fourth_five.get_right().is_none());

        assert_eq!(fifth_five.get_definition(), Definition::Number);
        assert_eq!(fifth_five.get_parent().unwrap(), 7);
        assert!(fifth_five.get_left().is_none());
        assert!(fifth_five.get_right().is_none());

        assert_eq!(sixth_five.get_definition(), Definition::Number);
        assert_eq!(sixth_five.get_parent().unwrap(), 11);
        assert!(sixth_five.get_left().is_none());
        assert!(sixth_five.get_right().is_none());

        assert_eq!(seventh_five.get_definition(), Definition::Number);
        assert_eq!(seventh_five.get_parent().unwrap(), 11);
        assert!(seventh_five.get_left().is_none());
        assert!(seventh_five.get_right().is_none());

        assert_eq!(eigth_five.get_definition(), Definition::Number);
        assert_eq!(eigth_five.get_parent().unwrap(), 13);
        assert!(eigth_five.get_left().is_none());
        assert!(eigth_five.get_right().is_none());

        // Additions

        assert_eq!(first_plus.get_definition(), Definition::Addition);
        assert_eq!(first_plus.get_parent().unwrap(), 1);
        assert_eq!(first_plus.get_left().unwrap(), 2);
        assert_eq!(first_plus.get_right().unwrap(), 4);

        assert_eq!(second_plus.get_definition(), Definition::Addition);
        assert_eq!(second_plus.get_parent().unwrap(), 5);
        assert_eq!(second_plus.get_left().unwrap(), 6);
        assert_eq!(second_plus.get_right().unwrap(), 8);

        assert_eq!(third_plus.get_definition(), Definition::Addition);
        assert_eq!(third_plus.get_parent().unwrap(), 9);
        assert_eq!(third_plus.get_left().unwrap(), 10);
        assert_eq!(third_plus.get_right().unwrap(), 12);

        // Equalities

        assert_eq!(first_equality.get_definition(), Definition::Equality);
        assert_eq!(first_equality.get_parent().unwrap(), 5);
        assert_eq!(first_equality.get_left().unwrap(), 0);
        assert_eq!(first_equality.get_right().unwrap(), 3);

        assert_eq!(second_equality.get_definition(), Definition::Equality);
        assert_eq!(second_equality.get_parent().unwrap(), 9);
        assert_eq!(second_equality.get_left().unwrap(), 1);
        assert_eq!(second_equality.get_right().unwrap(), 7);

        assert_eq!(third_equality.get_definition(), Definition::Equality);
        assert_eq!(third_equality.get_parent().unwrap(), 13);
        assert_eq!(third_equality.get_left().unwrap(), 5);
        assert_eq!(third_equality.get_right().unwrap(), 11);

        assert_eq!(fourth_equality.get_definition(), Definition::Equality);
        assert!(fourth_equality.get_parent().is_none());
        assert_eq!(fourth_equality.get_left().unwrap(), 9);
        assert_eq!(fourth_equality.get_right().unwrap(), 14);
    }
}
