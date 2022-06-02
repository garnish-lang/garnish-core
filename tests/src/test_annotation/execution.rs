use std::fmt::Debug;

use garnish_lang_compiler::{build_with_data, parse};
use garnish_traits::{ExpressionDataType, GarnishLangRuntimeData, GarnishLangRuntimeState, GarnishRuntime, TypeConstants, NO_CONTEXT};

use crate::test_annotation::{TestAnnotationDetails, TestDetails, TestExtractionError};

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
}

pub struct ExecutionResult<Data: GarnishLangRuntimeData> {
    test_results: Vec<TestResult<Data>>,
}

impl<Data: GarnishLangRuntimeData> ExecutionResult<Data> {
    fn new(test_results: Vec<TestResult<Data>>) -> Self {
        ExecutionResult { test_results }
    }

    fn get_results(&self) -> &Vec<TestResult<Data>> {
        &self.test_results
    }
}

pub fn execute_tests<Data: GarnishLangRuntimeData, Runtime: GarnishRuntime<Data>>(
    runtime: &mut Runtime,
    tests: &TestDetails,
) -> Result<ExecutionResult<Data>, TestExtractionError> {
    let mut results = vec![];

    for test in tests.get_annotations() {
        let parse_result = parse(test.get_expression().clone()).or_else(|err| Err(TestExtractionError::error(err.get_message())))?;

        let start = runtime.get_data().get_jump_table_len();

        build_with_data(parse_result.get_root(), parse_result.get_nodes().clone(), runtime.get_data_mut())
            .or_else(|err| Err(TestExtractionError::error(err.get_message())))?;

        runtime
            .get_data_mut()
            .set_instruction_cursor(start)
            .or_else(|err| Err(TestExtractionError::error(format!("{:?}", err).as_str())))?;

        // Run test expression to get list that should contain 2 items
        // a name string and an expression to execute
        loop {
            match runtime.execute_current_instruction(NO_CONTEXT) {
                Err(e) => return Err(TestExtractionError::error(e.to_string().as_str())),
                Ok(data) => match data.get_state() {
                    GarnishLangRuntimeState::Running => (),
                    GarnishLangRuntimeState::End => break,
                },
            }
        }

        let (name_value, expression_value) = match runtime.get_data().get_current_value() {
            None => (None, None),
            Some(value) => match runtime
                .get_data()
                .get_data_type(value)
                .or_else(|err| Err(TestExtractionError::error(format!("{:?}", err).as_str())))?
            {
                ExpressionDataType::List => {
                    let name_value = runtime.get_data().get_list_item(value, Data::Number::zero()).map_or(None, |v| Some(v));
                    let expression_value = runtime.get_data().get_list_item(value, Data::Number::one()).map_or(None, |v| Some(v));
                    (name_value, expression_value)
                }
                _ => (None, None),
            },
        };

        let (success, value, error) = match expression_value {
            None => (false, None, Some(format!("Could not retrieve expression"))),
            Some(expression_addr) => match runtime.get_data().get_expression(expression_addr) {
                Err(e) => (false, None, Some(format!("{:?}", e))),
                Ok(point) => match runtime.get_data().get_jump_point(point) {
                    None => (false, None, Some("Jump point not registered".to_string())),
                    Some(start) => {

                        runtime
                            .get_data_mut()
                            .set_instruction_cursor(start)
                            .or_else(|err| Err(TestExtractionError::error(format!("{:?}", err).as_str())))?;

                        // Run test expression to get list that should contain 2 items
                        // a name string and an expression to execute
                        loop {
                            match runtime.execute_current_instruction(NO_CONTEXT) {
                                Err(e) => return Err(TestExtractionError::error(e.to_string().as_str())),
                                Ok(data) => match data.get_state() {
                                    GarnishLangRuntimeState::Running => (),
                                    GarnishLangRuntimeState::End => break,
                                },
                            }
                        }

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
                }
            },
        };

        results.push(TestResult::<Data>::new(success, error, value, name_value, test.clone()));
    }

    Ok(ExecutionResult::new(results))
}

#[cfg(test)]
mod tests {
    use garnish_data::SimpleRuntimeData;
    use garnish_lang_compiler::{build_with_data, lex, parse};
    use garnish_lang_runtime::runtime_impls::SimpleGarnishRuntime;

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
        let results = execute_tests(&mut runtime, &tests).unwrap();

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
        let results = execute_tests(&mut runtime, &tests).unwrap();

        assert_eq!(results.get_results().len(), 1);

        let first = results.get_results().get(0).unwrap();
        assert_eq!(first.error(), None);
        assert!(!first.is_success());
        assert_eq!(first.value(), Some(1));
        assert_eq!(first.test_details(), tests.get_annotations().get(0).unwrap());
    }
}
