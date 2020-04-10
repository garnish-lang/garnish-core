use crate::{ParseResult, Node, Token, TokenType, Classification};
use expr_lang_common::Result;

#[derive(Debug)]
pub struct AST {
    pub(crate) nodes: Vec<Node>,
    pub(crate) root: usize,
    pub(crate) sub_roots: Vec<usize>
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
        (OpType::UnaryLeft, vec![]), // 0 - Symbol
        (OpType::Binary, vec![]), // 1 - Decimal
        (OpType::Binary, vec![]), // 2 - Access
        (OpType::UnaryLeft, vec![]), // 3 - Unary
        (OpType::Binary, vec![]), // 4 - TypeCast
        (OpType::Binary, vec![]), // 5 - Exponential
        (OpType::Binary, vec![]), // 6 - Multiply, Division, Modulo
        (OpType::Binary, vec![]), // 7 - Addition, Subtraction
        (OpType::Binary, vec![]), // 8 - Bit Shift
        (OpType::Binary, vec![]), // 9 - Range
        (OpType::Binary, vec![]), // 10 - Relational
        (OpType::Binary, vec![]), // 11 - Equality
        (OpType::Binary, vec![]), // 12 - Bitwise And
        (OpType::Binary, vec![]), // 13 - Bitwise Xor
        (OpType::Binary, vec![]), // 14 - Bitwise Or
        (OpType::Binary, vec![]), // 15 - Logical And
        (OpType::Binary, vec![]), // 16 - Logical Xor
        (OpType::Binary, vec![]), // 17 - Logical Or
        (OpType::Binary, vec![]), // 18 - Link
        (OpType::Binary, vec![]), // 19 - Pair
        (OpType::Binary, vec![]), // 20 - List
        (OpType::Binary, vec![]), // 21 - Partially Apply
        (OpType::Binary, vec![]), // 22 - Prefix Apply
        (OpType::Binary, vec![]), // 23 - Suffix Apply
        (OpType::Binary, vec![]), // 24 - Infix Apply
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
            Classification::Exponential => 5,
            Classification::Multiplication
            | Classification::Division
            | Classification::IntegerDivision
            | Classification::Modulo => 6,
            Classification::Addition
            | Classification::Subtraction => 7,
            Classification::BitwiseLeftShift
            | Classification::BitwiseRightShift => 8,
            Classification::MakeRange
            | Classification::MakeStartExclusiveRange
            | Classification::MakeEndExclusiveRange
            | Classification::MakeExclusiveRange => 9,
            Classification::LessThan
            | Classification::LessThanOrEqual
            | Classification::GreaterThan
            | Classification::GreaterThanOrEqual => 10,
            Classification::Equality
            | Classification::Inequality
            | Classification::TypeEqual => 11,
            Classification::BitwiseAnd => 12,
            Classification::BitwiseXor => 13,
            Classification::BitwiseOr => 14,
            Classification::LogicalAnd => 15,
            Classification::LogicalXor => 16,
            Classification::LogicalOr => 17,
            Classification::MakeLink => 18,
            Classification::MakePair => 19,
            Classification::ListSeparator => 20,
            Classification::PartiallyApply => 21,
            Classification::PrefixApply => 22,
            Classification::SuffixApply => 23,
            Classification::InfixApply => 24,
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
    use super::tests::{AssertNode, ast_from, assert_binary_op};
    
    #[test]
    fn type_cast() {
        assert_binary_op("\"10\" #> 0");
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
mod prefix_precedence_test {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op, assert_multi_op_least_first};
    
    #[test]
    fn prefix_apply() {
        assert_binary_op("10 `expr 2");
    }

    #[test]
    fn prefix_apply_with_partially_apply() {
        assert_multi_op_least_first("10 ~~ 9 `expr 2");
    }
}

#[cfg(test)]
mod suffix_precedence_test {
    use crate::{Lexer, TokenType, Token, Node, Parser, Classification};
    use super::tests::{AssertNode, ast_from, assert_binary_op, assert_multi_op_least_first};
    
    #[test]
    fn suffix_apply() {
        assert_binary_op("10 expr` 2");
    }

    #[test]
    fn suffix_apply_with_prefix_apply() {
        assert_multi_op_least_first("10 `expr 9 expr` 2");
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
    fn infix_apply_with_suffix_apply() {
        assert_multi_op_least_first("10 expr` 9 `expr` 2");
    }
}

