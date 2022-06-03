use std::fmt::Debug;

use garnish_lang_compiler::{build_with_data, parse};
use garnish_traits::{ExpressionDataType, GarnishLangRuntimeData, GarnishLangRuntimeState, GarnishNumber, GarnishRuntime, NO_CONTEXT, TypeConstants};

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

fn execute_test_annotation<Data: GarnishLangRuntimeData, Runtime: GarnishRuntime<Data>>(
    runtime: &mut Runtime,
    test: &TestAnnotationDetails,
) -> Result<TestResult<Data>, TestExtractionError> {
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
                    execute_until_end(runtime, start)?;

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
        },
    };

    Ok(TestResult::<Data>::new(success, error, value, name_value, test.clone()))
}

fn execute_until_end<Data: GarnishLangRuntimeData, Runtime: GarnishRuntime<Data>>(
    runtime: &mut Runtime,
    start: Data::Size,
) -> Result<(), TestExtractionError> {
    runtime
        .get_data_mut()
        .set_instruction_cursor(start)
        .or_else(|err| Err(TestExtractionError::error(format!("{:?}", err).as_str())))?;

    loop {
        match runtime.execute_current_instruction(NO_CONTEXT) {
            Err(e) => return Err(TestExtractionError::error(e.to_string().as_str()))?,
            Ok(data) => match data.get_state() {
                GarnishLangRuntimeState::Running => (),
                GarnishLangRuntimeState::End => break,
            },
        }
    }

    Ok(())
}

fn execute_case_annotation<Data: GarnishLangRuntimeData, Runtime: GarnishRuntime<Data>>(
    runtime: &mut Runtime,
    test: &TestAnnotationDetails,
    top_expression: Data::Size,
) -> Result<TestResult<Data>, TestExtractionError> {
    let (name_value, input_value, output_value) = match runtime.get_data().get_current_value() {
        None => (None, None, None),
        Some(value) => match runtime
            .get_data()
            .get_data_type(value)
            .or_else(|err| Err(TestExtractionError::error(format!("{:?}", err).as_str())))?
        {
            ExpressionDataType::List => {
                let name_value = runtime.get_data().get_list_item(value, Data::Number::zero()).map_or(None, |v| Some(v));
                let expression_value = runtime.get_data().get_list_item(value, Data::Number::one()).map_or(None, |v| Some(v));
                let output_value = runtime
                    .get_data()
                    .get_list_item(
                        value,
                        Data::Number::one().increment().map_or(
                            Err(TestExtractionError::error(
                                "Could not create the number 2 with Data's Size associated type.",
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

    let (success, value, error) = match input_value {
        None => (false, None, Some("No input value given for case".to_string())),
        Some(input_addr) => {
            // push input value to input stack
            runtime
                .get_data_mut()
                .push_value_stack(input_addr)
                .or_else(|err| Err(TestExtractionError::error(format!("{:?}", err).as_str())))?;

            // execute top expression
            execute_until_end(runtime, top_expression)?;

            // push current value and output value to registers
            // perform comparison
            match output_value {
                None => (false, None, Some("No output value given for case".to_string())),
                Some(output_addr) => match runtime.get_data().get_current_value() {
                    None => (false, None, Some("No current value available after top expression execution".to_string())),
                    Some(value_addr) => {
                        runtime
                            .get_data_mut()
                            .push_register(value_addr)
                            .or_else(|err| Err(TestExtractionError::error(format!("{:?}", err).as_str())))?;

                        runtime
                            .get_data_mut()
                            .push_register(output_addr)
                            .or_else(|err| Err(TestExtractionError::error(format!("{:?}", err).as_str())))?;

                        runtime
                            .equal()
                            .or_else(|err| Err(TestExtractionError::error(format!("{:?}", err).as_str())))?;

                        runtime.update_value()
                            .or_else(|err| Err(TestExtractionError::error(format!("{:?}", err).as_str())))?;

                        match runtime.get_data().get_current_value() {
                            None => (false, None, Some("No value available after equality comparison".to_string())),
                            Some(result_addr) => {
                                match runtime
                                    .get_data()
                                    .get_data_type(result_addr)
                                    .or_else(|err| Err(TestExtractionError::error(format!("{:?}", err).as_str())))?
                                {
                                    ExpressionDataType::True => (true, Some(value_addr), None),
                                    ExpressionDataType::False => (true, Some(value_addr), None),
                                    t => (false, Some(value_addr), Some(format!("Value after equality is {:?}, expected True or False", t))),
                                }
                            }
                        }
                    }
                },
            }
        }
    };

    Ok(TestResult::<Data>::new(success, error, value, name_value, test.clone()))
}

pub fn execute_tests<Data: GarnishLangRuntimeData, Runtime: GarnishRuntime<Data>>(
    runtime: &mut Runtime,
    tests: &TestDetails,
    top_expression: Option<Data::Size>,
) -> Result<ExecutionResult<Data>, TestExtractionError> {
    let mut results = vec![];

    for test in tests.get_annotations() {
        let parse_result = parse(test.get_expression().clone()).or_else(|err| Err(TestExtractionError::error(err.get_message())))?;

        let start = runtime.get_data().get_jump_table_len();

        build_with_data(parse_result.get_root(), parse_result.get_nodes().clone(), runtime.get_data_mut())
            .or_else(|err| Err(TestExtractionError::error(err.get_message())))?;

        match runtime.get_data().get_jump_point(start) {
            None => return Err(TestExtractionError::error("No starting jump point available")),
            Some(point) => {
                runtime
                    .get_data_mut()
                    .set_instruction_cursor(point)
                    .or_else(|err| Err(TestExtractionError::error(format!("{:?}", err).as_str())))?;
            }
        }

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

        let result = match test.get_annotation() {
            TestAnnotation::Test => execute_test_annotation(runtime, test)?,
            TestAnnotation::Case => match top_expression {
                None => TestResult::<Data>::new(
                    false,
                    Some("No top expression provided for Case annotation.".to_string()),
                    None,
                    None,
                    test.clone(),
                ),
                Some(top) => execute_case_annotation(runtime, test, top)?,
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
}
