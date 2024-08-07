use std::collections::HashMap;
use std::str::Chars;

use log::trace;

use crate::error::CompilerError;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub enum TokenType {
    Unknown,
    UnitLiteral,
    PlusSign,
    Subtraction,
    Division,
    MultiplicationSign,
    ExponentialSign,
    IntegerDivision,
    Remainder,
    AbsoluteValue,
    Opposite,
    BitwiseNot,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseLeftShift,
    BitwiseRightShift,
    And,
    Or,
    Xor,
    Not,
    Tis,
    StartExpression,
    EndExpression,
    StartGroup,
    EndGroup,
    StartSideEffect,
    EndSideEffect,
    Value,
    Comma,
    Symbol,
    Number,
    Identifier,
    CharList,
    ByteList,
    Whitespace,
    Subexpression,
    ExpressionTerminator,
    ExpressionSeparator,
    Annotation,
    LineAnnotation,
    Apply,
    JumpIfFalse,
    JumpIfTrue,
    ElseJump,
    TypeOf,
    ApplyTo,
    Reapply,
    EmptyApply,
    TypeCast,
    TypeEqual,
    Equality,
    Inequality,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Period,
    LeftInternal,
    RightInternal,
    LengthInternal,
    Pair,
    Concatenation,
    Range,
    StartExclusiveRange,
    EndExclusiveRange,
    ExclusiveRange,
    False,
    True,
    PrefixIdentifier,
    SuffixIdentifier,
    InfixIdentifier,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

    pub fn get_line(&self) -> usize {
        self.row
    }

    pub fn get_column(&self) -> usize {
        self.column
    }
}

