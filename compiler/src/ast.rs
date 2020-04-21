use crate::{ParseResult, Node, Token, TokenType, Classification};
use garnish_lang_common::Result;

#[derive(Debug)]
pub struct AST {
    pub(crate) nodes: Vec<Node>,
    pub(crate) root: usize,
    pub(crate) sub_roots: Vec<usize>
}

impl AST {
    pub fn new() -> Self {
        AST {
            nodes: vec![],
            root: 0,
            sub_roots: vec![],
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum OpType {
    Binary,
    UnaryLeft,
    UnaryRight,
}

pub fn make_ast(mut parse_result: ParseResult) -> Result<AST> {
    if parse_result.nodes.is_empty() {
        return Ok(AST {
            nodes: vec![],
            root: 0,
            sub_roots: vec![]
        });
    }

    let mut op_locations: Vec<(OpType, Vec<usize>)> = vec![
        (OpType::UnaryLeft, vec![]), // 0 - Symbol
        (OpType::Binary, vec![]), // 1 - Decimal
        (OpType::Binary, vec![]), // 2 - Access
        (OpType::UnaryLeft, vec![]), // 3 - Unary
        (OpType::UnaryRight, vec![]), // 4 - Suffix Apply
        (OpType::Binary, vec![]), // 5 - TypeCast
        (OpType::Binary, vec![]), // 6 - Exponential
        (OpType::Binary, vec![]), // 7 - Multiply, Division, Modulo
        (OpType::Binary, vec![]), // 8 - Addition, Subtraction
        (OpType::Binary, vec![]), // 9 - Bit Shift
        (OpType::Binary, vec![]), // 10 - Range
        (OpType::Binary, vec![]), // 11 - Relational
        (OpType::Binary, vec![]), // 12 - Equality
        (OpType::Binary, vec![]), // 13 - Bitwise And
        (OpType::Binary, vec![]), // 14 - Bitwise Xor
        (OpType::Binary, vec![]), // 15 - Bitwise Or
        (OpType::Binary, vec![]), // 16 - Logical And
        (OpType::Binary, vec![]), // 17 - Logical Xor
        (OpType::Binary, vec![]), // 18 - Logical Or
        (OpType::Binary, vec![]), // 19 - Link
        (OpType::Binary, vec![]), // 20 - Pair
        (OpType::Binary, vec![]), // 21 - List
        (OpType::Binary, vec![]), // 22 - Partially Apply
        (OpType::Binary, vec![]), // 23 - Infix Apply
        (OpType::Binary, vec![]), // 24 - Conditional
        (OpType::UnaryLeft, vec![]), // 25 - DefaultInvoke
        (OpType::Binary, vec![]), // 26 - Conditional Continuation
        (OpType::Binary, vec![]), // 27 - Functional
        (OpType::Binary, vec![]), // 28 - Iteration
        (OpType::Binary, vec![]), // 29 - Output Result
    ];

    // maintain separate list for groupings
    let mut groups: Vec<usize> = vec![];

    // maintain separate list of literals for checking for orphans
    let mut literals: Vec<usize> = vec![];
    let mut next_parent: Option<usize> = None;

    //for (i, n) in parse_result.nodes.iter().enumerate() {
        //println!("{}, {:?}", i, n);
    //}
    //println!("------");

    for i in 0..parse_result.nodes.len() {
        match next_parent {
            Some(p) if parse_result.nodes[i].classification != Classification::NoOp => {
                parse_result.nodes[i].parent = next_parent;
                parse_result.nodes[i].left = None;
                parse_result.nodes[p].right = Some(i);
                next_parent = None;
            }
            _ => ()
        }

        let node = &mut parse_result.nodes[i];

        if node.token.token_type == TokenType::StartGroup 
            || node.token.token_type == TokenType::StartExpression {
            next_parent = Some(i);
        }

        let p = match node.classification {
            Classification::IterationOutput 
            | Classification::IterationSkip 
            | Classification::IterationContinue
            | Classification::IterationComplete => {
                literals.push(i);
                continue;
            }
            Classification::NoOp => continue,
            Classification::Literal => {
                if node.token.token_type == TokenType::StartGroup 
                    || node.token.token_type == TokenType::StartExpression {
                    groups.push(i);
                } else {
                    literals.push(i);
                }
                continue;
            }
            Classification::Symbol => 0,
            Classification::Decimal => 1,
            Classification::Access => 2,
            Classification::Negation
            | Classification::AbsoluteValue 
            | Classification::LogicalNot 
            | Classification::BitwiseNot
            | Classification::PrefixApply => 3,
            Classification::SuffixApply => 4,
            Classification::TypeCast => 5,
            Classification::Exponential => 6,
            Classification::Multiplication
            | Classification::Division
            | Classification::IntegerDivision
            | Classification::Modulo => 7,
            Classification::Addition
            | Classification::Subtraction => 8,
            Classification::BitwiseLeftShift
            | Classification::BitwiseRightShift => 9,
            Classification::MakeRange
            | Classification::MakeStartExclusiveRange
            | Classification::MakeEndExclusiveRange
            | Classification::MakeExclusiveRange => 10,
            Classification::LessThan
            | Classification::LessThanOrEqual
            | Classification::GreaterThan
            | Classification::GreaterThanOrEqual => 11,
            Classification::Equality
            | Classification::Inequality
            | Classification::TypeEqual => 12,
            Classification::BitwiseAnd => 13,
            Classification::BitwiseXor => 14,
            Classification::BitwiseOr => 15,
            Classification::LogicalAnd => 16,
            Classification::LogicalXor => 17,
            Classification::LogicalOr => 18,
            Classification::MakeLink => 19,
            Classification::MakePair => 20,
            Classification::ListSeparator => 21,
            Classification::PartiallyApply => 22,
            Classification::InfixApply => 23,
            Classification::InvokeIfTrue
            | Classification::InvokeIfFalse
            | Classification::ResultCheckInvoke => 24,
            Classification::DefaultInvoke => 25,
            Classification::ConditionalContinuation => 26,
            Classification::Apply
            | Classification::PipeApply => 27,
            Classification::Iterate
            | Classification::IterateToSingleValue 
            | Classification::ReverseIterate
            | Classification::ReverseIterateToSingleValue 
            | Classification::MultiIterate => 28,
            Classification::OutputResult => 29,
            _ => unimplemented!("{:?}", node.classification)
        };

        op_locations[p].1.push(i);
    }

    // pair precedence has right to left associativity
    // so we're going to reverse that list before creating AST
    op_locations[20].1 = op_locations[20].1.iter().rev().map(|u| *u).collect();

    // for testing only
    //for (i, n) in parse_result.nodes.iter().enumerate() {
        //println!("{}, {:?}", i, n);
    //}
    //println!("------");

    for precedence in op_locations.iter() {
        for loc in precedence.1.iter() {
            // get op's left and right
            // update parent to be loc
            // if value set left and right to None

            let (left, right) = parse_result.nodes.get(*loc).map(|n| (n.left, n.right)).unwrap();

            if precedence.0 != OpType::UnaryLeft {
                match left {
                    Some(i) => {
                        match parse_result.nodes[i].parent {
                            Some(p) if parse_result.nodes[p].token.token_type == TokenType::StartGroup 
                                        || parse_result.nodes[p].token.token_type == TokenType::StartExpression 
                                    => {
                                parse_result.nodes[i].parent = Some(*loc);
                                parse_result.nodes[*loc].parent = Some(p);
                                parse_result.nodes[p].right = Some(*loc);
                            }
                            Some(p) => {
                                // go up to nodes root and update its right to be this node
                                let mut p_root = p;
                                while true {
                                    match parse_result.nodes[p_root].parent  {
                                        Some(p) if parse_result.nodes[p_root].token.token_type != TokenType::StartGroup 
                                            && parse_result.nodes[p_root].token.token_type != TokenType::StartExpression 
                                        => p_root = p,
                                        _ => {
                                            parse_result.nodes[p].parent = Some(*loc);    
                                            parse_result.nodes[*loc].left = Some(p);
                                            break;
                                        }
                                    }
                                }
                            }
                            _ => {
                                parse_result.nodes[i].parent = Some(*loc);
                            }
                        }

                        if parse_result.nodes[i].token.token_type != TokenType::StartExpression  
                            && parse_result.nodes[i].token.token_type != TokenType::StartGroup {
                            match parse_result.nodes[i].left {
                                Some(l) => {
                                    if parse_result.nodes[l].parent != Some(i) {
                                        parse_result.nodes[l].right = Some(*loc);
                                    }
                                }
                                None => ()
                            }

                            if parse_result.nodes[i].classification == Classification::Literal {
                                parse_result.nodes[i].left = None;
                                parse_result.nodes[i].right = None;
                            }
                        } else {
                            parse_result.nodes[i].left = None;
                        }
                    }
                    None => () // nothing to do
                };
            } else {
                // unary left means operand is on right side
                // set left side to None
                parse_result.nodes[*loc].left = None;
            }

            if precedence.0 != OpType::UnaryRight {
                match right {
                    Some(i) => {
                        match parse_result.nodes[i].parent {
                            Some(p) => {
                                // go up to nodes root and update its left to be this node
                                let mut p_root = p;
                                while true {
                                    match parse_result.nodes[p_root].parent {
                                        Some(p) => p_root = p,
                                        None => {
                                            parse_result.nodes[*loc].right = Some(p);    
                                            parse_result.nodes[p].parent = Some(*loc);
                                            break;
                                        }
                                    }
                                }
                            }
                            None => {
                                parse_result.nodes[i].parent = Some(*loc);
                            }
                        }
                        
                        if parse_result.nodes[i].token.token_type != TokenType::StartExpression 
                            && parse_result.nodes[i].token.token_type != TokenType::StartGroup {
                            match parse_result.nodes[i].right {
                                Some(r) => {
                                    if parse_result.nodes[r].parent != Some(i) {
                                        parse_result.nodes[r].left = Some(*loc);
                                    }
                                }
                                None => (),
                            }

                            if parse_result.nodes[i].classification == Classification::Literal {
                                parse_result.nodes[i].left = None;
                                parse_result.nodes[i].right = None;
                            }
                        } else {
                            parse_result.nodes[i].left = None;
                        }
                    }
                    None => () // nothing to do
                }
            } else {
                // unary right means operand is on left
                // set right side to None
                parse_result.nodes[*loc].right = None;
            }
                
            // for testing only
            //for (i, n) in parse_result.nodes.iter().enumerate() {
                //println!("{}, {:?}", i, n);
            //}
            //println!("------");
        }
    }

    for i in literals {
        // set left and right to None
        // might not be completly necessary but keeps the tree clean
        if parse_result.nodes[i].token.token_type != TokenType::StartGroup
            && parse_result.nodes[i].token.token_type != TokenType::StartExpression {
            parse_result.nodes[i].left = None;
            parse_result.nodes[i].right = None;
        }
    }

    // group sub trees will have been resolved
    // final touch is to assign the parent to the sub tree root
    // a node will already be assigned to the start group's right
    // crawl up until we find the root
    for i in groups {
        match parse_result.nodes[i].right {
            Some(r) => match parse_result.nodes[r].parent {
                Some(p) if p != i => {
                    let mut p_root = p;
                    while true {
                        match parse_result.nodes[p_root].parent {
                            Some(p) => p_root = p,
                            None => {
                                parse_result.nodes[p_root].parent = Some(i);
                                parse_result.nodes[i].right = Some(p_root);
                                break;
                            }
                        }
                    }
                }
                Some(_) => (), // right's parent is already assigned to the group
                None => parse_result.nodes[r].parent = Some(i),
            }
            None => unimplemented!("no right assign to group node {}", i),
        }
                
        // for testing only
        //for (i, n) in parse_result.nodes.iter().enumerate() {
            //println!("{}, {:?}", i, n);
        //}
        //println!("------");
    }

    let mut root_index = *parse_result.sub_expressions.get(0).unwrap(); // should always have 1
    let mut node = &parse_result.nodes[root_index];

    let mut count = 0;
    while node.parent.is_some() {
        root_index = node.parent.unwrap();
        node = &parse_result.nodes[root_index];

        count += 1;
        if count > parse_result.nodes.len() {
            break;
        }
    }

    return Ok(AST {
        nodes: parse_result.nodes.clone(),
        root: root_index,
        sub_roots: vec![]
    });
}

#[cfg(test)]
mod tests {
    use crate::{make_ast, AST, Lexer, TokenType, Token, Node, Parser, Classification};

    pub fn ast_from(s: &str) -> AST {
        let input = Lexer::new().lex(s).unwrap();
        let parser = Parser::new();
        let parse_result = parser.make_groups(&input).unwrap();
        
        return make_ast(parse_result).unwrap();
    }

    pub trait AssertNode {
        fn assert_node(&self, index: usize, parent: Option<usize>, left: Option<usize>, right: Option<usize>);
    }

    impl AssertNode for Vec<Node> {
        fn assert_node(&self, index: usize, parent: Option<usize>, left: Option<usize>, right: Option<usize>) {
            let node = self.get(index).unwrap();
            assert_eq!(node.parent, parent, "Expected parent node of {:?} to be {:?}, was {:?}", index, parent, node.parent);
            assert_eq!(node.left, left, "Expected left node of {:?} to be {:?}, was {:?}", index, left, node.left);
            assert_eq!(node.right, right, "Expected right node of {:?} to be {:?}, was {:?}", index, right, node.right);
        }
    }

    pub fn assert_binary_op(input: &str) {
        let ast = ast_from(input);

        ast.nodes.assert_node(0, Some(2), None, None);
        ast.nodes.assert_node(2, None, Some(0), Some(4));
        ast.nodes.assert_node(4, Some(2), None, None);

        assert_eq!(ast.root, 2);
    }

    pub fn assert_multi_op_least_first(input: &str) {
        let ast = ast_from(input);

        ast.nodes.assert_node(0, Some(2), None, None);
        ast.nodes.assert_node(2, Some(6), Some(0), Some(4));
        ast.nodes.assert_node(4, Some(2), None, None);
        ast.nodes.assert_node(6, None, Some(2), Some(8));
        ast.nodes.assert_node(8, Some(6), None, None);

        assert_eq!(ast.root, 6);
    }

    #[test]
    fn create_empty() {
        let ast = ast_from("");

        assert_eq!(ast.nodes, vec![]);
        assert_eq!(ast.root, 0);
        assert_eq!(ast.sub_roots, vec![]);
    }
}

// Precedence testing strategy
// Each precedence level will get its own module for tests
// each precedence level will be tested by first proving the indivudual operators are put in the AST
// and then tested along side operators from the previous precedence to ensure they are ordered correctly

#[cfg(test)]
mod value_precedence_tests {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::ast_from;
    
    #[test]
    fn number_only() {
        let ast = ast_from("10");

        assert_eq!(ast.nodes, vec![Node {
            classification: Classification::Literal,
            token: Token {
                value: String::from("10"),
                token_type: TokenType::Number,
            },
            left: None,
            right: None,
            parent: None
        }]);
        assert_eq!(ast.root, 0);
        assert_eq!(ast.sub_roots, vec![]);
    }

    #[test]
    fn character_only() {
        let ast = ast_from("'a'");

        assert_eq!(ast.nodes, vec![Node {
            classification: Classification::Literal,
            token: Token {
                value: String::from("a"),
                token_type: TokenType::Character,
            },
            left: None,
            right: None,
            parent: None
        }]);
        assert_eq!(ast.root, 0);
        assert_eq!(ast.sub_roots, vec![]);
    }

    #[test]
    fn character_list_only() {
        let ast = ast_from("\"hello world\"");

        assert_eq!(ast.nodes, vec![Node {
            classification: Classification::Literal,
            token: Token {
                value: String::from("hello world"),
                token_type: TokenType::CharacterList,
            },
            left: None,
            right: None,
            parent: None
        }]);
        assert_eq!(ast.root, 0);
        assert_eq!(ast.sub_roots, vec![]);
    }

    #[test]
    fn identifier_only() {
        let ast = ast_from("my_value");

        assert_eq!(ast.nodes, vec![Node {
            classification: Classification::Literal,
            token: Token {
                value: String::from("my_value"),
                token_type: TokenType::Identifier,
            },
            left: None,
            right: None,
            parent: None
        }]);
        assert_eq!(ast.root, 0);
        assert_eq!(ast.sub_roots, vec![]);
    }

    #[test]
    fn symbol_only() {
        let ast = ast_from(":");

        assert_eq!(ast.nodes, vec![Node {
            classification: Classification::Symbol,
            token: Token {
                value: String::from(":"),
                token_type: TokenType::SymbolOperator,
            },
            left: None,
            right: None,
            parent: None
        }]);
        assert_eq!(ast.root, 0);
        assert_eq!(ast.sub_roots, vec![]);
    }

    #[test]
    fn unit_only() {
        let ast = ast_from("()");

        assert_eq!(ast.nodes, vec![Node {
            classification: Classification::Literal,
            token: Token {
                value: String::from("()"),
                token_type: TokenType::UnitLiteral,
            },
            left: None,
            right: None,
            parent: None
        }]);
        assert_eq!(ast.root, 0);
        assert_eq!(ast.sub_roots, vec![]);
    }

    #[test]
    fn input_only() {
        let ast = ast_from("$");

        assert_eq!(ast.nodes, vec![Node {
            classification: Classification::Literal,
            token: Token {
                value: String::from("$"),
                token_type: TokenType::Input,
            },
            left: None,
            right: None,
            parent: None
        }]);
        assert_eq!(ast.root, 0);
        assert_eq!(ast.sub_roots, vec![]);
    }

    #[test]
    fn result_only() {
        let ast = ast_from("?");

        assert_eq!(ast.nodes, vec![Node {
            classification: Classification::Literal,
            token: Token {
                value: String::from("?"),
                token_type: TokenType::Result,
            },
            left: None,
            right: None,
            parent: None
        }]);
        assert_eq!(ast.root, 0);
        assert_eq!(ast.sub_roots, vec![]);
    }

    #[test]
    fn iteration_output() {
        let ast = ast_from("|>output");

        assert_eq!(ast.nodes, vec![Node {
            classification: Classification::IterationOutput,
            token: Token {
                value: String::from("|>output"),
                token_type: TokenType::IterationOutput,
            },
            left: None,
            right: None,
            parent: None
        }]);
        assert_eq!(ast.root, 0);
        assert_eq!(ast.sub_roots, vec![]);
    }

    #[test]
    fn iteration_skip() {
        let ast = ast_from("|>skip");

        assert_eq!(ast.nodes, vec![Node {
            classification: Classification::IterationSkip,
            token: Token {
                value: String::from("|>skip"),
                token_type: TokenType::IterationSkip,
            },
            left: None,
            right: None,
            parent: None
        }]);
        assert_eq!(ast.root, 0);
        assert_eq!(ast.sub_roots, vec![]);
    }

    #[test]
    fn iteration_continue() {
        let ast = ast_from("|>continue");

        assert_eq!(ast.nodes, vec![Node {
            classification: Classification::IterationContinue,
            token: Token {
                value: String::from("|>continue"),
                token_type: TokenType::IterationContinue,
            },
            left: None,
            right: None,
            parent: None
        }]);
        assert_eq!(ast.root, 0);
        assert_eq!(ast.sub_roots, vec![]);
    }

    #[test]
    fn iteration_complete() {
        let ast = ast_from("|>complete");

        assert_eq!(ast.nodes, vec![Node {
            classification: Classification::IterationComplete,
            token: Token {
                value: String::from("|>complete"),
                token_type: TokenType::IterationComplete,
            },
            left: None,
            right: None,
            parent: None
        }]);
        assert_eq!(ast.root, 0);
        assert_eq!(ast.sub_roots, vec![]);
    }
}

#[cfg(test)]
mod symbol_precedence_tests {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from};

    #[test]
    fn symbol() {
        let ast = ast_from(":my_symbol");

        ast.nodes.assert_node(0, None, None, Some(1));
        ast.nodes.assert_node(1, Some(0), None, None);

        assert_eq!(ast.root, 0);
    }
}

#[cfg(test)]
mod dot_access_precedence_tests {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from};

    #[test]
    fn decimal_is_above_numbers() {
        let ast = ast_from("3.14");

        ast.nodes.assert_node(0, Some(1), None, None);
        ast.nodes.assert_node(1, None, Some(0), Some(2));
        ast.nodes.assert_node(2, Some(1), None, None);

        assert_eq!(ast.root, 1);
    }

    #[test]
    fn access_is_above_identifiers() {
        let ast = ast_from("my_object.my_value");

        ast.nodes.assert_node(0, Some(1), None, None);
        ast.nodes.assert_node(1, None, Some(0), Some(2));
        ast.nodes.assert_node(2, Some(1), None, None);

        assert_eq!(ast.root, 1);
    }

    #[test]
    fn access_is_above_decimal() {
        let ast = ast_from("3.14.my_value");

        ast.nodes.assert_node(0, Some(1), None, None);
        ast.nodes.assert_node(1, Some(3), Some(0), Some(2));
        ast.nodes.assert_node(2, Some(1), None, None);
        ast.nodes.assert_node(3, None, Some(1), Some(4));
        ast.nodes.assert_node(4, Some(3), None, None);

        assert_eq!(ast.root, 3);
    }
}

#[cfg(test)]
mod unary_precedence_tests {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from};

