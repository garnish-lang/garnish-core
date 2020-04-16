use expr_lang_common::Result;
use crate::{AST, Classification, TokenType, Node};
use expr_lang_instruction_set_builder::InstructionSetBuilder;
use expr_lang_common::ExpressionValue;

pub fn build_byte_code(ast: AST) -> Result<InstructionSetBuilder> {
    let mut instructions = InstructionSetBuilder::new();
    instructions.start_expression("main");

    if ast.nodes.is_empty() {
        instructions.put(ExpressionValue::unit())?;
        instructions.end_expression();
        return Ok(instructions);
    }

    let extract = |o, s, p| -> Result<&Node> { 
        match o {
            Some(i) => match ast.nodes.get(i) {
                Some(n) => Ok(n),
                None => Err(format!("Node on {} side of parent {} with index {} not in AST", s, p, i).into())
            }
            None => Err(format!("Expected {} side node for parent {}", s, p).into())
        }
    };

    let extract_left = |o, p| extract(o, "left", p);
    let extract_right = |o, p| extract(o, "right", p);

    let first = &ast.nodes[ast.root];
    match first.classification {
        // put literal values in based on their type
        Classification::Literal => match first.token.token_type {
            TokenType::UnitLiteral => instructions.put(ExpressionValue::unit())?,
            TokenType::Number => {
                let i: i32 = match first.token.value.parse() {
                    Ok(f) => f,
                    Err(_) => return Err(format!("Invalid integer value ({}) at node {}", first.token.value, ast.root).into())
                };
                instructions.put(ExpressionValue::integer(i))?;
            }
            TokenType::Character => {
                instructions.put(ExpressionValue::character(first.token.value.clone()))?;
            }
            TokenType::CharacterList => {
                instructions.put(ExpressionValue::character_list(first.token.value.clone()))?;
            }
            TokenType::Identifier => {
                instructions.resolve(&first.token.value);
            }
            _ => unimplemented!()
        }
        Classification::Symbol => {
            // unary op literal
            instructions.put(ExpressionValue::symbol(&extract_right(first.right, ast.root)?.token.value))?;
        }
        Classification::Decimal => {
            // special literal value composed of two literal nodes
            let float_str = format!("{}.{}", 
                extract_left(first.left, ast.root)?.token.value, 
                extract_right(first.right, ast.root)?.token.value
            );
            let f: f32 = match float_str.parse() {
                Ok(f) => f,
                Err(_) => return Err(format!("Invalid float value ({}) at node {}", float_str, ast.root).into())
            };
            instructions.put(ExpressionValue::float(f))?;
        }
        _ => ()
    }

    instructions.end_expression();

    return Ok(instructions);
}

#[cfg(test)]
mod tests {
    use crate::{build_byte_code, make_ast, AST, Lexer, TokenType, Token, Node, Parser, Classification};
    use expr_lang_instruction_set_builder::InstructionSetBuilder;
    use expr_lang_common::{DataType, ExpressionValue};

    pub fn byte_code_from(s: &str) -> InstructionSetBuilder {
        let input = Lexer::new().lex(s).unwrap();
        let parser = Parser::new();
        let parse_result = parser.make_groups(&input).unwrap();
        let ast_result = make_ast(parse_result).unwrap();
        
        return build_byte_code(ast_result).unwrap();
    }

    #[test]
    fn empty() {
        let instructions = byte_code_from("");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.put(ExpressionValue::unit()).unwrap();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn unit() {
        let instructions = byte_code_from("()");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.put(ExpressionValue::unit()).unwrap();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn integer() {
        let instructions = byte_code_from("10");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.put(ExpressionValue::integer(10)).unwrap();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn invalid_integer() {
        let mut ast = AST::new();
        ast.root = 0;
        ast.nodes.push(Node {
            classification: Classification::Literal,
            token: Token {
                value: "abc".into(),
                token_type: TokenType::Number,
            },
            left: None,
            right: None,
            parent: None,
        });

        let result = build_byte_code(ast);
        assert_eq!(result.err().unwrap().get_message(), "Invalid integer value (abc) at node 0");
    }

    #[test]
    fn float() {
        let instructions = byte_code_from("3.14");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.put(ExpressionValue::float(3.14)).unwrap();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn invalid_float() {
        let mut ast = AST::new();
        ast.root = 1;
        ast.nodes.push(Node {
            classification: Classification::Literal,
            token: Token {
                value: "abc".into(),
                token_type: TokenType::Number,
            },
            left: None,
            right: None,
            parent: Some(1),
        });
        ast.nodes.push(Node {
            classification: Classification::Decimal,
            token: Token {
                value: ".".into(),
                token_type: TokenType::DotOperator,
            },
            left: Some(0),
            right: Some(2),
            parent: None,
        });
        ast.nodes.push(Node {
            classification: Classification::Literal,
            token: Token {
                value: "abc".into(),
                token_type: TokenType::Number,
            },
            left: None,
            right: None,
            parent: Some(1),
        });

        let result = build_byte_code(ast);
        assert_eq!(result.err().unwrap().get_message(), "Invalid float value (abc.abc) at node 1");
    }

    #[test]
    fn float_missing_left() {
        let mut ast = AST::new();
        ast.root = 0;
        ast.nodes.push(Node {
            classification: Classification::Decimal,
            token: Token {
                value: ".".into(),
                token_type: TokenType::DotOperator,
            },
            left: None,
            right: Some(1),
            parent: None,
        });
        ast.nodes.push(Node {
            classification: Classification::Literal,
            token: Token {
                value: "10".into(),
                token_type: TokenType::Number,
            },
            left: None,
            right: None,
            parent: Some(0),
        });

        let result = build_byte_code(ast);
        assert_eq!(result.err().unwrap().get_message(), "Expected left side node for parent 0");
    }

    #[test]
    fn float_missing_right() {
        let mut ast = AST::new();
        ast.root = 1;
        ast.nodes.push(Node {
            classification: Classification::Literal,
            token: Token {
                value: "10".into(),
                token_type: TokenType::Number,
            },
            left: None,
            right: None,
            parent: Some(1),
        });
        ast.nodes.push(Node {
            classification: Classification::Decimal,
            token: Token {
                value: ".".into(),
                token_type: TokenType::DotOperator,
            },
            left: Some(0),
            right: None,
            parent: None,
        });

        let result = build_byte_code(ast);
        assert_eq!(result.err().unwrap().get_message(), "Expected right side node for parent 1");
    }

    #[test]
    fn character() {
        let instructions = byte_code_from("'a'");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.put(ExpressionValue::character("a".into())).unwrap();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn character_list() {
        let instructions = byte_code_from("\"Hello, World!\"");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.put(ExpressionValue::character_list("Hello, World!".into())).unwrap();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn symbol() {
        let instructions = byte_code_from(":my_symbol");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.put(ExpressionValue::symbol("my_symbol")).unwrap();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn symbol_missing_right() {
        let mut ast = AST::new();
        ast.root = 0;
        ast.nodes.push(Node {
            classification: Classification::Symbol,
            token: Token {
                value: ":".into(),
                token_type: TokenType::SymbolOperator,
            },
            left: None,
            right: None,
            parent: None,
        });

        let result = build_byte_code(ast);
        assert_eq!(result.err().unwrap().get_message(), "Expected right side node for parent 0");
    }

    #[test]
    fn identifier() {
        let instructions = byte_code_from("my_value");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.resolve(&"my_value".into());
        expected.end_expression();

        assert_eq!(instructions, expected);
    }
}

