use expr_lang_common::Result;

use crate::{Token, TokenType};

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum Operation {
    Group,
    Addition,
    Subtraction,
    Negation,
    AbsoluteValue,
    NoOp,
    Literal,
    Multiplication,
    Division,
    IntegerDivision,
    Modulo,
    Exponential,
    LogicalAnd,
    LogicalOr,
    LogicalXor,
    Not,
    TypeCast,
    Equality,
    Inequality,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    TypeEqual,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseLeftShift,
    BitwiseRightShift,
    Access,
    MakeRange,
    MakeStartExclusiveRange,
    MakeEndExclusiveRange,
    MakeExclusiveRange,
    MakePair,
    MakeLink,
    MultiIterate,
    InvokeIfTrue,
    InvokeIfFalse,
    ResultCheckInvoke,
    Apply,
    PartiallyApply,
    PipeApply,
    PrefixApply,
    SuffixApply,
    InfixApply,
    Iterate,
    IterateToSingleValue,
    ReverseIterate,
    ReversIterateToSingleValue,
    IterationOutput,
    IterationSkip,
    IterationContinue,
    IterationComplete,
    OutputResult,
    CheckForResult,
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Node {
    operation: Operation,
    token: Token,
    left: Option<usize>,
    right: Option<usize>,
}

pub struct ParseResult {
    nodes: Vec<Node>,
    groups: Vec<usize>,
    sub_expressions: Vec<usize>,
}

pub struct Parser {}

impl Parser {
    pub fn new() -> Self {
        Parser {}
    }

    pub fn make_groups(&self, tokens: &[Token]) -> Result<ParseResult> {
        let mut nodes: Vec<Node> = vec![];
        let mut groups = vec![];
        let mut sub_expressions = vec![0]; // assuming first sub expression is always start of input

        let mut groups_stack = vec![];
        let mut next_left = None;

        let mut last_token_type = TokenType::HorizontalSpace;
        let mut last_operation = Operation::NoOp;
        let mut check_for_result: Option<usize> = None;
        let mut check_for_prefix: Option<usize> = None;
        let mut check_for_suffix: Option<usize> = None;
        let mut check_for_infix: Option<usize> = None;

        let trim_start = tokens.iter().position(non_white_space).unwrap_or(0);
        let trim_end = tokens.iter().rposition(non_white_space).unwrap_or(tokens.len());

        let trimmed_tokens = &tokens[trim_start..=trim_end];

        for (i, token) in trimmed_tokens.iter().enumerate() {
            let left = match next_left {
                Some(l) => {
                    // only use next left once
                    next_left = None;
                    Some(l)
                },
                None => if i as i32 - 1 >= 0 { Some(i - 1) } else { None }
            };

            let right = if i + 1 < trimmed_tokens.len() { Some(i + 1) } else { None };

            let _l = left.unwrap_or_default();

            let mut op = initial_operation_from_token_type(token.token_type);

            // check for result
            match check_for_result {
                Some(r) => {
                    if op == Operation::Literal {
                        // this is a separate expression
                        // set check for result node to be have output result
                        match nodes.get_mut(r) {
                            Some(n) => {
                                n.operation = Operation::OutputResult;
                            }
                            None => unreachable!()
                        }                      

                        sub_expressions.push(i);
                    } else {
                        // update left's right to point to this node
                        match left {
                            Some(l) => {
                                match nodes.get_mut(l) {
                                    Some(n) => {
                                        n.right = Some(i);
                                    }
                                    None => unreachable!()
                                }
                            }
                            None => () // end of input
                        }

                        // The new line before this now serves no purpose
                        match nodes.get_mut(r) {
                            Some(n) => {
                                n.operation = Operation::NoOp;
                            }
                            None => unreachable!()
                        }
                    }
                    check_for_result = None;
                }
                None => () // nothing to do
            }

            // match token type for special cases
            match token.token_type {
                TokenType::StartGroup | TokenType::StartExpression => {
                    groups.push(i);
                    groups_stack.push(i);
                }
                TokenType::EndGroup | TokenType::EndExpression => {
                    match groups_stack.pop() {
                        Some(group_start) => {
                            // set next left to group start to force next node to point there
                            next_left = Some(group_start);

                            // go back to group start node and update its right
                            // to point to one after this node
                            match nodes.get_mut(group_start) {
                                Some(n) => {
                                    if (n.token.token_type == TokenType::StartGroup && token.token_type != TokenType::EndGroup)
                                        || (n.token.token_type == TokenType::StartExpression && token.token_type != TokenType::EndExpression) {
                                        // group tokens don't match raise error
                                        let expected = if n.token.token_type == TokenType::StartGroup { "group" } else { "expression" };
                                        let found = if n.token.token_type == TokenType::StartGroup { "expression" } else { "group" };

                                        return Err(format!("Expected end of {}, found end of {}.", expected, found).into())
                                    }

                                    match trimmed_tokens.get(i + 1) {
                                        Some(rt) => {
                                            if rt.token_type == TokenType::HorizontalSpace {
                                                // skip the horizontal space
                                                n.right = Some(i + 2);
                                            } else {
                                                n.right = right;
                                            }
                                        }
                                        None => {
                                            n.right = None;
                                        }
                                    }
                                }
                                None => unreachable!("Group start not in nodes vec.")
                            }
                        }
                        None => return Err(format!("End of group found at {} but no start preceded it.", i).into())
                    }
                }
                TokenType::MinusSign => {
                    match left {
                        Some(l) => match nodes.get(l) {
                            Some(n) => if n.operation != Operation::Literal {
                                op = Operation::Negation;
                            }
                            None => unreachable!()
                        }
                        None => op = Operation::Negation
                    }
                }
                TokenType::PlusSign => {
                    match left {
                        Some(l) => match nodes.get(l) {
                            Some(n) => if n.operation != Operation::Literal {
                                op = Operation::AbsoluteValue;
                            }
                            None => unreachable!()
                        }
                        None => op = Operation::AbsoluteValue
                    }
                }
                TokenType::Identifier => {
                    match check_for_prefix {
                        Some(p) => match nodes.get_mut(p) {
                            Some(n) => {
                                n.operation = Operation::PrefixApply;

                                // also set check for infix
                                check_for_infix = Some(p);

                                check_for_prefix = None;
                            }
                            None => unreachable!()
                        }
                        // if not checking for prefix, is could still be a suffix apply
                        None => check_for_suffix = Some(i)
                    }
                }
                TokenType::InfixOperator => {
                    match check_for_infix {
                        Some(f) => {
                            // update this node and previous node to be infix apply
                            op = Operation::InfixApply;
                            match nodes.get_mut(f) {
                                Some(n) => n.operation = Operation::InfixApply,
                                None => unreachable!()
                            }

                            check_for_infix = None;
                        }
                        None => {
                            // not checking for infix
                            // first see if looking for suffix
                            // else this could still be a prefix apply
                            if check_for_suffix.is_some() {
                                op = Operation::SuffixApply;
                            } else {
                                check_for_prefix = Some(i);
                            }
                        }
                    }
                }
                TokenType::HorizontalSpace => {
                    // set next left to be this nodes left
                    next_left = left;
                    match left {
                        Some(l) => {
                            match nodes.get_mut(l) {
                                Some(n) => {
                                    n.right = right;
                                }
                                None => unreachable!()
                            }
                        }
                        None => () // end of input
                    }

                    // *fix termination
                    // *fix operators must be right next to their identifiers
                    // so if we are checking for any off them
                    // we stop checking now
                
                    if check_for_infix.is_some() {
                        check_for_infix = None;
                    }
                }
                TokenType::NewLine => {
                    if last_token_type == TokenType::NewLine {
                        // explicit starting new sub expression
                        // update this token to be a no op
                        // update previous token to be an output operation
                        // and push next token index to subexpressions if exists

                        op = Operation::NoOp;

                        match nodes.get_mut(i - 1) {
                            Some(n) => {
                                n.operation = Operation::OutputResult;
                            }
                            None => unreachable!()
                        }

                        match right {
                            Some(r) => {
                                sub_expressions.push(r);
                            }
                            None => () // were at end of input
                        }
                    } else {
                        // check for implicit new sub expression
                        // by looking at terminability of previous token
                        // and the startability of current expression
                        // an expression is terminable if it ends in a value type
                        // and an expression can only start with a value type

                        if last_operation == Operation::Literal {
                            // store node to be checked during next iteration
                            check_for_result = Some(i);
                            // set next left to be reassigned 
                            // if next token is not a literal
                            next_left = left;
                        } else {
                            // current expression continues to next node
                            // skip over this newline by linking last node
                            // and set this node to be a noop

                            next_left = left;
                            op = Operation::NoOp;
                            match left {
                                Some(l) => {
                                    match nodes.get_mut(l) {
                                        Some(n) => {
                                            n.right = right;
                                        }
                                        None => unreachable!()
                                    }
                                }
                                None => () // end of input
                            }
                        }
                    }
                }
                // not all token types have unique behavior
                _ => ()
            }

            last_token_type = token.token_type;
            last_operation = op;

            nodes.push(Node {
                operation: last_operation,
                token: token.clone(),
                left,
                right
            });
        }

        let limit = nodes.len();
        for i in 0..limit {
            let current = &nodes[i];

            // if new line wasn't reassigned to OutputResult
            if current.operation == Operation::CheckForResult {

            }
        }

        // groups stack should be empty
        // if not we have an unclosed group some where
        if !groups_stack.is_empty() {
            return Err(format!("Unclosed group starting at {}.", groups_stack.pop().unwrap()).into());
        }

        Ok(ParseResult {
            groups,
            nodes,
            sub_expressions
        })
    }
}

fn non_white_space(t: &Token) -> bool {
    t.token_type != TokenType::HorizontalSpace && t.token_type != TokenType::NewLine
}

fn initial_operation_from_token_type(token_type: TokenType) -> Operation {
    match token_type {
        TokenType::Number => Operation::Literal,
        TokenType::Identifier => Operation::Literal,
        TokenType::HorizontalSpace => Operation::NoOp,
        TokenType::PlusSign => Operation::Addition,
        TokenType::MinusSign => Operation::Subtraction,
        TokenType::MultiplicationSign => Operation::Multiplication,
        TokenType::DivisionSign => Operation::Division,
        TokenType::IntegerDivisionOperator => Operation::IntegerDivision,
        TokenType::ModuloOperator => Operation::Modulo,
        TokenType::ExponentialSign => Operation::Exponential,
        TokenType::LogicalAndOperator => Operation::LogicalAnd,
        TokenType::LogicalOrOperator => Operation::LogicalOr,
        TokenType::LogicalXorOperator => Operation::LogicalXor,
        TokenType::NotOperator => Operation::Not,
        TokenType::TypeCastOperator => Operation::TypeCast,
        TokenType::EqualityOperator => Operation::Equality,
        TokenType::InequalityOperator => Operation::Inequality,
        TokenType::LessThanOperator => Operation::LessThan,
        TokenType::LessThanOrEqualOperator => Operation::LessThanOrEqual,
        TokenType::GreaterThanOperator => Operation::GreaterThan,
        TokenType::GreaterThanOrEqualOperator => Operation::GreaterThanOrEqual,
        TokenType::TypeComparisonOperator => Operation::TypeEqual,
        TokenType::BitwiseAndOperator => Operation::BitwiseAnd,
        TokenType::BitwiseOrOperator => Operation::BitwiseOr,
        TokenType::BitwiseXorOperator => Operation::BitwiseXor,
        TokenType::BitwiseLeftShiftOperator => Operation::BitwiseLeftShift,
        TokenType::BitwiseRightShiftOperator => Operation::BitwiseRightShift,
        TokenType::DotOperator => Operation::Access,
        TokenType::RangeOperator => Operation::MakeRange,
        TokenType::StartExclusiveRangeOperator => Operation::MakeStartExclusiveRange,
        TokenType::EndExclusiveRangeOperator => Operation::MakeEndExclusiveRange,
        TokenType::ExclusiveRangeOperator => Operation::MakeExclusiveRange,
        TokenType::PairOperator => Operation::MakePair,
        TokenType::LinkOperator => Operation::MakeLink,
        TokenType::MultiIterationOperator => Operation::MultiIterate,
        TokenType::ConditionalTrueOperator => Operation::InvokeIfTrue,
        TokenType::ConditionalFalseOperator => Operation::InvokeIfFalse,
        TokenType::ConditionalResultOperator => Operation::ResultCheckInvoke,
        TokenType::SymbolOperator => Operation::Literal,
        TokenType::StartExpression => Operation::Literal,
        TokenType::EndExpression => Operation::NoOp,
        TokenType::StartGroup => Operation::Literal,
        TokenType::EndGroup => Operation::NoOp,
        TokenType::Result => Operation::Literal,
        TokenType::Input => Operation::Literal,
        TokenType::Comma => Operation::NoOp,
        TokenType::UnitLiteral => Operation::Literal,
        TokenType::ApplyOperator => Operation::Apply,
        TokenType::PartiallyApplyOperator => Operation::PartiallyApply,
        TokenType::PipeOperator => Operation::PipeApply,
        TokenType::InfixOperator => Operation::NoOp,
        TokenType::IterationOperator => Operation::Iterate,
        TokenType::SingleValueIterationOperator => Operation::IterateToSingleValue,
        TokenType::ReverseIterationOperator => Operation::ReverseIterate,
        TokenType::SingleValueReverseIterationOperator => Operation::ReversIterateToSingleValue,
        TokenType::IterationOutput => Operation::IterationOutput,
        TokenType::IterationSkip => Operation::IterationSkip,
        TokenType::IterationContinue => Operation::IterationContinue,
        TokenType::IterationComplete => Operation::IterationComplete,
        TokenType::Character => Operation::Literal,
        TokenType::CharacterList => Operation::Literal,
        TokenType::NewLine => Operation::CheckForResult,
    }
}

#[cfg(test)]
mod general_tests {
    use crate::{Operation, Parser, Token, TokenType, Lexer};

    #[test]
    fn create_parser() {
        Parser::new();
    }

    #[test]
    fn assigns_initial_operations() {
        let pairs = [
            (TokenType::PlusSign, Operation::AbsoluteValue),
            (TokenType::MinusSign, Operation::Negation),
            (TokenType::MultiplicationSign, Operation::Multiplication),
            (TokenType::DivisionSign, Operation::Division),
            (TokenType::IntegerDivisionOperator, Operation::IntegerDivision),
            (TokenType::ModuloOperator, Operation::Modulo),
            (TokenType::ExponentialSign, Operation::Exponential),
            (TokenType::LogicalAndOperator, Operation::LogicalAnd),
            (TokenType::LogicalOrOperator, Operation::LogicalOr),
            (TokenType::LogicalXorOperator, Operation::LogicalXor),
            (TokenType::NotOperator, Operation::Not),
            (TokenType::TypeCastOperator, Operation::TypeCast),
            (TokenType::EqualityOperator, Operation::Equality),
            (TokenType::InequalityOperator, Operation::Inequality),
            (TokenType::LessThanOperator, Operation::LessThan),
            (TokenType::LessThanOrEqualOperator, Operation::LessThanOrEqual),
            (TokenType::GreaterThanOperator, Operation::GreaterThan),
            (TokenType::GreaterThanOrEqualOperator, Operation::GreaterThanOrEqual),
            (TokenType::TypeComparisonOperator, Operation::TypeEqual),
            (TokenType::BitwiseAndOperator, Operation::BitwiseAnd),
            (TokenType::BitwiseOrOperator, Operation::BitwiseOr),
            (TokenType::BitwiseXorOperator, Operation::BitwiseXor),
            (TokenType::BitwiseLeftShiftOperator, Operation::BitwiseLeftShift),
            (TokenType::BitwiseRightShiftOperator, Operation::BitwiseRightShift),
            (TokenType::DotOperator, Operation::Access),
            (TokenType::RangeOperator, Operation::MakeRange),
            (TokenType::StartExclusiveRangeOperator, Operation::MakeStartExclusiveRange),
            (TokenType::EndExclusiveRangeOperator, Operation::MakeEndExclusiveRange),
            (TokenType::ExclusiveRangeOperator, Operation::MakeExclusiveRange),
            (TokenType::PairOperator, Operation::MakePair),
            (TokenType::LinkOperator, Operation::MakeLink),
            (TokenType::MultiIterationOperator, Operation::MultiIterate),
            (TokenType::ConditionalTrueOperator, Operation::InvokeIfTrue),
            (TokenType::ConditionalFalseOperator, Operation::InvokeIfFalse),
            (TokenType::ConditionalResultOperator, Operation::ResultCheckInvoke),
            (TokenType::SymbolOperator, Operation::Literal),
//            (TokenType::StartExpression, Operation::Literal), requires additional tokens to be valid
//            (TokenType::EndExpression, Operation::NoOp), requires additional tokens to be valid
//            (TokenType::StartGroup, Operation::Literal), requires additional tokens to be valid
//            (TokenType::EndGroup, Operation::NoOp), requires additional tokens to be valid
            (TokenType::Result, Operation::Literal),
            (TokenType::Input, Operation::Literal),
            (TokenType::Comma, Operation::NoOp),
            (TokenType::UnitLiteral, Operation::Literal),
            (TokenType::ApplyOperator, Operation::Apply),
            (TokenType::PartiallyApplyOperator, Operation::PartiallyApply),
            (TokenType::PipeOperator, Operation::PipeApply),
            (TokenType::InfixOperator, Operation::NoOp),
            (TokenType::IterationOperator, Operation::Iterate),
            (TokenType::SingleValueIterationOperator, Operation::IterateToSingleValue),
            (TokenType::ReverseIterationOperator, Operation::ReverseIterate),
            (TokenType::SingleValueReverseIterationOperator, Operation::ReversIterateToSingleValue),
            (TokenType::IterationOutput, Operation::IterationOutput),
            (TokenType::IterationSkip, Operation::IterationSkip),
            (TokenType::IterationContinue, Operation::IterationContinue),
            (TokenType::IterationComplete, Operation::IterationComplete),
            (TokenType::Character, Operation::Literal),
            (TokenType::CharacterList, Operation::Literal),
            (TokenType::Number, Operation::Literal),
            (TokenType::Identifier, Operation::Literal),
//            (TokenType::HorizontalSpace, Operation::NoOp), trimmed when by itself
//            (TokenType::NewLine, Operation::NoOp), trimmed when by itself
        ];

        for pair in pairs.iter() {
            let input = vec![Token {
                value: String::from(""),
                token_type: pair.0
            }];

            let parser = Parser::new();
            let result = parser.make_groups(&input).unwrap().nodes;

            assert_eq!(result.get(0).unwrap().operation, pair.1);
        }
    }

    #[test]
    fn parsing_creates_initial_links() {
        let input = Lexer::new().lex("5 + my_value").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node_1 = result.nodes.get(0).unwrap();
        assert_eq!(node_1.left, None);
        assert_eq!(node_1.right, Some(2));

        let node_2 = result.nodes.get(2).unwrap();
        assert_eq!(node_2.left, Some(0));
        assert_eq!(node_2.right, Some(4));

        let node_3 = result.nodes.get(4).unwrap();
        assert_eq!(node_3.left, Some(2));
        assert_eq!(node_3.right, None);
    }
}

#[cfg(test)]
mod group_tests {
    use crate::{Parser, Lexer};

    #[test]
    fn parenthesis_cause_group_to_be_created_with_proper_links() {
        let input = Lexer::new().lex("5 + (5 - 10) + my_value").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        assert_eq!(*result.groups.get(0).unwrap(), 4);

        // only check the links that should be non-sequential

        // start group right should now reference first space after end group
        let node_5 = result.nodes.get(4).unwrap();
        assert_eq!(node_5.left, Some(2));
        assert_eq!(node_5.right, Some(12));

        // first space after end group should now reference start group
        let node_11 = result.nodes.get(12).unwrap();
        assert_eq!(node_11.left, Some(4));
        assert_eq!(node_11.right, Some(14));
    }

    #[test]
    fn braces_cause_group_to_be_created_with_proper_links() {
        let input = Lexer::new().lex("5 + {5 - 10} + my_value").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        assert_eq!(*result.groups.get(0).unwrap(), 4);

        // only check the links that should be non-sequential

        // start group right should now reference addition sign after end group
        let node_5 = result.nodes.get(4).unwrap();
        assert_eq!(node_5.left, Some(2));
        assert_eq!(node_5.right, Some(12));

        // addition sign after end group should now reference start group
        let node_11 = result.nodes.get(12).unwrap();
        assert_eq!(node_11.left, Some(4));
        assert_eq!(node_11.right, Some(14));
    }

    #[test]
    fn brace_and_parenthesis_result_in_error() {
        let input = Lexer::new().lex("5 + {5 - 10) + my_value").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input);

        assert_eq!(result.err().unwrap().get_message(), "Expected end of expression, found end of group.");
    }

    #[test]
    fn parenthesis_braces_result_in_error() {
        let input = Lexer::new().lex("5 + (5 - 10} + my_value").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input);

        assert_eq!(result.err().unwrap().get_message(), "Expected end of group, found end of expression.");
    }

    #[test]
    fn only_group_has_none_references_on_start_group_node() {
        let input = Lexer::new().lex("(5 - 10)").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        // start group right should now reference first space after end group
        let node = result.nodes.get(0).unwrap();
        assert_eq!(node.left, None);
        assert_eq!(node.right, None);
    }

    #[test]
    fn unclosed_group_results_in_error() {
        let input = Lexer::new().lex("(5 - 10").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input);

        assert_eq!(result.err().unwrap().get_message(), "Unclosed group starting at 0.");
    }

    #[test]
    fn unstarted_group_results_in_error() {
        let input = Lexer::new().lex("5 - 10)").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input);

        assert_eq!(result.err().unwrap().get_message(), "End of group found at 5 but no start preceded it.");
    }

    #[test]
    fn nested_group_creates_links() {
        let input = Lexer::new().lex("(5 + (5 - 10) - 9) + 4").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node_1 = result.nodes.get(0).unwrap();
        assert_eq!(node_1.left, None);
        assert_eq!(node_1.right, Some(18));

        let node_6 = result.nodes.get(5).unwrap();
        assert_eq!(node_6.left, Some(3));
        assert_eq!(node_6.right, Some(13));

        let node_13 = result.nodes.get(13).unwrap();
        assert_eq!(node_13.left, Some(5));
        assert_eq!(node_13.right, Some(15));

        let node_18 = result.nodes.get(18).unwrap();
        assert_eq!(node_18.left, Some(0));
        assert_eq!(node_18.right, Some(20));
    }
}

