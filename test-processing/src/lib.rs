use garnish_lang_compiler::{LexerToken, TokenType};

pub struct TestInfo {
    tokens: Vec<LexerToken>,
}

impl TestInfo {
    pub fn tokens(&self) -> &Vec<LexerToken> {
        &self.tokens
    }
}

pub struct TestingInfo {
    tests: Vec<TestInfo>,
}

impl TestingInfo {
    pub fn tests(&self) -> &Vec<TestInfo> {
        &self.tests
    }
}

pub fn parse_tests(tokens: Vec<LexerToken>) -> Result<TestingInfo, String> {
    let mut info = TestingInfo { tests: vec![] };

    let mut current_tokens = vec![];
    let mut ingesting = false;

    for token in tokens {
        if !ingesting {
            match token.get_token_type() {
                TokenType::Annotation => {
                    match token.get_text().as_ref() {
                        "@Test" => {
                            ingesting = true;
                        }
                        _ => ()
                    }
                }
                _ => ()
            }
        } else {
            match token.get_token_type() {
                TokenType::Subexpression => {
                    ingesting = false;
                    info.tests.push(TestInfo {tokens: current_tokens});
                    current_tokens = vec![];
                }
                _ => {
                    current_tokens.push(token);
                }
            }
        }
    }

    // add any hanging tests
    if ingesting {
        info.tests.push(TestInfo {tokens: current_tokens});
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
}
