use std::collections::HashMap;

use expr_lang_common::Result;

#[derive(Debug, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub enum TokenType {
    PlusSign,
    MinusSign,
    MultiplicationSign,
    DivisionSign,
    IntegerDivisionOperator,
    ModuloOperator,
    ExponentialSign,
    LogicalAndOperator,
    LogicalOrOperator,
    LogicalXorOperator,
    NotOperator,
    TypeCastOperator,
    EqualityOperator,
    InequalityOperator,
    LessThanOperator,
    LessThanOrEqualOperator,
    GreaterThanOperator,
    GreaterThanOrEqualOperator,
    TypeComparisonOperator,
    BitwiseAndOperator,
    BitwiseOrOperator,
    BitwiseXorOperator,
    BitwiseLeftShiftOperator,
    BitwiseRightShiftOperator,
    DotOperator,
    RangeOperator,
    StartExclusiveRangeOperator,
    EndExclusiveRangeOperator,
    ExclusiveRangeOperator,
    PairOperator,
    LinkOperator,
    MultiIterationOperator,
    ConditionalTrueOperator,
    ConditionalFalseOperator,
    ConditionalResultOperator,
    SymbolOperator,
    StartExpression,
    EndExpression,
    StartGroup,
    EndGroup,
    Result,
    Input,
    Comma,
    UnitLiteral,
    ApplyOperator,
    PartiallyApplyOperator,
    PipeOperator,
    InfixOperator,
    IterationOperator,
    SingleValueIterationOperator,
    ReverseIterationOperator,
    SingleValueReverseIterationOperator,
    IterationOutput,
    IterationSkip,
    IterationContinue,
    IterationComplete,
    Character,
    CharacterList,
    Number,
    Identifier,
    HorizontalSpace,
    NewLine,
}

#[derive(Debug, PartialOrd, PartialEq, Clone)]
pub struct Token {
    pub(crate) value: String,
    pub(crate) token_type: TokenType,
}

impl Token {
    pub fn get_value(&self) -> &String {
        &self.value
    }

    pub fn get_token_type(&self) -> TokenType {
        self.token_type
    }
}

#[derive(PartialOrd, PartialEq, Debug, Copy, Clone)]
enum LexerState {
    NotParsing,
    ParsingSymbol,
    ParsingCharacter,
    ParsingCharacterList,
    ParsingNumber,
    ParsingIdentifier,
    ParsingHorizontalSpace,
    ParsingVerticalSpace,
}

struct LexerNode {
    value: String,
    token_type: Option<TokenType>,
    children: HashMap<String, LexerNode>,
}

pub struct Lexer {
    children: HashMap<String, LexerNode>,
}

fn insert_node(
    chars: &Vec<char>,
    i: usize,
    s: String,
    children: &mut HashMap<String, LexerNode>,
    t: TokenType,
) {
    match chars.get(i) {
        Some(c) => {
            let cs = c.to_string(); // for debugging, current debugger can't show char values
            match children.get_mut(&cs) {
                Some(child) => {
                    if i + 1 >= chars.len() {
                        // no more children
                        // update this node's token type
                        child.token_type = Some(t.clone());
                        child.value = s.clone();
                    } else {
                        insert_node(chars, i + 1, s, &mut child.children, t);
                    }
                }
                None => {
                    // only assign token type to last node in chain
                    let mut child = if i + 1 >= chars.len() {
                        LexerNode {
                            value: s.clone(),
                            token_type: Some(t.clone()),
                            children: HashMap::new(),
                        }
                    } else {
                        LexerNode {
                            value: String::new(),
                            token_type: None,
                            children: HashMap::new(),
                        }
                    };

                    insert_node(chars, i + 1, s, &mut child.children, t);

                    children.insert(c.to_string(), child);
                }
            }
        }
        None => (),
    }
}