    #[test]
    fn absolute_value() {
        let ast = ast_from("+10");

        ast.nodes.assert_node(0, None, None, Some(1));
        ast.nodes.assert_node(1, Some(0), None, None);
        
        assert_eq!(ast.root, 0);
    }

    #[test]
    fn negation() {
        let ast = ast_from("-10");

        ast.nodes.assert_node(0, None, None, Some(1));
        ast.nodes.assert_node(1, Some(0), None, None);

        assert_eq!(ast.root, 0);
    }
    
    #[test]
    fn logical_not() {
        let ast = ast_from("!!10");

        ast.nodes.assert_node(0, None, None, Some(1));
        ast.nodes.assert_node(1, Some(0), None, None);

        assert_eq!(ast.root, 0);
    }
    
    #[test]
    fn bitwise_not() {
        let ast = ast_from("!10");

        ast.nodes.assert_node(0, None, None, Some(1));
        ast.nodes.assert_node(1, Some(0), None, None);

        assert_eq!(ast.root, 0);
    }

    #[test]
    fn prefix_apply() {
        let ast = ast_from("`expr 10");

        ast.nodes.assert_node(0, None, None, Some(2));
        ast.nodes.assert_node(2, Some(0), None, None);

        assert_eq!(ast.root, 0);
    }