#[cfg(test)]
mod subexpression_tests {
    use crate::{Lexer, Parser, Operation};

    #[test]
    fn first_subexpression_starts_at_zero() {
        let input = Lexer::new().lex("5 + 4").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        assert_eq!(*result.sub_expressions.get(0).unwrap(), 0);
    }

    #[test]
    fn first_subexpression_starts_after_horizontal_space() {
        let input = Lexer::new().lex(" 5 + 4").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        assert_eq!(*result.sub_expressions.get(0).unwrap(), 0);
    }

    #[test]
    fn newlines_are_ignored_at_start_and_end_of_input() {
        let input = Lexer::new().lex(" \n\n 5 + 4 \n\n ").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        assert_eq!(result.sub_expressions.len(), 1);
    }

    #[test]
    fn double_new_line_between_tokens_converts_to_sub_expression() {
        let input = Lexer::new().lex("5 + 4\n\n10 + 6").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node_6 = result.nodes.get(5).unwrap();
        assert_eq!(node_6.operation, Operation::OutputResult);
        assert_eq!(node_6.left, Some(4));
        assert_eq!(node_6.right, Some(6));

        assert_eq!(*result.sub_expressions.get(0).unwrap(), 0);
        assert_eq!(*result.sub_expressions.get(1).unwrap(), 7);
    }

