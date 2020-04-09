use crate::{ParseResult, Node, Token, TokenType, Classification};
use expr_lang_common::Result;

pub struct AST {
    pub(crate) nodes: Vec<Node>,
    pub(crate) root: usize,
    pub(crate) sub_roots: Vec<usize>
}

#[derive(PartialEq, Copy, Clone)]
enum OpType {
    Binary,
    UnaryLeft,
    UnaryRight,
}

pub fn make_ast(mut parse_result: ParseResult) -> Result<AST> {
    if parse_result.nodes.is_empty() {
        return Ok(AST {
            nodes: vec![Node {
                classification: Classification::Literal,
                token: Token {
                    value: String::from(""),
                    token_type: TokenType::UnitLiteral
                },
                left: None,
                right: None,
                parent: None
            }],
            root: 0,
            sub_roots: vec![]
        });
    }

    let mut op_locations: Vec<(OpType, Vec<usize>)> = vec![
        (OpType::UnaryLeft, vec![]),
        (OpType::Binary, vec![]),
        (OpType::Binary, vec![]),
        (OpType::UnaryLeft, vec![]),
        (OpType::Binary, vec![]),
    ];

    for (i, node) in parse_result.nodes.iter().enumerate() {
        let p = match node.classification {
            Classification::Literal 
            | Classification::IterationOutput 
            | Classification::IterationSkip 
            | Classification::IterationContinue
            | Classification::IterationComplete 
            | Classification::NoOp => continue,
            Classification::Symbol => 0,
            Classification::Decimal => 1,
            Classification::Access => 2,
            Classification::Negation
            | Classification::AbsoluteValue 
            | Classification::Not => 3,
            Classification::TypeCast => 4,
            _ => unimplemented!("{:?}", node.classification)
        };

        op_locations[p].1.push(i);
    }

    for precedence in op_locations.iter() {
        for loc in precedence.1.iter() {
            // get op's left and right
            // update parent to be loc
            // if value set left and right to None

            let (left, right) = parse_result.nodes.get(*loc).map(|n| (n.left, n.right)).unwrap();

            if precedence.0 != OpType::UnaryLeft {
                match left {
                    Some(i) => {
                        parse_result.nodes[i].parent = Some(*loc);

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
                    }
                    None => () // nothing to do
                };
            } else {
                // unary left means operand is on right side
                // set left side to None
                parse_result.nodes[*loc].left = None;
            }

            match right {
                Some(i) => {
                    parse_result.nodes[i].parent = Some(*loc);

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
                }
                None => () // nothing to do
            }
        }
    }

    let mut root_index = *parse_result.sub_expressions.get(0).unwrap(); // should always have 1
    let mut node = &parse_result.nodes[root_index];

    while node.parent.is_some() {
        root_index = node.parent.unwrap();
        node = &parse_result.nodes[root_index];
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
            assert_eq!(node.parent, parent);
            assert_eq!(node.left, left);
            assert_eq!(node.right, right);
        }
    }

    #[test]
    fn create_empty() {
        let ast = ast_from("");

        assert_eq!(ast.nodes, vec![Node {
            classification: Classification::Literal,
            token: Token {
                value: String::from(""),
                token_type: TokenType::UnitLiteral,
            },
            left: None,
            right: None,
            parent: None
        }]);
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
    fn not() {
        let ast = ast_from("!10");

        ast.nodes.assert_node(0, None, None, Some(1));
        ast.nodes.assert_node(1, Some(0), None, None);

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
mod type_cast_precedence_tests {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from};
    
    #[test]
    fn type_cast() {
        let ast = ast_from("\"10\" #> 0");

        ast.nodes.assert_node(0, Some(2), None, None);
        ast.nodes.assert_node(2, None, Some(0), Some(4));
        ast.nodes.assert_node(4, Some(2), None, None);

        assert_eq!(ast.root, 2);
    }
    
    #[test]
    fn type_cast_with_unary() {
        let ast = ast_from("-10 #> \"\"");

        ast.nodes.assert_node(0, Some(3), None, Some(1));
        ast.nodes.assert_node(1, Some(0), None, None );
        ast.nodes.assert_node(3, None, Some(0), Some(5));
        ast.nodes.assert_node(5, Some(3), None, None);

        assert_eq!(ast.root, 3);
    }
}
