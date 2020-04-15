use expr_lang_common::Result;
use std::fmt;

use crate::{Token, TokenType};

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum Classification {
    Group,
    Addition,
    Subtraction,
    Negation,
    AbsoluteValue,
    NoOp,
    Literal,
    Symbol,
    Decimal,
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
    ListSeparator,
    InvokeIfTrue,
    InvokeIfFalse,
    DefaultInvoke,
    ResultCheckInvoke,
    ConditionalContinuation,
    Apply,
    PartiallyApply,
    PipeApply,
    PrefixApply,
    SuffixApply,
    InfixApply,
    Iterate,
    IterateToSingleValue,
    ReverseIterate,
    ReverseIterateToSingleValue,
    MultiIterate,
    IterationOutput,
    IterationSkip,
    IterationContinue,
    IterationComplete,
    OutputResult,
    CheckForResult,
}

#[derive(Clone, PartialOrd, PartialEq)]
pub struct Node {
    pub(crate) classification: Classification,
    pub(crate) token: Token,
    pub(crate) left: Option<usize>,
    pub(crate) right: Option<usize>,
    pub(crate) parent: Option<usize>,
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}, {:?}, {:?}, {:?}, {:?}, {:?}", self.classification, self.token.token_type, self.token.value, self.parent, self.left, self.right)
    }
}

#[derive(Debug)]
pub struct ParseResult {
    pub(crate) nodes: Vec<Node>,
    pub(crate) groups: Vec<usize>,
    pub(crate) sub_expressions: Vec<usize>,
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

        if tokens.is_empty() {
            return Ok(ParseResult {
                nodes,
                groups,
                sub_expressions: vec![]
            });
        }
        let mut groups_stack: Vec<usize> = vec![];
        let mut conditional_groups_stack = vec![None]; // keep extra in vec so condtitionals in group 0 can be used
        let mut next_left = None;

        let mut last_token_type = TokenType::HorizontalSpace;
        let mut last_classification = Classification::NoOp;
        let mut check_for_result: Option<usize> = None;
        let mut check_for_list_separator: Option<usize> = None;
        let mut check_for_decimal: Option<usize> = None;
        let mut in_access = false;

        let trim_start = tokens.iter().position(non_white_space).unwrap_or(0);
        let trim_end = tokens.iter().rposition(non_white_space).unwrap_or(tokens.len());

        let trimmed_tokens = &tokens[trim_start..=trim_end];

