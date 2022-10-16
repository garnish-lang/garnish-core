use garnish_lang_compiler::{LexerToken, TokenType};

pub struct TestInfo {
    annotation: TestAnnotation,
    tokens: Vec<LexerToken>,
    mocks: Vec<MockInfo>,
    tag_annotation: Vec<LexerToken>,
}

impl TestInfo {
    pub fn annotation(&self) -> TestAnnotation {
        self.annotation
    }

    pub fn tokens(&self) -> &Vec<LexerToken> {
        &self.tokens
    }

    pub fn mocks(&self) -> &Vec<MockInfo> {
        &self.mocks
    }

    pub fn tag_annotation(&self) -> &Vec<LexerToken> {
        &self.tag_annotation
    }
}

pub struct MockInfo {
    tokens: Vec<LexerToken>,
}

impl MockInfo {
    pub fn tokens(&self) -> &Vec<LexerToken> {
        &self.tokens
    }
}

pub struct TestingInfo {
    main: Vec<LexerToken>,
    tests: Vec<TestInfo>,
    mocks: Vec<MockInfo>,
    tags: Vec<LexerToken>,
    before_all: Vec<LexerToken>,
    after_all: Vec<LexerToken>,
    before_each: Vec<LexerToken>,
    after_each: Vec<LexerToken>,
}

impl TestingInfo {
    pub fn tests(&self) -> &Vec<TestInfo> {
        &self.tests
    }

    pub fn main(&self) -> &Vec<LexerToken> {
        &self.main
    }

    pub fn mocks(&self) -> &Vec<MockInfo> {
        &self.mocks
    }

    pub fn tags(&self) -> &Vec<LexerToken> {
        &self.tags
    }

    pub fn before_all(&self) -> &Vec<LexerToken> {
        &self.before_all
    }

    pub fn after_all(&self) -> &Vec<LexerToken> {
        &self.after_all
    }

    pub fn before_each(&self) -> &Vec<LexerToken> {
        &self.before_each
    }