    #[test]
    fn multiple_prefix_apply() {
        let ast = ast_from("`expr2 `expr 10");

        ast.nodes.assert_node(0, None, None, Some(2));
        ast.nodes.assert_node(2, Some(0), None, Some(4));
        ast.nodes.assert_node(4, Some(2), None, None);

        assert_eq!(ast.root, 0);
    }
    
    #[test]
    fn unary_with_access() {
        let ast = ast_from("!my_object.my_value");

        ast.nodes.assert_node(0, None, None, Some(2));
        ast.nodes.assert_node(1, Some(2), None, None);
        ast.nodes.assert_node(2, Some(0), Some(1), Some(3));
        ast.nodes.assert_node(3, Some(2), None, None);

        assert_eq!(ast.root, 0);
    }
    
    #[test]
    fn unary_with_symbol() {
        let ast = ast_from("!:true");

        ast.nodes.assert_node(0, None, None, Some(1));
        ast.nodes.assert_node(1, Some(0), None, Some(2));
        ast.nodes.assert_node(2, Some(1), None, None);

        assert_eq!(ast.root, 0);
    }
}

#[cfg(test)]
mod suffix_apply_precedence_tests {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op};

    #[test]
    fn multiple_suffix_apply() {
        let ast = ast_from("10 expr` expr2`");

        ast.nodes.assert_node(0, Some(2), None, None);
        ast.nodes.assert_node(2, Some(4), Some(0), None);
        ast.nodes.assert_node(4, None, Some(2), None);

        assert_eq!(ast.root, 4);
    }

    #[test]
    fn suffix_apply() {
        let ast = ast_from("10 expr`");

        ast.nodes.assert_node(0, Some(2), None, None);
        ast.nodes.assert_node(2, None, Some(0), None);

        assert_eq!(ast.root, 2);
    }

    #[test]
    fn suffix_apply_with_prefix_apply() {
        let ast = ast_from("`expr 10 expr`");

        ast.nodes.assert_node(0, Some(4), None, Some(2));
        ast.nodes.assert_node(2, Some(0), None, None);
        ast.nodes.assert_node(4, None, Some(0), None);

        assert_eq!(ast.root, 4);
    }
}