impl Lexer {
    pub fn new() -> Self {
        let mut children: HashMap<String, LexerNode> = HashMap::new();

        [
            ("+", TokenType::PlusSign),
            ("-", TokenType::MinusSign),
            ("*", TokenType::MultiplicationSign),
            ("**", TokenType::ExponentialSign),
            ("/", TokenType::DivisionSign),
            ("//", TokenType::IntegerDivisionOperator),
            ("%", TokenType::ModuloOperator),
            ("&&", TokenType::LogicalAndOperator),
            ("||", TokenType::LogicalOrOperator),
            ("^^", TokenType::LogicalXorOperator),
            ("!", TokenType::NotOperator),
            ("#>", TokenType::TypeCastOperator),
            ("==", TokenType::EqualityOperator),
            ("!=", TokenType::InequalityOperator),
            ("<", TokenType::LessThanOperator),
            ("<=", TokenType::LessThanOrEqualOperator),
            (">", TokenType::GreaterThanOperator),
            (">=", TokenType::GreaterThanOrEqualOperator),
            ("#=", TokenType::TypeComparisonOperator),
            ("&", TokenType::BitwiseAndOperator),
            ("|", TokenType::BitwiseOrOperator),
            ("^", TokenType::BitwiseXorOperator),
            ("<<", TokenType::BitwiseLeftShiftOperator),
            (">>", TokenType::BitwiseRightShiftOperator),
            (".", TokenType::DotOperator),
            ("..", TokenType::RangeOperator),
            (">..", TokenType::StartExclusiveRangeOperator),
            ("..<", TokenType::EndExclusiveRangeOperator),
            (">..<", TokenType::ExclusiveRangeOperator),
            ("=", TokenType::PairOperator),
            ("->", TokenType::LinkOperator),
            ("=>", TokenType::ConditionalTrueOperator),
            ("!>", TokenType::ConditionalFalseOperator),
            ("=?>", TokenType::ConditionalResultOperator),
            ("~", TokenType::ApplyOperator),
            ("~~", TokenType::PartiallyApplyOperator),
            ("~>", TokenType::PipeOperator),
            ("`", TokenType::InfixOperator),
            (":", TokenType::SymbolOperator),
            ("{", TokenType::StartExpression),
            ("}", TokenType::EndExpression),
            ("(", TokenType::StartGroup),
            (")", TokenType::EndGroup),
            ("?", TokenType::Result),
            ("$", TokenType::Input),
            ("()", TokenType::UnitLiteral),
            (",", TokenType::Comma),
            (">>>", TokenType::IterationOperator),
            (">>|", TokenType::SingleValueIterationOperator),
            ("|>>", TokenType::ReverseIterationOperator),
            ("|>|", TokenType::SingleValueReverseIterationOperator),
            ("<>>", TokenType::MultiIterationOperator),
            ("|>output", TokenType::IterationOutput),
            ("|>skip", TokenType::IterationSkip),
            ("|>continue", TokenType::IterationContinue),
            ("|>complete", TokenType::IterationComplete),
        ]
        .iter()
        .for_each(|(v, t)| {
            insert_node(
                &v.chars().map(|c| c).collect::<Vec<char>>(),
                0,
                String::from(*v),
                &mut children,
                *t,
            );
        });

        Lexer { children }
    }