    #[test]
    fn single_new_line_between_tokens_converts_to_expression_if_after_new_line_is_non_terminable() {
        let input = Lexer::new().lex("5 + 4\n+ 6").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node_5 = result.nodes.get(4).unwrap();
        assert_eq!(node_5.left, Some(2));
        assert_eq!(node_5.right, Some(6));

        let node_6 = result.nodes.get(5).unwrap();
        assert_eq!(node_6.operation, Operation::NoOp);

        let node_7 = result.nodes.get(6).unwrap();
        assert_eq!(node_7.left, Some(4));
        assert_eq!(node_7.right, Some(8));

        assert_eq!(*result.sub_expressions.get(0).unwrap(), 0);
    }

    #[test]
    fn single_new_line_between_tokens_converts_to_expression_if_before_new_line_is_non_terminable() {
        let input = Lexer::new().lex("5 +\n4 + 6").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node_3 = result.nodes.get(2).unwrap();
        assert_eq!(node_3.left, Some(0));
        assert_eq!(node_3.right, Some(4));

        let node_4 = result.nodes.get(3).unwrap();
        assert_eq!(node_4.operation, Operation::NoOp);

        let node_5 = result.nodes.get(4).unwrap();
        assert_eq!(node_5.left, Some(2));
        assert_eq!(node_5.right, Some(6));

        assert_eq!(*result.sub_expressions.get(0).unwrap(), 0);
    }