#[cfg(test)]
mod type_cast_precedence_tests {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op};
    
    #[test]
    fn type_cast() {
        assert_binary_op("\"10\" #> 0");
    }
    
    #[test]
    fn type_cast_with_suffix_apply() {
        let ast = ast_from("10 expr` #> \"\"");

        ast.nodes.assert_node(0, Some(2), None, None);
        ast.nodes.assert_node(2, Some(4), Some(0), None );
        ast.nodes.assert_node(4, None, Some(2), Some(6));
        ast.nodes.assert_node(6, Some(4), None, None);

        assert_eq!(ast.root, 4);
    }
}

#[cfg(test)]
mod exponential_precedence_test {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op, assert_multi_op_least_first};
    
    #[test]
    fn type_cast() {
        assert_binary_op("10 ** 2");
    }
    
    #[test]
    fn type_cast_with_unary() {
        assert_multi_op_least_first("\"10\" #> 0 ** 2");
    }
}

#[cfg(test)]
mod multiply_divide_modulo_precedence_test {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op, assert_multi_op_least_first};
    
    #[test]
    fn multiplication() {
        assert_binary_op("10 * 2");
    }
    
    #[test]
    fn division() {
        assert_binary_op("10 / 2");
    }
    
    #[test]
    fn integer_division() {
        assert_binary_op("10 // 2");
    }
    
    #[test]
    fn modulo() {
        assert_binary_op("10 % 2");
    }
    
    #[test]
    fn multiplication_with_exponential() {
        assert_multi_op_least_first("10 ** 9 * 2");
    }
}

