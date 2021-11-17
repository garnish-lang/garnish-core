use log::trace;
use std::{collections::HashMap, iter, vec};

#[derive(Debug, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub enum TokenType {
    Unknown,
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
    LogicalNotOperator,
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
    BitwiseNotOperator,
    BitwiseLeftShiftOperator,
    BitwiseRightShiftOperator,
    DotOperator,
    RangeOperator,
    StartExclusiveRangeOperator,
    EndExclusiveRangeOperator,
    ExclusiveRangeOperator,
    PairOperator,
    LinkOperator,
    ConditionalTrueOperator,
    ConditionalFalseOperator,
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
    PrefixOperator,
    SuffixOperator,
    Character,
    CharacterList,
    Number,
    Identifier,
    HorizontalSpace,
    NewLine,
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

// fn insert_node(chars: &Vec<char>, i: usize, s: String, children: &mut HashMap<String, LexerSymbolNode>, t: TokenType) {
//     match chars.get(i) {
//         Some(c) => {
//             let cs = c.to_string(); // for debugging, current debugger can't show char values
//             match children.get_mut(&cs) {
//                 Some(child) => {
//                     if i + 1 >= chars.len() {
//                         // no more children
//                         // update this node's token type
//                         child.token_type = Some(t.clone());
//                         child.value = s.clone();
//                     } else {
//                         insert_node(chars, i + 1, s, &mut child.children, t);
//                     }
//                 }
//                 None => {
//                     // only assign token type to last node in chain
//                     let mut child = if i + 1 >= chars.len() {
//                         LexerSymbolNode {
//                             value: s.clone(),
//                             token_type: Some(t.clone()),
//                             children: HashMap::new(),
//                         }
//                     } else {
//                         LexerSymbolNode {
//                             value: String::new(),
//                             token_type: None,
//                             children: HashMap::new(),
//                         }
//                     };

//                     insert_node(chars, i + 1, s, &mut child.children, t);

//                     children.insert(c.to_string(), child);
//                 }
//             }
//         }
//         None => (),
//     }
// }

// fn create_symbol_tree() -> HashMap<String, LexerSymbolNode> {
//     let mut children: HashMap<String, LexerSymbolNode> = HashMap::new();

//     [
//         ("+", TokenType::PlusSign),
//         ("-", TokenType::MinusSign),
//         ("*", TokenType::MultiplicationSign),
//         ("**", TokenType::ExponentialSign),
//         ("/", TokenType::DivisionSign),
//         ("//", TokenType::IntegerDivisionOperator),
//         ("%", TokenType::ModuloOperator),
//         ("&&", TokenType::LogicalAndOperator),
//         ("||", TokenType::LogicalOrOperator),
//         ("^^", TokenType::LogicalXorOperator),
//         ("!!", TokenType::LogicalNotOperator),
//         ("#>", TokenType::TypeCastOperator),
//         ("==", TokenType::EqualityOperator),
//         ("!=", TokenType::InequalityOperator),
//         ("<", TokenType::LessThanOperator),
//         ("<=", TokenType::LessThanOrEqualOperator),
//         (">", TokenType::GreaterThanOperator),
//         (">=", TokenType::GreaterThanOrEqualOperator),
//         ("#=", TokenType::TypeComparisonOperator),
//         ("&", TokenType::BitwiseAndOperator),
//         ("|", TokenType::BitwiseOrOperator),
//         ("^", TokenType::BitwiseXorOperator),
//         ("!", TokenType::BitwiseNotOperator),
//         ("<<", TokenType::BitwiseLeftShiftOperator),
//         (">>", TokenType::BitwiseRightShiftOperator),
//         (".", TokenType::DotOperator),
//         ("..", TokenType::RangeOperator),
//         (">..", TokenType::StartExclusiveRangeOperator),
//         ("..<", TokenType::EndExclusiveRangeOperator),
//         (">..<", TokenType::ExclusiveRangeOperator),
//         ("=", TokenType::PairOperator),
//         ("->", TokenType::LinkOperator),
//         ("!>", TokenType::ConditionalFalseOperator),
//         ("?>", TokenType::ConditionalTrueOperator),
//         ("~", TokenType::ApplyOperator),
//         ("~~", TokenType::PartiallyApplyOperator),
//         ("~>", TokenType::PipeOperator),
//         (":", TokenType::SymbolOperator),
//         ("{", TokenType::StartExpression),
//         ("}", TokenType::EndExpression),
//         ("(", TokenType::StartGroup),
//         (")", TokenType::EndGroup),
//         ("$?", TokenType::Result),
//         ("$", TokenType::Input),
//         ("()", TokenType::UnitLiteral),
//         (",", TokenType::Comma),
//     ]
//     .iter()
//     .for_each(|(v, t)| {
//         insert_node(&v.chars().map(|c| c).collect::<Vec<char>>(), 0, String::from(*v), &mut children, *t);
//     });

