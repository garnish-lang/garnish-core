use std::fmt::Debug;

use garnish_lang_compiler::{build_with_data, parse, LexerToken, TokenType};
use garnish_traits::{
    ExpressionDataType, GarnishLangRuntimeContext, GarnishLangRuntimeData, GarnishLangRuntimeState, GarnishNumber, GarnishRuntime, Instruction,
    RuntimeError, TypeConstants,
};

use crate::test_annotation::{TestAnnotation, TestAnnotationDetails, TestDetails, TestExtractionError};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TestResult<Data: GarnishLangRuntimeData> {
    test_details: TestAnnotationDetails,
    success: bool,
    error: Option<String>,
    actual_value: Option<Data::Size>,
    name_value: Option<Data::Size>,
}

impl<Data: GarnishLangRuntimeData> TestResult<Data> {
    fn new(
        success: bool,
        error: Option<String>,
        actual_value: Option<Data::Size>,
        name_value: Option<Data::Size>,
        test_details: TestAnnotationDetails,
    ) -> Self {
        TestResult {
            success,
            error,
            actual_value,
            name_value,
            test_details,
        }
    }

    pub fn is_success(&self) -> bool {
        self.success
    }

    pub fn error(&self) -> Option<&String> {
        self.error.as_ref()
    }

    pub fn value(&self) -> Option<Data::Size> {
        self.actual_value.clone()
    }

    pub fn test_details(&self) -> &TestAnnotationDetails {
        &self.test_details
    }

    pub fn name(&self) -> Option<Data::Size> {
        self.name_value
    }
}

pub struct ExecutionResult<Data: GarnishLangRuntimeData> {
    test_results: Vec<TestResult<Data>>,
}

impl<Data: GarnishLangRuntimeData> ExecutionResult<Data> {
    fn new(test_results: Vec<TestResult<Data>>) -> Self {
        ExecutionResult { test_results }
    }

    pub fn get_results(&self) -> &Vec<TestResult<Data>> {
        &self.test_results
    }
}

fn execute_test_annotation<Data: GarnishLangRuntimeData, Runtime: GarnishRuntime<Data>, Context: GarnishLangRuntimeContext<Data>>(
    runtime: &mut Runtime,
    context: &mut Context,
    test: &TestAnnotationDetails,
) -> Result<TestResult<Data>, TestExtractionError> {
    let (name_value, expression_value) = match runtime.get_data().get_current_value() {
        None => (None, None),
        Some(value) => match runtime
            .get_data()
            .get_data_type(value)
            .or_else(|err| Err(TestExtractionError::with_tokens(format!("{:?}", err).as_str(), test.get_expression())))?
        {
            ExpressionDataType::List => {
                let name_value = runtime.get_data().get_list_item(value, Data::Number::zero()).map_or(None, |v| Some(v));
                let expression_value = runtime.get_data().get_list_item(value, Data::Number::one()).map_or(None, |v| Some(v));
                (name_value, expression_value)
            }
            _ => (None, None),
        },
    };

    let (_, expression_addr) = match (name_value, expression_value) {
        (Some(name_addr), Some(expression_addr)) => (name_addr, expression_addr),
        _ => {
            return Ok(TestResult::new(
                false,
                Some("Missing test annotation parameters. Expected a name and nested expression.".to_string()),
                None,
                name_value,
                test.clone(),
            ));
        }
    };

    let (success, value, error) = match runtime.get_data().get_expression(expression_addr) {
        Err(e) => (
            false,
            None,
            Some(format!(
                "Non-expression value found when trying to execute test annotation.\nError: {}",
                e
            )),
        ),
        Ok(point) => match runtime.get_data().get_jump_point(point) {
            None => (false, None, Some("Jump point not registered".to_string())),
            Some(start) => {
                execute_until_end(runtime, context, start)?;

                match runtime.get_data().get_current_value() {
                    None => (false, None, Some("Failed to get data from test".to_string())),
                    Some(value) => match runtime.get_data().get_data_type(value) {
                        Err(e) => (false, None, Some(format!("{:?}", e))),
                        Ok(t) => match t {
                            ExpressionDataType::True => (true, Some(value), None),
                            ExpressionDataType::False => (false, Some(value), None),
                            _ => (false, None, Some("Value not of type True or False".to_string())),
                        },
                    },
                }
            }
        },
    };

    Ok(TestResult::<Data>::new(success, error, value, name_value, test.clone()))
}

