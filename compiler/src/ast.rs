use crate::{ParseResult, Node, Token, TokenType, Classification};
use expr_lang_common::Result;

pub struct AST {
    pub(crate) nodes: Vec<Node>,
    pub(crate) root: usize,
    pub(crate) sub_roots: Vec<usize>
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

    let mut op_locations: Vec<usize> = vec![];

    for (i, node) in parse_result.nodes.iter().enumerate() {
        if node.classification == Classification::Decimal {
            op_locations.push(i);
        }
    }

    for loc in op_locations.iter() {
        // get op's left and right
        // update parent to be loc
        // if value set left and right to None

        let (left, right) = parse_result.nodes.get(*loc).map(|n| (n.left, n.right)).unwrap();

        match left {
            Some(i) => {
                parse_result.nodes[i].parent = Some(*loc);
                parse_result.nodes[i].left = None;
                parse_result.nodes[i].right = None;
            }
            None => () // nothing to do
        }

        match right {
            Some(i) => {
                parse_result.nodes[i].parent = Some(*loc);
                parse_result.nodes[i].left = None;
                parse_result.nodes[i].right = None;
            }
            None => () // nothing to do
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
mod dot_access_precedence_tests {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::ast_from;

    #[test]
    fn decimal_is_above_numbers() {
        let ast = ast_from("3.14");

        let node = ast.nodes.get(0).unwrap();
        assert_eq!(node.parent, Some(1));
        assert_eq!(node.left, None);
        assert_eq!(node.right, None);

        let node = ast.nodes.get(2).unwrap();
        assert_eq!(node.parent, Some(1));
        assert_eq!(node.left, None);
        assert_eq!(node.right, None);

        let node = ast.nodes.get(1).unwrap();
        assert_eq!(node.parent, None);
        assert_eq!(node.left, Some(0));
        assert_eq!(node.right, Some(2));

        assert_eq!(ast.root, 1);
    }
}