    pub fn after_each(&self) -> &Vec<LexerToken> {
        &self.after_each
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum TestAnnotation {
    Test,
    Case,
    Mock,
    Tag,
    MockAll,
    TagAll,
    BeforeAll,
    AfterAll,
    BeforeEach,
    AfterEach,
}

pub fn parse_tests(tokens: Vec<LexerToken>) -> Result<TestingInfo, String> {
    let mut info = TestingInfo {
        tests: vec![],
        mocks: vec![],
        tags: vec![],
        before_all: vec![],
        after_all: vec![],
        before_each: vec![],
        main: vec![],
        after_each: vec![]
    };

    let mut current_mocks = vec![];
    let mut current_tokens = vec![];
    let mut tag_annotation = vec![];
    let mut ingesting_annotation = None;

    // keep track of how many groupings we enter when ingesting
    // no need to match types, that will be done by parser
    let mut nested_depth = 0;

    for token in tokens {
        match ingesting_annotation {
            None => match token.get_token_type() {
                TokenType::Annotation => match token.get_text().as_ref() {
                    "@Test" => {
                        ingesting_annotation = Some(TestAnnotation::Test);
                    }
                    "@Case" => {
                        ingesting_annotation = Some(TestAnnotation::Case);
                    }
                    "@Mock" => ingesting_annotation = Some(TestAnnotation::Mock),
                    "@Tag" => ingesting_annotation = Some(TestAnnotation::Tag),
                    "@MockAll" => ingesting_annotation = Some(TestAnnotation::MockAll),
                    "@TagAll" => ingesting_annotation = Some(TestAnnotation::TagAll),
                    "@BeforeAll" => ingesting_annotation = Some(TestAnnotation::BeforeAll),
                    "@AfterAll" => ingesting_annotation = Some(TestAnnotation::AfterAll),
                    "@BeforeEach" => ingesting_annotation = Some(TestAnnotation::BeforeEach),
                    "@AfterEach" => ingesting_annotation = Some(TestAnnotation::AfterEach),
                    _ => (),
                },
                _ => {
                    info.main.push(token);
                }
            },
            Some(a) => match token.get_token_type() {
                TokenType::Whitespace | TokenType::Subexpression if token.get_text().contains("\n") && nested_depth == 0 => {
                    // end ingestion
                    match a {
                        TestAnnotation::Test | TestAnnotation::Case => {
                            info.tests.push(TestInfo {
                                annotation: a,
                                tokens: current_tokens,
                                mocks: current_mocks,
                                tag_annotation
                            });

                            tag_annotation = vec![];
                            current_mocks = vec![];
                        }
                        TestAnnotation::Mock => {
                            current_mocks.push(MockInfo { tokens: current_tokens });
                        }
                        TestAnnotation::Tag => {
                            tag_annotation = current_tokens;
                        }
                        TestAnnotation::MockAll => {
                            info.mocks.push(MockInfo { tokens: current_tokens });
                        }
                        TestAnnotation::TagAll => {
                            info.tags = current_tokens;
                        }
                        TestAnnotation::BeforeAll => {
                            info.before_all = current_tokens;
                        }
                        TestAnnotation::AfterAll => {
                            info.after_all = current_tokens;
                        }
                        TestAnnotation::BeforeEach => {
                            info.before_each = current_tokens;
                        }
                        TestAnnotation::AfterEach => {
                            info.after_each = current_tokens;
                        }
                    }

                    current_tokens = vec![];
                    ingesting_annotation = None;
                }
                TokenType::StartExpression | TokenType::StartGroup | TokenType::StartSideEffect => {
                    nested_depth += 1;
                    current_tokens.push(token);
                }
                TokenType::EndExpression | TokenType::EndGroup | TokenType::EndSideEffect => {
                    nested_depth -= 1;
                    current_tokens.push(token);
                }
                _ => current_tokens.push(token)
            }
        }
    }

    // add any hanging tests
    match ingesting_annotation {
        Some(a) => {
            info.tests.push(TestInfo {
                annotation: a,
                tokens: current_tokens,
                mocks: current_mocks,
                tag_annotation,
            });
        }
        None => (),
    }

    if nested_depth > 0 {
        Err(format!("Unterminated grouping"))
    } else {
        Ok(info)
    }
}

#[cfg(test)]
mod tests {
    use garnish_lang_compiler::lex;

    use super::*;

    #[test]
    fn single_test() {
        let input = "@Test \"Five equals Five\" { 5 == 5 }";

        let tokens = lex(input).unwrap();

        let testing_info = parse_tests(tokens.clone()).unwrap();

        assert_eq!(testing_info.tests().len(), 1);
        assert_eq!(testing_info.tests().get(0).unwrap().tokens(), &Vec::from(&tokens[1..]));
        assert_eq!(testing_info.tests().get(0).unwrap().annotation(), TestAnnotation::Test);
    }

    #[test]
    fn single_test_across_multiple_lines() {
        let input = "@Test \"Five equals Five\" { \n(5 == 5\n&& 4 == 4)\n\n[log ~ \"Debug message\"]\n\n$ && 3 == 3\n }";

        let tokens = lex(input).unwrap();

        let testing_info = parse_tests(tokens.clone()).unwrap();

        assert_eq!(testing_info.tests().len(), 1);
        assert_eq!(testing_info.tests().get(0).unwrap().tokens(), &Vec::from(&tokens[1..]));
        assert_eq!(testing_info.tests().get(0).unwrap().annotation(), TestAnnotation::Test);
    }

    #[test]
    fn unterminated_grouping_is_error() {
        let input = "@Test \"Five equals Five\" { \n(5 == 5\n&& 4 == 4)\n\n[log ~ \"Debug message\"\n\n$ && 3 == 3\n }";

        let tokens = lex(input).unwrap();

        let testing_info = parse_tests(tokens.clone());

        assert!(testing_info.is_err());
    }

    #[test]
    fn multiple_tests_separated_by_sub_expression() {
        let input = "@Test \"Five equals Five\" { 5 == 5 }\n\n@Test \"Ten equals Ten\" { 10 == 10 }\n\n@Test \"Twenty equals Twenty\" { 20 == 20 }";

        let tokens = lex(input).unwrap();

        let testing_info = parse_tests(tokens.clone()).unwrap();

        assert_eq!(testing_info.tests().len(), 3);
        assert_eq!(testing_info.tests().get(0).unwrap().tokens(), &Vec::from(&tokens[1..13]));
        assert_eq!(testing_info.tests().get(1).unwrap().tokens(), &Vec::from(&tokens[15..27]));
        assert_eq!(testing_info.tests().get(2).unwrap().tokens(), &Vec::from(&tokens[29..]));
    }

    #[test]
    fn multiple_tests_separated_by_whitespace() {
        let input = "@Test \"Five equals Five\" { 5 == 5 }\n@Test \"Ten equals Ten\" { 10 == 10 }\n@Test \"Twenty equals Twenty\" { 20 == 20 }";

        let tokens = lex(input).unwrap();

        let testing_info = parse_tests(tokens.clone()).unwrap();

        assert_eq!(testing_info.tests().len(), 3);
        assert_eq!(testing_info.tests().get(0).unwrap().tokens(), &Vec::from(&tokens[1..13]));
        assert_eq!(testing_info.tests().get(1).unwrap().tokens(), &Vec::from(&tokens[15..27]));
        assert_eq!(testing_info.tests().get(2).unwrap().tokens(), &Vec::from(&tokens[29..]));
    }

    #[test]
    fn non_test_stored_as_main() {
        let input = "$ + 5";

        let tokens = lex(input).unwrap();

        let testing_info = parse_tests(tokens.clone()).unwrap();

        assert_eq!(testing_info.tests().len(), 0);
        assert_eq!(testing_info.main(), &Vec::from(&tokens[..]));
    }

    #[test]
    fn main_stored_between_tests() {
        let input = "$ + 5\n\n@Test \"Five equals Five\" { 5 == 5 }\n\n$ + 10";

        let tokens = lex(input).unwrap();

        let testing_info = parse_tests(tokens.clone()).unwrap();

        let main_tokens: Vec<LexerToken> = tokens.iter().take(6).chain(tokens.iter().skip(20)).map(|t| t.clone()).collect();

        assert_eq!(testing_info.tests().len(), 1);
        assert_eq!(testing_info.main(), &main_tokens);
    }

    #[test]
    fn single_case() {
        let input = "@Case \"Five plus Five equals 10\" 5 10";

        let tokens = lex(input).unwrap();

        let testing_info = parse_tests(tokens.clone()).unwrap();

        assert_eq!(testing_info.tests().len(), 1);
        assert_eq!(testing_info.tests().get(0).unwrap().tokens(), &Vec::from(&tokens[1..]));
        assert_eq!(testing_info.tests().get(0).unwrap().annotation(), TestAnnotation::Case);
    }

    #[test]
    fn case_with_main() {
        let input = "$ + 5\n\n@Case \"Five plus Five equals 10\" 5 10";

        let tokens = lex(input).unwrap();

        let testing_info = parse_tests(tokens.clone()).unwrap();

        let main_tokens: Vec<LexerToken> = tokens.iter().take(6).map(|t| t.clone()).collect();

        assert_eq!(testing_info.tests().len(), 1);
        assert_eq!(testing_info.main(), &main_tokens);
        assert_eq!(testing_info.tests().get(0).unwrap().annotation(), TestAnnotation::Case);
    }

    #[test]
    fn multiple_cases() {
        let input =
            "$ + 5\n\n@Case \"Five plus Five equals 10\" 5 10\n\n@Case \"Five plus 10 equals 15\" 10 15\n\n@Case \"Five plus 20 equals 10\" 20 25";

        let tokens = lex(input).unwrap();

        let testing_info = parse_tests(tokens.clone()).unwrap();

        assert_eq!(testing_info.tests().len(), 3);
        assert_eq!(testing_info.tests().get(0).unwrap().tokens(), &Vec::from(&tokens[7..13]));
        assert_eq!(testing_info.tests().get(0).unwrap().annotation(), TestAnnotation::Case);
        assert_eq!(testing_info.tests().get(1).unwrap().tokens(), &Vec::from(&tokens[15..21]));
        assert_eq!(testing_info.tests().get(1).unwrap().annotation(), TestAnnotation::Case);
        assert_eq!(testing_info.tests().get(2).unwrap().tokens(), &Vec::from(&tokens[23..]));
        assert_eq!(testing_info.tests().get(2).unwrap().annotation(), TestAnnotation::Case);
    }

    #[test]
    fn case_with_mock() {
        let input = "get_points~~ + $\n\n@Mock get_points 10\n@Case \"Add 5 to point value\" 5 15";

        let tokens = lex(input).unwrap();

        let testing_info = parse_tests(tokens.clone()).unwrap();

        assert_eq!(testing_info.tests().len(), 1);
        assert_eq!(testing_info.tests().get(0).unwrap().tokens(), &Vec::from(&tokens[14..]));
        assert_eq!(testing_info.tests().get(0).unwrap().annotation(), TestAnnotation::Case);
        assert_eq!(
            testing_info.tests().get(0).unwrap().mocks().get(0).unwrap().tokens(),
            &Vec::from(&tokens[8..12])
        );
    }

    #[test]
    fn test_with_tags() {
        let input = "@Tag optional database\n@Test \"Five equals Five\" { 5 == 5 }";

        let tokens = lex(input).unwrap();

        let testing_info = parse_tests(tokens.clone()).unwrap();

        assert_eq!(testing_info.tests().len(), 1);
        assert_eq!(testing_info.tests().get(0).unwrap().tokens(), &Vec::from(&tokens[7..]));
        assert_eq!(testing_info.tests().get(0).unwrap().annotation(), TestAnnotation::Test);
        assert_eq!(testing_info.tests().get(0).unwrap().tag_annotation(), &Vec::from(&tokens[1..5]));
    }

    #[test]
    fn mock_all() {
        let input = "@MockAll get_points 10\n\n@Test \"Five equals Five\" { 5 == 5 }";

        let tokens = lex(input).unwrap();

        let testing_info = parse_tests(tokens.clone()).unwrap();

        assert_eq!(testing_info.mocks().get(0).unwrap().tokens(), &Vec::from(&tokens[1..5]));
    }

    #[test]
    fn tag_all() {
        let input = "@TagAll optional database\n\n@Test \"Five equals Five\" { 5 == 5 }";

        let tokens = lex(input).unwrap();

        let testing_info = parse_tests(tokens.clone()).unwrap();

        assert_eq!(testing_info.tags(), &Vec::from(&tokens[1..5]));
    }

    #[test]
    fn before_all() {
        let input = "@BeforeAll { setup~~ }\n\n@Test \"Five equals Five\" { 5 == 5 }";

        let tokens = lex(input).unwrap();

        let testing_info = parse_tests(tokens.clone()).unwrap();

        assert_eq!(testing_info.before_all(), &Vec::from(&tokens[1..8]));
    }

    #[test]
    fn after_all() {
        let input = "@AfterAll { tear_down~~ }\n\n@Test \"Five equals Five\" { 5 == 5 }";

        let tokens = lex(input).unwrap();

        let testing_info = parse_tests(tokens.clone()).unwrap();

        assert_eq!(testing_info.after_all(), &Vec::from(&tokens[1..8]));
    }

    #[test]
    fn before_each() {
        let input = "@BeforeEach { add_data~~ }\n\n@Test \"Five equals Five\" { 5 == 5 }";

        let tokens = lex(input).unwrap();

        let testing_info = parse_tests(tokens.clone()).unwrap();

        assert_eq!(testing_info.before_each(), &Vec::from(&tokens[1..8]));
    }

    #[test]
    fn after_each() {
        let input = "@AfterEach { clear_data~~ }\n\n@Test \"Five equals Five\" { 5 == 5 }";

        let tokens = lex(input).unwrap();

        let testing_info = parse_tests(tokens.clone()).unwrap();

        assert_eq!(testing_info.after_each(), &Vec::from(&tokens[1..8]));
    }
}
