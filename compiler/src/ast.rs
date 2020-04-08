use crate::{ParseResult, Node, Token, TokenType, Classification};
use expr_lang_common::Result;

pub struct AST {
    pub(crate) nodes: Vec<Node>,
    pub(crate) root: usize,
    pub(crate) sub_roots: Vec<usize>
}

pub fn make_ast(parse_result: ParseResult) -> Result<AST> {
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

    let mut nodes = Vec::with_capacity(parse_result.nodes.len());

    for node in parse_result.nodes.iter() {
        nodes.push(node.clone());
    }

    return Ok(AST {
        nodes,
        root: 0,
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
    use crate::{make_ast, AST, Lexer, TokenType, Token, Node, Parser, Classification};
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

}