fn execute_until_end<Data: GarnishLangRuntimeData, Runtime: GarnishRuntime<Data>, Context: GarnishLangRuntimeContext<Data>>(
    runtime: &mut Runtime,
    context: &mut Context,
    start: Data::Size,
) -> Result<(), TestExtractionError> {
    runtime
        .get_data_mut()
        .set_instruction_cursor(start)
        .or_else(|err| Err(TestExtractionError::error(format!("{:?}", err).as_str())))?;

    loop {
        match runtime.execute_current_instruction(Some(context)) {
            Err(e) => return Err(TestExtractionError::error(e.to_string().as_str()))?,
            Ok(data) => match data.get_state() {
                GarnishLangRuntimeState::Running => (),
                GarnishLangRuntimeState::End => break,
            },
        }
    }

    Ok(())
}

fn execute_case_annotation<Data: GarnishLangRuntimeData, Runtime: GarnishRuntime<Data>, Context: GarnishLangRuntimeContext<Data>>(
    runtime: &mut Runtime,
    context: &mut Context,
    test: &TestAnnotationDetails,
    top_expression: Data::Size,
) -> Result<TestResult<Data>, TestExtractionError> {
    let (name_value, input_value, output_value) = match runtime.get_data().get_current_value() {
        None => (None, None, None),
        Some(value) => match runtime
            .get_data()
            .get_data_type(value)
            .or_else(|err| Err(TestExtractionError::with_tokens(format!("{:?}", err).as_str(), test.get_expression())))?
        {
            ExpressionDataType::List => {
                let name_value = runtime.get_data().get_list_item(value, Data::Number::zero()).map_or(None, |v| Some(v));
                let expression_value = runtime.get_data().get_list_item(value, Data::Number::one()).map_or(None, |v| Some(v));
                let output_value = runtime
                    .get_data()
                    .get_list_item(
                        value,
                        Data::Number::one().increment().map_or(
                            Err(TestExtractionError::with_tokens(
                                "Could not create the number 2 with Data's Size associated type.",
                                test.get_expression(),
                            )),
                            |v| Ok(v),
                        )?,
                    )
                    .map_or(None, |v| Some(v));
                (name_value, expression_value, output_value)
            }
            _ => (None, None, None),
        },
    };

    let (input_addr, output_addr) = match (name_value, input_value, output_value) {
        (Some(_), Some(input), Some(output)) => (input, output),
        _ => {
            return Ok(TestResult::new(
                false,
                Some("Missing case annotation parameters. Expected a name, input and expected output.".to_string()),
                None,
                name_value,
                test.clone(),
            ));
        }
    };

    // push input value to input stack
    runtime
        .get_data_mut()
        .push_value_stack(input_addr)
        .or_else(|err| Err(TestExtractionError::with_tokens(format!("{:?}", err).as_str(), test.get_expression())))?;

    // execute top expression
    execute_until_end(runtime, context, top_expression)?;

    let (success, value, error) = match runtime.get_data().get_current_value() {
        None => (false, None, Some("No current value available after top expression execution".to_string())),
        Some(value_addr) => {
            runtime
                .get_data_mut()
                .push_register(value_addr)
                .or_else(|err| Err(TestExtractionError::with_tokens(format!("{:?}", err).as_str(), test.get_expression())))?;

            runtime
                .get_data_mut()
                .push_register(output_addr)
                .or_else(|err| Err(TestExtractionError::with_tokens(format!("{:?}", err).as_str(), test.get_expression())))?;

            runtime
                .equal()
                .or_else(|err| Err(TestExtractionError::with_tokens(format!("{:?}", err).as_str(), test.get_expression())))?;

            runtime
                .update_value()
                .or_else(|err| Err(TestExtractionError::with_tokens(format!("{:?}", err).as_str(), test.get_expression())))?;

            match runtime.get_data().get_current_value() {
                None => (false, None, Some("No value available after equality comparison".to_string())),
                Some(result_addr) => {
                    match runtime
                        .get_data()
                        .get_data_type(result_addr)
                        .or_else(|err| Err(TestExtractionError::with_tokens(format!("{:?}", err).as_str(), test.get_expression())))?
                    {
                        ExpressionDataType::True => (true, Some(value_addr), None),
                        ExpressionDataType::False => (false, Some(value_addr), None),
                        t => (
                            false,
                            Some(value_addr),
                            Some(format!("Value after equality is {:?}, expected True or False", t)),
                        ),
                    }
                }
            }
        }
    };

    Ok(TestResult::<Data>::new(success, error, value, name_value, test.clone()))
}

