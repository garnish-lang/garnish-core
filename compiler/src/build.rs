use expr_lang_common::Result;
use crate::{AST, Classification, TokenType};
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

    let first = &ast.nodes[ast.root];
    match first.classification {
        // put literal values in based on their type
        Classification::Literal => match first.token.token_type {
            TokenType::UnitLiteral => instructions.put(ExpressionValue::unit())?,
            TokenType::Number => {
                let i: i32 = first.token.value.parse().unwrap();
                instructions.put(ExpressionValue::integer(i))?;
            }
            _ => unimplemented!()
        }
        Classification::Decimal => {
            // special literal value composed of two literal nodes
            let float_str = format!("{}.{}", 
                ast.nodes[first.left.unwrap()].token.value, 
                ast.nodes[first.right.unwrap()].token.value
            );
            let f: f32 = float_str.parse().unwrap();
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
    fn float() {
        let instructions = byte_code_from("3.14");

        let mut expected = InstructionSetBuilder::new();
        expected.start_expression("main");
        expected.put(ExpressionValue::float(3.14)).unwrap();
        expected.end_expression();

        assert_eq!(instructions, expected);
    }
}

