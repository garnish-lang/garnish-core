use garnish_lang_common::Result;
use crate::{AST, Classification, TokenType, Node};
use garnish_lang_instruction_set_builder::InstructionSetBuilder;
use garnish_lang_common::ExpressionValue;

struct SubData {
    name: String,
    start: usize,
    id: usize,
    right: Option<String>
}

impl SubData {
    fn new(name: String, start: usize, id: usize, right: Option<String>) -> Self {
        SubData { name, start, id, right }
    }
}

struct SubInfo {
    count: usize,
    subs: Vec<SubData>,
    current: SubData
}

pub fn build_byte_code(root_name: &str, ast: AST) -> Result<InstructionSetBuilder> {
    let mut instructions = InstructionSetBuilder::new();

    append_ast_to_instructions(root_name, ast, &mut instructions)?;

    return Ok(instructions);
}

pub fn append_ast_to_instructions(root_name: &str, ast: AST, instructions: &mut InstructionSetBuilder) -> Result<()> {
    if ast.nodes.is_empty() {
        instructions.start_expression(root_name);
        instructions.put(ExpressionValue::unit())?;
        instructions.end_expression();
        return Ok(());
    }
    
    let mut info = SubInfo {
        count: 0,
        subs: vec![],
        current: SubData::new("".into(), 0, 0, None),
    };

    info.subs.push(SubData::new(String::from(root_name), ast.root, 0, None));

    while !info.subs.is_empty() {
        info.current = info.subs.pop().unwrap();

        instructions.start_expression(info.current.name.clone());
        process_node(&root_name, info.current.start.clone(), &ast, instructions, false, None, &mut info)?;
        instructions.end_expression();
    }
    
    Ok(())
}
 