struct TestExecutionContext<Data: GarnishLangRuntimeData> {
    caller_context: Option<Box<dyn GarnishLangRuntimeContext<Data>>>,
    mocks: Vec<(Data::Symbol, Data::Size)>,
}

impl<Data: GarnishLangRuntimeData> TestExecutionContext<Data> {
    fn new(c: Option<Box<dyn GarnishLangRuntimeContext<Data>>>) -> Self {
        TestExecutionContext {
            caller_context: c,
            mocks: vec![],
        }
    }

    fn register_mock(&mut self, sym: Data::Symbol, value: Data::Size) {
        self.mocks.push((sym, value))
    }
}

impl<Data: GarnishLangRuntimeData> GarnishLangRuntimeContext<Data> for TestExecutionContext<Data> {
    fn resolve(&mut self, symbol: Data::Symbol, runtime: &mut Data) -> Result<bool, RuntimeError<Data::Error>> {
        for (sym, value) in self.mocks.iter() {
            if sym == &symbol {
                runtime.push_register(*value)?;
                return Ok(true);
            }
        }

        match &mut self.caller_context {
            None => Ok(false),
            Some(context) => context.resolve(symbol, runtime),
        }
    }

    fn apply(&mut self, external_value: Data::Size, input_addr: Data::Size, runtime: &mut Data) -> Result<bool, RuntimeError<Data::Error>> {
        match &mut self.caller_context {
            None => Ok(false),
            Some(context) => context.apply(external_value, input_addr, runtime),
        }
    }

    fn defer_op(
        &mut self,
        runtime: &mut Data,
        operation: Instruction,
        left: (ExpressionDataType, Data::Size),
        right: (ExpressionDataType, Data::Size),
    ) -> Result<bool, RuntimeError<Data::Error>> {
        match &mut self.caller_context {
            None => Ok(false),
            Some(context) => context.defer_op(runtime, operation, left, right),
        }
    }
}

fn is_expression_empty(tokens: &Vec<LexerToken>) -> bool {
    for t in tokens {
        match t.get_token_type() {
            TokenType::Whitespace | TokenType::Subexpression | TokenType::Unknown => (),
            // any other token means expression is not empty
            _ => return false,
        }
    }

    return true;
}

pub fn execute_tests<Data: GarnishLangRuntimeData, Runtime: GarnishRuntime<Data>>(
    runtime: &mut Runtime,
    tests: &TestDetails,
    top_expression: Option<Data::Size>,
) -> Result<ExecutionResult<Data>, TestExtractionError> {
    execute_tests_with_context(runtime, tests, top_expression, || None)
}

pub(crate) fn lex_token_string(tokens: &Vec<LexerToken>) -> String {
    match tokens.get(0) {
        None => String::new(),
        Some(first) => {
            format!(
                "{}; line {} col {}",
                tokens.iter().map(|t| t.get_text().clone()).collect::<Vec<String>>().join(""),
                first.get_line(),
                first.get_column()
            )
        }
    }
}