    #[test]
    fn group_before_and_newline_before_unstartable_expression_is_a_single_expression() {
        let input = Lexer::new().lex("(5 + 4)\n + 6").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node_3 = result.nodes.get(0).unwrap();
        assert_eq!(node_3.left, None);
        assert_eq!(node_3.right, Some(9));

        let node_4 = result.nodes.get(9).unwrap();
        assert_eq!(node_4.left, Some(0));
        assert_eq!(node_4.right, Some(11));

        assert_eq!(*result.sub_expressions.get(0).unwrap(), 0);
    }

    #[test]
    fn group_after_newline_and_unterminated_expression_is_a_single_expression() {
        let input = Lexer::new().lex("6 + \n(5 + 4)").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node_3 = result.nodes.get(2).unwrap();
        assert_eq!(node_3.left, Some(0));
        assert_eq!(node_3.right, Some(5));

        let node_4 = result.nodes.get(5).unwrap();
        assert_eq!(node_4.left, Some(2));
        assert_eq!(node_4.right, None);

        assert_eq!(*result.sub_expressions.get(0).unwrap(), 0);
    }

    #[test]
    fn expression_before_and_newline_before_unstartable_expression_is_a_single_expression() {
        let input = Lexer::new().lex("{5 + 4}\n + 6").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node_3 = result.nodes.get(0).unwrap();
        assert_eq!(node_3.left, None);
        assert_eq!(node_3.right, Some(9));

        let node_4 = result.nodes.get(9).unwrap();
        assert_eq!(node_4.left, Some(0));
        assert_eq!(node_4.right, Some(11));

        assert_eq!(*result.sub_expressions.get(0).unwrap(), 0);
    }

