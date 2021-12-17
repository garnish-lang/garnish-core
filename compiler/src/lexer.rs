use log::trace;
use std::{collections::HashMap, iter, vec};

use crate::{LexerAnnotationProcessor, LexerAnnotationProcessorInstruction, NoOpProcessor};

#[derive(Debug, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub enum TokenType {
    Unknown,
    PlusSign,
    AbsoluteValue,
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
    Whitespace,
    Subexpression,
    Annotation,
    LineAnnotation,
    Apply,
    ApplyIfFalse,
    ApplyIfTrue,
    DefaultConditional,
    ApplyTo,
    Reapply,
    EmptyApply,
    Equality,
    Period,
    LeftInternal,
    RightInternal,
    LengthInternal,
    Pair,
    False,
    True,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct LexerOperatorNode {
    value: char,
    token_type: Option<TokenType>,
    children: HashMap<char, LexerOperatorNode>,
}

impl LexerOperatorNode {
    pub fn get_value(&self) -> char {
        self.value
    }

    pub fn get_token_type(&self) -> Option<TokenType> {
        self.token_type
    }

    pub fn get_child(&self, key: &char) -> Option<&LexerOperatorNode> {
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

impl LexerToken {
    pub fn new(text: String, token_type: TokenType, row: usize, column: usize) -> LexerToken {
        LexerToken {
            text,
            token_type,
            row,
            column,
        }
    }

    pub fn empty() -> LexerToken {
        LexerToken {
            text: "".to_string(),
            token_type: TokenType::Unknown,
            row: 0,
            column: 0,
        }
    }

    pub fn get_text(&self) -> &String {
        &self.text
    }

    pub fn get_token_type(&self) -> TokenType {
        self.token_type
    }
}

pub fn create_operator_tree(symbol_list: Vec<(&str, TokenType)>) -> LexerOperatorNode {
    let mut root = LexerOperatorNode {
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
                    LexerOperatorNode {
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

fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == ':'
}

#[derive(Debug, PartialOrd, Eq, PartialEq, Clone, Copy)]
enum LexingState {
    NoToken,
    Operator,
    Spaces,
    Subexpression,
    Number,
    Indentifier,
    Annotation,
    LineAnnotation,
    Symbol,
}

fn start_token<'a>(
    c: char,
    current_operator: &mut &'a LexerOperatorNode,
    operator_tree: &'a LexerOperatorNode,
    state: &mut LexingState,
    current_characters: &mut String,
    current_token_type: &mut Option<TokenType>,
    token_start_column: &mut usize,
    token_start_row: &mut usize,
    text_column: &mut usize,
    text_row: &mut usize,
) {
    trace!("Beginning new token");
    *current_characters = String::new();
    *current_operator = operator_tree;
    *current_token_type = None;

    *token_start_row = *text_row;
    *token_start_column = *text_column;

    // start new token
    if current_operator.get_child(&c).is_some() {
        *state = LexingState::Operator;
        match current_operator.get_child(&c) {
            None => unreachable!(),
            Some(node) => {
                current_characters.push(c);
                *current_token_type = node.token_type;
                *current_operator = node;
            }
        }
    } else if c == ' ' || c == '\t' {
        current_characters.push(c);
        *state = LexingState::Spaces;
        *current_token_type = Some(TokenType::Whitespace);
    } else if c.is_ascii_whitespace() {
        // any other white space, all some form of new line
        current_characters.push(c);
        *state = LexingState::Subexpression;
        *current_token_type = Some(TokenType::Subexpression);

        *text_column = 0;
        *text_row += 1;
    } else if c.is_numeric() {
        current_characters.push(c);
        *state = LexingState::Number;
        *current_token_type = Some(TokenType::Number);
    } else if c == '\0' {
        *state = LexingState::NoToken;
        *current_token_type = None;
        trace!("Null character found, skipping.");
        return;
    } else if is_identifier_char(c) {
        // catch all to create identifiers
        // any disallowed characters will have a blacklist made for them

        current_characters.push(c);
        *state = LexingState::Indentifier;
        *current_token_type = Some(TokenType::Identifier);
    } else if c == '@' {
        current_characters.push(c);
        *state = LexingState::Annotation;
        *current_token_type = Some(TokenType::Annotation);
    } else if c == ';' {
        current_characters.push(c);
        *state = LexingState::Symbol;
        *current_token_type = Some(TokenType::Symbol);
    }

    trace!(
        "Starting token with character {:?} token type '{:?}' state '{:?}'",
        &c,
        current_token_type,
        state
    );
}

pub fn lex(input: &String) -> Result<Vec<LexerToken>, String> {
    lex_with_processor(input, &mut NoOpProcessor::new())
}

pub fn lex_with_processor<T: LexerAnnotationProcessor>(input: &String, processor: &mut T) -> Result<Vec<LexerToken>, String> {
    trace!("Beginning lexing");

    let mut tokens = vec![];
    let mut current_characters = String::new();

    let operator_tree = create_operator_tree(vec![
        ("+", TokenType::PlusSign),
        ("++", TokenType::AbsoluteValue),
        ("*", TokenType::MultiplicationSign),
        ("**", TokenType::ExponentialSign),
        ("()", TokenType::UnitLiteral),
        ("{", TokenType::StartExpression),
        ("}", TokenType::EndExpression),
        ("(", TokenType::StartGroup),
        (")", TokenType::EndGroup),
        ("$", TokenType::Input),
        ("$?", TokenType::True),
        ("$!", TokenType::False),
        ("$$", TokenType::Result),
        (",", TokenType::Comma),
        ("~", TokenType::Apply),
        ("!>", TokenType::ApplyIfFalse),
        ("?>", TokenType::ApplyIfTrue),
        ("|>", TokenType::DefaultConditional),
        ("~>", TokenType::ApplyTo),
        ("^~", TokenType::Reapply),
        ("~~", TokenType::EmptyApply),
        ("==", TokenType::Equality),
        ("=", TokenType::Pair),
        (".", TokenType::Period),
        ("._", TokenType::RightInternal),
        ("_.", TokenType::LeftInternal),
        (".|", TokenType::LengthInternal),
    ]);
    let mut current_operator = &operator_tree;
    let mut current_token_type = None;

    let mut text_row = 0;
    let mut text_column = 0;
    let mut token_start_column = 0;
    let mut token_start_row = 0;

    let mut should_create = true;
    let mut state = LexingState::NoToken;

    let mut processor_instruction = None;

    for c in input.chars().chain(iter::once('\0')) {
        trace!("Character {:?} at ({:?}, {:?})", c, text_column, text_row);
        trace!("Current state: {:?}", state);

        let start_new = match state {
            LexingState::NoToken => true,
            LexingState::Operator => {
                trace!("Continuing operator");
                match current_operator.get_child(&c) {
                    Some(node) => {
                        // set 'current' values
                        current_characters.push(c);
                        current_token_type = node.token_type;
                        current_operator = node;

                        trace!("Switched to operator token '{:?}'", current_token_type);

                        false
                    }
                    None => {
                        // One token (LeftInternal) starts with an underscore
                        // which can also be start of an identifier
                        // if on this token and current character can be identifier
                        // switch to lexing identifier
                        if current_characters == "_" && is_identifier_char(c) {
                            trace!("Switching to lexing identifier after starting with an underscore.");
                            current_characters.push(c);
                            current_token_type = Some(TokenType::Identifier);
                            state = LexingState::Indentifier;

                            false
                        } else {
                            trace!("Ending operator");
                            true
                        }
                    }
                }
            }
            LexingState::Symbol => {
                trace!("Continuing symbol");
                if current_characters.len() == 1 {
                    // just the colon character
                    // need to make sure the first character is only alpha or underscore
                    if is_identifier_char(c) {
                        current_characters.push(c);
                        false
                    } else {
                        // end token
                        true
                    }
                } else {
                    if is_identifier_char(c) {
                        current_characters.push(c);
                        false
                    } else {
                        // end token
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
                if is_identifier_char(c) {
                    current_characters.push(c);
                    false
                } else {
                    trace!("Ending identifier");

                    // check for invalid identifiers
                    // currently only need to check for single colon character ':'
                    if current_characters == ":" {
                        current_token_type = None;
                    }
                    true
                }
            }
            LexingState::Spaces => {
                if c == '\n' {
                    trace!("Found newline character, switching state");
                    current_characters.push(c);

                    state = LexingState::Subexpression;
                    // wrap coordinates to new line
                    text_column = 0;
                    text_row += 1;

                    false
                } else if c != ' ' && c != '\t' {
                    trace!("Ending horizontal space");
                    true
                } else {
                    current_characters.push(c);
                    false
                }
            }
            LexingState::Subexpression => {
                if c.is_ascii_whitespace() && !(c == '\t' || c == ' ') {
                    trace!("Found second new line character. Creating subexpression token");
                    // first time through will be the second new line type character

                    // add character and create token
                    current_characters.push(c);

                    // could've arrived here by passing through whitespace state
                    // check len of characters so see if 2 tokens need to be created
                    if current_characters.len() > 2 {
                        trace!("Creating whitespace token from extra characters");
                        // have extra characters, split them into a whitespace token
                        let spaces_characters = &current_characters[..current_characters.len() - 2];
                        tokens.push(LexerToken::new(
                            spaces_characters.to_string(),
                            TokenType::Whitespace,
                            token_start_row,
                            // actual token is determined after current, minus 1 to make accurate
                            token_start_column,
                        ));

                        current_characters = current_characters[(current_characters.len() - 2)..].to_string();
                    }

                    // wrap coordinates to new line
                    text_column = 0;
                    text_row += 1;

                    // skip start new token for this character since it is a part of this token
                    should_create = false;

                    true
                } else {
                    trace!("Non-newline character found");
                    // change to white space token and start new
                    current_token_type = Some(TokenType::Whitespace);

                    // if other white space, switch state and continue
                    if c == '\t' || c == ' ' {
                        trace!("Is whitespace character, switching state");
                        current_characters.push(c);
                        state = LexingState::Spaces;
                        false
                    } else {
                        trace!("Is non-whitespace character, ending token");
                        // else end this token and start new
                        true
                    }
                }
            }
            LexingState::Annotation => {
                // change to LineAnnotation if second character is also an '@'
                if c == '@' && current_characters.len() == 1 {
                    trace!("Another '@' character found, changing to LineAnnotation.");
                    current_characters.push(c);
                    state = LexingState::LineAnnotation;
                    current_token_type = Some(TokenType::LineAnnotation);
                    false
                } else if c.is_alphanumeric() || c == '_' {
                    // continue token
                    current_characters.push(c);
                    false
                } else {
                    // end token

                    match processor.yield_annotation(&current_characters, token_start_row, token_start_column) {
                        Err(e) => Err(format!("{:?}", e.to_string()))?,
                        Ok(v) => match v.get_instruction() {
                            LexerAnnotationProcessorInstruction::Drop => {
                                state = LexingState::NoToken;
                            }
                            LexerAnnotationProcessorInstruction::UntilToken => {
                                // processor is given chance to consume token so we drop it here
                                state = LexingState::NoToken;

                                processor_instruction = Some(v);
                            }
                            LexerAnnotationProcessorInstruction::NoOp => (),
                        },
                    }

                    true
                }
            }
            LexingState::LineAnnotation => {
                // line annotations continue until end of line
                // for simplicity we include entire line as the token
                if c == '\n' || c == '\0' {
                    match processor.yield_line_annotation(&current_characters, token_start_row, token_start_column) {
                        Err(e) => Err(format!("{:?}", e.to_string()))?,
                        Ok(v) => match v.get_instruction() {
                            LexerAnnotationProcessorInstruction::Drop => {
                                state = LexingState::NoToken;
                            }
                            LexerAnnotationProcessorInstruction::UntilToken => {
                                // processor is given chance to consume token so we drop it here
                                state = LexingState::NoToken;

                                processor_instruction = Some(v);
                            }
                            LexerAnnotationProcessorInstruction::NoOp => (),
                        },
                    }

                    true
                } else {
                    current_characters.push(c);
                    false
                }
            }
        };

        trace!("Will start new: {:?}", start_new);

        if start_new {
            if state != LexingState::NoToken {
                trace!("Pushing new token: {:?}", current_token_type);

                let token = LexerToken::new(
                    current_characters,
                    match current_token_type {
                        Some(t) => t,
                        None => Err(format!("No token type at row {:?} and column {:?}", text_row, text_column))?,
                    },
                    token_start_row,
                    // actual token is determined after current, minus 1 to make accurate
                    token_start_column,
                );

                // check for annotation processing before pushing token
                match processor_instruction {
                    None => {
                        tokens.push(token);
                    }
                    Some(info) => match info.get_instruction() {
                        // if yielding to processor, its assumed that the processor is taking ownership of the token
                        LexerAnnotationProcessorInstruction::UntilToken => {
                            // check now for borrow to happen before move to processor
                            if token.get_token_type() == info.get_token_type() {
                                // end yielding
                                processor_instruction = None;
                            }

                            processor.yield_token(token).or_else(|e| Err(format!("{:?}", e.to_string())))?;
                        }
                        // no other instruction is supported at this point
                        _ => tokens.push(token),
                    },
                }
            }

            // set default for new if next token isn't a symbol
            state = LexingState::NoToken;
            current_characters = String::new();
            current_operator = &operator_tree;
            current_token_type = None;

            if should_create {
                trace!("Starting new token");
                start_token(
                    c,
                    &mut current_operator,
                    &operator_tree,
                    &mut state,
                    &mut current_characters,
                    &mut current_token_type,
                    &mut token_start_column,
                    &mut &mut token_start_row,
                    &mut text_column,
                    &mut text_row,
                );
            } else {
                // Used only for single skips, flip back for next iteration
                trace!("Did not start new token");
                should_create = true;
            }
        }

        // need to relook at column count when deep diving into line feed, form feed, carriage return parsing
        if c != '\0' && c != '\n' {
            text_column += 1;
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::{lex, lexer::create_operator_tree, LexerToken, TokenType};

    #[test]
    fn access_single_string() {
        let root = create_operator_tree(vec![("+", TokenType::PlusSign)]);

        let child = root.get_child(&'+').unwrap();
        assert_eq!(child.get_value(), '+');
        assert_eq!(child.get_token_type(), Some(TokenType::PlusSign));
        assert!(child.children.is_empty());
    }

    #[test]
    fn nested_symbols() {
        let root = create_operator_tree(vec![("*", TokenType::MultiplicationSign), ("**", TokenType::ExponentialSign)]);

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
        let root = create_operator_tree(vec![("**", TokenType::ExponentialSign), ("*", TokenType::MultiplicationSign)]);

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
    fn pair() {
        let result = lex(&"=".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "=".to_string(),
                token_type: TokenType::Pair,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn period() {
        let result = lex(&".".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: ".".to_string(),
                token_type: TokenType::Period,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn access_left_internal() {
        let result = lex(&"_.".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "_.".to_string(),
                token_type: TokenType::LeftInternal,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn access_right_internal() {
        let result = lex(&"._".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "._".to_string(),
                token_type: TokenType::RightInternal,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn access_length_internal() {
        let result = lex(&".|".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: ".|".to_string(),
                token_type: TokenType::LengthInternal,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn absolute_value() {
        let result = lex(&"++".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "++".to_string(),
                token_type: TokenType::AbsoluteValue,
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
    fn default_conditional() {
        let result = lex(&"|>".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "|>".to_string(),
                token_type: TokenType::DefaultConditional,
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
    fn true_symbol() {
        let result = lex(&"$?".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "$?".to_string(),
                token_type: TokenType::True,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn false_symbol() {
        let result = lex(&"$!".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "$!".to_string(),
                token_type: TokenType::False,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn result_symbol() {
        let result = lex(&"$$".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "$$".to_string(),
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
        let result = lex(&"^~".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "^~".to_string(),
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
    fn symbol() {
        let result = lex(&";my_symbol".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: ";my_symbol".to_string(),
                token_type: TokenType::Symbol,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn empty_symbol() {
        let result = lex(&";".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: ";".to_string(),
                token_type: TokenType::Symbol,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn symbol_with_more_colons() {
        let result = lex(&";my_symbol:my_sub_symbol".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: ";my_symbol:my_sub_symbol".to_string(),
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
        let result = lex(&"{{{".to_string()).unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "{".to_string(),
                    token_type: TokenType::StartExpression,
                    column: 0,
                    row: 0
                },
                LexerToken {
                    text: "{".to_string(),
                    token_type: TokenType::StartExpression,
                    column: 1,
                    row: 0
                },
                LexerToken {
                    text: "{".to_string(),
                    token_type: TokenType::StartExpression,
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
                    token_type: TokenType::Whitespace,
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
                    token_type: TokenType::Whitespace,
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
                    token_type: TokenType::Whitespace,
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
                    token_type: TokenType::Whitespace,
                    column: 18,
                    row: 0
                },
            ]
        );
    }

    #[test]
    fn newline_whitespace() {
        let result = lex(&"\n  \n  \t\n \t \n".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "\n  \n  \t\n \t \n".to_string(),
                token_type: TokenType::Whitespace,
                column: 0,
                row: 0
            },]
        )
    }

    #[test]
    fn subexpression() {
        let result = lex(&"\n\n".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "\n\n".to_string(),
                token_type: TokenType::Subexpression,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn double_subexpression() {
        let result = lex(&"\n\n\n\n".to_string()).unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "\n\n".to_string(),
                    token_type: TokenType::Subexpression,
                    column: 0,
                    row: 0
                },
                LexerToken {
                    text: "\n\n".to_string(),
                    token_type: TokenType::Subexpression,
                    column: 0,
                    row: 2
                }
            ]
        )
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
                    token_type: TokenType::Whitespace,
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
                    token_type: TokenType::Whitespace,
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
                    token_type: TokenType::Whitespace,
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
                    token_type: TokenType::Whitespace,
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
    fn lex_identifier_starting_with_underscore() {
        let result = lex(&"_value".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "_value".to_string(),
                token_type: TokenType::Identifier,
                column: 0,
                row: 0
            }]
        );
    }

    #[test]
    fn lex_identifier_only_underscore_is_err() {
        let result = lex(&"_".to_string());

        assert!(result.is_err());
    }

    #[test]
    fn lex_identifier_start_with_colon() {
        let result = lex(&":value".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: ":value".to_string(),
                token_type: TokenType::Identifier,
                column: 0,
                row: 0
            }]
        );
    }

    #[test]
    fn lex_identifier_with_only_colon_is_error() {
        let result = lex(&":".to_string());

        assert!(result.is_err());
    }

    #[test]
    fn identifiers_cannot_start_with_number() {
        let result = lex(&"3value".to_string()).unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "3".to_string(),
                    token_type: TokenType::Number,
                    column: 0,
                    row: 0
                },
                LexerToken {
                    text: "value".to_string(),
                    token_type: TokenType::Identifier,
                    column: 1,
                    row: 0
                }
            ]
        );
    }

    #[test]
    fn lex_identifiers() {
        let result = lex(&"value_1 Value_2 namespace::value::property".to_string()).unwrap();

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
                    token_type: TokenType::Whitespace,
                    column: 7,
                    row: 0
                },
                LexerToken {
                    text: "Value_2".to_string(),
                    token_type: TokenType::Identifier,
                    column: 8,
                    row: 0
                },
                LexerToken {
                    text: " ".to_string(),
                    token_type: TokenType::Whitespace,
                    column: 15,
                    row: 0
                },
                LexerToken {
                    text: "namespace::value::property".to_string(),
                    token_type: TokenType::Identifier,
                    column: 16,
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
        let result = lex(&"@annotation".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "@annotation".to_string(),
                token_type: TokenType::Annotation,
                column: 0,
                row: 0
            },]
        );
    }

    #[test]
    fn annotation_with_token_after() {
        let result = lex(&"@annotation my_value".to_string()).unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "@annotation".to_string(),
                    token_type: TokenType::Annotation,
                    column: 0,
                    row: 0
                },
                LexerToken {
                    text: " ".to_string(),
                    token_type: TokenType::Whitespace,
                    column: 11,
                    row: 0
                },
                LexerToken {
                    text: "my_value".to_string(),
                    token_type: TokenType::Identifier,
                    column: 12,
                    row: 0
                }
            ]
        );
    }

    #[test]
    fn line_annotation() {
        let result = lex(&"@@ This is a comment".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "@@ This is a comment".to_string(),
                token_type: TokenType::LineAnnotation,
                column: 0,
                row: 0
            },]
        );
    }

    #[test]
    fn line_annotation_no_space_after() {
        let result = lex(&"@@This is a comment".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "@@This is a comment".to_string(),
                token_type: TokenType::LineAnnotation,
                column: 0,
                row: 0
            },]
        );
    }

    #[test]
    fn line_annotation_with_newline_and_identifier() {
        let result = lex(&"@@This is a comment\nmy_value".to_string()).unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "@@This is a comment".to_string(),
                    token_type: TokenType::LineAnnotation,
                    column: 0,
                    row: 0
                },
                LexerToken {
                    text: "\n".to_string(),
                    token_type: TokenType::Whitespace,
                    column: 19,
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
