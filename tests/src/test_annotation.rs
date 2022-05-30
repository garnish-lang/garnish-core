use garnish_lang_compiler::LexerToken;

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

pub struct TestAnnotationDetails {
    name: String,
    expression: Vec<LexerToken>,
}

pub struct TestDetails {
    tests: Vec<TestAnnotationDetails>
}

impl TestDetails {
    pub fn get_tests(&self) -> &Vec<TestAnnotationDetails> {
        &self.tests
    }
}



#[cfg(test)]
mod tests {
    use garnish_lang_compiler::lex;

    #[test]
    fn gathers_all_parts() {
        let tokens = lex("@Test \"Plus 10\" { 5 + 10 == 15 }").unwrap();

        let test_details = extract_tests(&tokens);

        assert_eq!(test_details.get_tests().len(), 1);
    }
}