#[cfg(test)]
mod addition_subtraction_precedence_test {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op, assert_multi_op_least_first};
    
    #[test]
    fn addition() {
        assert_binary_op("10 + 2");
    }
    
    #[test]
    fn subtraction() {
        assert_binary_op("10 - 2");
    }

    #[test]
    fn addition_with_multiplication() {
        assert_multi_op_least_first("10 * 9 + 2");
    }
}

#[cfg(test)]
mod bit_shift_precedence_test {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op, assert_multi_op_least_first};
    
    #[test]
    fn shift_right() {
        assert_binary_op("10 >> 2");
    }
    
    #[test]
    fn shift_left() {
        assert_binary_op("10 << 2");
    }

    #[test]
    fn shift_left_with_addition() {
        assert_multi_op_least_first("10 + 9 << 2");
    }
}

#[cfg(test)]
mod range_precedence_test {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op, assert_multi_op_least_first};
    
    #[test]
    fn inclusive_range() {
        assert_binary_op("10 .. 2");
    }
    
    #[test]
    fn start_exclusive_range() {
        assert_binary_op("10 >.. 2");
    }

    #[test]
    fn end_exclusive_range() {
        assert_binary_op("10 ..< 2");
    }

    #[test]
    fn exclusive_range() {
        assert_binary_op("10 >..< 2");
    }

    #[test]
    fn exclusive_range_with_shift_left() {
        assert_multi_op_least_first("10 << 9 >..< 2");
    }
}