        for (i, token) in trimmed_tokens.iter().enumerate() {
            let mut left = match next_left {
                Some(l) => {
                    // only use next left once
                    next_left = None;
                    Some(l)
                },
                None => if i as i32 - 1 >= 0 { Some(i - 1) } else { None }
            };

            let right = if i + 1 < trimmed_tokens.len() { Some(i + 1) } else { None };

            let _l = left.unwrap_or_default();

            let mut classification = initial_classification_from_token_type(token.token_type);

            let mut token_type = token.token_type;
            let in_group = match groups_stack.iter().peekable().peek() {
                Some(g) => match nodes.get(**g) {
                    Some(n) => n.token.token_type == TokenType::StartGroup,
                    None => unreachable!()
                }
                None => false
            };

            // check for result
            match check_for_result {
                Some(r) => {
                    if classification == Classification::Literal {
                        // this is a separate expression
                        // set check for result node to be have output result
                        match nodes.get_mut(r) {
                            Some(n) => {
                                n.classification = Classification::OutputResult;
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
                                n.classification = Classification::NoOp;
                            }
                            None => unreachable!()
                        }
                    }
                    check_for_result = None;
                }
                None => () // nothing to do
            }

            match check_for_list_separator {
                // previous node was a space with value on the left
                // if this node is a value the space is a list separator
                Some(r) => {
                    if classification == Classification::Literal || (in_group && token.token_type == TokenType::NewLine) {
                        // update space to be list separator
                        match nodes.get_mut(r) {
                            Some(n) => n.classification = Classification::ListSeparator,
                            None => unreachable!()
                        }

                        left = Some(r);
                    } else {
                        // update space's left's right to point to this node
                        let space_left = match nodes.get(r) {
                            Some(n) => {
                                left = n.left;
                                match n.left {
                                    Some(nl) => match nodes.get_mut(nl) {
                                        Some(n) => n.right = Some(i),
                                        None => unreachable!()
                                    }
                                    None => () // no left
                                }
                            }
                            None => unreachable!()
                        };
                    }
                    check_for_list_separator = None;
                }
                None => ()
            }

            // check to cancel access chain
            if in_access {
                if (last_token_type == TokenType::Number || last_token_type == TokenType::Identifier)
                    && token.token_type != TokenType::DotOperator {
                    in_access = false;
                } else if last_token_type == TokenType::DotOperator &&
                    token.token_type != TokenType::Number && token.token_type != TokenType::Identifier {
                    return Err(format!("Trailing access operator at {}.", i - 1).into());
                }
            }

            // Outputing a result isn't allowed in a group
            // so all newlines get space logic applied to them
            if in_group && token_type == TokenType::NewLine {
                token_type = TokenType::HorizontalSpace;
            }


            // match token type for special cases
            match token_type {
                TokenType::StartGroup | TokenType::StartExpression => {
                    groups.push(i);
                    groups_stack.push(i);
                    // keep conditional groups in sync
                    conditional_groups_stack.push(None);
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
                            
                            // remove conditional group that might've in this group
                            conditional_groups_stack.pop();
                        }
                        None => return Err(format!("End of group found at {} but no start preceded it.", i).into())
                    }
                }
                TokenType::ConditionalTrueOperator |
                TokenType::ConditionalResultOperator => {
                    conditional_groups_stack[groups_stack.len()] = Some(groups_stack.len());
                }
                TokenType::ConditionalFalseOperator => {
                    match left {
                        Some(l) => match nodes.get(l) {
                            Some(n) => if n.classification == Classification::ConditionalContinuation {
                                classification = Classification::DefaultInvoke;
                            } else {
                                conditional_groups_stack[groups_stack.len()] = Some(groups_stack.len());
                            }
                            None => unreachable!()
                        }
                        None => ()
                    }
                }
                TokenType::Comma => {
                    match conditional_groups_stack.get(groups_stack.len()) {
                        Some(item) => match item {
                            Some(g) => classification = Classification::ConditionalContinuation,
                            None => () // nothing to do
                        }
                        None => unreachable!() // conditional groups should be in sync with groups
                    }

                    conditional_groups_stack[groups_stack.len()] = None;
                }
                TokenType::MinusSign => {
                    match left {
                        Some(l) => match nodes.get(l) {
                            Some(n) => if n.classification != Classification::Literal {
                                classification = Classification::Negation;
                            }
                            None => unreachable!()
                        }
                        None => classification = Classification::Negation
                    }
                }
                TokenType::PlusSign => {
                    match left {
                        Some(l) => match nodes.get(l) {
                            Some(n) => if n.classification != Classification::Literal {
                                classification = Classification::AbsoluteValue;
                            }
                            None => unreachable!()
                        }
                        None => classification = Classification::AbsoluteValue
                    }
                }
                TokenType::Number => {
                    match check_for_decimal {
                        Some(d) => match nodes.get_mut(d) {
                            Some(n) => n.classification = Classification::Decimal,
                            None => unreachable!()
                        }
                        None => () // nothing to do
                    }
                }
                TokenType::DotOperator => {
                    match left {
                        Some(l) => match nodes.get(l) {
                            Some(n) => {
                                if n.token.token_type == TokenType::Number && !in_access {
                                    check_for_decimal = Some(i);
                                } else {
                                    // flag as access chain
                                    in_access = true;
                                }
                            }
                            None => unreachable!()
                        }
                        None => () // begining of input
                    }
                }
                TokenType::HorizontalSpace => {
                    // set next left to be this nodes left
                    next_left = left;
                    match left {
                        Some(l) => {
                            let is_literal = match nodes.get(l) {
                                Some(nl) => nl.classification == Classification::Literal && nl.token.token_type != TokenType::StartGroup && nl.token.token_type != TokenType::StartExpression,
                                None => false
                            };

                            if is_literal {
                                check_for_list_separator = Some(i);
                            } else {
                                match nodes.get_mut(l) {
                                    Some(n) => {
                                        n.right = right;
                                    }
                                    None => unreachable!()
                                }
                            }
                        }
                        None => () // end of input
                    }
                }
                TokenType::NewLine => {
                    if last_token_type == TokenType::NewLine {
                        // explicit starting new sub expression
                        // update this token to be a no op
                        // update previous token to be an output classification
                        // and push next token index to subexpressions if exists

                        classification = Classification::NoOp;

                        match nodes.get_mut(i - 1) {
                            Some(n) => {
                                n.classification = Classification::OutputResult;
                                n.right = right;
                            }
                            None => unreachable!()
                        }

                        next_left = Some(i - 1);

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

                        if last_classification == Classification::Literal {
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
                            classification = Classification::NoOp;
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
            last_classification = classification;

            nodes.push(Node {
               classification: last_classification,
                token: token.clone(),
                left,
                right,
                parent: None
            });
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

fn initial_classification_from_token_type(token_type: TokenType) -> Classification {
    match token_type {
        TokenType::Number => Classification::Literal,
        TokenType::Identifier => Classification::Literal,
        TokenType::HorizontalSpace => Classification::NoOp,
        TokenType::PlusSign => Classification::Addition,
        TokenType::MinusSign => Classification::Subtraction,
        TokenType::MultiplicationSign => Classification::Multiplication,
        TokenType::DivisionSign => Classification::Division,
        TokenType::IntegerDivisionOperator => Classification::IntegerDivision,
        TokenType::ModuloOperator => Classification::Modulo,
        TokenType::ExponentialSign => Classification::Exponential,
        TokenType::LogicalAndOperator => Classification::LogicalAnd,
        TokenType::LogicalOrOperator => Classification::LogicalOr,
        TokenType::LogicalXorOperator => Classification::LogicalXor,
        TokenType::NotOperator => Classification::Not,
        TokenType::TypeCastOperator => Classification::TypeCast,
        TokenType::EqualityOperator => Classification::Equality,
        TokenType::InequalityOperator => Classification::Inequality,
        TokenType::LessThanOperator => Classification::LessThan,
        TokenType::LessThanOrEqualOperator => Classification::LessThanOrEqual,
        TokenType::GreaterThanOperator => Classification::GreaterThan,
        TokenType::GreaterThanOrEqualOperator => Classification::GreaterThanOrEqual,
        TokenType::TypeComparisonOperator => Classification::TypeEqual,
        TokenType::BitwiseAndOperator => Classification::BitwiseAnd,
        TokenType::BitwiseOrOperator => Classification::BitwiseOr,
        TokenType::BitwiseXorOperator => Classification::BitwiseXor,
        TokenType::BitwiseLeftShiftOperator => Classification::BitwiseLeftShift,
        TokenType::BitwiseRightShiftOperator => Classification::BitwiseRightShift,
        TokenType::DotOperator => Classification::Access,
        TokenType::RangeOperator => Classification::MakeRange,
        TokenType::StartExclusiveRangeOperator => Classification::MakeStartExclusiveRange,
        TokenType::EndExclusiveRangeOperator => Classification::MakeEndExclusiveRange,
        TokenType::ExclusiveRangeOperator => Classification::MakeExclusiveRange,
        TokenType::PairOperator => Classification::MakePair,
        TokenType::LinkOperator => Classification::MakeLink,
        TokenType::MultiIterationOperator => Classification::MultiIterate,
        TokenType::ConditionalTrueOperator => Classification::InvokeIfTrue,
        TokenType::ConditionalFalseOperator => Classification::InvokeIfFalse,
        TokenType::ConditionalResultOperator => Classification::ResultCheckInvoke,
        TokenType::SymbolOperator => Classification::Symbol,
        TokenType::StartExpression => Classification::Literal,
        TokenType::EndExpression => Classification::NoOp,
        TokenType::StartGroup => Classification::Literal,
        TokenType::EndGroup => Classification::NoOp,
        TokenType::Result => Classification::Literal,
        TokenType::Input => Classification::Literal,
        TokenType::Comma => Classification::ListSeparator,
        TokenType::UnitLiteral => Classification::Literal,
        TokenType::ApplyOperator => Classification::Apply,
        TokenType::PartiallyApplyOperator => Classification::PartiallyApply,
        TokenType::PipeOperator => Classification::PipeApply,
        TokenType::InfixOperator => Classification::InfixApply,
        TokenType::PrefixOperator => Classification::PrefixApply,
        TokenType::SuffixOperator => Classification::SuffixApply,
        TokenType::IterationOperator => Classification::Iterate,
        TokenType::SingleValueIterationOperator => Classification::IterateToSingleValue,
        TokenType::ReverseIterationOperator => Classification::ReverseIterate,
        TokenType::SingleValueReverseIterationOperator => Classification::ReverseIterateToSingleValue,
        TokenType::IterationOutput => Classification::IterationOutput,
        TokenType::IterationSkip => Classification::IterationSkip,
        TokenType::IterationContinue => Classification::IterationContinue,
        TokenType::IterationComplete => Classification::IterationComplete,
        TokenType::Character => Classification::Literal,
        TokenType::CharacterList => Classification::Literal,
        TokenType::NewLine => Classification::NoOp,
    }
}

#[cfg(test)]
mod general_tests {
    use crate::{Classification, Parser, Token, TokenType, Lexer};

    #[test]
    fn create_parser() {
        Parser::new();
    }

    #[test]
    fn parse_empty() {
        let input = Lexer::new().lex("").unwrap();
        let result = Parser::new().make_groups(&input).unwrap();

        assert_eq!(result.nodes, vec![]);
    }

    #[test]
    fn assigns_initial_classifications() {
        let pairs = [
            (TokenType::PlusSign, Classification::AbsoluteValue),
            (TokenType::MinusSign, Classification::Negation),
            (TokenType::MultiplicationSign, Classification::Multiplication),
            (TokenType::DivisionSign, Classification::Division),
            (TokenType::IntegerDivisionOperator, Classification::IntegerDivision),
            (TokenType::ModuloOperator, Classification::Modulo),
            (TokenType::ExponentialSign, Classification::Exponential),
            (TokenType::LogicalAndOperator, Classification::LogicalAnd),
            (TokenType::LogicalOrOperator, Classification::LogicalOr),
            (TokenType::LogicalXorOperator, Classification::LogicalXor),
            (TokenType::NotOperator, Classification::Not),
            (TokenType::TypeCastOperator, Classification::TypeCast),
            (TokenType::EqualityOperator, Classification::Equality),
            (TokenType::InequalityOperator, Classification::Inequality),
            (TokenType::LessThanOperator, Classification::LessThan),
            (TokenType::LessThanOrEqualOperator, Classification::LessThanOrEqual),
            (TokenType::GreaterThanOperator, Classification::GreaterThan),
            (TokenType::GreaterThanOrEqualOperator, Classification::GreaterThanOrEqual),
            (TokenType::TypeComparisonOperator, Classification::TypeEqual),
            (TokenType::BitwiseAndOperator, Classification::BitwiseAnd),
            (TokenType::BitwiseOrOperator, Classification::BitwiseOr),
            (TokenType::BitwiseXorOperator, Classification::BitwiseXor),
            (TokenType::BitwiseLeftShiftOperator, Classification::BitwiseLeftShift),
            (TokenType::BitwiseRightShiftOperator, Classification::BitwiseRightShift),
            (TokenType::DotOperator, Classification::Access),
            (TokenType::RangeOperator, Classification::MakeRange),
            (TokenType::StartExclusiveRangeOperator, Classification::MakeStartExclusiveRange),
            (TokenType::EndExclusiveRangeOperator, Classification::MakeEndExclusiveRange),
            (TokenType::ExclusiveRangeOperator, Classification::MakeExclusiveRange),
            (TokenType::PairOperator, Classification::MakePair),
            (TokenType::LinkOperator, Classification::MakeLink),
            (TokenType::MultiIterationOperator, Classification::MultiIterate),
            (TokenType::ConditionalTrueOperator, Classification::InvokeIfTrue),
            (TokenType::ConditionalFalseOperator, Classification::InvokeIfFalse),
            (TokenType::ConditionalResultOperator, Classification::ResultCheckInvoke),
            (TokenType::SymbolOperator, Classification::Symbol),
//            (TokenType::StartExpression, Classification::Literal), requires additional tokens to be valid
//            (TokenType::EndExpression, Classification::NoOp), requires additional tokens to be valid
//            (TokenType::StartGroup, Classification::Literal), requires additional tokens to be valid
//            (TokenType::EndGroup, Classification::NoOp), requires additional tokens to be valid
            (TokenType::Result, Classification::Literal),
            (TokenType::Input, Classification::Literal),
            (TokenType::Comma, Classification::ListSeparator),
            (TokenType::UnitLiteral, Classification::Literal),
            (TokenType::ApplyOperator, Classification::Apply),
            (TokenType::PartiallyApplyOperator, Classification::PartiallyApply),
            (TokenType::PipeOperator, Classification::PipeApply),
            (TokenType::InfixOperator, Classification::InfixApply),
            (TokenType::PrefixOperator, Classification::PrefixApply),
            (TokenType::SuffixOperator, Classification::SuffixApply),
            (TokenType::IterationOperator, Classification::Iterate),
            (TokenType::SingleValueIterationOperator, Classification::IterateToSingleValue),
            (TokenType::ReverseIterationOperator, Classification::ReverseIterate),
            (TokenType::SingleValueReverseIterationOperator, Classification::ReverseIterateToSingleValue),
            (TokenType::IterationOutput, Classification::IterationOutput),
            (TokenType::IterationSkip, Classification::IterationSkip),
            (TokenType::IterationContinue, Classification::IterationContinue),
            (TokenType::IterationComplete, Classification::IterationComplete),
            (TokenType::Character, Classification::Literal),
            (TokenType::CharacterList, Classification::Literal),
            (TokenType::Number, Classification::Literal),
            (TokenType::Identifier, Classification::Literal),
//            (TokenType::HorizontalSpace, Classification::NoOp), trimmed when by itself
//            (TokenType::NewLine, Classification::NoOp), trimmed when by itself
        ];

        for pair in pairs.iter() {
            let input = vec![Token {
                value: String::from(""),
                token_type: pair.0
            }];

            let parser = Parser::new();
            let result = parser.make_groups(&input).unwrap().nodes;

            assert_eq!(result.get(0).unwrap().classification, pair.1);
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
    use crate::{Lexer, Parser, Classification};

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
        assert_eq!(node_6.classification, Classification::OutputResult);
        assert_eq!(node_6.left, Some(4));
        assert_eq!(node_6.right, Some(7));

        let node = result.nodes.get(7).unwrap();
        assert_eq!(node.left, Some(5));

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
        assert_eq!(node_6.classification, Classification::NoOp);

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
        assert_eq!(node_4.classification, Classification::NoOp);

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
        assert_eq!(node_3.classification, Classification::OutputResult);
        assert_eq!(node_3.left, Some(4));
        assert_eq!(node_3.right, Some(6));
        
        assert_eq!(*result.sub_expressions.get(0).unwrap(), 0);
        assert_eq!(*result.sub_expressions.get(1).unwrap(), 6);
    }

    #[test]
    fn single_newline_in_group_is_treated_like_space() {
        let input = Lexer::new().lex("(5 + 4\n9 + 2)").unwrap();
        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node = result.nodes.get(6).unwrap();
        assert_eq!(node.classification, Classification::ListSeparator);
    }

    #[test]
    fn double_newline_in_group_is_treated_like_space() {
        let input = Lexer::new().lex("(5 + 4\n\n9 + 2)").unwrap();
        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node = result.nodes.get(6).unwrap();
        assert_eq!(node.classification, Classification::ListSeparator);

        let node = result.nodes.get(7).unwrap();
        assert_eq!(node.classification, Classification::NoOp);
    }
}

#[cfg(test)]
mod reassignment_tests {
    use crate::{Lexer, Parser, Classification};

    #[test]
    fn minus_sign_gets_reassigned_to_negation() {
        let input = Lexer::new().lex("5 + -4").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node_3 = result.nodes.get(4).unwrap();
        assert_eq!(node_3.classification, Classification::Negation);
    }

    #[test]
    fn minus_sign_remains_subtraction_if_value_is_before() {
        let input = Lexer::new().lex("5 - 4").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node_3 = result.nodes.get(2).unwrap();
        assert_eq!(node_3.classification, Classification::Subtraction);
    }

    #[test]
    fn minus_sign_is_negation_when_only_value() {
        let input = Lexer::new().lex("-4").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node_3 = result.nodes.get(0).unwrap();
        assert_eq!(node_3.classification, Classification::Negation);
    }
    
    #[test]
    fn plus_sign_gets_reassigned_to_absolute_value() {
        let input = Lexer::new().lex("5 - +4").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node_3 = result.nodes.get(4).unwrap();
        assert_eq!(node_3.classification, Classification::AbsoluteValue);
    }

    #[test]
    fn plus_sign_remains_addition_if_value_is_before() {
        let input = Lexer::new().lex("5 + 4").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node_3 = result.nodes.get(2).unwrap();
        assert_eq!(node_3.classification, Classification::Addition);
    }

    #[test]
    fn plus_sign_is_absolute_value_when_only_value() {
        let input = Lexer::new().lex("+4").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node_3 = result.nodes.get(0).unwrap();
        assert_eq!(node_3.classification, Classification::AbsoluteValue);
    }

    #[test]
    fn space_between_literals_on_same_line_is_list_separator() {
        let input = Lexer::new().lex("5 10 15").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node = result.nodes.get(1).unwrap();
        assert_eq!(node.left, Some(0));
        assert_eq!(node.right, Some(2));
        assert_eq!(node.classification, Classification::ListSeparator);

        let node = result.nodes.get(3).unwrap();
        assert_eq!(node.left, Some(2));
        assert_eq!(node.right, Some(4));
        assert_eq!(node.classification, Classification::ListSeparator);
    }

    #[test]
    fn space_after_open_group_and_before_close_group_are_not_list_separators() {
        for item in ["( 5 10 15 )", "{ 5 10 15 }"].iter() {
            let input = Lexer::new().lex(item).unwrap();

            let parser = Parser::new();
            let result = parser.make_groups(&input).unwrap();

            let node = result.nodes.get(1).unwrap();
            assert_eq!(node.classification, Classification::NoOp);

            let node = result.nodes.get(8).unwrap();
            assert_eq!(node.classification, Classification::NoOp);
        }
    }

    #[test]
    fn first_comma_in_same_group_after_conditional_is_conditional_continuation() {
        for text in ["=>", "!>", "=?>"].iter() {
            let input = Lexer::new().lex(&format!("value == 5 {} 100, value == 10 {} 1000", text, text)).unwrap();

            let parser = Parser::new();
            let result = parser.make_groups(&input).unwrap();

            let node = result.nodes.get(9).unwrap();
            assert_eq!(node.classification, Classification::ConditionalContinuation);
        }
    }

    #[test]
    fn commas_in_different_group_than_conditional_are_unchanged() {
        let input = Lexer::new().lex("value == 5 => (25, 50, 75, 100), value == 10 => 1000").unwrap();

        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node = result.nodes.get(10).unwrap();
        assert_eq!(node.classification, Classification::ListSeparator);

        let node = result.nodes.get(13).unwrap();
        assert_eq!(node.classification, Classification::ListSeparator);

        let node = result.nodes.get(16).unwrap();
        assert_eq!(node.classification, Classification::ListSeparator);

        let node = result.nodes.get(20).unwrap();
        assert_eq!(node.classification, Classification::ConditionalContinuation);
    }

    #[test]
    fn conditional_ends_with_group() {
        let input = Lexer::new().lex("(value == 5 => 100), (10, 20, 30)").unwrap();
        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node = result.nodes.get(11).unwrap();
        assert_eq!(node.classification, Classification::ListSeparator);

        let node = result.nodes.get(15).unwrap();
        assert_eq!(node.classification, Classification::ListSeparator);

        let node = result.nodes.get(18).unwrap();
        assert_eq!(node.classification, Classification::ListSeparator);
    }

    #[test]
    fn conditional_with_nested_conditional_chain_has_its_continuation_set() {
        let input = Lexer::new().lex("value == 5 => (x == 1 => :true, x == 2 => :false), value == 10 => 100").unwrap();
        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node = result.nodes.get(19).unwrap();
        assert_eq!(node.classification, Classification::ConditionalContinuation);

        let node = result.nodes.get(32).unwrap();
        assert_eq!(node.classification, Classification::ConditionalContinuation);
    }

    #[test]
    fn conditional_check_terminates_after_first_comma() {
        let input = Lexer::new().lex("value == 5 => 100, 10, 20, 30").unwrap();
        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node = result.nodes.get(9).unwrap();
        assert_eq!(node.classification, Classification::ConditionalContinuation);

        let node = result.nodes.get(12).unwrap();
        assert_eq!(node.classification, Classification::ListSeparator);
    }

    #[test]
    fn invoke_if_false_reassigned_to_default_if_at_end_of_chain_and_no_left_side() {
        let input = Lexer::new().lex("value == 5 => 100, !> 50").unwrap();
        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node = result.nodes.get(11).unwrap();
        assert_eq!(node.classification, Classification::DefaultInvoke);
    }

    #[test]
    fn conditional_continues_through_new_lines() {
        let input = Lexer::new().lex("value == 5 => 5 + \n4 - 3, value == 10 => 1000").unwrap();
        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node = result.nodes.get(18).unwrap();
        assert_eq!(node.classification, Classification::ConditionalContinuation);
    }

    #[test]
    fn dot_reassigned_to_decimal_when_between_numbers() {
        let input = Lexer::new().lex("3.14").unwrap();
        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node = result.nodes.get(1).unwrap();
        assert_eq!(node.classification, Classification::Decimal);
    }

    #[test]
    fn contiguous_dots_after_non_number_remain_access_classification() {
        let input = Lexer::new().lex("value.1.10.5").unwrap();
        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node = result.nodes.get(1).unwrap();
        assert_eq!(node.classification, Classification::Access);

        let node = result.nodes.get(3).unwrap();
        assert_eq!(node.classification, Classification::Access);

        let node = result.nodes.get(5).unwrap();
        assert_eq!(node.classification, Classification::Access);
    }

    #[test]
    fn dot_chain_ends_after_no_more_dots() {
        let input = Lexer::new().lex("value.1.10.5 10.5 20").unwrap();
        let parser = Parser::new();
        let result = parser.make_groups(&input).unwrap();

        let node = result.nodes.get(1).unwrap();
        assert_eq!(node.classification, Classification::Access);

        let node = result.nodes.get(3).unwrap();
        assert_eq!(node.classification, Classification::Access);

        let node = result.nodes.get(5).unwrap();
        assert_eq!(node.classification, Classification::Access);

        let node = result.nodes.get(9).unwrap();
        assert_eq!(node.classification, Classification::Decimal);
    }

    #[test]
    fn dot_chain_raises_error_if_ended_with_dot() {
        let input = Lexer::new().lex("value.1.10.5. + 5").unwrap();
        let parser = Parser::new();
        let result = parser.make_groups(&input);

        assert_eq!(result.err().unwrap().get_message(), "Trailing access operator at 7.");
    }
}