pub fn execute_tests_with_context<Data: GarnishLangRuntimeData, Runtime: GarnishRuntime<Data>, MakeContextFn>(
    runtime: &mut Runtime,
    tests: &TestDetails,
    top_expression: Option<Data::Size>,
    context_fn: MakeContextFn,
) -> Result<ExecutionResult<Data>, TestExtractionError>
where
    MakeContextFn: FnOnce() -> Option<Box<dyn GarnishLangRuntimeContext<Data>>>,
{
    let mut results = vec![];

    let mut context = TestExecutionContext::new(match context_fn() {
        Some(c) => Some(c),
        None => None,
    });

    'test_loop: for test in tests.get_annotations() {
        // process mocks first
        for mock in test.get_mocks() {
            if is_expression_empty(mock.get_expression()) {
                results.push(TestResult::new(
                    false,
                    Some("Mock expression empty".to_string()),
                    None,
                    None,
                    test.clone(),
                ));
                continue 'test_loop;
            }

            // remove first identifier from mock tokens
            // this is symbol to mock
            let mut identifier = None;
            let mut identifier_loc = 0;
            for (i, token) in mock.get_expression().iter().enumerate() {
                if token.get_token_type() == TokenType::Identifier {
                    identifier = Some(
                        Data::parse_symbol(token.get_text().as_str())
                            .or_else(|e| Err(TestExtractionError::with_tokens(format!("{}", e).as_str(), mock.get_expression())))?,
                    );
                    identifier_loc = i;
                    break;
                }
            }

            let identifier = match identifier {
                Some(i) => i,
                None => {
                    // cannot mock
                    // skip test
                    results.push(TestResult::new(
                        false,
                        Some("No identifier found in mock.".to_string()),
                        None,
                        None,
                        test.clone(),
                    ));
                    continue 'test_loop;
                }
            };

            let mock_expression = Vec::from(&mock.get_expression()[identifier_loc + 1..]);
            if is_expression_empty(&mock_expression) {
                results.push(TestResult::new(
                    false,
                    Some("No expression after identifier in mock".to_string()),
                    None,
                    None,
                    test.clone(),
                ));
                continue 'test_loop;
            }

            // exclude identifier from parse
            let parse_result =
                parse(mock_expression).or_else(|err| Err(TestExtractionError::with_tokens(format!("{}", err).as_str(), mock.get_expression())))?;

            let start = runtime.get_data().get_jump_table_len();

            build_with_data(parse_result.get_root(), parse_result.get_nodes().clone(), runtime.get_data_mut())
                .or_else(|err| Err(TestExtractionError::with_tokens(err.get_message(), mock.get_expression())))?;

            let point = match runtime.get_data().get_jump_point(start) {
                None => {
                    return Err(TestExtractionError::with_tokens(
                        "No starting jump point available",
                        mock.get_expression(),
                    ))
                }
                Some(point) => point,
            };

            execute_until_end(runtime, &mut context, point)?;

            // current value after execution is mock's value
            // register pair in context
            match runtime.get_data().get_current_value() {
                None => Err(TestExtractionError::with_tokens(
                    "No value in runtime after mock execution.",
                    mock.get_expression(),
                ))?,
                Some(current_value) => {
                    context.register_mock(identifier, current_value);
                }
            }
        }

        if is_expression_empty(test.get_expression()) {
            results.push(TestResult::new(
                false,
                Some("Test expression empty".to_string()),
                None,
                None,
                test.clone(),
            ));
            continue;
        }

        let parse_result =
            parse(test.get_expression().clone()).or_else(|err| Err(TestExtractionError::with_tokens(err.get_message(), test.get_expression())))?;

        let start = runtime.get_data().get_jump_table_len();

        build_with_data(parse_result.get_root(), parse_result.get_nodes().clone(), runtime.get_data_mut())
            .or_else(|err| Err(TestExtractionError::with_tokens(err.get_message(), test.get_expression())))?;

        let point = match runtime.get_data().get_jump_point(start) {
            None => {
                return Err(TestExtractionError::with_tokens(
                    "No starting jump point available",
                    test.get_expression(),
                ))
            }
            Some(point) => point,
        };

        execute_until_end(runtime, &mut context, point)?;

        let result = match test.get_annotation() {
            TestAnnotation::Test => execute_test_annotation(runtime, &mut context, test)?,
            TestAnnotation::Case => match top_expression {
                None => TestResult::<Data>::new(
                    false,
                    Some("No top expression provided for Case annotation.".to_string()),
                    None,
                    None,
                    test.clone(),
                ),
                Some(top) => execute_case_annotation(runtime, &mut context, test, top)?,
            },
        };

        results.push(result);
    }

    Ok(ExecutionResult::new(results))
}

#[cfg(test)]
mod tests {
    use garnish_data::SimpleRuntimeData;
    use garnish_lang_compiler::{build_with_data, lex, parse};
    use garnish_lang_runtime::runtime_impls::SimpleGarnishRuntime;
    use garnish_traits::GarnishLangRuntimeData;