#[cfg(test)]
mod relational_precedence_test {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op, assert_multi_op_least_first};
    
    #[test]
    fn less_than() {
        assert_binary_op("10 < 2");
    }
    
    #[test]
    fn less_than_or_equal() {
        assert_binary_op("10 <= 2");
    }

    #[test]
    fn greater_than() {
        assert_binary_op("10 > 2");
    }

    #[test]
    fn greater_than_or_equal() {
        assert_binary_op("10 >= 2");
    }

    #[test]
    fn greater_than_with_range() {
        assert_multi_op_least_first("10 .. 9 > 2");
    }
}

#[cfg(test)]
mod equality_precedence_test {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op, assert_multi_op_least_first};
    
    #[test]
    fn equality() {
        assert_binary_op("10 == 2");
    }
    
    #[test]
    fn inequality() {
        assert_binary_op("10 != 2");
    }

    #[test]
    fn type_equality() {
        assert_binary_op("10 #= 2");
    }

    #[test]
    fn equality_with_less_than() {
        assert_multi_op_least_first("10 < 9 == 2");
    }
}

#[cfg(test)]
mod bit_and_precedence_test {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op, assert_multi_op_least_first};
    
    #[test]
    fn bit_and() {
        assert_binary_op("10 & 2");
    }

    #[test]
    fn bit_and_with_equality() {
        assert_multi_op_least_first("10 == 9 & 2");
    }
}

#[cfg(test)]
mod bit_xor_precedence_test {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op, assert_multi_op_least_first};
    
    #[test]
    fn bit_xor() {
        assert_binary_op("10 ^ 2");
    }

    #[test]
    fn bit_xor_with_bit_and() {
        assert_multi_op_least_first("10 & 9 ^ 2");
    }
}

#[cfg(test)]
mod bit_or_precedence_test {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op, assert_multi_op_least_first};
    
    #[test]
    fn bit_or() {
        assert_binary_op("10 | 2");
    }

    #[test]
    fn bit_or_with_bit_xor() {
        assert_multi_op_least_first("10 ^ 9 | 2");
    }
}

#[cfg(test)]
mod logical_and_precedence_test {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op, assert_multi_op_least_first};
    
    #[test]
    fn logical_and() {
        assert_binary_op("10 && 2");
    }

    #[test]
    fn logical_and_with_bit_or() {
        assert_multi_op_least_first("10 | 9 && 2");
    }
}

#[cfg(test)]
mod logical_xor_precedence_test {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op, assert_multi_op_least_first};
    
    #[test]
    fn logical_xor() {
        assert_binary_op("10 ^^ 2");
    }

    #[test]
    fn logical_xor_with_logical_and() {
        assert_multi_op_least_first("10 && 9 ^^ 2");
    }
}

#[cfg(test)]
mod logical_or_precedence_test {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op, assert_multi_op_least_first};
    
    #[test]
    fn logical_or() {
        assert_binary_op("10 || 2");
    }

    #[test]
    fn logical_or_with_logial_xor() {
        assert_multi_op_least_first("10 ^^ 9 || 2");
    }
}

#[cfg(test)]
mod link_precedence_test {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op, assert_multi_op_least_first};
    
    #[test]
    fn link() {
        assert_binary_op("10 -> 2");
    }

    #[test]
    fn link_with_logical_or() {
        assert_multi_op_least_first("10 || 9 -> 2");
    }
}

#[cfg(test)]
mod pair_precedence_test {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op, assert_multi_op_least_first};
    
    #[test]
    fn pair() {
        assert_binary_op("10 = 2");
    }

    #[test]
    fn pair_with_link() {
        assert_multi_op_least_first("10 -> 9 = 2");
    }

    #[test]
    fn multiple_pairs() {
        let ast = ast_from("10 = 5 = 1");

        ast.nodes.assert_node(0, Some(2), None, None);
        ast.nodes.assert_node(2, None, Some(0), Some(6));
        ast.nodes.assert_node(4, Some(6), None, None);
        ast.nodes.assert_node(6, Some(2), Some(4), Some(8));
        ast.nodes.assert_node(8, Some(6), None, None);

        assert_eq!(ast.root, 2);
    }
}

#[cfg(test)]
mod list_precedence_test {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op, assert_multi_op_least_first};
    
    #[test]
    fn list_comma() {
        assert_binary_op("10 , 2");
    }
    
    #[test]
    fn list_space() {
        let ast = ast_from("10 2");

        ast.nodes.assert_node(0, Some(1), None, None);
        ast.nodes.assert_node(1, None, Some(0), Some(2));
        ast.nodes.assert_node(2, Some(1), None, None);

        assert_eq!(ast.root, 1);
    }

    #[test]
    fn list_with_pair() {
        assert_multi_op_least_first("10 = 9 , 2");
    }
}