//     return children;
// }

pub fn create_symbol_tree(symbol_list: Vec<(String, TokenType)>) -> LexerSymbolNode {
    let mut root = LexerSymbolNode {
        value: '\0',
        token_type: None,
        children: HashMap::new(),
    };

    for (characters, token_type) in symbol_list {
        let mut current = &mut root;

        for c in characters.chars() {
            if !current.children.contains_key(&c) {
                current.children.insert(
                    c,
                    LexerSymbolNode {
                        value: c,
                        token_type: Some(token_type),
                        children: HashMap::new(),
                    },
                );
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
}

pub fn lex(input: &String) -> Result<Vec<LexerToken>, String> {
    trace!("Begining lexing");

    let mut tokens = vec![];
    let mut current_characters = String::new();

    let symbol_tree = create_symbol_tree(vec![("+".to_string(), TokenType::PlusSign)]);
    let mut current_symbol = &symbol_tree;
    let mut current_token_type = None;

    let mut text_row = 0;
    let mut text_column = 0;
    let mut token_start_column = 0;

    let mut state = LexingState::NoToken;

    fn start_token<'a>(
        c: char,
        current_symbol: &mut &'a LexerSymbolNode,
        symbol_tree: &'a LexerSymbolNode,
        state: &mut LexingState,
        current_characters: &mut String,
        current_token_type: &mut Option<TokenType>,
        token_start_column: &mut usize,
        text_column: usize,
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
            *state = LexingState::Spaces;
            *current_token_type = Some(TokenType::HorizontalSpace);
            trace!("Horizontal spaces started");
        }

        *token_start_column = text_column;
    }

    for c in input.chars().chain(iter::once('\0')) {
        trace!("Character {:?} at ({:?}, {:?})", c, text_column, text_row);

        if state == LexingState::NoToken {
            start_token(
                c,
                &mut current_symbol,
                &symbol_tree,
                &mut state,
                &mut current_characters,
                &mut current_token_type,
                &mut token_start_column,
                text_column,
            );
        } else {
            trace!("Continuing token");

            // continue/end tokens
            let start_new = match state {
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
                            state = LexingState::NoToken;

                            // no sub token, end current
                            tokens.push(LexerToken {
                                text: current_characters,
                                token_type: match current_token_type {
                                    Some(t) => t,
                                    None => Err(format!("No token type at row {:?} and column {:?}", text_row, text_column))?,
                                },
                                row: text_row,
                                // actual token is determined after current, minus 1 to make accurate
                                column: token_start_column,
                            });

                            // set default for new if next token isn't a symbol
                            current_characters = String::new();
                            current_symbol = &symbol_tree;
                            current_token_type = None;

                            true
                        }
                    }
                }
                LexingState::Spaces => {
                    if c != ' ' && c != '\t' {
                        trace!("Ending horizontal space");
                        state = LexingState::NoToken;

                        // end of horizontal space, create token and start new
                        tokens.push(LexerToken {
                            text: " ".to_string(),
                            token_type: TokenType::HorizontalSpace,
                            row: text_row,
                            // actual token is determined after current, minus 1 to make accurate
                            column: token_start_column,
                        });

                        true
                    } else {
                        false
                    }
                }
            };

            if start_new {
                start_token(
                    c,
                    &mut current_symbol,
                    &symbol_tree,
                    &mut state,
                    &mut current_characters,
                    &mut current_token_type,
                    &mut token_start_column,
                    text_column,
                );
            }
        }

        text_column += 1;
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::{lex, lexer::create_symbol_tree, LexerToken, TokenType};

    #[test]
    fn access_single_string() {
        let root = create_symbol_tree(vec![("+".to_string(), TokenType::PlusSign)]);

        let child = root.get_child(&'+').unwrap();
        assert_eq!(child.get_value(), '+');
        assert_eq!(child.get_token_type(), Some(TokenType::PlusSign));
        assert!(child.children.is_empty());
    }

    #[test]
    fn nested_symbols() {
        let root = create_symbol_tree(vec![
            ("*".to_string(), TokenType::MultiplicationSign),
            ("**".to_string(), TokenType::ExponentialSign),
        ]);

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
    fn lex_three_one_character_symbol_with_spaces() {
        let result = lex(&"    +     +      +      ".to_string()).unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: " ".to_string(),
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
                    text: " ".to_string(),
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
                    text: " ".to_string(),
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
                    text: " ".to_string(),
                    token_type: TokenType::HorizontalSpace,
                    column: 18,
                    row: 0
                },
            ]
        );
    }
}
