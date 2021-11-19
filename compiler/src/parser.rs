use log::trace;
use std::{collections::HashMap, hash::Hash};

use crate::lexer::*;

#[derive(Debug, PartialOrd, Eq, PartialEq, Clone, Copy, Hash)]
pub enum Definition {
    Addition,
    Number,
    AbsoluteValue,
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
    map.insert(Definition::AbsoluteValue, 5);
    map.insert(Definition::Addition, 9);

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
                let parent = match last_left {
                    None => assumed_right,
                    Some(left) => Some(left),
                };

                (Definition::Number, parent, None, None)
            }
            TokenType::AbsoluteValue => (Definition::AbsoluteValue, None, None, assumed_right),
            TokenType::PlusSign => (Definition::Addition, None, last_left, assumed_right),
            t => Err(format!("Definition from token type {:?} not defined.", t))?,
        };

        trace!(
            "Defined as {:?} with relation ships parent {:?} left {:?} right {:?}",
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
        if count >= nodes.len() {
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
        let left = result.get_node(1).unwrap();

        assert_eq!(root.get_definition(), Definition::AbsoluteValue);
        assert_eq!(root.get_right().unwrap(), 1);
        assert!(root.get_left().is_none());

        assert_eq!(left.get_definition(), Definition::Number);
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

        assert_eq!(result.get_root(), 1);

        let right = result.get_node(0).unwrap();
        let root = result.get_node(1).unwrap();

        assert_eq!(root.get_definition(), Definition::AbsoluteValue);
        assert!(root.get_left().is_none());
        assert!(root.get_right().is_none());

        assert_eq!(right.get_definition(), Definition::Number);
        assert!(right.get_left().is_none());
        assert!(right.get_right().is_none());
    }
}
