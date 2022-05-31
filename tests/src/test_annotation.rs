use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::iter;

use garnish_lang_compiler::{LexerToken, TokenType};

/// Test Annotations
///
/// @Test
/// Parameters: Name, Expression
/// Runs given expression and expects the resulting value to be True
///
/// @Case
/// Parameters: Name, Input, Expected Output
/// Runs associated expression and dose equality comparison against expected output
///
/// @Mock
/// Parameters: Expression Symbol, Value
/// For associated test, given expression will always result in given value.
///
/// @MockAll
/// Parameters: Expression Symbol, Value
/// For associated file, given expression will always result in given value.
///
///

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum TestAnnotation {
    Test,
    Case,
    Mock,
    MockAll
}

pub struct TestAnnotationDetails {
    annotation: TestAnnotation,
    expression: Vec<LexerToken>,
}

impl TestAnnotationDetails {
    fn new(annotation: TestAnnotation, expression: Vec<LexerToken>) -> Self {
        TestAnnotationDetails { annotation, expression }
    }

    pub fn get_annotation(&self) -> TestAnnotation {
        self.annotation
    }

    pub fn get_expression(&self) -> &Vec<LexerToken> {
        &self.expression
    }
}

pub struct TestDetails {
    tests: Vec<TestAnnotationDetails>,
}

impl TestDetails {
    fn new() -> Self {
        TestDetails { tests: Vec::new() }
    }

    pub fn get_tests(&self) -> &Vec<TestAnnotationDetails> {
        &self.tests
    }
}

#[derive(Debug)]
pub struct TestExtractionError {
    reason: String,
}

impl TestExtractionError {
    pub fn error(s: &str) -> Self {
        TestExtractionError { reason: s.to_string() }
    }
}

impl Display for TestExtractionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.reason.as_str())
    }
}

impl Error for TestExtractionError {}

enum ExtractionState {
    Searching,
    InTest,
}

fn get_first_non_space(tokens: &Vec<LexerToken>, start: usize) -> (usize, LexerToken) {
    let unknown_token= LexerToken::new("".to_string(), TokenType::Unknown, 0, 0);
    let mut non_space = (0, unknown_token);
    for (i, token) in tokens.iter().enumerate().skip(start) {
        match token.get_token_type() {
            TokenType::Whitespace => continue,
            _ => {
                non_space = (i, token.clone());
                break;
            }
        }
    }

    non_space
}

pub fn extract_tests(tokens: &Vec<LexerToken>) -> Result<TestDetails, TestExtractionError> {
    let mut extraction_details = TestDetails::new();
    let mut state = ExtractionState::Searching;
    let unknown_token= LexerToken::new("".to_string(), TokenType::Unknown, 0, 0);
    let mut iter = tokens.iter().chain(iter::once(&unknown_token));

    let mut current_extraction = Vec::new();

    while let Some(next) = iter.next() {
        match state {
            ExtractionState::Searching => {
                match next.get_token_type() {
                    TokenType::Annotation => match next.get_text().as_str() {
                        "@Test" => {
                            state = ExtractionState::InTest;
                        }
                        _ => (), // none test annotation
                    },
                    _ => (), // go to next token
                }
            }
            ExtractionState::InTest => {
                // get all tokens until first un-nested Subexpression token
                match next.get_token_type() {
                    TokenType::Unknown => {
                        // finalize test annotation details

                        // first non space token should be a string for name
                        let non_space = get_first_non_space(&current_extraction, 0);

                        // create details
                        let expression = Vec::from(&current_extraction[non_space.0..]);
                        let details = TestAnnotationDetails::new(TestAnnotation::Test, expression);
                        extraction_details.tests.push(details);

                        // reset
                        current_extraction = Vec::new();
                        state = ExtractionState::Searching;
                    }
                    _ => {
                        current_extraction.push(next.clone());
                    }
                }
            }
        }
    }

    Ok(extraction_details)
}

#[cfg(test)]
mod tests {
    use garnish_lang_compiler::lex;
    use garnish_lang_runtime::runtime_impls::SimpleGarnishRuntime;

    use crate::test_annotation::{extract_tests, TestAnnotation};

    #[test]
    fn create_test_detail() {
        let tokens = lex("@Test \"Plus 10\" { 5 + 10 == 15 }").unwrap();

        let test_details = extract_tests(&tokens).unwrap();

        assert_eq!(test_details.get_tests().len(), 1);
        let detail = test_details.get_tests().get(0).unwrap();

        assert_eq!(detail.get_annotation(), TestAnnotation::Test);
        assert_eq!(detail.get_expression(), &Vec::from(&tokens[2..]));
    }
}
