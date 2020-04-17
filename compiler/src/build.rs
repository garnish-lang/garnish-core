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

    process_node(ast.root, &ast, &mut instructions)?;

    instructions.end_expression();

    return Ok(instructions);
}
 
fn process_node(index: usize, ast: &AST, i: &mut InstructionSetBuilder) -> Result<()> {
    let node = &ast.nodes[index];

    let extract_index = |o, s, p| -> Result<usize> {
        match o {
            Some(i) => Ok(i),
            None => Err(format!("Expected {} side node for parent {}", s, p).into())
        }
    };

    let extract = |o, s, p| -> Result<&Node> { 
        let i = extract_index(o, s, p)?;
        match ast.nodes.get(i) {
            Some(n) => Ok(n),
            None => Err(format!("Node on {} side of parent {} with index {} not in AST", s, p, i).into())
        }
    };

    let extract_left = || extract(node.left, "left", index);
    let extract_right = || extract(node.right, "right", index);

    let right_index = || extract_index(node.right, "right", index);
    let left_index = || extract_index(node.left, "left", index);

    let process_left_right = |i: &mut InstructionSetBuilder, op: fn(&mut InstructionSetBuilder) -> ()| -> Result<()> {
        process_node(left_index()?, ast, i)?;
        process_node(right_index()?, ast, i)?;
        op(i);
        Ok(())
    };

    let process_unary_right = |i: &mut InstructionSetBuilder, op: fn(&mut InstructionSetBuilder) -> ()| -> Result<()> {
        process_node(right_index()?, ast, i)?;
        op(i);
        Ok(())
    };

    match node.classification {
        // put literal values in based on their type
        Classification::Literal => match node.token.token_type {
            TokenType::UnitLiteral => i.put(ExpressionValue::unit())?,
            TokenType::Number => {
                let num: i32 = match node.token.value.parse() {
                    Ok(f) => f,
                    Err(_) => return Err(format!("Invalid integer value ({}) at node {}", node.token.value, ast.root).into())
                };
                i.put(ExpressionValue::integer(num))?;
            }
            TokenType::Character => {
                i.put(ExpressionValue::character(node.token.value.clone()))?;
            }
            TokenType::CharacterList => {
                i.put(ExpressionValue::character_list(node.token.value.clone()))?;
            }
            TokenType::Identifier => {
                i.resolve(&node.token.value)?;
            }
            _ => unimplemented!()
        }
        Classification::Symbol => {
            // unary op literal
            i.put(ExpressionValue::symbol(&extract_right()?.token.value))?;
        }
        Classification::Decimal => {
            // special literal value composed of two literal nodes
            let float_str = format!("{}.{}", 
                extract_left()?.token.value, 
                extract_right()?.token.value
            );
            let f: f32 = match float_str.parse() {
                Ok(f) => f,
                Err(_) => return Err(format!("Invalid float value ({}) at node {}", float_str, ast.root).into())
            };
            i.put(ExpressionValue::float(f))?;
        }
        Classification::Negation => process_unary_right(i, InstructionSetBuilder::perform_negation)?,
        Classification::AbsoluteValue => process_unary_right(i, InstructionSetBuilder::perform_absolute_value)?,
        Classification::LogicalNot => process_unary_right(i, InstructionSetBuilder::perform_logical_not)?,
        Classification::BitwiseNot => process_unary_right(i, InstructionSetBuilder::perform_bitwise_not)?,
        Classification::PrefixApply => {
            process_node(right_index()?, ast, i)?;
            i.push_input();
            i.execute_expression(&node.token.value);
        }
        Classification::SuffixApply => {
            process_node(left_index()?, ast, i)?;
            i.push_input();
            i.execute_expression(&node.token.value);
        }
        Classification::Access => process_left_right(i, InstructionSetBuilder::perform_access)?,
        Classification::TypeCast => process_left_right(i, InstructionSetBuilder::perform_type_cast)?,
        _ => ()
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{build_byte_code, make_ast, AST, Lexer, TokenType, Token, Node, Parser, Classification};
    use expr_lang_instruction_set_builder::InstructionSetBuilder;
    use expr_lang_common::{ExpressionValue};

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
        expected.resolve(&"my_value".into()).unwrap();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn access() {
        let instructions = byte_code_from("my_object.my_value");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.resolve(&"my_object".into()).unwrap();
        expected.resolve(&"my_value".into()).unwrap();
        expected.perform_access();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn multi_access() {
        let instructions = byte_code_from("my_object.my_sub_object.my_property.my_value");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.resolve(&"my_object".into()).unwrap();
        expected.resolve(&"my_sub_object".into()).unwrap();
        expected.perform_access();
        expected.resolve(&"my_property".into()).unwrap();
        expected.perform_access();
        expected.resolve(&"my_value".into()).unwrap();
        expected.perform_access();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }
}

#[cfg(test)]
mod unary_tests {
    use expr_lang_instruction_set_builder::InstructionSetBuilder;
    use expr_lang_common::{ExpressionValue};
    use super::tests::byte_code_from;

    #[test]
    fn negation() {
        let instructions = byte_code_from("-10");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.put(ExpressionValue::integer(10)).unwrap();
        expected.perform_negation();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn absolute_value() {
        let instructions = byte_code_from("+10");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.put(ExpressionValue::integer(10)).unwrap();
        expected.perform_absolute_value();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn logcial_not() {
        let instructions = byte_code_from("!!10");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.put(ExpressionValue::integer(10)).unwrap();
        expected.perform_logical_not();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn bitwise_not() {
        let instructions = byte_code_from("!10");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.put(ExpressionValue::integer(10)).unwrap();
        expected.perform_bitwise_not();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn prefix_invoke() {
        let instructions = byte_code_from("`expr 10");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.put(ExpressionValue::integer(10)).unwrap();
        expected.push_input();
        expected.execute_expression("expr");
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn suffix_invoke() {
        let instructions = byte_code_from("10 expr`");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.put(ExpressionValue::integer(10)).unwrap();
        expected.push_input();
        expected.execute_expression("expr");
        expected.end_expression();

        assert_eq!(instructions, expected);
    }
}

#[cfg(test)]
mod binary_tests {
    use expr_lang_instruction_set_builder::InstructionSetBuilder;
    use expr_lang_common::{ExpressionValue};
    use super::tests::byte_code_from;

    #[test]
    fn type_cast() {
        let instructions = byte_code_from("10 #> \"\"");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.put(ExpressionValue::integer(10)).unwrap();
        expected.put(ExpressionValue::character_list("".into())).unwrap();
        expected.perform_type_cast();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }
}