#[cfg(test)]
mod partially_apply_precedence_test {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op, assert_multi_op_least_first};
    
    #[test]
    fn partially_apply() {
        assert_binary_op("10 ~~ 2");
    }

    #[test]
    fn partially_apply_with_list() {
        assert_multi_op_least_first("10 , 9 ~~ 2");
    }
}

#[cfg(test)]
mod infix_precedence_test {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op, assert_multi_op_least_first};
    
    #[test]
    fn infix_apply() {
        assert_binary_op("10 `expr` 2");
    }

    #[test]
    fn infix_apply_partially_apply() {
        assert_multi_op_least_first("10 ~~ 9 `expr` 2");
    }
}

#[cfg(test)]
mod conditional_precedence_test {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op, assert_multi_op_least_first};
    
    #[test]
    fn invoke_if_true() {
        assert_binary_op("10 => 2");
    }
    
    #[test]
    fn invoke_if_false() {
        assert_binary_op("10 !> 2");
    }
    
    #[test]
    fn result_check_invoke() {
        assert_binary_op("10 =?> 2");
    }

    #[test]
    fn invoke_if_true_with_infix() {
        assert_multi_op_least_first("10 `expr` 9 => 2");
    }

    #[test]
    fn conditional_chain() {
        let ast = ast_from("10 => 5, 20 => 15, !> 25");

        ast.nodes.assert_node(2, Some(5), Some(0), Some(4));
        ast.nodes.assert_node(5, Some(12), Some(2), Some(9));
        ast.nodes.assert_node(9, Some(5), Some(7), Some(11));
        ast.nodes.assert_node(12, None, Some(5), Some(14));
        ast.nodes.assert_node(14, Some(12), None, Some(16));

        assert_eq!(ast.root, 12);
    }
}

#[cfg(test)]
mod funtional_precedence_test {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op, assert_multi_op_least_first};
    
    #[test]
    fn apply() {
        assert_binary_op("10 ~ 2");
    }
    
    #[test]
    fn pipe_apply() {
        assert_binary_op("10 ~> 2");
    }

    #[test]
    fn pipe_apply_with_invoke_if_true() {
        assert_multi_op_least_first("10 => 9 ~> 2");
    }
}

#[cfg(test)]
mod iteration_precedence_test {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op, assert_multi_op_least_first};
    
    #[test]
    fn iteration() {
        assert_binary_op("10 >>> 2");
    }
    
    #[test]
    fn iteration_to_single_value() {
        assert_binary_op("10 >>| 2");
    }
    
    #[test]
    fn reverse_iteration() {
        assert_binary_op("10 |>> 2");
    }
    
    #[test]
    fn reverse_iteration_to_single_value() {
        assert_binary_op("10 |>| 2");
    }

    #[test]
    fn iteration_with_apply() {
        assert_multi_op_least_first("10 ~ 9 >>> 2");
    }
}

#[cfg(test)]
mod output_result_tests {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op, assert_multi_op_least_first};

    #[test]
    fn output_result() {
        let ast = ast_from("5\n\n6");

        ast.nodes.assert_node(0, Some(1), None, None);
        ast.nodes.assert_node(1, None, Some(0), Some(3));
        ast.nodes.assert_node(3, Some(1), None, None);

        assert_eq!(ast.root, 1);
    }

    #[test]
    fn output_result_with_iteration() {
        let ast = ast_from("my_list >>> expr\n\nmy_list >> expr");

        ast.nodes.assert_node(0, Some(2), None, None);
        ast.nodes.assert_node(2, Some(5), Some(0), Some(4));
        ast.nodes.assert_node(4, Some(2), None, None);
        ast.nodes.assert_node(5, None, Some(2), Some(9));
        ast.nodes.assert_node(7, Some(9), None, None);
        ast.nodes.assert_node(9, Some(5), Some(7), Some(11));
        ast.nodes.assert_node(11, Some(9), None, None);

        assert_eq!(ast.root, 5);
    }
}

#[cfg(test)]
mod multi_precedence_tests {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op, assert_multi_op_least_first};

    #[test]
    fn pyrimid_greatest_precedence_on_outsides() {
        //                  0                        1          2
        //                  0        12        4 6 8 0 2  4 6 8 0 2        34
        let ast = ast_from("my_object.my_value * 4 + 8 .. 8 + 4 * my_object.my_value");

        ast.nodes.assert_node(0, Some(1), None, None);
        ast.nodes.assert_node(1, Some(4), Some(0), Some(2));
        ast.nodes.assert_node(2, Some(1), None, None);
        ast.nodes.assert_node(4, Some(8), Some(1), Some(6));
        ast.nodes.assert_node(6, Some(4), None, None);
        ast.nodes.assert_node(8, Some(12), Some(4), Some(10));
        ast.nodes.assert_node(10, Some(8), None, None);
        ast.nodes.assert_node(12, None, Some(8), Some(16));
        ast.nodes.assert_node(14, Some(16), None, None);
        ast.nodes.assert_node(16, Some(12), Some(14), Some(20));
        ast.nodes.assert_node(18, Some(20), None, None);
        ast.nodes.assert_node(20, Some(16), Some(18), Some(23));
        ast.nodes.assert_node(22, Some(23), None, None);
        ast.nodes.assert_node(23, Some(20), Some(22), Some(24));
        ast.nodes.assert_node(24, Some(23), None, None);

        assert_eq!(ast.root, 12);
    }

    #[test]
    fn pyrimid_greatest_precedence_on_insides() {
        //                  0           1                        2   
        //                  0 2  4  6 8 0 2        34        6 8 0 2  4  6 
        let ast = ast_from("5 && 10 + 6 * my_object.my_value * 6 + 10 && 5");

        ast.nodes.assert_node(0, Some(2), None, None);
        ast.nodes.assert_node(2, Some(24), Some(0), Some(20));
        ast.nodes.assert_node(4, Some(6), None, None);
        ast.nodes.assert_node(6, Some(20), Some(4), Some(16));
        ast.nodes.assert_node(8, Some(10), None, None);
        ast.nodes.assert_node(10, Some(16), Some(8), Some(13));
        ast.nodes.assert_node(12, Some(13), None, None);
        ast.nodes.assert_node(13, Some(10), Some(12), Some(14));
        ast.nodes.assert_node(14, Some(13), None, None);
        ast.nodes.assert_node(16, Some(6), Some(10), Some(18));
        ast.nodes.assert_node(18, Some(16), None, None);
        ast.nodes.assert_node(20, Some(2), Some(6), Some(22));
        ast.nodes.assert_node(22, Some(20), None, None);
        ast.nodes.assert_node(24, None, Some(2), Some(26));
        ast.nodes.assert_node(26, Some(24), None, None);

        assert_eq!(ast.root, 24);
    }
}

