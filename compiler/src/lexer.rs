use log::trace;
use std::{collections::HashMap, iter, vec};

#[derive(Debug, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub enum TokenType {
    Unknown,
    PlusSign,
    MultiplicationSign,
    ExponentialSign,
    UnitLiteral,
    StartExpression,
    EndExpression,
    StartGroup,
    EndGroup,
    Result,
    Input,
    Comma,
    Symbol,
    Number,
    Identifier,
    HorizontalSpace,
    NewLine,
    Annotation,
    Apply,
    ApplyIfFalse,
    ApplyIfTrue,
    ApplyTo,
    Reapply,
    EmptyApply,
    Equality,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct LexerSymbolNode {
    value: char,
    token_type: Option<TokenType>,
    children: HashMap<char, LexerSymbolNode>,
}

impl LexerSymbolNode {
    pub fn get_value(&self) -> char {
        self.value
    }

    pub fn get_token_type(&self) -> Option<TokenType> {
        self.token_type
    }

    pub fn get_child(&self, key: &char) -> Option<&LexerSymbolNode> {
        self.children.get(key)
    }
}

#[derive(Debug, PartialOrd, Eq, PartialEq, Clone)]
pub struct LexerToken {
    text: String,
    token_type: TokenType,
    row: usize,
    column: usize,
}

pub fn create_symbol_tree(symbol_list: Vec<(&str, TokenType)>) -> LexerSymbolNode {
    let mut root = LexerSymbolNode {
        value: '\0',
        token_type: None,
        children: HashMap::new(),
    };

    for (characters, token_type) in symbol_list {
        let mut current = &mut root;
        let len = characters.len();

        for (i, c) in characters.chars().enumerate() {
            let last = i >= len - 1;
            if !current.children.contains_key(&c) {
                let t = match last {
                    true => Some(token_type),
                    false => None,
                };

                current.children.insert(
                    c,
                    LexerSymbolNode {
                        value: c,
                        token_type: t,
                        children: HashMap::new(),
                    },
                );
            } else {
                // has child
                if last {
                    // update token type
                    match current.children.get_mut(&c) {
                        Some(node) => {
                            node.token_type = Some(token_type);
                        }
                        None => unreachable!(),
                    }
                }
            }

            match current.children.get_mut(&c) {
                Some(child) => {
                    current = child;
                }
                None => unreachable!(),
            }
        }
    }

    root
}

#[derive(Debug, PartialOrd, Eq, PartialEq, Clone, Copy)]
enum LexingState {
    NoToken,
    Symbol,
    Spaces,
    NewLine,
    Number,
    Indentifier,
    Annotation,
}

fn start_token<'a>(
    c: char,
    current_symbol: &mut &'a LexerSymbolNode,
    symbol_tree: &'a LexerSymbolNode,
    state: &mut LexingState,
    current_characters: &mut String,
    current_token_type: &mut Option<TokenType>,
    token_start_column: &mut usize,
    token_start_row: &mut usize,
    text_column: usize,
    text_row: usize,
) {
    trace!("Beginning new token");
    *current_characters = String::new();
    *current_symbol = symbol_tree;
    *current_token_type = None;

    // start new token
    if current_symbol.get_child(&c).is_some() {
        *state = LexingState::Symbol;
        match current_symbol.get_child(&c) {
            None => unreachable!(),
            Some(node) => {
                current_characters.push(c);
                *current_token_type = node.token_type;
                *current_symbol = node;
            }
        }
        trace!("Symbol started");
    } else if c == ' ' || c == '\t' {
        current_characters.push(c);
        *state = LexingState::Spaces;
        *current_token_type = Some(TokenType::HorizontalSpace);
        trace!("Horizontal spaces started");
    } else if c.is_ascii_whitespace() {
        // any other white space
        current_characters.push(c);
        *state = LexingState::NewLine;
        *current_token_type = Some(TokenType::NewLine);
        trace!("New line started");
    } else if c.is_numeric() {
        current_characters.push(c);
        *state = LexingState::Number;
        *current_token_type = Some(TokenType::Number);
        trace!("Number started");
    } else if c == '\0' {
        *state = LexingState::NoToken;
        *current_token_type = None;
        trace!("Null character found, skipping.");
        return;
    } else if c.is_alphanumeric() || c == '_' {
        // catch all to create identifiers
        // any disallowed characters will have a blacklist made for them

        current_characters.push(c);
        *state = LexingState::Indentifier;
        *current_token_type = Some(TokenType::Identifier);
        trace!("Identifier started");
    } else if c == '@' {
        current_characters.push(c);
        *state = LexingState::Annotation;
        *current_token_type = Some(TokenType::Annotation);
    }

    *token_start_row = text_row;
    *token_start_column = text_column;
}