    use crate::test_annotation::{execute_tests, extract_tests};

    #[test]
    fn execute_true_test() {
        let mut data = SimpleRuntimeData::new();

        let input = lex("@Test \"5 equals 5\" { 5 == 5 }").unwrap();
        let tests = extract_tests(&input).unwrap();

        // caller needs space to set up data as well, let them build top expression
        let parse_result = parse(tests.get_expression().clone()).unwrap();
        // let top_expression = data.get_jump_table_len();
        build_with_data(parse_result.get_root(), parse_result.get_nodes().clone(), &mut data).unwrap();

        let mut runtime = SimpleGarnishRuntime::new(data);
        let results = execute_tests(&mut runtime, &tests, None).unwrap();

        assert_eq!(results.get_results().len(), 1);

        let first = results.get_results().get(0).unwrap();
        assert_eq!(first.error(), None);
        assert!(first.is_success());
        assert_eq!(first.value(), Some(2));
        assert_eq!(first.test_details(), tests.get_annotations().get(0).unwrap());
    }

    #[test]
    fn execute_false_test() {
        let mut data = SimpleRuntimeData::new();

        let input = lex("@Test \"5 equals 10\" { 5 == 10 }").unwrap();
        let tests = extract_tests(&input).unwrap();

        // caller needs space to set up data as well, let them build top expression
        let parse_result = parse(tests.get_expression().clone()).unwrap();
        // let top_expression = data.get_jump_table_len();
        build_with_data(parse_result.get_root(), parse_result.get_nodes().clone(), &mut data).unwrap();

        let mut runtime = SimpleGarnishRuntime::new(data);
        let results = execute_tests(&mut runtime, &tests, None).unwrap();

        assert_eq!(results.get_results().len(), 1);

        let first = results.get_results().get(0).unwrap();
        assert_eq!(first.error(), None);
        assert!(!first.is_success());
        assert_eq!(first.value(), Some(1));
        assert_eq!(first.test_details(), tests.get_annotations().get(0).unwrap());
    }

    #[test]
    fn test_without_expression_has_error_in_result() {
        let mut data = SimpleRuntimeData::new();

        let input = lex("@Test \"5 equals 5\" not_an_expression").unwrap();
        let tests = extract_tests(&input).unwrap();

        // caller needs space to set up data as well, let them build top expression
        let parse_result = parse(tests.get_expression().clone()).unwrap();
        // let top_expression = data.get_jump_table_len();
        build_with_data(parse_result.get_root(), parse_result.get_nodes().clone(), &mut data).unwrap();

        let mut runtime = SimpleGarnishRuntime::new(data);
        let results = execute_tests(&mut runtime, &tests, None).unwrap();

        assert_eq!(results.get_results().len(), 1);

        let first = results.get_results().get(0).unwrap();
        assert!(first
            .error()
            .unwrap()
            .contains("Non-expression value found when trying to execute test annotation."));
        assert!(!first.is_success());
        assert_eq!(first.value(), None);
        assert_eq!(first.test_details(), tests.get_annotations().get(0).unwrap());
    }

    #[test]
    fn test_with_no_parameters_has_error_in_result() {
        let mut data = SimpleRuntimeData::new();

        let input = lex("@Test ").unwrap();
        let tests = extract_tests(&input).unwrap();

        // caller needs space to set up data as well, let them build top expression
        let parse_result = parse(tests.get_expression().clone()).unwrap();
        // let top_expression = data.get_jump_table_len();
        build_with_data(parse_result.get_root(), parse_result.get_nodes().clone(), &mut data).unwrap();

        let mut runtime = SimpleGarnishRuntime::new(data);
        let results = execute_tests(&mut runtime, &tests, None).unwrap();

        assert_eq!(results.get_results().len(), 1);

        let first = results.get_results().get(0).unwrap();
        assert!(first.error().unwrap().contains("Test expression empty"));
        assert!(!first.is_success());
        assert_eq!(first.value(), None);
        assert_eq!(first.test_details(), tests.get_annotations().get(0).unwrap());
    }