fn process_node(name: &str,
        index: usize,
        ast: &AST,
        i: &mut InstructionSetBuilder,
        list_root: bool,
        conditional: Option<&String>,
        refs: &mut SubInfo
    ) -> Result<()> {
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

    let process_left_right = |refs: &mut SubInfo, i: &mut InstructionSetBuilder, op: fn(&mut InstructionSetBuilder) -> ()| -> Result<()> {
        process_node(name, left_index()?, ast, i, list_root, conditional, refs)?;
        process_node(name, right_index()?, ast, i, list_root, conditional, refs)?;
        op(i);
        Ok(())
    };

    let process_unary_right = |refs: &mut SubInfo, i: &mut InstructionSetBuilder, op: fn(&mut InstructionSetBuilder) -> ()| -> Result<()> {
        process_node(name, right_index()?, ast, i, list_root, conditional, refs)?;
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
            TokenType::StartGroup => process_node(name, right_index()?, ast, i, false, conditional, refs)?,
            TokenType::StartExpression => {
                let id = refs.count;
                refs.count += 1;
                let name = format!("{}@sub_{}", name, id);
                refs.subs.push(SubData::new(name.clone(), right_index()?, id, None));
                i.put(ExpressionValue::expression(name.clone()))?;
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
        Classification::Negation => process_unary_right(refs, i, InstructionSetBuilder::perform_negation)?,
        Classification::AbsoluteValue => process_unary_right(refs, i, InstructionSetBuilder::perform_absolute_value)?,
        Classification::LogicalNot => process_unary_right(refs, i, InstructionSetBuilder::perform_logical_not)?,
        Classification::BitwiseNot => process_unary_right(refs, i, InstructionSetBuilder::perform_bitwise_not)?,
        Classification::PrefixApply => {
            process_node(name, right_index()?, ast, i, list_root, conditional, refs)?;
            i.push_input();
            i.execute_expression(&node.token.value);
        }
        Classification::SuffixApply => {
            process_node(name, left_index()?, ast, i, list_root, conditional, refs)?;
            i.push_input();
            i.execute_expression(&node.token.value);
        }
        Classification::Access => process_left_right(refs, i, InstructionSetBuilder::perform_access)?,
        Classification::TypeCast => process_left_right(refs, i, InstructionSetBuilder::perform_type_cast)?,
        Classification::Exponential => process_left_right(refs, i, InstructionSetBuilder::perform_exponential)?,
        Classification::Multiplication => process_left_right(refs, i, InstructionSetBuilder::perform_multiplication)?,
        Classification::Division => process_left_right(refs, i, InstructionSetBuilder::perform_division)?,
        Classification::IntegerDivision => process_left_right(refs, i, InstructionSetBuilder::perform_integer_division)?,
        Classification::Modulo => process_left_right(refs, i, InstructionSetBuilder::perform_remainder)?,
        Classification::Addition => process_left_right(refs, i, InstructionSetBuilder::perform_addition)?,
        Classification::Subtraction => process_left_right(refs, i, InstructionSetBuilder::perform_subtraction)?,
        Classification::BitwiseLeftShift => process_left_right(refs, i, InstructionSetBuilder::perform_bitwise_left_shift)?,
        Classification::BitwiseRightShift => process_left_right(refs, i, InstructionSetBuilder::perform_bitwise_right_shift)?,
        Classification::MakeRange => process_left_right(refs, i, InstructionSetBuilder::make_inclusive_range)?,
        Classification::MakeStartExclusiveRange => process_left_right(refs, i, InstructionSetBuilder::make_start_exclusive_range)?,
        Classification::MakeEndExclusiveRange => process_left_right(refs, i, InstructionSetBuilder::make_end_exclusive_range)?,
        Classification::MakeExclusiveRange => process_left_right(refs, i, InstructionSetBuilder::make_exclusive_range)?,
        Classification::LessThan => process_left_right(refs, i, InstructionSetBuilder::perform_less_than_comparison)?,
        Classification::LessThanOrEqual => process_left_right(refs, i, InstructionSetBuilder::perform_less_than_or_equal_comparison)?,
        Classification::GreaterThan => process_left_right(refs, i, InstructionSetBuilder::perform_greater_than_comparison)?,
        Classification::GreaterThanOrEqual => process_left_right(refs, i, InstructionSetBuilder::perform_greater_than_or_equal_comparison)?,
        Classification::Equality => process_left_right(refs, i, InstructionSetBuilder::perform_equality_comparison)?,
        Classification::Inequality => process_left_right(refs, i, InstructionSetBuilder::perform_inequality_comparison)?,
        Classification::TypeEqual => process_left_right(refs, i, InstructionSetBuilder::perform_type_comparison)?,
        Classification::BitwiseAnd => process_left_right(refs, i, InstructionSetBuilder::perform_bitwise_and)?,
        Classification::BitwiseOr => process_left_right(refs, i, InstructionSetBuilder::perform_bitwise_or)?,
        Classification::BitwiseXor => process_left_right(refs, i, InstructionSetBuilder::perform_bitwise_xor)?,
        Classification::LogicalAnd => process_left_right(refs, i, InstructionSetBuilder::perform_logical_and)?,
        Classification::LogicalOr => process_left_right(refs, i, InstructionSetBuilder::perform_logical_or)?,
        Classification::LogicalXor => process_left_right(refs, i, InstructionSetBuilder::perform_logical_xor)?,
        Classification::MakeLink => process_left_right(refs, i, InstructionSetBuilder::make_link)?,
        Classification::MakePair => process_left_right(refs, i, InstructionSetBuilder::make_pair)?,
        Classification::PartiallyApply => process_left_right(refs, i, InstructionSetBuilder::partially_apply)?,
        Classification::Apply => process_left_right(refs, i, InstructionSetBuilder::apply)?,
        Classification::PipeApply => {
            process_node(name, right_index()?, ast, i, list_root, conditional, refs)?;
            process_node(name, left_index()?, ast, i, list_root, conditional, refs)?;
            i.apply();
        }
        Classification::Iterate => process_left_right(refs, i, InstructionSetBuilder::iterate)?,
        Classification::IterateToSingleValue => process_left_right(refs, i, InstructionSetBuilder::iterate_to_single_value)?,
        Classification::ReverseIterate => process_left_right(refs, i, InstructionSetBuilder::reverse_iterate)?,
        Classification::ReverseIterateToSingleValue => process_left_right(refs, i, InstructionSetBuilder::reverse_iterate_to_single_value)?,
        Classification::MultiIterate => process_left_right(refs, i, InstructionSetBuilder::multi_iterate)?,
        Classification::OutputResult => process_left_right(refs, i, InstructionSetBuilder::output_result)?,
        Classification::ListSeparator => {
            match list_root {
                false => {
                    i.start_list();
                    process_node(name, left_index()?, ast, i, true, conditional, refs)?;
                    process_node(name, right_index()?, ast, i, true, conditional, refs)?;
                    i.make_list();
                }
                true => {
                    process_node(name, left_index()?, ast, i, true, conditional, refs)?;
                    process_node(name, right_index()?, ast, i, true, conditional, refs)?;
                }
            }
        }
        Classification::InfixApply => {
            i.start_list();
            process_node(name, left_index()?, ast, i, list_root, conditional, refs)?;
            process_node(name, right_index()?, ast, i, list_root, conditional, refs)?;
            i.make_list();
            i.push_input();
            i.execute_expression(&node.token.value);
        }
        Classification::InvokeIfTrue => {
            process_node(name, left_index()?, ast, i, list_root, conditional, refs)?;
            let id = refs.count;
            refs.count += 1;
            let name = format!("{}@sub_{}", name, id);
            refs.subs.push(SubData::new(name.clone(), right_index()?, id, None));
            let right = match conditional {
                Some(c) => Some(c.clone()),
                None => match &refs.current.right {
                    Some(c) => Some(c.clone()),
                    None => None,
                }
            };
            i.conditional_execute(Some(name), right);
        }
        Classification::InvokeIfFalse => {
            process_node(name, left_index()?, ast, i, list_root, conditional, refs)?;
            let id = refs.count;
            refs.count += 1;
            let name = format!("{}@sub_{}", name, id);
            refs.subs.push(SubData::new(name.clone(), right_index()?, id, None));
            let right = match conditional {
                Some(c) => Some(c.clone()),
                None => match &refs.current.right {
                    Some(c) => Some(c.clone()),
                    None => None,
                }
            };
            i.conditional_execute(right, Some(name));
        }
        Classification::ResultCheckInvoke => {
            process_node(name, left_index()?, ast, i, list_root, conditional, refs)?;
            let id = refs.count;
            refs.count += 1;
            let name = format!("{}@sub_{}", name, id);
            refs.subs.push(SubData::new(name.clone(), right_index()?, id, None));
            let right = match conditional {
                Some(c) => Some(c.clone()),
                None => match &refs.current.right {
                    Some(c) => Some(c.clone()),
                    None => None,
                }
            };
            i.result_conditional_execute(name, right);
        }
        Classification::DefaultInvoke => {
            process_node(name, right_index()?, ast, i, list_root, conditional, refs)?;
        }
        Classification::ConditionalContinuation => {
            let id = refs.count;
            refs.count += 1;
            let right_name = format!("{}@sub_{}", name, id);
            refs.subs.push(SubData::new(right_name.clone(), right_index()?, id, match conditional {
                Some(c) => Some(c.clone()),
                None => None
            }));
            process_node(name, left_index()?, ast, i, list_root, Some(&right_name), refs)?;
        }
        _ => ()
    };
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{build_byte_code, make_ast, AST, Lexer, TokenType, Token, Node, Parser, Classification};
    use garnish_lang_instruction_set_builder::InstructionSetBuilder;
    use garnish_lang_common::{ExpressionValue};

    pub fn byte_code_from(s: &str) -> InstructionSetBuilder {
        let input = Lexer::new().lex(s).unwrap();
        let parser = Parser::new();
        let parse_result = parser.make_groups(&input).unwrap();
        let ast_result = make_ast(parse_result).unwrap();
        
        return build_byte_code("main", ast_result).unwrap();
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

        let result = build_byte_code("main", ast);
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

        let result = build_byte_code("main", ast);
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

        let result = build_byte_code("main", ast);
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

        let result = build_byte_code("main", ast);
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

        let result = build_byte_code("main", ast);
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
    use garnish_lang_instruction_set_builder::InstructionSetBuilder;
    use garnish_lang_common::{ExpressionValue};
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
    use garnish_lang_instruction_set_builder::InstructionSetBuilder;
    use garnish_lang_common::{ExpressionValue};
    use super::tests::byte_code_from;

    fn assert_binary_op(op_str: &str, op: fn(&mut InstructionSetBuilder) -> ()) {
        let instructions = byte_code_from(&format!("10 {} 5", op_str));

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.put(ExpressionValue::integer(10)).unwrap();
        expected.put(ExpressionValue::integer(5)).unwrap();
        op(&mut expected);
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn type_cast() {
        assert_binary_op("#>", InstructionSetBuilder::perform_type_cast);
    }

    #[test]
    fn exponential() {
        assert_binary_op("**", InstructionSetBuilder::perform_exponential);
    }

    #[test]
    fn multiplication() {
        assert_binary_op("*", InstructionSetBuilder::perform_multiplication);
    }

    #[test]
    fn division() {
        assert_binary_op("/", InstructionSetBuilder::perform_division);
    }

    #[test]
    fn integer_division() {
        assert_binary_op("//", InstructionSetBuilder::perform_integer_division);
    }

    #[test]
    fn modulo() {
        assert_binary_op("%", InstructionSetBuilder::perform_remainder);
    }

    #[test]
    fn addition() {
        assert_binary_op("+", InstructionSetBuilder::perform_addition);
    }

    #[test]
    fn subtraction() {
        assert_binary_op("-", InstructionSetBuilder::perform_subtraction);
    }

    #[test]
    fn bitwise_left_shift() {
        assert_binary_op("<<", InstructionSetBuilder::perform_bitwise_left_shift);
    }

    #[test]
    fn bitwise_right_shift() {
        assert_binary_op(">>", InstructionSetBuilder::perform_bitwise_right_shift);
    }

    #[test]
    fn inclusive_range() {
        assert_binary_op("..", InstructionSetBuilder::make_inclusive_range);
    }

    #[test]
    fn start_exclusive_range() {
        assert_binary_op(">..", InstructionSetBuilder::make_start_exclusive_range);
    }

    #[test]
    fn end_exclusive_range() {
        assert_binary_op("..<", InstructionSetBuilder::make_end_exclusive_range);
    }

    #[test]
    fn exclusive_range() {
        assert_binary_op(">..<", InstructionSetBuilder::make_exclusive_range);
    }

    #[test]
    fn less_than() {
        assert_binary_op("<", InstructionSetBuilder::perform_less_than_comparison);
    }

    #[test]
    fn less_than_or_equal() {
        assert_binary_op("<=", InstructionSetBuilder::perform_less_than_or_equal_comparison);
    }

    #[test]
    fn greater_than() {
        assert_binary_op(">", InstructionSetBuilder::perform_greater_than_comparison);
    }

    #[test]
    fn greater_than_or_equal() {
        assert_binary_op(">=", InstructionSetBuilder::perform_greater_than_or_equal_comparison);
    }

    #[test]
    fn equality() {
        assert_binary_op("==", InstructionSetBuilder::perform_equality_comparison);
    }

    #[test]
    fn inequality_comparison() {
        assert_binary_op("!=", InstructionSetBuilder::perform_inequality_comparison);
    }

    #[test]
    fn type_comparison() {
        assert_binary_op("#=", InstructionSetBuilder::perform_type_comparison);
    }

    #[test]
    fn bitwise_and() {
        assert_binary_op("&", InstructionSetBuilder::perform_bitwise_and);
    }

    #[test]
    fn bitwise_or() {
        assert_binary_op("|", InstructionSetBuilder::perform_bitwise_or);
    }

    #[test]
    fn bitwise_xor() {
        assert_binary_op("^", InstructionSetBuilder::perform_bitwise_xor);
    }

    #[test]
    fn logical_and() {
        assert_binary_op("&&", InstructionSetBuilder::perform_logical_and);
    }

    #[test]
    fn logical_or() {
        assert_binary_op("||", InstructionSetBuilder::perform_logical_or);
    }

    #[test]
    fn logical_xor() {
        assert_binary_op("^^", InstructionSetBuilder::perform_logical_xor);
    }

    #[test]
    fn make_link() {
        assert_binary_op("->", InstructionSetBuilder::make_link);
    }

    #[test]
    fn make_pair() {
        assert_binary_op("=", InstructionSetBuilder::make_pair);
    }

    #[test]
    fn make_list() {
        let instructions = byte_code_from("10, 20, 30, 40, 50");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.start_list();
        expected.put(ExpressionValue::integer(10)).unwrap();
        expected.put(ExpressionValue::integer(20)).unwrap();
        expected.put(ExpressionValue::integer(30)).unwrap();
        expected.put(ExpressionValue::integer(40)).unwrap();
        expected.put(ExpressionValue::integer(50)).unwrap();
        expected.make_list();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn partially_apply() {
        assert_binary_op("~~", InstructionSetBuilder::partially_apply);
    }

    #[test]
    fn infix() {
        let instructions = byte_code_from("10 `expr` 20");
        let mut expected = InstructionSetBuilder::new();

        expected.start_expression("main");
        expected.start_list();
        expected.put(ExpressionValue::integer(10)).unwrap();
        expected.put(ExpressionValue::integer(20)).unwrap();
        expected.make_list();
        expected.push_input();
        expected.execute_expression("expr");
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn invoke_if_true() {
        let instructions = byte_code_from("10 => 5");
        let mut expected = InstructionSetBuilder::new();

        expected.start_expression("main");
        expected.put(ExpressionValue::integer(10)).unwrap();
        expected.conditional_execute(Some("main@sub_0".into()), None);
        expected.end_expression();

        expected.start_expression("main@sub_0");
        expected.put(ExpressionValue::integer(5)).unwrap();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn invoke_if_false() {
        let instructions = byte_code_from("10 !> 5");
        let mut expected = InstructionSetBuilder::new();

        expected.start_expression("main");
        expected.put(ExpressionValue::integer(10)).unwrap();
        expected.conditional_execute(None, Some("main@sub_0".into()));
        expected.end_expression();

        expected.start_expression("main@sub_0");
        expected.put(ExpressionValue::integer(5)).unwrap();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn result_check_invoke() {
        let instructions = byte_code_from("10 =?> 5");
        let mut expected = InstructionSetBuilder::new();

        expected.start_expression("main");
        expected.put(ExpressionValue::integer(10)).unwrap();
        expected.result_conditional_execute("main@sub_0".into(), None);
        expected.end_expression();

        expected.start_expression("main@sub_0");
        expected.put(ExpressionValue::integer(5)).unwrap();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn apply() {
        assert_binary_op("~", InstructionSetBuilder::apply);
    }

    #[test]
    fn pipe_apply() {
        let instructions = byte_code_from("10 ~> expr");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.resolve(&"expr".into()).unwrap();
        expected.put(ExpressionValue::integer(10)).unwrap();
        expected.apply();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn iterate() {
        assert_binary_op(">>>", InstructionSetBuilder::iterate);
    }

    #[test]
    fn iterate_to_single_value() {
        assert_binary_op(">>|", InstructionSetBuilder::iterate_to_single_value);
    }

    #[test]
    fn reverse_iterate() {
        assert_binary_op("|>>", InstructionSetBuilder::reverse_iterate);
    }

    #[test]
    fn reverse_iterate_to_single_value() {
        assert_binary_op("|>|", InstructionSetBuilder::reverse_iterate_to_single_value);
    }

    #[test]
    fn multi_iterate() {
        assert_binary_op("<>>", InstructionSetBuilder::multi_iterate);
    }

    #[test]
    fn output_result() {
        assert_binary_op("\n\n", InstructionSetBuilder::output_result);
    }
}

#[cfg(test)]
mod conditional_chain_tests {
    use garnish_lang_instruction_set_builder::InstructionSetBuilder;
    use garnish_lang_common::{ExpressionValue};
    use super::tests::byte_code_from;

    #[test]
    fn two_true_chain() {
        let instructions = byte_code_from("10 => 5, 20 => 15");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.put(ExpressionValue::integer(10)).unwrap();
        expected.conditional_execute(Some("main@sub_1".into()), Some("main@sub_0".into()));
        expected.end_expression();

        expected.start_expression("main@sub_1");
        expected.put(ExpressionValue::integer(5)).unwrap();
        expected.end_expression();

        expected.start_expression("main@sub_0");
        expected.put(ExpressionValue::integer(20)).unwrap();
        expected.conditional_execute(Some("main@sub_2".into()), None);
        expected.end_expression();

        expected.start_expression("main@sub_2");
        expected.put(ExpressionValue::integer(15)).unwrap();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn two_false_chain() {
        let instructions = byte_code_from("10 !> 5, 20 !> 15");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.put(ExpressionValue::integer(10)).unwrap();
        expected.conditional_execute(Some("main@sub_0".into()), Some("main@sub_1".into()));
        expected.end_expression();

        expected.start_expression("main@sub_1");
        expected.put(ExpressionValue::integer(5)).unwrap();
        expected.end_expression();

        expected.start_expression("main@sub_0");
        expected.put(ExpressionValue::integer(20)).unwrap();
        expected.conditional_execute(None, Some("main@sub_2".into()));
        expected.end_expression();

        expected.start_expression("main@sub_2");
        expected.put(ExpressionValue::integer(15)).unwrap();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn two_result_check_chain() {
        let instructions = byte_code_from("10 =?> 5, 20 =?> 15");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.put(ExpressionValue::integer(10)).unwrap();
        expected.result_conditional_execute("main@sub_1".into(), Some("main@sub_0".into()));
        expected.end_expression();

        expected.start_expression("main@sub_1");
        expected.put(ExpressionValue::integer(5)).unwrap();
        expected.end_expression();

        expected.start_expression("main@sub_0");
        expected.put(ExpressionValue::integer(20)).unwrap();
        expected.result_conditional_execute("main@sub_2".into(), None);
        expected.end_expression();

        expected.start_expression("main@sub_2");
        expected.put(ExpressionValue::integer(15)).unwrap();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn three_true_chain() {
        let instructions = byte_code_from("10 => 5, 20 => 15, 30 => 25");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.put(ExpressionValue::integer(10)).unwrap();
        expected.conditional_execute(Some("main@sub_2".into()), Some("main@sub_1".into()));
        expected.end_expression();

        expected.start_expression("main@sub_2");
        expected.put(ExpressionValue::integer(5)).unwrap();
        expected.end_expression();

        expected.start_expression("main@sub_1");
        expected.put(ExpressionValue::integer(20)).unwrap();
        expected.conditional_execute(Some("main@sub_3".into()), Some("main@sub_0".into()));
        expected.end_expression();

        expected.start_expression("main@sub_3");
        expected.put(ExpressionValue::integer(15)).unwrap();
        expected.end_expression();

        expected.start_expression("main@sub_0");
        expected.put(ExpressionValue::integer(30)).unwrap();
        expected.conditional_execute(Some("main@sub_4".into()), None);
        expected.end_expression();

        expected.start_expression("main@sub_4");
        expected.put(ExpressionValue::integer(25)).unwrap();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn conditional_with_default() {
        let instructions = byte_code_from("10 => 5, !> 15");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.put(ExpressionValue::integer(10)).unwrap();
        expected.conditional_execute(Some("main@sub_1".into()), Some("main@sub_0".into()));
        expected.end_expression();

        expected.start_expression("main@sub_1");
        expected.put(ExpressionValue::integer(5)).unwrap();
        expected.end_expression();

        expected.start_expression("main@sub_0");
        expected.put(ExpressionValue::integer(15)).unwrap();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }
}

#[cfg(test)]
mod groups_and_sub_expressions {
    use garnish_lang_instruction_set_builder::InstructionSetBuilder;
    use garnish_lang_common::{ExpressionValue};
    use super::tests::byte_code_from;

    #[test]
    fn single_group() {
        let instructions = byte_code_from("5 * (4 + 3) * 9");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.put(ExpressionValue::integer(5)).unwrap();
        expected.put(ExpressionValue::integer(4)).unwrap();
        expected.put(ExpressionValue::integer(3)).unwrap();
        expected.perform_addition();
        expected.perform_multiplication();
        expected.put(ExpressionValue::integer(9)).unwrap();
        expected.perform_multiplication();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn single_expression() {
        let instructions = byte_code_from("5 -> {4 + 3} ~~ 9");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.put(ExpressionValue::integer(5)).unwrap();
        expected.put(ExpressionValue::expression("main@sub_0")).unwrap();
        expected.make_link();
        expected.put(ExpressionValue::integer(9)).unwrap();
        expected.partially_apply();
        expected.end_expression();

        expected.start_expression("main@sub_0");
        expected.put(ExpressionValue::integer(4)).unwrap();
        expected.put(ExpressionValue::integer(3)).unwrap();
        expected.perform_addition();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn nested_group() {
        let instructions = byte_code_from("5 * (4 + (3 - 2) + 3) * 9");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.put(ExpressionValue::integer(5)).unwrap();
        expected.put(ExpressionValue::integer(4)).unwrap();
        expected.put(ExpressionValue::integer(3)).unwrap();
        expected.put(ExpressionValue::integer(2)).unwrap();
        expected.perform_subtraction();
        expected.perform_addition();
        expected.put(ExpressionValue::integer(3)).unwrap();
        expected.perform_addition();
        expected.perform_multiplication();
        expected.put(ExpressionValue::integer(9)).unwrap();
        expected.perform_multiplication();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }

    #[test]
    fn nested_list() {
        let instructions = byte_code_from("10, 20, (30, 40, 50), 60");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.start_list();
        expected.put(ExpressionValue::integer(10)).unwrap();
        expected.put(ExpressionValue::integer(20)).unwrap();
        expected.start_list();
        expected.put(ExpressionValue::integer(30)).unwrap();
        expected.put(ExpressionValue::integer(40)).unwrap();
        expected.put(ExpressionValue::integer(50)).unwrap();
        expected.make_list();
        expected.put(ExpressionValue::integer(60)).unwrap();
        expected.make_list();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }
}