pub fn lex(input: &String) -> Result<Vec<LexerToken>, String> {
    trace!("Begining lexing");

    let mut tokens = vec![];
    let mut current_characters = String::new();

    let symbol_tree = create_symbol_tree(vec![
        ("+", TokenType::PlusSign),
        ("*", TokenType::MultiplicationSign),
        ("**", TokenType::ExponentialSign),
        ("()", TokenType::UnitLiteral),
        ("{", TokenType::StartExpression),
        ("}", TokenType::EndExpression),
        ("(", TokenType::StartGroup),
        (")", TokenType::EndGroup),
        ("$?", TokenType::Result),
        ("$", TokenType::Input),
        (",", TokenType::Comma),
        (":", TokenType::Symbol),
        ("~", TokenType::Apply),
        ("!>", TokenType::ApplyIfFalse),
        ("?>", TokenType::ApplyIfTrue),
        ("~>", TokenType::ApplyTo),
        ("~^", TokenType::Reapply),
        ("~~", TokenType::EmptyApply),
        ("==", TokenType::Equality),
    ]);
    let mut current_symbol = &symbol_tree;
    let mut current_token_type = None;

    let mut text_row = 0;
    let mut text_column = 0;
    let mut token_start_column = 0;
    let mut token_start_row = 0;

    let mut state = LexingState::NoToken;

    for c in input.chars().chain(iter::once('\0')) {
        trace!("Character {:?} at ({:?}, {:?})", c, text_column, text_row);

        let start_new = if state == LexingState::NoToken {
            true
        } else {
            trace!("Continuing token");

            // continue/end tokens
            match state {
                LexingState::NoToken => false,
                LexingState::Symbol => {
                    trace!("Continuing symbol");
                    match current_symbol.get_child(&c) {
                        Some(node) => {
                            // set 'current' values
                            current_characters.push(c);
                            current_token_type = node.token_type;
                            current_symbol = node;

                            false
                        }
                        None => {
                            trace!("Ending symbol");
                            true
                        }
                    }
                }
                LexingState::Number => {
                    if c.is_numeric() {
                        current_characters.push(c);
                        false
                    } else {
                        trace!("Ending number");
                        true
                    }
                }
                LexingState::Indentifier => {
                    if c.is_alphanumeric() || c == '_' {
                        current_characters.push(c);
                        false
                    } else {
                        trace!("Ending identifier");
                        true
                    }
                }
                LexingState::Spaces => {
                    if c != ' ' && c != '\t' {
                        trace!("Ending horizontal space");
                        true
                    } else {
                        current_characters.push(c);
                        false
                    }
                }
                LexingState::NewLine => {
                    // wrap coordinates to new line
                    text_column = 0;
                    text_row += 1;

                    // one new line charcter per token
                    // end immediately
                    true
                }
                LexingState::Annotation => {
                    // annotations continue until end of line
                    // for simplicity we include entire line as the token
                    if c == '\n' || c == '\0' {
                        true
                    } else {
                        current_characters.push(c);
                        false
                    }
                }
            }
        };

        if start_new {
            if state != LexingState::NoToken {
                tokens.push(LexerToken {
                    text: current_characters,
                    token_type: match current_token_type {
                        Some(t) => t,
                        None => Err(format!("No token type at row {:?} and column {:?}", text_row, text_column))?,
                    },
                    row: token_start_row,
                    // actual token is determined after current, minus 1 to make accurate
                    column: token_start_column,
                });
            }

            // set default for new if next token isn't a symbol
            state = LexingState::NoToken;
            current_characters = String::new();
            current_symbol = &symbol_tree;
            current_token_type = None;

            start_token(
                c,
                &mut current_symbol,
                &symbol_tree,
                &mut state,
                &mut current_characters,
                &mut current_token_type,
                &mut token_start_column,
                &mut &mut token_start_row,
                text_column,
                text_row,
            );
        }

        if c != '\0' {
            text_column += 1;
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::{lex, lexer::create_symbol_tree, LexerToken, TokenType};

    #[test]
    fn access_single_string() {
        let root = create_symbol_tree(vec![("+", TokenType::PlusSign)]);

        let child = root.get_child(&'+').unwrap();
        assert_eq!(child.get_value(), '+');
        assert_eq!(child.get_token_type(), Some(TokenType::PlusSign));
        assert!(child.children.is_empty());
    }

    #[test]
    fn nested_symbols() {
        let root = create_symbol_tree(vec![("*", TokenType::MultiplicationSign), ("**", TokenType::ExponentialSign)]);

        let child_1 = root.get_child(&'*').unwrap();
        let child_2 = child_1.get_child(&'*').unwrap();

        assert_eq!(child_1.value, '*');
        assert_eq!(child_1.token_type, Some(TokenType::MultiplicationSign));

        assert_eq!(child_2.value, '*');
        assert_eq!(child_2.token_type, Some(TokenType::ExponentialSign));
        assert!(child_2.children.is_empty());
    }

    #[test]
    fn nested_symbols_longer_first() {
        let root = create_symbol_tree(vec![("**", TokenType::ExponentialSign), ("*", TokenType::MultiplicationSign)]);

        let child_1 = root.get_child(&'*').unwrap();
        let child_2 = child_1.get_child(&'*').unwrap();

        assert_eq!(child_1.value, '*');
        assert_eq!(child_1.token_type, Some(TokenType::MultiplicationSign));

        assert_eq!(child_2.value, '*');
        assert_eq!(child_2.token_type, Some(TokenType::ExponentialSign));
        assert!(child_2.children.is_empty());
    }

    #[test]
    fn lex_single_one_character_symbol() {
        let result = lex(&"+".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "+".to_string(),
                token_type: TokenType::PlusSign,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn apply_if_true_symbol() {
        let result = lex(&"?>".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "?>".to_string(),
                token_type: TokenType::ApplyIfTrue,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn apply_if_false_symbol() {
        let result = lex(&"!>".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "!>".to_string(),
                token_type: TokenType::ApplyIfFalse,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn equality_symbol() {
        let result = lex(&"==".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "==".to_string(),
                token_type: TokenType::Equality,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn input_symbol() {
        let result = lex(&"$".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "$".to_string(),
                token_type: TokenType::Input,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn result_symbol() {
        let result = lex(&"$?".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "$?".to_string(),
                token_type: TokenType::Result,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn apply_symbol() {
        let result = lex(&"~".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "~".to_string(),
                token_type: TokenType::Apply,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn empty_apply_symbol() {
        let result = lex(&"~~".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "~~".to_string(),
                token_type: TokenType::EmptyApply,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn reapply_symbol() {
        let result = lex(&"~^".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "~^".to_string(),
                token_type: TokenType::Reapply,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn apply_to_symbol() {
        let result = lex(&"~>".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "~>".to_string(),
                token_type: TokenType::ApplyTo,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn symbol_symbol() {
        let result = lex(&":".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: ":".to_string(),
                token_type: TokenType::Symbol,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn start_group_symbol() {
        let result = lex(&"(".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "(".to_string(),
                token_type: TokenType::StartGroup,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn end_group_symbol() {
        let result = lex(&")".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: ")".to_string(),
                token_type: TokenType::EndGroup,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn start_expression_symbol() {
        let result = lex(&"{".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "{".to_string(),
                token_type: TokenType::StartExpression,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn end_expression_symbol() {
        let result = lex(&"}".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "}".to_string(),
                token_type: TokenType::EndExpression,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn comma_symbol() {
        let result = lex(&",".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: ",".to_string(),
                token_type: TokenType::Comma,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn unit_literal_symbol() {
        let result = lex(&"()".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "()".to_string(),
                token_type: TokenType::UnitLiteral,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn lex_three_one_character_symbol() {
        let result = lex(&"+++".to_string()).unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "+".to_string(),
                    token_type: TokenType::PlusSign,
                    column: 0,
                    row: 0
                },
                LexerToken {
                    text: "+".to_string(),
                    token_type: TokenType::PlusSign,
                    column: 1,
                    row: 0
                },
                LexerToken {
                    text: "+".to_string(),
                    token_type: TokenType::PlusSign,
                    column: 2,
                    row: 0
                }
            ]
        );
    }

    #[test]
    fn skip_null_characters() {
        let result = lex(&"+\0+\0+".to_string()).unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "+".to_string(),
                    token_type: TokenType::PlusSign,
                    column: 0,
                    row: 0
                },
                LexerToken {
                    text: "+".to_string(),
                    token_type: TokenType::PlusSign,
                    column: 1,
                    row: 0
                },
                LexerToken {
                    text: "+".to_string(),
                    token_type: TokenType::PlusSign,
                    column: 2,
                    row: 0
                }
            ]
        );
    }

    #[test]
    fn lex_three_one_character_symbol_with_spaces() {
        let result = lex(&"    +  \t  +\t\t\t\t\t\t+      ".to_string()).unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "    ".to_string(),
                    token_type: TokenType::HorizontalSpace,
                    column: 0,
                    row: 0
                },
                LexerToken {
                    text: "+".to_string(),
                    token_type: TokenType::PlusSign,
                    column: 4,
                    row: 0
                },
                LexerToken {
                    text: "  \t  ".to_string(),
                    token_type: TokenType::HorizontalSpace,
                    column: 5,
                    row: 0
                },
                LexerToken {
                    text: "+".to_string(),
                    token_type: TokenType::PlusSign,
                    column: 10,
                    row: 0
                },
                LexerToken {
                    text: "\t\t\t\t\t\t".to_string(),
                    token_type: TokenType::HorizontalSpace,
                    column: 11,
                    row: 0
                },
                LexerToken {
                    text: "+".to_string(),
                    token_type: TokenType::PlusSign,
                    column: 17,
                    row: 0
                },
                LexerToken {
                    text: "      ".to_string(),
                    token_type: TokenType::HorizontalSpace,
                    column: 18,
                    row: 0
                },
            ]
        );
    }

    #[test]
    fn lex_new_lines() {
        let result = lex(&"+\n+\n+\n".to_string()).unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "+".to_string(),
                    token_type: TokenType::PlusSign,
                    column: 0,
                    row: 0
                },
                LexerToken {
                    text: "\n".to_string(),
                    token_type: TokenType::NewLine,
                    column: 1,
                    row: 0
                },
                LexerToken {
                    text: "+".to_string(),
                    token_type: TokenType::PlusSign,
                    column: 0,
                    row: 1
                },
                LexerToken {
                    text: "\n".to_string(),
                    token_type: TokenType::NewLine,
                    column: 1,
                    row: 1
                },
                LexerToken {
                    text: "+".to_string(),
                    token_type: TokenType::PlusSign,
                    column: 0,
                    row: 2
                },
                LexerToken {
                    text: "\n".to_string(),
                    token_type: TokenType::NewLine,
                    column: 1,
                    row: 2
                },
            ]
        );
    }

    #[test]
    fn lex_numbers() {
        let result = lex(&"12345 67890".to_string()).unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "12345".to_string(),
                    token_type: TokenType::Number,
                    column: 0,
                    row: 0
                },
                LexerToken {
                    text: " ".to_string(),
                    token_type: TokenType::HorizontalSpace,
                    column: 5,
                    row: 0
                },
                LexerToken {
                    text: "67890".to_string(),
                    token_type: TokenType::Number,
                    column: 6,
                    row: 0
                },
            ]
        );
    }

    #[test]
    fn lex_numbers_with_symbol() {
        let result = lex(&"12345+67890".to_string()).unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "12345".to_string(),
                    token_type: TokenType::Number,
                    column: 0,
                    row: 0
                },
                LexerToken {
                    text: "+".to_string(),
                    token_type: TokenType::PlusSign,
                    column: 5,
                    row: 0
                },
                LexerToken {
                    text: "67890".to_string(),
                    token_type: TokenType::Number,
                    column: 6,
                    row: 0
                },
            ]
        );
    }

    #[test]
    fn lex_identifiers() {
        let result = lex(&"value_1 Value_2".to_string()).unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "value_1".to_string(),
                    token_type: TokenType::Identifier,
                    column: 0,
                    row: 0
                },
                LexerToken {
                    text: " ".to_string(),
                    token_type: TokenType::HorizontalSpace,
                    column: 7,
                    row: 0
                },
                LexerToken {
                    text: "Value_2".to_string(),
                    token_type: TokenType::Identifier,
                    column: 8,
                    row: 0
                },
            ]
        );
    }

    #[test]
    fn lex_identifiers_with_symbol() {
        let result = lex(&"value_1+Value_2".to_string()).unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "value_1".to_string(),
                    token_type: TokenType::Identifier,
                    column: 0,
                    row: 0
                },
                LexerToken {
                    text: "+".to_string(),
                    token_type: TokenType::PlusSign,
                    column: 7,
                    row: 0
                },
                LexerToken {
                    text: "Value_2".to_string(),
                    token_type: TokenType::Identifier,
                    column: 8,
                    row: 0
                },
            ]
        );
    }

    #[test]
    fn annotation() {
        let result = lex(&"@ This is a comment".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "@ This is a comment".to_string(),
                token_type: TokenType::Annotation,
                column: 0,
                row: 0
            },]
        );
    }

    #[test]
    fn annotation_no_space_after() {
        let result = lex(&"@This is a comment".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "@This is a comment".to_string(),
                token_type: TokenType::Annotation,
                column: 0,
                row: 0
            },]
        );
    }

    #[test]
    fn annotation_with_newline_and_identifier() {
        let result = lex(&"@This is a comment\nmy_value".to_string()).unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "@This is a comment".to_string(),
                    token_type: TokenType::Annotation,
                    column: 0,
                    row: 0
                },
                LexerToken {
                    text: "\n".to_string(),
                    token_type: TokenType::NewLine,
                    column: 18,
                    row: 0
                },
                LexerToken {
                    text: "my_value".to_string(),
                    token_type: TokenType::Identifier,
                    column: 0,
                    row: 1
                }
            ]
        );
    }
}