    #[test]
    fn expression_after_newline_and_unterminated_expression_is_a_single_expression() {
        let input = Lexer::new().lex("6 + \n{5 + 4}").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node_3 = result.nodes.get(2).unwrap();
        assert_eq!(node_3.left, Some(0));
        assert_eq!(node_3.right, Some(5));

        let node_4 = result.nodes.get(5).unwrap();
        assert_eq!(node_4.left, Some(2));
        assert_eq!(node_4.right, None);

        assert_eq!(*result.sub_expressions.get(0).unwrap(), 0);
    }

    #[test]
    fn two_terminated_expressions_separated_by_newline_are_separate_expressions() {
        let input = Lexer::new().lex("6 + 9\n5 + 4").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node_3 = result.nodes.get(5).unwrap();
        assert_eq!(node_3.operation, Operation::OutputResult);
        assert_eq!(node_3.left, Some(4));
        assert_eq!(node_3.right, Some(6));
        
        assert_eq!(*result.sub_expressions.get(0).unwrap(), 0);
        assert_eq!(*result.sub_expressions.get(1).unwrap(), 6);
    }
}

#[cfg(test)]
mod reassignment_tests {
    use crate::{Lexer, Parser, Operation};

    #[test]
    fn minus_sign_gets_reassigned_to_negation() {
        let input = Lexer::new().lex("5 + -4").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node_3 = result.nodes.get(4).unwrap();
        assert_eq!(node_3.operation, Operation::Negation);
    }