pub struct Lexer<'a> {
    input: &'a str,
    input_iter: Chars<'a>,
    operator_tree: LexerOperatorNode,
    current_characters: String,
    current_token_type: Option<TokenType>,
    text_row: usize,
    text_column: usize,
    token_start_column: usize,
    token_start_row: usize,
    should_create: bool,
    state: LexingState,
    can_float: bool,
    start_quote_count: usize,
    end_quote_count: usize,
    could_be_sub_expression: bool,
    result: Result<(), CompilerError>,
    characters_lexed: usize,
    at_end: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let operator_tree = create_operator_tree(vec![
            ("+", TokenType::PlusSign),
            ("++", TokenType::AbsoluteValue),
            ("-", TokenType::Subtraction),
            ("--", TokenType::Opposite),
            ("*", TokenType::MultiplicationSign),
            ("**", TokenType::ExponentialSign),
            ("/", TokenType::Division),
            ("//", TokenType::IntegerDivision),
            ("%", TokenType::Remainder),
            ("!", TokenType::BitwiseNot),
            ("&", TokenType::BitwiseAnd),
            ("|", TokenType::BitwiseOr),
            ("^", TokenType::BitwiseXor),
            ("<<", TokenType::BitwiseLeftShift),
            (">>", TokenType::BitwiseRightShift),
            ("&&", TokenType::And),
            ("||", TokenType::Or),
            ("^^", TokenType::Xor),
            ("!!", TokenType::Not),
            ("??", TokenType::Tis),
            ("()", TokenType::UnitLiteral),
            ("{", TokenType::StartExpression),
            ("}", TokenType::EndExpression),
            ("(", TokenType::StartGroup),
            (")", TokenType::EndGroup),
            ("[", TokenType::StartSideEffect),
            ("]", TokenType::EndSideEffect),
            ("$", TokenType::Value),
            ("$?", TokenType::True),
            ("$!", TokenType::False),
            (",", TokenType::Comma),
            ("~", TokenType::Apply),
            ("!>", TokenType::JumpIfFalse),
            ("?>", TokenType::JumpIfTrue),
            ("|>", TokenType::ElseJump),
            ("~>", TokenType::ApplyTo),
            ("^~", TokenType::Reapply),
            ("~~", TokenType::EmptyApply),
            ("#", TokenType::TypeOf),
            ("~#", TokenType::TypeCast),
            ("#=", TokenType::TypeEqual),
            ("==", TokenType::Equality),
            ("!=", TokenType::Inequality),
            ("<", TokenType::LessThan),
            ("<=", TokenType::LessThanOrEqual),
            (">", TokenType::GreaterThan),
            (">=", TokenType::GreaterThanOrEqual),
            ("=", TokenType::Pair),
            (".", TokenType::Period),
            ("._", TokenType::RightInternal),
            ("_.", TokenType::LeftInternal),
            (".|", TokenType::LengthInternal),
            ("<>", TokenType::Concatenation),
            ("..", TokenType::Range),
            (">..", TokenType::StartExclusiveRange),
            ("..<", TokenType::EndExclusiveRange),
            (">..<", TokenType::ExclusiveRange),
            // (";;", TokenType::ExpressionTerminator),
            (";", TokenType::ExpressionSeparator),
        ]);

        let current_characters = String::new();
        let current_token_type = None;

        let text_row = 0;
        let text_column = 0;
        let token_start_column = 0;
        let token_start_row = 0;

        let should_create = true;
        let state = LexingState::NoToken;
        let can_float = true; // whether the next period can be a float
        let start_quote_count = 0;
        let end_quote_count = 0;
        let could_be_sub_expression = false;

        Self {
            input,
            input_iter: input.chars(),
            operator_tree,
            current_characters,
            current_token_type,
            text_row,
            text_column,
            token_start_column,
            token_start_row,
            should_create,
            state,
            can_float,
            start_quote_count,
            end_quote_count,
            could_be_sub_expression,
            result: Ok(()),
            characters_lexed: 0,
            at_end: false,
        }
    }

    pub fn get_input(&self) -> &'a str {
        self.input
    }

    fn current_operator(&self) -> Option<&LexerOperatorNode> {
        let mut node = &self.operator_tree;

        for c in self.current_characters.chars() {
            match node.get_child(&c) {
                None => return None,
                Some(n) => node = n,
            }
        }

        Some(node)
    }

    fn start_token(&mut self, c: char) {
        trace!("Beginning new token");
        self.current_characters = String::new();
        self.current_token_type = None;

        self.token_start_row = self.text_row;
        self.token_start_column = self.text_column;

        self.current_characters.push(c);

        // start new token
        if self.current_operator().is_some() {
            self.state = LexingState::Operator;
            self.current_token_type = match self.current_operator() {
                None => unreachable!(),
                Some(node) => node.token_type,
            };
        } else if c == ' ' || c == '\t' {
            self.state = LexingState::Spaces;
            self.current_token_type = Some(TokenType::Whitespace);
        } else if c.is_ascii_whitespace() {
            // any other white space, all some form of new line
            self.state = LexingState::Subexpression;
            self.current_token_type = Some(TokenType::Subexpression);

            self.text_column = 0;
            self.text_row += 1;
        } else if c.is_numeric() {
            self.state = LexingState::Number;
            self.current_token_type = Some(TokenType::Number);
        } else if is_identifier_char(c) {
            self.state = LexingState::Identifier;
            self.current_token_type = Some(TokenType::Identifier);
        } else if c == '`' {
            self.state = LexingState::Identifier;
            self.current_token_type = Some(TokenType::SuffixIdentifier);
        } else if c == '@' {
            self.state = LexingState::Annotation;
            self.current_token_type = Some(TokenType::Annotation);
        } else if c == '"' {
            self.state = LexingState::StartCharList;
            self.current_token_type = Some(TokenType::CharList);
        } else if c == '\'' {
            self.state = LexingState::StartByteList;
            self.current_token_type = Some(TokenType::ByteList);
        } else if c == '\0' && self.at_end {
            // allowing for now as final loop character
            self.state = LexingState::NoToken;
            self.current_token_type = None;
            self.current_characters = String::new();
        } else {
            self.result = Err(CompilerError::new(
                format!("Invalid start to token: {:?}", c),
                self.text_row,
                self.text_column,
            ))
        }

        trace!(
            "Starting token with character {:?} token type '{:?}' state '{:?}'",
            &c,
            self.current_token_type,
            self.state
        );
    }

    fn can_create_valid_token(&self) -> Result<(), CompilerError> {
        match self.current_token_type {
            Some(t) => match t {
                TokenType::Identifier => {
                    if self.current_characters == "_" || self.current_characters == ":" {
                        Err(CompilerError::new(
                            "Identifiers must contain more than 1 character when starting with '_' or ':'",
                            self.token_start_row,
                            self.token_start_column,
                        ))
                    } else {
                        Ok(())
                    }
                }
                _ => Ok(()),
            },
            None => Ok(()),
        }
    }

    fn process_char(&mut self, c: char) -> Option<LexerToken> {
        trace!("Character {:?} at ({:?}, {:?})", c, self.text_column, self.text_row);
        trace!("Current state: {:?}", self.state);

        let mut next_token = None;

        self.characters_lexed += 1;

        let start_new = match self.state {
            LexingState::NoToken => {
                self.start_token(c);
                false
            }
            LexingState::Operator => {
                trace!("Continuing operator");

                self.current_characters.push(c);
                match self.current_operator() {
                    Some(node) => {
                        // update token type and continue
                        self.current_token_type = node.token_type;
                        trace!("Switched to operator token '{:?}'", self.current_token_type);

                        false
                    }
                    None => {
                        // One token (LeftInternal) starts with an underscore
                        // which can also be start of an identifier
                        // if on this token and current character can be identifier
                        // switch to lexing identifier
                        if self.current_characters.starts_with('_') && is_identifier_char(c) {
                            trace!("Switching to lexing identifier after starting with an underscore.");
                            self.current_token_type = Some(TokenType::Identifier);
                            self.state = LexingState::Identifier;

                            false
                        } else if self.current_characters.starts_with(".")
                            && self.current_characters.len() == 2 // To be here should only have the decimal and the current number
                            && c.is_numeric()
                            && self.can_float
                        {
                            // Range tokens start with a period
                            // but Float numbers can also start with period
                            // If a range token ends up here it should already have length of at least 2
                            //      so it should be skipped by length check in previous condition
                            trace!("Found number after period. Switching to float.");
                            self.current_token_type = Some(TokenType::Number);
                            self.state = LexingState::Float;
                            false
                        } else {
                            // remove added character then end
                            self.current_characters.pop();
                            trace!("Ending operator");
                            true
                        }
                    }
                }
            }
            LexingState::Number => {
                // after initial number, underscores are allowed as visual separator
                if c.is_numeric() || c == '_' || c.is_alphanumeric() {
                    self.current_characters.push(c);
                    false
                } else if c == '.' && self.can_float {
                    trace!("Period found, switching to float");
                    self.current_characters.push(c);
                    self.current_token_type = Some(TokenType::Number);
                    self.state = LexingState::Float;
                    false
                } else {
                    trace!("Ending number");
                    true
                }
            }
            LexingState::Float => {
                if c.is_numeric() || c == '_' || c.is_alphanumeric() {
                    self.current_characters.push(c);
                    false
                } else if c == '.' && self.current_characters.ends_with(".") {
                    // split current token into just an integer
                    // and new Range token
                    let s = self.current_characters.trim_matches('.');
                    let token = LexerToken::new(
                        s.to_string(),
                        TokenType::Number,
                        self.token_start_row,
                        // actual token is determined after current, minus 1 to make accurate
                        self.token_start_column,
                    );

                    next_token = Some(token);

                    self.token_start_row = self.text_row;

                    let correct_start_column = self.text_column - 1;

                    self.start_token('.');

                    // set to two back for exisitng period
                    self.token_start_column = correct_start_column;
                    self.current_characters.push(c);

                    match self.current_operator() {
                        Some(node) => {
                            // set 'current' values
                            self.current_token_type = node.token_type;
                            trace!("Switched to operator token '{:?}'", self.current_token_type);

                            false
                        }
                        None => {
                            self.result = Err(CompilerError::new(
                                "Could not setup range token.",
                                self.token_start_row,
                                self.token_start_column,
                            ));
                            return None;
                        }
                    }
                } else {
                    trace!("Ending float");
                    true
                }
            }
            LexingState::Identifier => {
                if is_identifier_char(c) {
                    self.current_characters.push(c);
                    false
                } else if c == '`' {
                    self.current_characters.push(c);
                    self.should_create = false;
                    self.current_token_type = Some(if self.current_token_type == Some(TokenType::SuffixIdentifier) {
                        trace!("Switching to infix identifier");
                        TokenType::InfixIdentifier
                    } else {
                        trace!("Switching to prefix identifier");
                        TokenType::PrefixIdentifier
                    });

                    true
                } else {
                    trace!("Ending identifier");

                    // check if identifier is a symbol
                    if self.current_characters.starts_with(":") && self.current_characters.chars().nth(1) != Some(':') {
                        self.current_token_type = Some(TokenType::Symbol);
                    }
                    true
                }
            }
            LexingState::StartCharList => {
                let end = if c != '"' {
                    // Only supporting char list surrounded by 1 pair of double quotes
                    // and surrounded by 3+ pairs of double quotes
                    // reserved 2 double quotes for empty char lists
                    if self.current_characters.len() == 2 {
                        // meaning, we have 2 double quotes already
                        true
                    } else {
                        self.start_quote_count = self.current_characters.len();
                        self.state = LexingState::CharList;

                        false
                    }
                } else {
                    false
                };

                // so far the only token type that can have a null character reach push
                // because it adds all chars, mostly indiscriminately
                if !end && c != '\0' {
                    self.current_characters.push(c);
                }

                end
            }
            LexingState::CharList => {
                if c == '"' {
                    self.end_quote_count += 1;
                    self.current_characters.push(c);

                    if self.start_quote_count == self.end_quote_count {
                        trace!("Ending CharList");
                        self.should_create = false;
                        true
                    } else {
                        false
                    }
                } else {
                    // reset end quote count every non-quote character
                    self.end_quote_count = 0;
                    self.current_characters.push(c);
                    false
                }
            }
            LexingState::StartByteList => {
                let end = if c != '\'' {
                    if self.current_characters.len() == 2 {
                        // Only supporting byte lists surrounded by 1 pair of quotes
                        // and surrounded by 3+ pairs of double quotes
                        // reserved 2 quotes for empty byte lists
                        self.should_create = false;
                        true
                    } else {
                        self.start_quote_count = self.current_characters.len();
                        self.state = LexingState::ByteList;

                        false
                    }
                } else {
                    false
                };

                // so far the only token type that can have a null character reach push
                // because it adds all chars, mostly indiscriminately
                if c != '\0' {
                    self.current_characters.push(c);
                }

                end
            }
            LexingState::ByteList => {
                if c == '\'' {
                    self.end_quote_count += 1;
                    self.current_characters.push(c);

                    if self.start_quote_count == self.end_quote_count {
                        trace!("Ending CharList");
                        self.should_create = false;
                        true
                    } else {
                        false
                    }
                } else {
                    self.end_quote_count = 0;
                    self.current_characters.push(c);
                    false
                }
            }
            LexingState::Spaces => {
                if c == '\n' {
                    // wrap coordinates to new line
                    self.text_column = 0;
                    self.text_row += 1;

                    match self.could_be_sub_expression {
                        true => {
                            trace!("Found second newline in whitespace sequence. Creating subexpression.");
                            // second newline character in whitespace sequence
                            // end token as subexpression
                            self.current_token_type = Some(TokenType::Subexpression);
                            self.current_characters.push(c);
                            // current character is a part of this token
                            // skip creation of new token with it
                            self.should_create = false;

                            true
                        }
                        false => {
                            // first newline character in whitespace sequence
                            // switch to subexpression
                            trace!("Found newline character, switching state");
                            self.current_characters.push(c);

                            self.state = LexingState::Subexpression;

                            false
                        }
                    }
                } else if c != ' ' && c != '\t' {
                    trace!("Ending horizontal space");
                    true
                } else {
                    self.current_characters.push(c);
                    false
                }
            }
            LexingState::Subexpression => {
                if c.is_ascii_whitespace() && !(c == '\t' || c == ' ') {
                    trace!("Found second new line character. Creating subexpression token");
                    // first time through will be the second new line type character

                    // add character and create token
                    self.current_characters.push(c);

                    // could've arrived here by passing through whitespace state
                    // check len of characters so see if 2 tokens need to be created
                    if self.current_characters.len() > 2 {
                        trace!("Creating whitespace token from extra characters");
                        // have extra characters, split them into a whitespace token
                        let spaces_characters = &self.current_characters[..self.current_characters.len() - 2];
                        next_token = Some(LexerToken::new(
                            spaces_characters.to_string(),
                            TokenType::Whitespace,
                            self.token_start_row,
                            // actual token is determined after current, minus 1 to make accurate
                            self.token_start_column,
                        ));

                        self.current_characters = self.current_characters[(self.current_characters.len() - 2)..].to_string();
                    }

                    // wrap coordinates to new line
                    self.text_column = 0;
                    self.text_row += 1;

                    // skip start new token for this character since it is a part of this token
                    self.should_create = false;

                    true
                } else {
                    trace!("Non-newline character found");

                    // change to white space token and start new
                    self.current_token_type = Some(TokenType::Whitespace);
                    // but could be a subexpression still if another newline is found
                    self.could_be_sub_expression = true;

                    // if other white space, switch state and continue
                    if c == '\t' || c == ' ' {
                        trace!("Is whitespace character, switching state");
                        self.current_characters.push(c);
                        self.state = LexingState::Spaces;
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
                if c == '@' && self.current_characters.len() == 1 {
                    trace!("Another '@' character found, changing to LineAnnotation.");
                    self.current_characters.push(c);
                    self.state = LexingState::LineAnnotation;
                    self.current_token_type = Some(TokenType::LineAnnotation);
                    false
                } else if c.is_alphanumeric() || c == '_' {
                    // continue token
                    self.current_characters.push(c);
                    false
                } else {
                    // end token

                    true
                }
            }
            LexingState::LineAnnotation => {
                // line annotations continue until end of line
                // for simplicity we include entire line as the token

                if c == '\n' {
                    self.current_characters.push(c);
                    self.should_create = false;

                    // wrap coordinates to new line
                    self.text_column = 0;
                    self.text_row += 1;
                    true
                } else if c == '\0' {
                    true
                } else {
                    self.current_characters.push(c);
                    false
                }
            }
        };

        trace!("Will start new: {:?}", start_new);

        if start_new {
            // determine if the next token can be a float
            // cannot immediately follow identifiers, a period, other floats
            self.can_float = ![
                // $.1 value.1 "text".1
                // the 1 will be an integer for access operation
                Some(TokenType::Value),
                Some(TokenType::CharList),
                Some(TokenType::ByteList),
                Some(TokenType::Identifier),
                // value.1.1
                // since the period and 1 after value is for access, the next period is considered the same
                Some(TokenType::Period),
                // works alongside the above, the 1 is forced to be a number since it can't be a float
                // forceing the next period to not be a float, etc
                Some(TokenType::Number),
            ]
            .contains(&self.current_token_type);

            if self.state != LexingState::NoToken {
                trace!("Pushing new token: {:?}", self.current_token_type);

                self.result = self.can_create_valid_token();
                if self.result.is_ok() {
                    let token = LexerToken::new(
                        self.current_characters.clone(),
                        match self.current_token_type {
                            Some(t) => t,
                            None => {
                                self.result = Err(CompilerError::new("No token", self.token_start_row, self.token_start_column));
                                return None;
                            }
                        },
                        self.token_start_row,
                        // actual token is determined after current, minus 1 to make accurate
                        self.token_start_column,
                    );

                    next_token = Some(token);
                }
            }

            // set default for new if next token isn't a symbol
            self.state = LexingState::NoToken;
            self.current_characters = String::new();
            self.current_token_type = None;
            self.start_quote_count = 0;
            self.end_quote_count = 0;
            self.could_be_sub_expression = false;

            if self.should_create {
                trace!("Starting new token");
                self.start_token(c);
            } else {
                // Used only for single skips, flip back for next iteration
                trace!("Did not start new token");
                self.should_create = true;
            }
        }

        // need to relook at column count when deep diving into line feed, form feed, carriage return parsing
        if c != '\n' {
            self.text_column += 1;
        }

        next_token
    }

    fn internal_next(&mut self) -> Option<LexerToken> {
        if self.result.is_err() {
            // Invalid lexing state
            // do not continue consuming characters
            return None;
        }

        let mut next_token = None;

        loop {
            match self.input_iter.next() {
                Some(c) => match self.process_char(c) {
                    Some(t) => {
                        next_token = Some(t);
                        break;
                    }
                    None => (),
                },
                None => {
                    self.at_end = true;
                    // run all checks again to finalize last token
                    // by pushing through null character
                    match self.process_char('\0') {
                        Some(t) => {
                            next_token = Some(t);
                            break;
                        }
                        None => (),
                    }

                    // if we have a lingering token an don't already have an err
                    if self.current_characters.len() > 0 && self.result.is_ok() {
                        self.result = Err(CompilerError::new(
                            format!("Unterminated token. Might be {:?}.", self.current_token_type),
                            self.token_start_row,
                            self.token_start_column,
                        ));
                    }

                    break;
                }
            }
        }

        next_token
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = LexerToken;

    fn next(&mut self) -> Option<Self::Item> {
        self.internal_next()
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
    Float,
    Identifier,
    Annotation,
    LineAnnotation,
    CharList,
    StartCharList,
    ByteList,
    StartByteList,
}

pub fn lex(input: &str) -> Result<Vec<LexerToken>, CompilerError> {
    let mut tokens = vec![];

    let mut lexer = Lexer::new(input);

    while let Some(token) = lexer.next() {
        match lexer.result {
            Ok(_) => tokens.push(token),
            Err(e) => return Err(e),
        }
    }

    match lexer.result {
        Ok(_) => Ok(tokens),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod errors {
    use crate::error::CompilerError;
    use crate::lex::*;

    #[test]
    fn error_from_unknown_token() {
        let result = lex(&"?".to_string());

        assert_eq!(result.err().unwrap(), CompilerError::new("No token", 0, 0));
    }
}

#[cfg(test)]
mod iterator {
    use crate::lex::*;

    #[test]
    fn plus_sign() {
        let mut iter = Lexer::new("+");

        let result = iter.next().unwrap();

        assert_eq!(
            result,
            LexerToken {
                text: "+".to_string(),
                token_type: TokenType::PlusSign,
                column: 0,
                row: 0
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::lex::lexer::create_operator_tree;
    use crate::lex::*;

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
    fn empty_gives_empty() {
        let result = lex(&"".to_string()).unwrap();

        assert_eq!(result, vec![])
    }

    #[test]
    fn plus_sign() {
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
    fn subtraction() {
        let result = lex(&"-".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "-".to_string(),
                token_type: TokenType::Subtraction,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn multiplication() {
        let result = lex(&"*".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "*".to_string(),
                token_type: TokenType::MultiplicationSign,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn division() {
        let result = lex(&"/".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "/".to_string(),
                token_type: TokenType::Division,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn exponential() {
        let result = lex(&"**".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "**".to_string(),
                token_type: TokenType::ExponentialSign,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn integer_division() {
        let result = lex(&"//".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "//".to_string(),
                token_type: TokenType::IntegerDivision,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn remainder() {
        let result = lex(&"%".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "%".to_string(),
                token_type: TokenType::Remainder,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn opposite() {
        let result = lex(&"--".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "--".to_string(),
                token_type: TokenType::Opposite,
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
    fn bitwise_not() {
        let result = lex(&"!".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "!".to_string(),
                token_type: TokenType::BitwiseNot,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn bitwise_and() {
        let result = lex(&"&".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "&".to_string(),
                token_type: TokenType::BitwiseAnd,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn bitwise_or() {
        let result = lex(&"|".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "|".to_string(),
                token_type: TokenType::BitwiseOr,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn bitwise_xor() {
        let result = lex(&"^".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "^".to_string(),
                token_type: TokenType::BitwiseXor,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn bitwise_left_shift() {
        let result = lex(&"<<".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "<<".to_string(),
                token_type: TokenType::BitwiseLeftShift,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn bitwise_right_shift() {
        let result = lex(&">>".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: ">>".to_string(),
                token_type: TokenType::BitwiseRightShift,
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
    fn value_period_number() {
        let result = lex(&"$.0".to_string()).unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "$".to_string(),
                    token_type: TokenType::Value,
                    column: 0,
                    row: 0
                },
                LexerToken {
                    text: ".".to_string(),
                    token_type: TokenType::Period,
                    column: 1,
                    row: 0
                },
                LexerToken {
                    text: "0".to_string(),
                    token_type: TokenType::Number,
                    column: 2,
                    row: 0
                }
            ]
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
    fn apply_if_true_symbol() {
        let result = lex(&"?>".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "?>".to_string(),
                token_type: TokenType::JumpIfTrue,
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
                token_type: TokenType::JumpIfFalse,
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
                token_type: TokenType::ElseJump,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn and() {
        let result = lex(&"&&".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "&&".to_string(),
                token_type: TokenType::And,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn or() {
        let result = lex(&"||".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "||".to_string(),
                token_type: TokenType::Or,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn xor() {
        let result = lex(&"^^".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "^^".to_string(),
                token_type: TokenType::Xor,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn not() {
        let result = lex(&"!!".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "!!".to_string(),
                token_type: TokenType::Not,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn tis() {
        let result = lex(&"??".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "??".to_string(),
                token_type: TokenType::Tis,
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
    fn inequality() {
        let result = lex(&"!=".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "!=".to_string(),
                token_type: TokenType::Inequality,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn less_than() {
        let result = lex(&"<".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "<".to_string(),
                token_type: TokenType::LessThan,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn less_than_or_equal() {
        let result = lex(&"<=".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "<=".to_string(),
                token_type: TokenType::LessThanOrEqual,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn greater_than() {
        let result = lex(&">".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: ">".to_string(),
                token_type: TokenType::GreaterThan,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn greater_than_or_equal() {
        let result = lex(&">=".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: ">=".to_string(),
                token_type: TokenType::GreaterThanOrEqual,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn type_of() {
        let result = lex(&"#".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "#".to_string(),
                token_type: TokenType::TypeOf,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn type_cast() {
        let result = lex(&"~#".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "~#".to_string(),
                token_type: TokenType::TypeCast,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn type_equal() {
        let result = lex(&"#=".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "#=".to_string(),
                token_type: TokenType::TypeEqual,
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
                token_type: TokenType::Value,
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
    fn concatenation() {
        let result = lex(&"<>".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "<>".to_string(),
                token_type: TokenType::Concatenation,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn range() {
        let result = lex(&"..".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "..".to_string(),
                token_type: TokenType::Range,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn exclusive_start_range() {
        let result = lex(&">..".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: ">..".to_string(),
                token_type: TokenType::StartExclusiveRange,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn exclusive_end_range() {
        let result = lex(&"..<".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "..<".to_string(),
                token_type: TokenType::EndExclusiveRange,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn exclusive_range() {
        let result = lex(&">..<".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: ">..<".to_string(),
                token_type: TokenType::ExclusiveRange,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn symbol() {
        let result = lex(&":my_symbol".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: ":my_symbol".to_string(),
                token_type: TokenType::Symbol,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn empty_symbol() {
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
    fn symbol_with_more_colons() {
        let result = lex(&":my_symbol:my_sub_symbol".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: ":my_symbol:my_sub_symbol".to_string(),
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
    fn start_side_effect_symbol() {
        let result = lex(&"[".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "[".to_string(),
                token_type: TokenType::StartSideEffect,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn end_side_effect_symbol() {
        let result = lex(&"]".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "]".to_string(),
                token_type: TokenType::EndSideEffect,
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
    fn null_characters_cause_error() {
        let result = lex(&"+\0+\0+".to_string());

        assert!(result.is_err())
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
    fn split_newlines_still_subexpression() {
        let result = lex(&"\n   \t   \n".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "\n   \t   \n".to_string(),
                token_type: TokenType::Subexpression,
                column: 0,
                row: 0
            },]
        )
    }

    #[test]
    fn adjacent_split_newlines_still_subexpression() {
        let result = lex(&"\n   \t   \n\n   \t   \n".to_string()).unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "\n   \t   \n".to_string(),
                    token_type: TokenType::Subexpression,
                    column: 0,
                    row: 0
                },
                LexerToken {
                    text: "\n   \t   \n".to_string(),
                    token_type: TokenType::Subexpression,
                    column: 0,
                    row: 2
                },
            ]
        )
    }

    #[test]
    fn adjacent_split_newlines_separated_by_spaces_still_subexpression() {
        let result = lex(&"\n   \t   \n     \n   \t   \n".to_string()).unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "\n   \t   \n".to_string(),
                    token_type: TokenType::Subexpression,
                    column: 0,
                    row: 0
                },
                LexerToken {
                    text: "     \n   \t   \n".to_string(),
                    token_type: TokenType::Subexpression,
                    column: 0,
                    row: 2
                },
            ]
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
    #[should_panic] // pending implementation of stack frames to prevent memory leak
    fn expression_terminator() {
        let result = lex(&";;".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: ";;".to_string(),
                token_type: TokenType::ExpressionTerminator,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn expression_separator() {
        let result = lex(&";".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: ";".to_string(),
                token_type: TokenType::ExpressionSeparator,
                column: 0,
                row: 0
            }]
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
    fn integer_range() {
        let result = lex("3..6").unwrap();

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
                    text: "..".to_string(),
                    token_type: TokenType::Range,
                    column: 1,
                    row: 0
                },
                LexerToken {
                    text: "6".to_string(),
                    token_type: TokenType::Number,
                    column: 3,
                    row: 0
                },
            ]
        )
    }

    #[test]
    fn integer_end_exclusive_range() {
        let result = lex("3..<").unwrap();

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
                    text: "..<".to_string(),
                    token_type: TokenType::EndExclusiveRange,
                    column: 1,
                    row: 0
                }
            ]
        )
    }

    #[test]
    fn float_period_integer() {
        let result = lex("3.14.1").unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "3.14".to_string(),
                    token_type: TokenType::Number,
                    column: 0,
                    row: 0
                },
                LexerToken {
                    text: ".".to_string(),
                    token_type: TokenType::Period,
                    column: 4,
                    row: 0
                },
                LexerToken {
                    text: "1".to_string(),
                    token_type: TokenType::Number,
                    column: 5,
                    row: 0
                }
            ]
        )
    }

    #[test]
    fn identifier_period_integer() {
        let result = lex("value.1").unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "value".to_string(),
                    token_type: TokenType::Identifier,
                    column: 0,
                    row: 0
                },
                LexerToken {
                    text: ".".to_string(),
                    token_type: TokenType::Period,
                    column: 5,
                    row: 0
                },
                LexerToken {
                    text: "1".to_string(),
                    token_type: TokenType::Number,
                    column: 6,
                    row: 0
                }
            ]
        )
    }

    #[test]
    fn identifier_space_float() {
        let result = lex("value .1").unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "value".to_string(),
                    token_type: TokenType::Identifier,
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
                    text: ".1".to_string(),
                    token_type: TokenType::Number,
                    column: 6,
                    row: 0
                }
            ]
        )
    }

    #[test]
    fn identifier_period_integer_period_integer() {
        let result = lex("value.1.1").unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "value".to_string(),
                    token_type: TokenType::Identifier,
                    column: 0,
                    row: 0
                },
                LexerToken {
                    text: ".".to_string(),
                    token_type: TokenType::Period,
                    column: 5,
                    row: 0
                },
                LexerToken {
                    text: "1".to_string(),
                    token_type: TokenType::Number,
                    column: 6,
                    row: 0
                },
                LexerToken {
                    text: ".".to_string(),
                    token_type: TokenType::Period,
                    column: 7,
                    row: 0
                },
                LexerToken {
                    text: "1".to_string(),
                    token_type: TokenType::Number,
                    column: 8,
                    row: 0
                }
            ]
        )
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
    fn lex_identifier_start_with_two_colons() {
        let result = lex(&"::value".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "::value".to_string(),
                token_type: TokenType::Identifier,
                column: 0,
                row: 0
            }]
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
    fn prefix_identifier() {
        let result = lex(&"expression`".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "expression`".to_string(),
                token_type: TokenType::PrefixIdentifier,
                column: 0,
                row: 0
            },]
        );
    }

    #[test]
    fn suffix_identifier() {
        let result = lex(&"`expression".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "`expression".to_string(),
                token_type: TokenType::SuffixIdentifier,
                column: 0,
                row: 0
            },]
        );
    }

    #[test]
    fn infix_identifier() {
        let result = lex(&"`expression`".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "`expression`".to_string(),
                token_type: TokenType::InfixIdentifier,
                column: 0,
                row: 0
            },]
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
                    text: "@@This is a comment\n".to_string(),
                    token_type: TokenType::LineAnnotation,
                    column: 0,
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

#[cfg(test)]
mod numbers {
    use std::vec;

    use crate::lex::*;

    #[test]
    fn with_visual_separator_underscore() {
        let result = lex(&"12345_67890".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "12345_67890".to_string(),
                token_type: TokenType::Number,
                column: 0,
                row: 0
            }]
        );
    }

    #[test]
    fn with_letters() {
        let result = lex(&"12_ABCDF".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "12_ABCDF".to_string(),
                token_type: TokenType::Number,
                column: 0,
                row: 0
            }]
        );
    }
    #[test]
    fn with_visual_separator_underscore_and_decimal() {
        let result = lex(&"0.12345_67890".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "0.12345_67890".to_string(),
                token_type: TokenType::Number,
                column: 0,
                row: 0
            }]
        );
    }

    #[test]
    fn with_letters_and_decimal() {
        let result = lex(&"0.12_ABCDF".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "0.12_ABCDF".to_string(),
                token_type: TokenType::Number,
                column: 0,
                row: 0
            }]
        );
    }

    #[test]
    fn lex_integers() {
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
    fn lex_integers_with_symbol() {
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
    fn float() {
        let result = lex("3.14").unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "3.14".to_string(),
                token_type: TokenType::Number,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn float_start_with_period() {
        let result = lex(".14").unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: ".14".to_string(),
                token_type: TokenType::Number,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn float_end_with_period() {
        let result = lex("3.").unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "3.".to_string(),
                token_type: TokenType::Number,
                column: 0,
                row: 0
            }]
        )
    }
}

#[cfg(test)]
mod chars_and_bytes {
    use std::vec;

    use crate::lex::*;

    #[test]
    fn character_list_unclosed() {
        let result = lex(&"\"Hello World!".to_string());

        assert!(result.is_err())
    }

    #[test]
    fn byte_list_unclosed() {
        let result = lex(&"'Hello World!".to_string());

        assert!(result.is_err())
    }

    #[test]
    fn character_list_multi_quote() {
        let result = lex(&"\"\"\"Hello \"sub quote\" World!\"\"\"".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "\"\"\"Hello \"sub quote\" World!\"\"\"".to_string(),
                token_type: TokenType::CharList,
                column: 0,
                row: 0
            }]
        );
    }

    #[test]
    fn byte_list_multi_quote() {
        let result = lex(&"'''Hello ' World!'''".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "'''Hello ' World!'''".to_string(),
                token_type: TokenType::ByteList,
                column: 0,
                row: 0
            }]
        );
    }

    #[test]
    fn byte_list_dot_and_number() {
        let result = lex(&"'Hello'.4".to_string()).unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "'Hello'".to_string(),
                    token_type: TokenType::ByteList,
                    column: 0,
                    row: 0
                },
                LexerToken {
                    text: ".".to_string(),
                    token_type: TokenType::Period,
                    column: 7,
                    row: 0
                },
                LexerToken {
                    text: "4".to_string(),
                    token_type: TokenType::Number,
                    column: 8,
                    row: 0
                }
            ]
        );
    }

    #[test]
    fn empty_character_list() {
        let result = lex(&"\"\"".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "\"\"".to_string(),
                token_type: TokenType::CharList,
                column: 0,
                row: 0
            }]
        );
    }

    #[test]
    fn empty_character_list_with_surrounding_text() {
        let result = lex(&"5 \"\" 5".to_string()).unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "5".to_string(),
                    token_type: TokenType::Number,
                    column: 0,
                    row: 0
                },
                LexerToken {
                    text: " ".to_string(),
                    token_type: TokenType::Whitespace,
                    column: 1,
                    row: 0
                },
                LexerToken {
                    text: "\"\"".to_string(),
                    token_type: TokenType::CharList,
                    column: 2,
                    row: 0
                },
                LexerToken {
                    text: " ".to_string(),
                    token_type: TokenType::Whitespace,
                    column: 4,
                    row: 0
                },
                LexerToken {
                    text: "5".to_string(),
                    token_type: TokenType::Number,
                    column: 5,
                    row: 0
                }
            ]
        );
    }

    #[test]
    fn character_list() {
        let result = lex(&"\"Hello World!\"".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "\"Hello World!\"".to_string(),
                token_type: TokenType::CharList,
                column: 0,
                row: 0
            }]
        );
    }

    #[test]
    fn character_list_then_access() {
        let result = lex(&"\"Hello World!\".4".to_string()).unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "\"Hello World!\"".to_string(),
                    token_type: TokenType::CharList,
                    column: 0,
                    row: 0
                },
                LexerToken {
                    text: ".".to_string(),
                    token_type: TokenType::Period,
                    column: 14,
                    row: 0
                },
                LexerToken {
                    text: "4".to_string(),
                    token_type: TokenType::Number,
                    column: 15,
                    row: 0
                }
            ]
        );
    }

    #[test]
    fn character_list_followed_by_operations() {
        let result = lex(&"\"Hello World!\" ~ 10".to_string()).unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "\"Hello World!\"".to_string(),
                    token_type: TokenType::CharList,
                    column: 0,
                    row: 0
                },
                LexerToken {
                    text: " ".to_string(),
                    token_type: TokenType::Whitespace,
                    column: 14,
                    row: 0
                },
                LexerToken {
                    text: "~".to_string(),
                    token_type: TokenType::Apply,
                    column: 15,
                    row: 0
                },
                LexerToken {
                    text: " ".to_string(),
                    token_type: TokenType::Whitespace,
                    column: 16,
                    row: 0
                },
                LexerToken {
                    text: "10".to_string(),
                    token_type: TokenType::Number,
                    column: 17,
                    row: 0
                }
            ]
        );
    }

    #[test]
    fn empty_byte_list() {
        let result = lex(&"''".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "''".to_string(),
                token_type: TokenType::ByteList,
                column: 0,
                row: 0
            }]
        );
    }

    #[test]
    fn byte_list() {
        let result = lex(&"'Hello World!'".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "'Hello World!'".to_string(),
                token_type: TokenType::ByteList,
                column: 0,
                row: 0
            }]
        );
    }

    #[test]
    fn byte_list_followed_by_operations() {
        let result = lex(&"'Hello World!' ~ 10".to_string()).unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "'Hello World!'".to_string(),
                    token_type: TokenType::ByteList,
                    column: 0,
                    row: 0
                },
                LexerToken {
                    text: " ".to_string(),
                    token_type: TokenType::Whitespace,
                    column: 14,
                    row: 0
                },
                LexerToken {
                    text: "~".to_string(),
                    token_type: TokenType::Apply,
                    column: 15,
                    row: 0
                },
                LexerToken {
                    text: " ".to_string(),
                    token_type: TokenType::Whitespace,
                    column: 16,
                    row: 0
                },
                LexerToken {
                    text: "10".to_string(),
                    token_type: TokenType::Number,
                    column: 17,
                    row: 0
                }
            ]
        );
    }
}