    #[test]
    fn test_with_only_one_parameter_has_error_in_result() {
        let mut data = SimpleRuntimeData::new();

        let input = lex("@Test \"5 equals 5\"").unwrap();
        let tests = extract_tests(&input).unwrap();

        // caller needs space to set up data as well, let them build top expression
        let parse_result = parse(tests.get_expression().clone()).unwrap();
        // let top_expression = data.get_jump_table_len();
        build_with_data(parse_result.get_root(), parse_result.get_nodes().clone(), &mut data).unwrap();

        let mut runtime = SimpleGarnishRuntime::new(data);
        let results = execute_tests(&mut runtime, &tests, None).unwrap();

        assert_eq!(results.get_results().len(), 1);

        let first = results.get_results().get(0).unwrap();
        assert!(first
            .error()
            .unwrap()
            .contains("Missing test annotation parameters. Expected a name and nested expression."));
        assert!(!first.is_success());
        assert_eq!(first.value(), None);
        assert_eq!(first.test_details(), tests.get_annotations().get(0).unwrap());
    }

    #[test]
    fn execute_true_case() {
        let mut data = SimpleRuntimeData::new();

        let input = lex("$ + 5\n\n@Case \"5 + 5 is 10\" 5 10").unwrap();
        let tests = extract_tests(&input).unwrap();

        // caller needs space to set up data as well, let them build top expression
        let parse_result = parse(tests.get_expression().clone()).unwrap();
        let top_expression = data.get_jump_table_len();
        build_with_data(parse_result.get_root(), parse_result.get_nodes().clone(), &mut data).unwrap();

        let mut runtime = SimpleGarnishRuntime::new(data);
        let results = execute_tests(&mut runtime, &tests, Some(top_expression)).unwrap();

        assert_eq!(results.get_results().len(), 1);

        let first = results.get_results().get(0).unwrap();
        assert_eq!(first.error(), None);
        assert!(first.is_success());
        assert_eq!(first.value(), Some(5));
        assert_eq!(first.test_details(), tests.get_annotations().get(0).unwrap());
    }

    #[test]
    fn execute_false_case() {
        let mut data = SimpleRuntimeData::new();

        let input = lex("$ + 10\n\n@Case \"5 + 5 is 10\" 5 10").unwrap();
        let tests = extract_tests(&input).unwrap();

        // caller needs space to set up data as well, let them build top expression
        let parse_result = parse(tests.get_expression().clone()).unwrap();
        let top_expression = data.get_jump_table_len();
        build_with_data(parse_result.get_root(), parse_result.get_nodes().clone(), &mut data).unwrap();

        let mut runtime = SimpleGarnishRuntime::new(data);
        let results = execute_tests(&mut runtime, &tests, Some(top_expression)).unwrap();

        assert_eq!(results.get_results().len(), 1);

        let first = results.get_results().get(0).unwrap();
        assert_eq!(first.error(), None);
        assert!(!first.is_success());
        assert_eq!(first.value(), Some(7));
        assert_eq!(first.test_details(), tests.get_annotations().get(0).unwrap());
    }

    #[test]
    fn case_with_only_two_parameters() {
        let mut data = SimpleRuntimeData::new();

        let input = lex("$ + 5\n\n@Case \"5 + 5 is 10\" 5").unwrap();
        let tests = extract_tests(&input).unwrap();

        // caller needs space to set up data as well, let them build top expression
        let parse_result = parse(tests.get_expression().clone()).unwrap();
        let top_expression = data.get_jump_table_len();
        build_with_data(parse_result.get_root(), parse_result.get_nodes().clone(), &mut data).unwrap();

        let mut runtime = SimpleGarnishRuntime::new(data);
        let results = execute_tests(&mut runtime, &tests, Some(top_expression)).unwrap();

        assert_eq!(results.get_results().len(), 1);

        let first = results.get_results().get(0).unwrap();
        assert!(first
            .error()
            .unwrap()
            .contains("Missing case annotation parameters. Expected a name, input and expected output."));
        assert!(!first.is_success());
        assert_eq!(first.value(), None);
        assert_eq!(first.test_details(), tests.get_annotations().get(0).unwrap());
    }