    #[test]
    fn minus_sign_remains_subtraction_if_value_is_before() {
        let input = Lexer::new().lex("5 - 4").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node_3 = result.nodes.get(2).unwrap();
        assert_eq!(node_3.operation, Operation::Subtraction);
    }

    #[test]
    fn minus_sign_is_negation_when_only_value() {
        let input = Lexer::new().lex("-4").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node_3 = result.nodes.get(0).unwrap();
        assert_eq!(node_3.operation, Operation::Negation);
    }
    
    #[test]
    fn plus_sign_gets_reassigned_to_absolute_value() {
        let input = Lexer::new().lex("5 - +4").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node_3 = result.nodes.get(4).unwrap();
        assert_eq!(node_3.operation, Operation::AbsoluteValue);
    }

    #[test]
    fn plus_sign_remains_addition_if_value_is_before() {
        let input = Lexer::new().lex("5 + 4").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node_3 = result.nodes.get(2).unwrap();
        assert_eq!(node_3.operation, Operation::Addition);
    }

    #[test]
    fn plus_sign_is_absolute_value_when_only_value() {
        let input = Lexer::new().lex("+4").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node_3 = result.nodes.get(0).unwrap();
        assert_eq!(node_3.operation, Operation::AbsoluteValue);
    }

    #[test]
    fn fix_operator_reassigned_to_prefix_when_before_an_identifier() {
        let input = Lexer::new().lex("`expr 5").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node_3 = result.nodes.get(0).unwrap();
        assert_eq!(node_3.operation, Operation::PrefixApply);
    }

    #[test]
    fn fix_operator_reassigned_to_suffix_when_after_an_identifier() {
        let input = Lexer::new().lex("5 expr`").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node_3 = result.nodes.get(3).unwrap();
        assert_eq!(node_3.operation, Operation::SuffixApply);
    }

    #[test]
    fn fix_operator_reassigned_to_infix_when_surrounding_an_identifier() {
        let input = Lexer::new().lex("5 `expr` 4").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node_3 = result.nodes.get(2).unwrap();
        assert_eq!(node_3.operation, Operation::InfixApply);

        let node_3 = result.nodes.get(4).unwrap();
        assert_eq!(node_3.operation, Operation::InfixApply);
    }

    #[test]
    fn prefix_followed_by_suffix_parse_correctly() {
        let input = Lexer::new().lex("`expr 4 + 5 expr`").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node_3 = result.nodes.get(0).unwrap();
        assert_eq!(node_3.operation, Operation::PrefixApply);

        let node_3 = result.nodes.get(10).unwrap();
        assert_eq!(node_3.operation, Operation::SuffixApply);
    }
}