    pub fn lex(&self, s: &str) -> Result<Vec<Token>> {
        let mut tokens = vec![];

        let mut last_node: Option<&LexerNode> = None;
        let mut last_children = &self.children;
        let mut state = LexerState::NotParsing;
        let mut counter = 0;
        let mut current_value = String::new();
        let mut current_char: Option<char> = None;

        let mut chars = s.chars().peekable();

        while chars.peek().is_some() {
            let c = match current_char {
                Some(c) => {
                    current_char = None;
                    c
                }
                None => match chars.next() {
                    Some(c) => c,
                    None => unreachable!(),
                },
            };
            let _cs = c.to_string();

            if state == LexerState::NotParsing {
                state = Lexer::state_from_char(c)
            }

            match state {
                LexerState::ParsingSymbol => {
                    let cs = c.to_string();
                    match last_children.get(&cs) {
                        // go down child chain until we don't find a child
                        Some(child) => {
                            last_node = Some(child);
                            last_children = &child.children;
                        }
                        // current character is not a part of the current token
                        // end the current token
                        None => self.match_node(&cs, last_node, |node, token_type| {
                            tokens.push(Token {
                                value: node.value.clone(),
                                token_type: token_type.clone(),
                            });

                            last_children = &self.children;

                            // haven't used current character yet
                            // start next token
                            match last_children.get(&cs) {
                                Some(child) => {
                                    last_node = Some(child);
                                    last_children = &child.children;
                                }
                                // no last node means we're not matching any known tokens
                                // effectively ignore the character
                                None => {
                                    current_char = Some(c);
                                    last_node = None;
                                    state = LexerState::NotParsing;
                                }
                            }
                        })?,
                    }
                }
                LexerState::ParsingCharacter => match c {
                    '\'' if counter == 0 => {
                        counter += 1;
                    }
                    '\'' if counter == 1 => {
                        // end of character
                        tokens.push(Token {
                            value: current_value,
                            token_type: TokenType::Character,
                        });

                        counter = 0;
                        current_value = String::new();
                        state = LexerState::NotParsing;
                    }
                    _ => {
                        current_value.push(c);
                        if current_value.len() > 1 {
                            return Err(
                                "Character literals can only contain a single character.".into()
                            );
                        }
                    }
                },
                LexerState::ParsingCharacterList => match c {
                    '"' if counter == 0 => {
                        counter += 1;
                    }
                    '"' if counter == 1 => {
                        // end of character
                        tokens.push(Token {
                            value: current_value,
                            token_type: TokenType::CharacterList,
                        });

                        counter = 0;
                        current_value = String::new();
                        state = LexerState::NotParsing;
                    }
                    _ => current_value.push(c),
                },
                LexerState::ParsingNumber => {
                    if c.is_numeric() {
                        current_value.push(c);
                    } else {
                        tokens.push(Token {
                            value: current_value,
                            token_type: TokenType::Number,
                        });

                        counter = 0;
                        current_value = String::new();
                        state = LexerState::NotParsing;
                        current_char = Some(c);
                    }
                }
                LexerState::ParsingIdentifier => {
                    if c.is_alphanumeric() || c == '_' {
                        current_value.push(c);
                    } else {
                        tokens.push(Token {
                            value: current_value,
                            token_type: TokenType::Identifier,
                        });

                        counter = 0;
                        current_value = String::new();
                        state = LexerState::NotParsing;
                        current_char = Some(c);
                    }
                }
                LexerState::ParsingHorizontalSpace => {
                    if c != ' ' && c != '\t' {
                        tokens.push(Token {
                            value: ' '.to_string(),
                            token_type: TokenType::HorizontalSpace,
                        });

                        counter = 0;
                        current_value = String::new();
                        state = LexerState::NotParsing;
                        current_char = Some(c);
                    }
                }
                LexerState::ParsingVerticalSpace => {
                    if c != '\n' {
                        counter = 0;
                        current_value = String::new();
                        state = LexerState::NotParsing;
                        current_char = Some(c);
                    } else if counter >= 2 {
                        // only lex two new lines
                        // ignore any others
                        counter += 1;
                    } else {
                        counter += 1;
                        tokens.push(Token {
                            value: "\n".into(),
                            token_type: TokenType::NewLine,
                        });
                    }
                }
                LexerState::NotParsing => unreachable!(),
            }
        }

        // for token that has last character of the input
        // need to perform clean up

        // single character tokens have't benn started yet
        // match on current_char to start it
        match current_char {
            Some(c) => {
                let _cs = c.to_string();

                current_value.push(c);

                if state == LexerState::NotParsing {
                    state = Lexer::state_from_char(c)
                }
            }
            None => (),
        }

        // finish up last token by pushing to tokens
        match state {
            LexerState::ParsingSymbol => {
                self.match_node(&current_value, last_node, |node, token_type| {
                    tokens.push(Token {
                        value: node.value.clone(),
                        token_type: token_type.clone(),
                    });
                })?;
            }
            LexerState::ParsingNumber => {
                tokens.push(Token {
                    value: current_value,
                    token_type: TokenType::Number,
                });
            }
            LexerState::ParsingIdentifier => {
                tokens.push(Token {
                    value: current_value,
                    token_type: TokenType::Identifier,
                });
            }
            // reaching here while parsing a character or character list means the closing quote wasn't present
            LexerState::ParsingCharacter => {
                return Err("Expected end of Character literal ('). Found end of input.".into())
            }
            LexerState::ParsingCharacterList => {
                return Err(
                    "Expected end of CharacterList literal (\"). Found end of input.".into(),
                )
            }
            LexerState::ParsingHorizontalSpace => {
                tokens.push(Token {
                    value: " ".into(),
                    token_type: TokenType::HorizontalSpace,
                });
            }
            LexerState::ParsingVerticalSpace => (),
            LexerState::NotParsing => (),
        }

        Ok(tokens)
    }