    #[test]
    fn case_with_only_one_parameters() {
        let mut data = SimpleRuntimeData::new();

        let input = lex("$ + 5\n\n@Case \"5 + 5 is 10\"").unwrap();
        let tests = extract_tests(&input).unwrap();

        // caller needs space to set up data as well, let them build top expression
        let parse_result = parse(tests.get_expression().clone()).unwrap();
        let top_expression = data.get_jump_table_len();
        build_with_data(parse_result.get_root(), parse_result.get_nodes().clone(), &mut data).unwrap();

        let mut runtime = SimpleGarnishRuntime::new(data);
        let results = execute_tests(&mut runtime, &tests, Some(top_expression)).unwrap();

        assert_eq!(results.get_results().len(), 1);

        let first = results.get_results().get(0).unwrap();
        assert!(first
            .error()
            .unwrap()
            .contains("Missing case annotation parameters. Expected a name, input and expected output."));
        assert!(!first.is_success());
        assert_eq!(first.value(), None);
        assert_eq!(first.test_details(), tests.get_annotations().get(0).unwrap());
    }

    #[test]
    fn case_with_no_parameters() {
        let mut data = SimpleRuntimeData::new();

        let input = lex("$ + 5\n\n@Case ").unwrap();
        let tests = extract_tests(&input).unwrap();

        // caller needs space to set up data as well, let them build top expression
        let parse_result = parse(tests.get_expression().clone()).unwrap();
        let top_expression = data.get_jump_table_len();
        build_with_data(parse_result.get_root(), parse_result.get_nodes().clone(), &mut data).unwrap();

        let mut runtime = SimpleGarnishRuntime::new(data);
        let results = execute_tests(&mut runtime, &tests, Some(top_expression)).unwrap();

        assert_eq!(results.get_results().len(), 1);

        let first = results.get_results().get(0).unwrap();
        assert!(first.error().unwrap().contains("Test expression empty"));
        assert!(!first.is_success());
        assert_eq!(first.value(), None);
        assert_eq!(first.test_details(), tests.get_annotations().get(0).unwrap());
    }
}

#[cfg(test)]
mod context {
    use garnish_data::data::SimpleNumber;
    use garnish_data::{symbol_value, DataError, SimpleRuntimeData};
    use garnish_lang_compiler::{build_with_data, lex, parse};
    use garnish_lang_runtime::runtime_impls::SimpleGarnishRuntime;
    use garnish_traits::{GarnishLangRuntimeContext, GarnishLangRuntimeData, GarnishRuntime, RuntimeError};

    use crate::test_annotation::{execute_tests_with_context, extract_tests};

    struct TestContext {}

    impl GarnishLangRuntimeContext<SimpleRuntimeData> for TestContext {
        fn resolve(&mut self, symbol: u64, runtime: &mut SimpleRuntimeData) -> Result<bool, RuntimeError<DataError>> {
            if symbol == symbol_value("value1") {
                runtime.add_number(SimpleNumber::Integer(100)).and_then(|r| runtime.push_register(r))?;
                return Ok(true);
            }

            Ok(false)
        }
    }

    #[test]
    fn caller_context_is_used_in_top_expression() {
        let mut data = SimpleRuntimeData::new();

        let input = lex("$ + value1\n\n@Case \"5 + value1 is 105\" 5 105").unwrap();
        let tests = extract_tests(&input).unwrap();

        // caller needs space to set up data as well, let them build top expression
        let parse_result = parse(tests.get_expression().clone()).unwrap();
        let top_expression = data.get_jump_table_len();
        build_with_data(parse_result.get_root(), parse_result.get_nodes().clone(), &mut data).unwrap();

        let mut runtime = SimpleGarnishRuntime::new(data);
        let results = execute_tests_with_context(&mut runtime, &tests, Some(top_expression), || Some(Box::new(TestContext {}))).unwrap();

        assert_eq!(results.get_results().len(), 1);

        let first = results.get_results().get(0).unwrap();
        assert_eq!(first.error(), None);
        assert!(first.is_success());
        assert_eq!(first.value(), Some(6));
        assert_eq!(runtime.get_data().get_number(6).unwrap(), SimpleNumber::Integer(105));
        assert_eq!(first.test_details(), tests.get_annotations().get(0).unwrap());
    }

