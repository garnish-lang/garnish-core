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

    fn ast_from(s: &str) -> AST {
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
}