    fn state_from_char(c: char) -> LexerState {
        match c {
            '\'' => LexerState::ParsingCharacter,
            '"' => LexerState::ParsingCharacterList,
            x if x.is_numeric() => LexerState::ParsingNumber,
            x if x.is_alphabetic() || x == '_' => LexerState::ParsingIdentifier,
            ' ' | '\t' => LexerState::ParsingHorizontalSpace,
            '\n' => LexerState::ParsingVerticalSpace,
            _ => LexerState::ParsingSymbol,
        }
    }

    fn match_node<F>(&self, key: &str, node: Option<&LexerNode>, f: F) -> Result
        where
            F: FnOnce(&LexerNode, &TokenType) -> (),
    {
        match node {
            Some(node) => match &node.token_type {
                Some(token_type) => f(node, token_type),
                // No token type means we're in the middle of a token and trying to end it
                None => return Err("Unknown token".into()),
            },
            // last node could be single character node
            // which means it hasn't been started yet
            // pull from
            None => match self.children.get(&key.to_string()) {
                Some(child) => match &child.token_type {
                    Some(token_type) => f(child, token_type),
                    // No token type means we're in the middle of a token and trying to end it
                    None => return Err("Unknown token".into()),
                }
                // last character isn't a token
                // ignore it
                None => ()
            },
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Lexer, Token, TokenType};

    #[test]
    fn create_lexer() {
        Lexer::new();
    }

    #[test]
    fn empty_string_makes_empty_token_list() {
        assert_eq!(Lexer::new().lex("").unwrap(), vec![]);
    }

    #[test]
    fn lex_plus_sign() {
        let token = Lexer::new().lex("+").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("+"),
                token_type: TokenType::PlusSign,
            }
        );
    }

    #[test]
    fn lex_double_plus_sign() {
        let tokens = Lexer::new().lex("++").unwrap();
        let token_1 = tokens.get(0).unwrap().clone();
        let token_2 = tokens.get(1).unwrap().clone();

        assert_eq!(
            token_1,
            Token {
                value: String::from("+"),
                token_type: TokenType::PlusSign,
            }
        );
        assert_eq!(
            token_2,
            Token {
                value: String::from("+"),
                token_type: TokenType::PlusSign,
            }
        );
    }

    #[test]
    fn lex_minus_sign() {
        let token = Lexer::new().lex("-").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("-"),
                token_type: TokenType::MinusSign,
            }
        );
    }

    #[test]
    fn lex_asterisk() {
        let token = Lexer::new().lex("*").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("*"),
                token_type: TokenType::MultiplicationSign,
            }
        );
    }

    #[test]
    fn lex_double_asterisk() {
        let token = Lexer::new().lex("**").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("**"),
                token_type: TokenType::ExponentialSign,
            }
        );
    }

    #[test]
    fn lex_division_sign() {
        let token = Lexer::new().lex("/").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("/"),
                token_type: TokenType::DivisionSign,
            }
        );
    }

    #[test]
    fn lex_integer_division_sign() {
        let token = Lexer::new().lex("//").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("//"),
                token_type: TokenType::IntegerDivisionOperator,
            }
        );
    }

    #[test]
    fn lex_modulo_operator() {
        let token = Lexer::new().lex("%").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("%"),
                token_type: TokenType::ModuloOperator,
            }
        );
    }

    #[test]
    fn lex_type_cast_operator() {
        let token = Lexer::new().lex("#>").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("#>"),
                token_type: TokenType::TypeCastOperator,
            }
        );
    }

    #[test]
    fn lex_and_logical_operator() {
        let token = Lexer::new().lex("&&").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("&&"),
                token_type: TokenType::LogicalAndOperator,
            }
        );
    }

    #[test]
    fn lex_or_logical_operator() {
        let token = Lexer::new().lex("||").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("||"),
                token_type: TokenType::LogicalOrOperator,
            }
        );
    }

    #[test]
    fn lex_xor_logical_operator() {
        let token = Lexer::new().lex("^^").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("^^"),
                token_type: TokenType::LogicalXorOperator,
            }
        );
    }

    #[test]
    fn lex_not_operator() {
        let token = Lexer::new().lex("!").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("!"),
                token_type: TokenType::NotOperator,
            }
        );
    }

    #[test]
    fn lex_equality_operator() {
        let token = Lexer::new().lex("==").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("=="),
                token_type: TokenType::EqualityOperator,
            }
        );
    }

    #[test]
    fn lex_inequality_operator() {
        let token = Lexer::new().lex("!=").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("!="),
                token_type: TokenType::InequalityOperator,
            }
        );
    }

    #[test]
    fn lex_less_than_operator() {
        let token = Lexer::new().lex("<").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("<"),
                token_type: TokenType::LessThanOperator,
            }
        );
    }

    #[test]
    fn lex_less_than_or_equal_operator() {
        let token = Lexer::new().lex("<=").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("<="),
                token_type: TokenType::LessThanOrEqualOperator,
            }
        );
    }

    #[test]
    fn lex_greater_than_operator() {
        let token = Lexer::new().lex(">").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from(">"),
                token_type: TokenType::GreaterThanOperator,
            }
        );
    }

    #[test]
    fn lex_greater_than_or_equal_operator() {
        let token = Lexer::new().lex(">=").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from(">="),
                token_type: TokenType::GreaterThanOrEqualOperator,
            }
        );
    }

    #[test]
    fn lex_type_comparison_operator() {
        let token = Lexer::new().lex("#=").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("#="),
                token_type: TokenType::TypeComparisonOperator,
            }
        );
    }

    #[test]
    fn lex_bitwise_and_operator() {
        let token = Lexer::new().lex("&").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("&"),
                token_type: TokenType::BitwiseAndOperator,
            }
        );
    }

    #[test]
    fn lex_bitwise_or_operator() {
        let token = Lexer::new().lex("|").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("|"),
                token_type: TokenType::BitwiseOrOperator,
            }
        );
    }

    #[test]
    fn lex_bitwise_xor_operator() {
        let token = Lexer::new().lex("^").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("^"),
                token_type: TokenType::BitwiseXorOperator,
            }
        );
    }

    #[test]
    fn lex_bitwise_left_shift_operator() {
        let token = Lexer::new().lex("<<").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("<<"),
                token_type: TokenType::BitwiseLeftShiftOperator,
            }
        );
    }

    #[test]
    fn lex_bitwise_right_shift_operator() {
        let token = Lexer::new().lex(">>").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from(">>"),
                token_type: TokenType::BitwiseRightShiftOperator,
            }
        );
    }

    #[test]
    fn lex_dot_operator() {
        let token = Lexer::new().lex(".").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("."),
                token_type: TokenType::DotOperator,
            }
        );
    }

    #[test]
    fn lex_range_operator() {
        let token = Lexer::new().lex("..").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from(".."),
                token_type: TokenType::RangeOperator,
            }
        );
    }

    #[test]
    fn lex_start_exclusive_operator() {
        let token = Lexer::new().lex(">..").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from(">.."),
                token_type: TokenType::StartExclusiveRangeOperator,
            }
        );
    }

    #[test]
    fn lex_end_exclusive_operator() {
        let token = Lexer::new().lex("..<").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("..<"),
                token_type: TokenType::EndExclusiveRangeOperator,
            }
        );
    }

    #[test]
    fn lex_exclusive_range_operator() {
        let token = Lexer::new().lex(">..<").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from(">..<"),
                token_type: TokenType::ExclusiveRangeOperator,
            }
        );
    }

    #[test]
    fn lex_pair_operator() {
        let token = Lexer::new().lex("=").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("="),
                token_type: TokenType::PairOperator,
            }
        );
    }

    #[test]
    fn lex_link_operator() {
        let token = Lexer::new().lex("->").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("->"),
                token_type: TokenType::LinkOperator,
            }
        );
    }

    #[test]
    fn lex_conditional_true_operator() {
        let token = Lexer::new().lex("=>").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("=>"),
                token_type: TokenType::ConditionalTrueOperator,
            }
        );
    }

    #[test]
    fn lex_conditional_false_operator() {
        let token = Lexer::new().lex("!>").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("!>"),
                token_type: TokenType::ConditionalFalseOperator,
            }
        );
    }

    #[test]
    fn lex_conditional_result_operator() {
        let token = Lexer::new().lex("=?>").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("=?>"),
                token_type: TokenType::ConditionalResultOperator,
            }
        );
    }

    #[test]
    fn lex_apply_operator() {
        let token = Lexer::new().lex("~").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("~"),
                token_type: TokenType::ApplyOperator,
            }
        );
    }

    #[test]
    fn lex_partially_apply_operator() {
        let token = Lexer::new().lex("~~").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("~~"),
                token_type: TokenType::PartiallyApplyOperator,
            }
        );
    }

    #[test]
    fn lex_pipe_operator() {
        let token = Lexer::new().lex("~>").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("~>"),
                token_type: TokenType::PipeOperator,
            }
        );
    }

    #[test]
    fn lex_infix_operator() {
        let token = Lexer::new().lex("`").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("`"),
                token_type: TokenType::InfixOperator,
            }
        );
    }

    #[test]
    fn lex_symbol_operator() {
        let token = Lexer::new().lex(":").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from(":"),
                token_type: TokenType::SymbolOperator,
            }
        );
    }

    #[test]
    fn lex_start_expression() {
        let token = Lexer::new().lex("{").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("{"),
                token_type: TokenType::StartExpression,
            }
        );
    }

    #[test]
    fn lex_end_expression() {
        let token = Lexer::new().lex("}").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("}"),
                token_type: TokenType::EndExpression,
            }
        );
    }

    #[test]
    fn lex_start_group() {
        let token = Lexer::new().lex("(").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("("),
                token_type: TokenType::StartGroup,
            }
        );
    }

    #[test]
    fn lex_end_group() {
        let token = Lexer::new().lex(")").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from(")"),
                token_type: TokenType::EndGroup,
            }
        );
    }

    #[test]
    fn lex_result() {
        let token = Lexer::new().lex("?").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("?"),
                token_type: TokenType::Result,
            }
        );
    }

    #[test]
    fn lex_input() {
        let token = Lexer::new().lex("$").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("$"),
                token_type: TokenType::Input,
            }
        );
    }

    #[test]
    fn lex_comma() {
        let token = Lexer::new().lex(",").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from(","),
                token_type: TokenType::Comma,
            }
        );
    }

    #[test]
    fn lex_unit() {
        let token = Lexer::new().lex("()").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("()"),
                token_type: TokenType::UnitLiteral,
            }
        );
    }

    #[test]
    fn lex_iterate_operator() {
        let token = Lexer::new().lex(">>>").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from(">>>"),
                token_type: TokenType::IterationOperator,
            }
        );
    }

    #[test]
    fn lex_single_value_iteration_operator() {
        let token = Lexer::new().lex(">>|").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from(">>|"),
                token_type: TokenType::SingleValueIterationOperator,
            }
        );
    }

    #[test]
    fn lex_reverse_iteration_operator() {
        let token = Lexer::new().lex("|>>").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("|>>"),
                token_type: TokenType::ReverseIterationOperator,
            }
        );
    }

    #[test]
    fn lex_single_value_reverse_iteration_operator() {
        let token = Lexer::new().lex("|>|").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("|>|"),
                token_type: TokenType::SingleValueReverseIterationOperator,
            }
        );
    }

    #[test]
    fn lex_collection_iterate_operator() {
        let token = Lexer::new().lex("<>>").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("<>>"),
                token_type: TokenType::MultiIterationOperator,
            }
        );
    }

    #[test]
    fn lex_iteration_output_operator() {
        let token = Lexer::new()
            .lex("|>output")
            .unwrap()
            .get(0)
            .unwrap()
            .clone();
        assert_eq!(
            token,
            Token {
                value: String::from("|>output"),
                token_type: TokenType::IterationOutput,
            }
        );
    }

    #[test]
    fn lex_iteration_skip_operator() {
        let token = Lexer::new().lex("|>skip").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("|>skip"),
                token_type: TokenType::IterationSkip,
            }
        );
    }

    #[test]
    fn lex_iteration_continue_operator() {
        let token = Lexer::new()
            .lex("|>continue")
            .unwrap()
            .get(0)
            .unwrap()
            .clone();
        assert_eq!(
            token,
            Token {
                value: String::from("|>continue"),
                token_type: TokenType::IterationContinue,
            }
        );
    }

    #[test]
    fn lex_iteration_complete_operator() {
        let token = Lexer::new()
            .lex("|>complete")
            .unwrap()
            .get(0)
            .unwrap()
            .clone();
        assert_eq!(
            token,
            Token {
                value: String::from("|>complete"),
                token_type: TokenType::IterationComplete,
            }
        );
    }

    #[test]
    fn lex_invalid_symbol_returns_error() {
        let error = Lexer::new().lex("<>");
        assert!(error.is_err());
        assert_eq!(error.err().unwrap().get_message(), "Unknown token")
    }

    #[test]
    fn lex_character() {
        let token = Lexer::new().lex("'a'").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("a"),
                token_type: TokenType::Character,
            }
        );
    }

    #[test]
    fn lex_character_without_closing_quote() {
        let result = Lexer::new().lex("'a");
        assert_eq!(
            result.err().unwrap().get_message(),
            "Expected end of Character literal ('). Found end of input."
        );
    }

    #[test]
    fn lex_character_that_is_more_than_one_character() {
        let result = Lexer::new().lex("'ab'");
        assert_eq!(
            result.err().unwrap().get_message(),
            "Character literals can only contain a single character."
        );
    }

    #[test]
    fn lex_character_list() {
        let token = Lexer::new()
            .lex("\"The quick brown fox.\"")
            .unwrap()
            .get(0)
            .unwrap()
            .clone();
        assert_eq!(
            token,
            Token {
                value: String::from("The quick brown fox."),
                token_type: TokenType::CharacterList,
            }
        );
    }

    #[test]
    fn lex_character_list_without_closing_quote() {
        let result = Lexer::new().lex("\"The quick brown fox.");
        assert_eq!(
            result.err().unwrap().get_message(),
            "Expected end of CharacterList literal (\"). Found end of input."
        );
    }

    #[test]
    fn lex_number() {
        let token = Lexer::new()
            .lex("0123456789")
            .unwrap()
            .get(0)
            .unwrap()
            .clone();
        assert_eq!(
            token,
            Token {
                value: String::from("0123456789"),
                token_type: TokenType::Number,
            }
        );
    }

    #[test]
    fn lex_identifier() {
        let token = Lexer::new().lex("my_name").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("my_name"),
                token_type: TokenType::Identifier,
            }
        );
    }

    #[test]
    fn lex_space() {
        let token = Lexer::new().lex(" ").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from(" "),
                token_type: TokenType::HorizontalSpace,
            }
        );
    }

    #[test]
    fn lex_tab() {
        let token = Lexer::new().lex("\t").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from(" "),
                token_type: TokenType::HorizontalSpace,
            }
        );
    }

    #[test]
    fn lex_multiple_spaces_and_tabs_makes_only_own_toke() {
        let tokens = Lexer::new().lex("    \t\t\t\t     \t\t\t    ").unwrap();

        assert_eq!(
            tokens,
            [Token {
                value: String::from(" "),
                token_type: TokenType::HorizontalSpace,
            }]
        );
    }

    #[test]
    fn lex_new_line() {
        let token = Lexer::new().lex("\n").unwrap().get(0).unwrap().clone();
        assert_eq!(
            token,
            Token {
                value: String::from("\n"),
                token_type: TokenType::NewLine,
            }
        );
    }

    #[test]
    fn lex_maximum_of_two_new_lines_in_a_row() {
        let tokens = Lexer::new().lex("\n\n\n\n").unwrap();
        assert_eq!(
            tokens,
            [
                Token {
                    value: String::from("\n"),
                    token_type: TokenType::NewLine,
                },
                Token {
                    value: String::from("\n"),
                    token_type: TokenType::NewLine,
                }
            ]
        );
    }

    #[test]
    fn lex_mix_of_symbols_and_literals_with_no_space() {
        let tokens = Lexer::new().lex("5+my_value").unwrap();
        assert_eq!(
            tokens,
            [
                Token {
                    value: String::from("5"),
                    token_type: TokenType::Number,
                },
                Token {
                    value: String::from("+"),
                    token_type: TokenType::PlusSign,
                },
                Token {
                    value: String::from("my_value"),
                    token_type: TokenType::Identifier,
                }
            ]
        );
    }

    #[test]
    fn lex_multi_digit_number_in_group() {
        let tokens = Lexer::new().lex("(10 + 10)").unwrap();
        assert_eq!(
            tokens,
            [
                Token {
                    value: String::from("("),
                    token_type: TokenType::StartGroup,
                },
                Token {
                    value: String::from("10"),
                    token_type: TokenType::Number,
                },
                Token {
                    value: String::from(" "),
                    token_type: TokenType::HorizontalSpace,
                },
                Token {
                    value: String::from("+"),
                    token_type: TokenType::PlusSign,
                },
                Token {
                    value: String::from(" "),
                    token_type: TokenType::HorizontalSpace,
                },
                Token {
                    value: String::from("10"),
                    token_type: TokenType::Number,
                },
                Token {
                    value: String::from(")"),
                    token_type: TokenType::EndGroup,
                },
            ]
        );
    }

    #[test]
    fn lex_mix_of_symbols_and_literals_with_no_space_reverse() {
        let tokens = Lexer::new().lex("my_value+5").unwrap();
        assert_eq!(
            tokens,
            [
                Token {
                    value: String::from("my_value"),
                    token_type: TokenType::Identifier,
                },
                Token {
                    value: String::from("+"),
                    token_type: TokenType::PlusSign,
                },
                Token {
                    value: String::from("5"),
                    token_type: TokenType::Number,
                },
            ]
        );
    }

    #[test]
    fn lex_single_character_symbol_end_of_input() {
        let tokens = Lexer::new().lex("5+?").unwrap();
        assert_eq!(
            tokens,
            [
                Token {
                    value: String::from("5"),
                    token_type: TokenType::Number,
                },
                Token {
                    value: String::from("+"),
                    token_type: TokenType::PlusSign,
                },
                Token {
                    value: String::from("?"),
                    token_type: TokenType::Result,
                }
            ]
        );
    }

    #[test]
    fn lex_mix_of_symbols_and_literals_with_space() {
        let tokens = Lexer::new().lex("5 + my_value").unwrap();
        assert_eq!(
            tokens,
            [
                Token {
                    value: String::from("5"),
                    token_type: TokenType::Number,
                },
                Token {
                    value: String::from(" "),
                    token_type: TokenType::HorizontalSpace,
                },
                Token {
                    value: String::from("+"),
                    token_type: TokenType::PlusSign,
                },
                Token {
                    value: String::from(" "),
                    token_type: TokenType::HorizontalSpace,
                },
                Token {
                    value: String::from("my_value"),
                    token_type: TokenType::Identifier,
                }
            ]
        );
    }

    #[test]
    fn lex_mix_of_symbols_and_literals_with_spaces_and_newlines() {
        let tokens = Lexer::new()
            .lex(
                "5 + my_value\n\n\"The quick brown fox\" -> ' ' -> \"jumped over the lazy dog.\"\n",
            )
            .unwrap();
        assert_eq!(
            tokens,
            [
                Token {
                    value: String::from("5"),
                    token_type: TokenType::Number,
                },
                Token {
                    value: String::from(" "),
                    token_type: TokenType::HorizontalSpace,
                },
                Token {
                    value: String::from("+"),
                    token_type: TokenType::PlusSign,
                },
                Token {
                    value: String::from(" "),
                    token_type: TokenType::HorizontalSpace,
                },
                Token {
                    value: String::from("my_value"),
                    token_type: TokenType::Identifier,
                },
                Token {
                    value: String::from("\n"),
                    token_type: TokenType::NewLine,
                },
                Token {
                    value: String::from("\n"),
                    token_type: TokenType::NewLine,
                },
                Token {
                    value: String::from("The quick brown fox"),
                    token_type: TokenType::CharacterList,
                },
                Token {
                    value: String::from(" "),
                    token_type: TokenType::HorizontalSpace,
                },
                Token {
                    value: String::from("->"),
                    token_type: TokenType::LinkOperator,
                },
                Token {
                    value: String::from(" "),
                    token_type: TokenType::HorizontalSpace,
                },
                Token {
                    value: String::from(" "),
                    token_type: TokenType::Character,
                },
                Token {
                    value: String::from(" "),
                    token_type: TokenType::HorizontalSpace,
                },
                Token {
                    value: String::from("->"),
                    token_type: TokenType::LinkOperator,
                },
                Token {
                    value: String::from(" "),
                    token_type: TokenType::HorizontalSpace,
                },
                Token {
                    value: String::from("jumped over the lazy dog."),
                    token_type: TokenType::CharacterList,
                },
                Token {
                    value: String::from("\n"),
                    token_type: TokenType::NewLine,
                },
            ]
        );
    }
}