    #[test]
    fn mocks_is_used_instead_of_caller_context() {
        let mut data = SimpleRuntimeData::new();

        let input = lex("$ + value1\n\n@Mock value1 200\n@Case \"5 + value1 is 105\" 5 205").unwrap();
        let tests = extract_tests(&input).unwrap();

        // caller needs space to set up data as well, let them build top expression
        let parse_result = parse(tests.get_expression().clone()).unwrap();
        let top_expression = data.get_jump_table_len();
        build_with_data(parse_result.get_root(), parse_result.get_nodes().clone(), &mut data).unwrap();

        let mut runtime = SimpleGarnishRuntime::new(data);
        let results = execute_tests_with_context(&mut runtime, &tests, Some(top_expression), || Some(Box::new(TestContext {}))).unwrap();

        assert_eq!(results.get_results().len(), 1);

        let first = results.get_results().get(0).unwrap();
        assert_eq!(first.error(), None);
        assert!(first.is_success());
        assert_eq!(first.value(), Some(7));
        assert_eq!(runtime.get_data().get_number(7).unwrap(), SimpleNumber::Integer(205));
        assert_eq!(first.test_details(), tests.get_annotations().get(0).unwrap());
    }

    #[test]
    fn mock_with_non_identifier_for_first_parameter() {
        let mut data = SimpleRuntimeData::new();

        let input = lex("$ + value1\n\n@Mock 10 20\n@Case \"5 + value1 is 105\" 5 205").unwrap();
        let tests = extract_tests(&input).unwrap();

        // caller needs space to set up data as well, let them build top expression
        let parse_result = parse(tests.get_expression().clone()).unwrap();
        let top_expression = data.get_jump_table_len();
        build_with_data(parse_result.get_root(), parse_result.get_nodes().clone(), &mut data).unwrap();

        let mut runtime = SimpleGarnishRuntime::new(data);
        let results = execute_tests_with_context(&mut runtime, &tests, Some(top_expression), || Some(Box::new(TestContext {}))).unwrap();

        assert_eq!(results.get_results().len(), 1);

        let first = results.get_results().get(0).unwrap();
        assert!(first.error().unwrap().contains("No identifier found in mock."));
        assert!(!first.is_success());
        assert_eq!(first.value(), None);
        assert_eq!(first.test_details(), tests.get_annotations().get(0).unwrap());
    }

    #[test]
    fn mock_with_only_one_parameter() {
        let mut data = SimpleRuntimeData::new();

        let input = lex("$ + value1\n\n@Mock value1\n@Case \"5 + value1 is 105\" 5 205").unwrap();
        let tests = extract_tests(&input).unwrap();

        // caller needs space to set up data as well, let them build top expression
        let parse_result = parse(tests.get_expression().clone()).unwrap();
        let top_expression = data.get_jump_table_len();
        build_with_data(parse_result.get_root(), parse_result.get_nodes().clone(), &mut data).unwrap();

        let mut runtime = SimpleGarnishRuntime::new(data);
        let results = execute_tests_with_context(&mut runtime, &tests, Some(top_expression), || Some(Box::new(TestContext {}))).unwrap();

        assert_eq!(results.get_results().len(), 1);

        let first = results.get_results().get(0).unwrap();
        assert!(first.error().unwrap().contains("No expression after identifier in mock"));
        assert!(!first.is_success());
        assert_eq!(first.value(), None);
        assert_eq!(first.test_details(), tests.get_annotations().get(0).unwrap());
    }

    #[test]
    fn mock_with_no_parameter() {
        let mut data = SimpleRuntimeData::new();

        let input = lex("$ + value1\n\n@Mock\n@Case \"5 + value1 is 105\" 5 205").unwrap();
        let tests = extract_tests(&input).unwrap();

        // caller needs space to set up data as well, let them build top expression
        let parse_result = parse(tests.get_expression().clone()).unwrap();
        let top_expression = data.get_jump_table_len();
        build_with_data(parse_result.get_root(), parse_result.get_nodes().clone(), &mut data).unwrap();

        let mut runtime = SimpleGarnishRuntime::new(data);
        let results = execute_tests_with_context(&mut runtime, &tests, Some(top_expression), || Some(Box::new(TestContext {}))).unwrap();

        assert_eq!(results.get_results().len(), 1);

        let first = results.get_results().get(0).unwrap();
        assert!(first.error().unwrap().contains("Mock expression empty"));
        assert!(!first.is_success());
        assert_eq!(first.value(), None);
        assert_eq!(first.test_details(), tests.get_annotations().get(0).unwrap());
    }
}