#[cfg(test)]
mod group_tests {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op, assert_multi_op_least_first};

    #[test]
    fn group_resolves_first() {
        for input in ["5 * {4 + 9} * 5", "5 * (4 + 9) * 5"].iter() {
            let ast = ast_from(input);

            ast.nodes.assert_node(0, Some(2), None, None);
            ast.nodes.assert_node(2, Some(12), Some(0), Some(4));
            ast.nodes.assert_node(4, Some(2), None, Some(7));
            ast.nodes.assert_node(5, Some(7), None, None);
            ast.nodes.assert_node(7, Some(4), Some(5), Some(9));
            ast.nodes.assert_node(9, Some(7), None, None);
            ast.nodes.assert_node(12, None, Some(2), Some(14));
            ast.nodes.assert_node(14, Some(12), None, None);

            assert_eq!(ast.root, 12);
        }
    }

    #[test]
    fn nested_group() {
        let ast = ast_from("5 ** (4 * (3 + 3) * 4) ** 5");

        ast.nodes.assert_node(0, Some(2), None, None);
        ast.nodes.assert_node(2, Some(22), Some(0), Some(4));
        ast.nodes.assert_node(4, Some(2), None, Some(17));
        ast.nodes.assert_node(5, Some(7), None, None);
        ast.nodes.assert_node(7, Some(17), Some(5), Some(9));
        ast.nodes.assert_node(9, Some(7), None, Some(12));
        ast.nodes.assert_node(10, Some(12), None, None);
        ast.nodes.assert_node(12, Some(9), Some(10), Some(14));
        ast.nodes.assert_node(14, Some(12), None, None);
        ast.nodes.assert_node(17, Some(4), Some(7), Some(19));
        ast.nodes.assert_node(19, Some(17), None, None);
        ast.nodes.assert_node(22, None, Some(2), Some(24));
        ast.nodes.assert_node(24, Some(22), None, None);

        assert_eq!(ast.root, 22);
    }

    #[test]
    fn compact_nesting() {
        let ast = ast_from("(3+((4)+(4))+3)");

        ast.nodes.assert_node(0, None, None, Some(12));
        ast.nodes.assert_node(1, Some(2), None, None);
        ast.nodes.assert_node(2, Some(12), Some(1), Some(3));
        ast.nodes.assert_node(3, Some(2), None, Some(7));
        ast.nodes.assert_node(4, Some(7), None, Some(5));
        ast.nodes.assert_node(5, Some(4), None, None);
        ast.nodes.assert_node(7, Some(3), Some(4), Some(8));
        ast.nodes.assert_node(8, Some(7), None, Some(9));
        ast.nodes.assert_node(9, Some(8), None, None);
        ast.nodes.assert_node(12, Some(0), Some(2), Some(13));
        ast.nodes.assert_node(13, Some(12), None, None);

        assert_eq!(ast.root, 0);
    }

    #[test]
    fn surrounded_expression() {
        //                  0            1
        //                  0 2  45 7 9  2  4
        let ast = ast_from("5 -> {4 + 3} ~~ 9");

        ast.nodes.assert_node(0, Some(2), None, None);
        ast.nodes.assert_node(2, Some(12), Some(0), Some(4));
        ast.nodes.assert_node(4, Some(2), None, Some(7));
        ast.nodes.assert_node(5, Some(7), None, None);
        ast.nodes.assert_node(7, Some(4), Some(5), Some(9));
        ast.nodes.assert_node(9, Some(7), None, None);
        ast.nodes.assert_node(12, None, Some(2), Some(14));
        ast.nodes.assert_node(14, Some(12), None, None);

        assert_eq!(ast.root, 12);
    }

    #[test]
    fn surrounded_group() {
        //                  0            1
        //                  0 2  45 7 9  2  4
        let ast = ast_from("5 -> (4 + 3) ~~ 9");

        ast.nodes.assert_node(0, Some(2), None, None);
        ast.nodes.assert_node(2, Some(12), Some(0), Some(4));
        ast.nodes.assert_node(4, Some(2), None, Some(7));
        ast.nodes.assert_node(5, Some(7), None, None);
        ast.nodes.assert_node(7, Some(4), Some(5), Some(9));
        ast.nodes.assert_node(9, Some(7), None, None);
        ast.nodes.assert_node(12, None, Some(2), Some(14));
        ast.nodes.assert_node(14, Some(12), None, None);

        assert_eq!(ast.root, 12);
    }
}

