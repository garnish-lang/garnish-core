use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::iter;

use garnish_lang_compiler::{LexerToken, TokenType};
use crate::test_annotation::TestAnnotation::{Mock, Test};

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

pub struct MockAnnotationDetails {
    expression: Vec<LexerToken>
}

impl MockAnnotationDetails {
    fn new(expression: Vec<LexerToken>) -> Self {
        MockAnnotationDetails { expression }
    }

    pub fn get_expression(&self) -> &Vec<LexerToken> {
        &self.expression
    }
}

pub struct TestAnnotationDetails {
    annotation: TestAnnotation,
    expression: Vec<LexerToken>,
    mocks: Vec<MockAnnotationDetails>
}

impl TestAnnotationDetails {
    fn new(annotation: TestAnnotation, expression: Vec<LexerToken>, mocks: Vec<MockAnnotationDetails>) -> Self {
        TestAnnotationDetails { annotation, expression, mocks }
    }

    pub fn get_annotation(&self) -> TestAnnotation {
        self.annotation
    }

    pub fn get_expression(&self) -> &Vec<LexerToken> {
        &self.expression
    }

    pub fn get_mocks(&self) -> &Vec<MockAnnotationDetails> {
        &self.mocks
    }
}

pub struct TestDetails {
    annotations: Vec<TestAnnotationDetails>,
    expression: Vec<LexerToken>,
}

impl TestDetails {
    fn new(expression: Vec<LexerToken>, annotations: Vec<TestAnnotationDetails>) -> Self {
        TestDetails { expression, annotations }
    }

    pub fn get_annotations(&self) -> &Vec<TestAnnotationDetails> {
        &self.annotations
    }

