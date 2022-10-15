use garnish_lang_compiler::{LexerToken, TokenType};

pub struct TestInfo {
    annotation: TestAnnotation,
    tokens: Vec<LexerToken>,
}

impl TestInfo {
    pub fn annotation(&self) -> TestAnnotation {
        self.annotation
    }

    pub fn tokens(&self) -> &Vec<LexerToken> {
        &self.tokens
    }
}

pub struct TestingInfo {
    main: Vec<LexerToken>,
    tests: Vec<TestInfo>,
}

impl TestingInfo {
    pub fn tests(&self) -> &Vec<TestInfo> {
        &self.tests
    }

    pub fn main(&self) -> &Vec<LexerToken> {
        &self.main
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum TestAnnotation {
    Test,
    Case,
}

pub fn parse_tests(tokens: Vec<LexerToken>) -> Result<TestingInfo, String> {
    let mut info = TestingInfo { tests: vec![], main: vec![] };

    let mut current_tokens = vec![];
    let mut ingesting_annotation = None;

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
                    _ => (),
                },
                _ => {
                    info.main.push(token);
                }
            },
            Some(a) => match token.get_token_type() {
                TokenType::Subexpression => {
                    info.tests.push(TestInfo {
                        annotation: a,
                        tokens: current_tokens,
                    });
                    current_tokens = vec![];
                    ingesting_annotation = None;
                }
                _ => {
                    current_tokens.push(token);
                }
            },
        }
    }

    // add any hanging tests
    match ingesting_annotation {
        Some(a) => {
            info.tests.push(TestInfo {
                annotation: a,
                tokens: current_tokens,
            });
        }
        None => (),
    }

    Ok(info)
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
    fn multiple_tests() {
        let input = "@Test \"Five equals Five\" { 5 == 5 }\n\n@Test \"Ten equals Ten\" { 10 == 10 }\n\n@Test \"Twenty equals Twenty\" { 20 == 20 }";

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
        let input = "$ + 5\n\n@Case \"Five plus Five equals 10\" 5 10\n\n@Case \"Five plus 10 equals 15\" 10 15\n\n@Case \"Five plus 20 equals 10\" 20 25";

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
}
