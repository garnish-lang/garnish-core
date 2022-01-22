use crate::error::CompilerError;
use log::trace;
use std::{collections::HashMap, iter, vec};

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
    AppendLink,
    PrependLink,
    Range,
    StartExclusiveRange,
    EndExclusiveRange,
    ExclusiveRange,
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

    pub fn get_line(&self) -> usize {
        self.row
    }

    pub fn get_column(&self) -> usize {
        self.column
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
    Indentifier,
    Annotation,
    LineAnnotation,
    Symbol,
    CharList,
    ByteList,
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
    } else if c == '"' {
        current_characters.push(c);
        *state = LexingState::CharList;
        *current_token_type = Some(TokenType::CharList);
    } else if c == '\'' {
        current_characters.push(c);
        *state = LexingState::ByteList;
        *current_token_type = Some(TokenType::ByteList);
    }

    trace!(
        "Starting token with character {:?} token type '{:?}' state '{:?}'",
        &c,
        current_token_type,
        state
    );
}

pub fn lex(input: &str) -> Result<Vec<LexerToken>, CompilerError> {
    lex_with_processor(input)
}

pub fn lex_with_processor(input: &str) -> Result<Vec<LexerToken>, CompilerError> {
    trace!("Beginning lexing");

    let mut tokens = vec![];
    let mut current_characters = String::new();

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
        ("->", TokenType::AppendLink),
        ("<-", TokenType::PrependLink),
        ("..", TokenType::Range),
        (">..", TokenType::StartExclusiveRange),
        ("..<", TokenType::EndExclusiveRange),
        (">..<", TokenType::ExclusiveRange),
    ]);
    let mut current_operator = &operator_tree;
    let mut current_token_type = None;

    let mut text_row = 0;
    let mut text_column = 0;
    let mut token_start_column = 0;
    let mut token_start_row = 0;

    let mut should_create = true;
    let mut state = LexingState::NoToken;
    let mut can_float = true; // whether the next period can be a float

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
                        } else if current_characters == "." && c.is_numeric() && can_float {
                            // Another token (Float) can start with a period
                            trace!("Found number after period. Switching to flaot.");
                            current_characters.push(c);
                            current_token_type = Some(TokenType::Number);
                            state = LexingState::Float;
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
                // after initial number, underscores are allowed as visual separator
                if c.is_numeric() || c == '_' || c.is_alphanumeric() {
                    current_characters.push(c);
                    false
                } else if c == '.' && can_float {
                    trace!("Period found, switching to float");
                    current_characters.push(c);
                    current_token_type = Some(TokenType::Number);
                    state = LexingState::Float;
                    false
                } else {
                    trace!("Ending number");
                    true
                }
            }
            LexingState::Float => {
                if c.is_numeric() || c == '_' || c.is_alphanumeric() {
                    current_characters.push(c);
                    false
                } else if c == '.' && current_characters.ends_with(".") {
                    // split current token into just an integer
                    // and new Range token
                    let s = current_characters.trim_matches('.');
                    let token = LexerToken::new(
                        s.to_string(),
                        TokenType::Number,
                        token_start_row,
                        // actual token is determined after current, minus 1 to make accurate
                        token_start_column,
                    );

                    tokens.push(token);

                    token_start_row = text_row;

                    let correct_start_column = text_column - 1;

                    start_token(
                        '.',
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

                    // set to two back for exisitng period
                    token_start_column = correct_start_column;

                    match current_operator.get_child(&c) {
                        Some(node) => {
                            // set 'current' values
                            current_characters.push(c);
                            current_token_type = node.token_type;
                            current_operator = node;

                            trace!("Switched to operator token '{:?}'", current_token_type);

                            false
                        }
                        None => Err(CompilerError::new("Could not setup range token.", token_start_row, token_start_column))?,
                    }
                } else {
                    trace!("Ending float");
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
            LexingState::CharList => {
                if c == '"' {
                    trace!("Ending CharList");
                    current_characters.push(c);
                    should_create = false;
                    true
                } else {
                    current_characters.push(c);
                    false
                }
            }
            LexingState::ByteList => {
                if c == '\'' {
                    trace!("Ending ByteList");
                    current_characters.push(c);
                    should_create = false;
                    true
                } else {
                    current_characters.push(c);
                    false
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

                    true
                }
            }
            LexingState::LineAnnotation => {
                // line annotations continue until end of line
                // for simplicity we include entire line as the token
                if c == '\n' || c == '\0' {
                    true
                } else {
                    current_characters.push(c);
                    false
                }
            }
        };

        trace!("Will start new: {:?}", start_new);

        if start_new {
            // determine if the next token can be a float
            // cannot immediately follow identifiers, a period, other floats
            can_float = ![
                // value.1
                // the 1 will be an integer for access operation
                Some(TokenType::Identifier),
                // value.1.1
                // since the period and 1 after value is for access, the next period is considered the same
                Some(TokenType::Period),
                // works alongside the above, the 1 is forced to be a number since it can't be a float
                // forceing the next period to not be a float, etc
                Some(TokenType::Number),
                // 3.14.1
                // floats can only have one decimal, this can also cause the above cascade
                Some(TokenType::Number),
            ]
            .contains(&current_token_type);

            if state != LexingState::NoToken {
                trace!("Pushing new token: {:?}", current_token_type);

                let token = LexerToken::new(
                    current_characters,
                    match current_token_type {
                        Some(t) => t,
                        None => Err(CompilerError::new("No token", token_start_row, token_start_column))?,
                    },
                    token_start_row,
                    // actual token is determined after current, minus 1 to make accurate
                    token_start_column,
                );

                tokens.push(token);
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
mod errors {
    use crate::error::CompilerError;
    use crate::lex;

    #[test]
    fn error_from_unknown_token() {
        let result = lex(&"?".to_string());

        assert_eq!(result.err().unwrap(), CompilerError::new("No token", 0, 0));
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::lexing::lexer::create_operator_tree;
    use crate::{lex, LexerToken, TokenType};

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
    fn append_link() {
        let result = lex(&"->".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "->".to_string(),
                token_type: TokenType::AppendLink,
                column: 0,
                row: 0
            }]
        )
    }

    #[test]
    fn prepend_link() {
        let result = lex(&"<-".to_string()).unwrap();

        assert_eq!(
            result,
            vec![LexerToken {
                text: "<-".to_string(),
                token_type: TokenType::PrependLink,
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
    fn integer_range() {
        let result = lex("3..").unwrap();

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
                }
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

#[cfg(test)]
mod numbers {
    use std::vec;

    use crate::{lex, LexerToken, TokenType};

    #[test]
    fn with_visual_separator_underscore() {
        let result = lex(&"12345_67890".to_string()).unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "12345_67890".to_string(),
                    token_type: TokenType::Number,
                    column: 0,
                    row: 0
                }
            ]
        );
    }

    #[test]
    fn with_letters() {
        let result = lex(&"12_ABCDF".to_string()).unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "12_ABCDF".to_string(),
                    token_type: TokenType::Number,
                    column: 0,
                    row: 0
                }
            ]
        );
    }
    #[test]
    fn with_visual_separator_underscore_and_decimal() {
        let result = lex(&"0.12345_67890".to_string()).unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "0.12345_67890".to_string(),
                    token_type: TokenType::Number,
                    column: 0,
                    row: 0
                }
            ]
        );
    }

    #[test]
    fn with_letters_and_decimal() {
        let result = lex(&"0.12_ABCDF".to_string()).unwrap();

        assert_eq!(
            result,
            vec![
                LexerToken {
                    text: "0.12_ABCDF".to_string(),
                    token_type: TokenType::Number,
                    column: 0,
                    row: 0
                }
            ]
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

    use crate::{lex, LexerToken, TokenType};

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