    pub fn get_expression(&self) -> &Vec<LexerToken> {
        &self.expression
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
    InMock,
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
    let mut state = ExtractionState::Searching;
    let unknown_token= LexerToken::new("".to_string(), TokenType::Unknown, 0, 0);
    let mut iter = tokens.iter().chain(iter::once(&unknown_token));

    let mut top_expression = Vec::new();
    let mut annotations = Vec::new();

    let mut current_mocks = Vec::new();
    let mut current_extraction = Vec::new();

    while let Some(next) = iter.next() {
        match state {
            ExtractionState::Searching => {
                match next.get_token_type() {
                    TokenType::Annotation => match next.get_text().as_str() {
                        "@Test" => {
                            state = ExtractionState::InTest;
                        }
                        "@Mock" => {
                            state = ExtractionState::InMock;
                        }
                        _ => (), // none test annotation
                    },
                    _ => {
                        // push to top expression
                        top_expression.push(next.clone())
                    }
                }
            }
            ExtractionState::InTest => {
                // get all tokens until first un-nested Subexpression token
                match next.get_token_type() {
                    TokenType::Unknown | TokenType::Subexpression => {
                        // finalize test annotation details

                        // first non space token should be a string for name
                        let non_space = get_first_non_space(&current_extraction, 0);

                        // create details
                        let expression = Vec::from(&current_extraction[non_space.0..]);
                        let details = TestAnnotationDetails::new(TestAnnotation::Test, expression, current_mocks);
                        annotations.push(details);

                        // reset
                        current_extraction = Vec::new();
                        current_mocks = Vec::new();
                        state = ExtractionState::Searching;
                    }
                    _ => {
                        current_extraction.push(next.clone());
                    }
                }
            }
            ExtractionState::InMock => {
                match next.get_token_type() {
                    TokenType::Whitespace if next.get_text().contains('\n') => {
                        // finalize mock details
                        let non_space = get_first_non_space(&current_extraction, 0);
                        let expression = Vec::from(&current_extraction[non_space.0..]);
                        let details = MockAnnotationDetails::new(expression);
                        current_mocks.push(details);

                        current_extraction = Vec::new();
                        state = ExtractionState::Searching;
                    }
                    TokenType::Subexpression => {
                        // finalize mock details
                        let non_space = get_first_non_space(&current_extraction, 0);
                        let expression = Vec::from(&current_extraction[non_space.0..]);
                        let details = MockAnnotationDetails::new(expression);
                        current_mocks.push(details);

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

    let extraction_details = TestDetails::new(top_expression, annotations);

    Ok(extraction_details)
}

#[cfg(test)]
mod tests {
    use garnish_lang_compiler::{lex, LexerToken};

    use crate::test_annotation::{extract_tests, TestAnnotation};

    #[test]
    fn create_test_detail() {
        let tokens = lex("@Test \"Plus 10\" { 5 + 10 == 15 }").unwrap();

        let test_details = extract_tests(&tokens).unwrap();

        assert_eq!(test_details.get_annotations().len(), 1);
        let detail = test_details.get_annotations().get(0).unwrap();

        assert_eq!(detail.get_annotation(), TestAnnotation::Test);
        assert_eq!(detail.get_expression(), &Vec::from(&tokens[2..]));
    }

    #[test]
    fn non_test_expression_is_output() {
        let tokens = lex("5 + 5\n\n@Test \"Plus 10\" { 5 + 10 == 15 }").unwrap();

        let test_details = extract_tests(&tokens).unwrap();

        assert_eq!(test_details.get_expression(), &Vec::from(&tokens[..6]));
        assert_eq!(test_details.get_annotations().len(), 1);

        let detail = test_details.get_annotations().get(0).unwrap();
        assert_eq!(detail.get_annotation(), TestAnnotation::Test);
        assert_eq!(detail.get_expression(), &Vec::from(&tokens[8..]));
    }

    #[test]
    fn separate_multiple_tests_with_sub_expression() {
        let tokens = lex("5 + 5\n\n@Test \"Plus 10\" { 5 + 10 == 15 }\n\n@Test \"Plus 20\" { 15 + 20 == 25 }").unwrap();

        let test_details = extract_tests(&tokens).unwrap();

        assert_eq!(test_details.get_expression(), &Vec::from(&tokens[..6]));
        assert_eq!(test_details.get_annotations().len(), 2);

        let detail = test_details.get_annotations().get(0).unwrap();
        assert_eq!(detail.get_annotation(), TestAnnotation::Test);
        assert_eq!(detail.get_expression(), &Vec::from(&tokens[8..23]));

        let detail = test_details.get_annotations().get(1).unwrap();
        assert_eq!(detail.get_annotation(), TestAnnotation::Test);
        assert_eq!(detail.get_expression(), &Vec::from(&tokens[26..]));
    }

    #[test]
    fn separate_top_expression_in_between_multiple_tests() {
        let tokens = lex("5 + 5\n\n@Test \"Plus 10\" { 5 + 10 == 15 }\n\n$ + 10\n\n@Test \"Plus 20\" { 15 + 20 == 25 }").unwrap();

        let test_details = extract_tests(&tokens).unwrap();
        let full_top = tokens[..6].iter().chain(tokens[24..30].iter());
        assert_eq!(test_details.get_expression(), &full_top.cloned().collect::<Vec<LexerToken>>());
        assert_eq!(test_details.get_annotations().len(), 2);

        let detail = test_details.get_annotations().get(0).unwrap();
        assert_eq!(detail.get_annotation(), TestAnnotation::Test);
        assert_eq!(detail.get_expression(), &Vec::from(&tokens[8..23]));

        let detail = test_details.get_annotations().get(1).unwrap();
        assert_eq!(detail.get_annotation(), TestAnnotation::Test);
        assert_eq!(detail.get_expression(), &Vec::from(&tokens[32..]));
    }

    #[test]
    fn mock_with_new_line() {
        let tokens = lex("5 + 5\n\n@Mock value 20\n@Test \"Plus 10\" { 5 + 10 == 15 }").unwrap();

        let test_details = extract_tests(&tokens).unwrap();

        assert_eq!(test_details.get_expression(), &Vec::from(&tokens[..6]));
        assert_eq!(test_details.get_annotations().len(), 1);

        let detail = test_details.get_annotations().get(0).unwrap();
        assert_eq!(detail.get_annotation(), TestAnnotation::Test);
        assert_eq!(detail.get_expression(), &Vec::from(&tokens[14..]));
        assert_eq!(detail.get_mocks().len(), 1);

        let mock = detail.get_mocks().get(0).unwrap();
        assert_eq!(mock.get_expression(), &Vec::from(&tokens[8..11]));
    }

    #[test]
    fn mock_with_sub_expression() {
        let tokens = lex("5 + 5\n\n@Mock value 20\n\n@Test \"Plus 10\" { 5 + 10 == 15 }").unwrap();

        let test_details = extract_tests(&tokens).unwrap();

        assert_eq!(test_details.get_expression(), &Vec::from(&tokens[..6]));
        assert_eq!(test_details.get_annotations().len(), 1);

        let detail = test_details.get_annotations().get(0).unwrap();
        assert_eq!(detail.get_annotation(), TestAnnotation::Test);
        assert_eq!(detail.get_expression(), &Vec::from(&tokens[14..]));
        assert_eq!(detail.get_mocks().len(), 1);

        let mock = detail.get_mocks().get(0).unwrap();
        assert_eq!(mock.get_expression(), &Vec::from(&tokens[8..11]));
    }
}
