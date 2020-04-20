use std::convert::TryFrom;

use garnish_common::{
    is_end_exclusive, is_start_exclusive, skip_sizes, skip_type, skip_type_and_2_sizes,
    skip_type_and_size, DataType, Result,
};

use crate::runtime::CallFrame;
use crate::ExpressionRuntime;

#[derive(Clone)]
pub(crate) enum IterationType {
    ReferenceList,
    Range,
}

#[derive(Clone)]
pub(crate) struct IterationData {
    iteration_type: IterationType,
    single_result: bool,
    expression_start: usize,
    counter: usize,
    end: usize,
    elements_start: usize,
    last_value_ref: usize,
    result_ref_cursor: usize,
    call_stack_index: usize,
}

impl ExpressionRuntime {
    pub(crate) fn perform_iterate_to_single_result(&mut self) -> Result {
        self.perform_iterate_to_result(true)
    }

    pub(crate) fn perform_iterate(&mut self) -> Result {
        self.perform_iterate_to_result(false)
    }

    pub(crate) fn perform_reverse_iterate(&mut self) -> Result {
        Err("Unimplemented".into())
    }

    pub(crate) fn perform_reverse_iterate_to_single_result(&mut self) -> Result {
        Err("Unimplemented".into())
    }

    pub(crate) fn perform_multi_iterate(&mut self) -> Result {
        Err("Unimplemented".into())
    }

    pub(crate) fn perform_iteration_output(&mut self) -> Result {
        let last = self.iteration_stack.len() - 1;
        match self.iteration_stack.get_mut(last) {
            Some(data) => {
                data.result_ref_cursor += 1;
                Ok(())
            }
            // no-op if no current iteration
            None => Ok(()),
        }
    }

    pub(crate) fn perform_iteration_continue(&mut self) -> Result {
        self.perform_reiterate()
    }

    pub(crate) fn perform_iteration_skip(&mut self) -> Result {
        let last = self.iteration_stack.len() - 1;
        match self.iteration_stack.get_mut(last) {
            Some(data) => {
                data.result_ref_cursor -= 1;

                // drop all call frames above the iteration call frame
                let to_pop = self.call_stack.len() - data.call_stack_index;
                for _ in 0..to_pop {
                    self.call_stack.pop();
                }
            }
            // no-op if no current iteration
            None => (),
        }

        self.perform_reiterate()?;

        Ok(())
    }

    pub(crate) fn perform_iteration_complete(&mut self) -> Result {
        let last = self.iteration_stack.len() - 1;
        match self.iteration_stack.get_mut(last) {
            // no-op if no current iteration
            None => return Ok(()),
            Some(data) => {
                // drop all call frames above the iteration call frame
                let to_pop = self.call_stack.len() - data.call_stack_index;
                for _ in 0..to_pop {
                    self.call_stack.pop();
                }

                self.update_iteration_data()?
            }
        };

        self.end_iteration()?;

        Ok(())
    }

    pub fn perform_iterate_to_result(&mut self, single_result: bool) -> Result {
        let (left_ref, right_ref) = self.consume_last_two_refs()?;

        // get expression start and result from right ref
        let (expression_start, result_ref) = match DataType::try_from(self.data[right_ref])? {
            DataType::Expression => {
                let expression_index = self.read_size_at(skip_type(right_ref))?;
                let expression_start = self.get_expression_start(expression_index)?;

                (expression_start, 0)
            }
            DataType::Partial => {
                let expression_index_ref = self.read_size_at(skip_type(right_ref))?;
                let expression_index = self.read_size_at(skip_type(expression_index_ref))?;
                let expression_start = self.get_expression_start(expression_index)?;

                let result_ref = self.read_size_at(skip_type_and_size(right_ref))?;

                (expression_start, result_ref)
            }
            _ => return self.insert_unit(),
        };

        // left should be an iterable value.
        // i.e. string, range, list, associative list
        match DataType::try_from(self.data[left_ref])? {
            DataType::List => {
                self.start_list();

                let length = self.read_size_at(skip_type(left_ref))?;

                let first_element = skip_type_and_2_sizes(left_ref);
                let first_value_ref = self.read_size_at(first_element)?;

                let input_ref = self.push_iteration_input_list(first_value_ref, result_ref)?;

                self.iteration_stack.push(IterationData {
                    iteration_type: IterationType::ReferenceList,
                    single_result,
                    expression_start,
                    counter: 0,
                    end: length,
                    elements_start: first_element,
                    last_value_ref: input_ref,
                    result_ref_cursor: self.ref_cursor,
                    call_stack_index: self.call_stack.len(),
                });

                self.push_iteration_frame(expression_start);
            }
            DataType::CharacterList => {
                self.start_list();

                let length = self.read_size_at(skip_type(left_ref))?;

                let first_element = skip_type_and_size(left_ref);
                let first_value_ref = self.read_size_at(first_element)?;

                let input_ref = self.push_iteration_input_list(first_value_ref, result_ref)?;

                self.iteration_stack.push(IterationData {
                    iteration_type: IterationType::ReferenceList,
                    single_result,
                    expression_start,
                    counter: 0,
                    end: length,
                    elements_start: first_element,
                    last_value_ref: input_ref,
                    result_ref_cursor: self.ref_cursor,
                    call_stack_index: self.call_stack.len(),
                });

                self.push_iteration_frame(expression_start);
            }
            DataType::Expression => {
                let input_expression_index = self.read_size_at(skip_type(left_ref))?;
                let input_expression_start = self.get_expression_start(input_expression_index)?;

                // add incomplete data to iteration stack to be updated when input expression completes
                self.iteration_stack.push(IterationData {
                    iteration_type: IterationType::ReferenceList,
                    single_result,
                    expression_start,
                    counter: 0,
                    end: 0,                     // will be changed
                    elements_start: 0,          // will be changed
                    last_value_ref: result_ref, // will be changed
                    result_ref_cursor: 0,       // will be changed
                    call_stack_index: 0,
                });

                // push input expression frame
                // mark as iterating to call back here
                self.push_expression_iteration_frame(input_expression_start);

                // add unit input for input expression to use
                self.push_unit_input()?;
            }
            DataType::Range => {
                self.start_list();

                let (flags, start_ref_opt, _, _) = self.get_range_values_at(left_ref)?;
                let (start_ref, counter) = if is_start_exclusive(flags) {
                    // get first value, increment it push it to stack for use as input
                    match start_ref_opt {
                        Some(start_ref) => match DataType::try_from(self.data[start_ref])? {
                            DataType::Integer => {
                                let r = self.value_cursor;
                                let value = self.read_integer_at(skip_type(start_ref))? + 1;
                                self.insert_integer(value)?;
                                self.ref_cursor -= 1;
                                (r, 1)
                            }
                            DataType::Float => {
                                let r = self.value_cursor;
                                let value = self.read_float_at(skip_type(start_ref))? + 1.0;
                                self.insert_float(value)?;
                                self.ref_cursor -= 1;
                                (r, 1)
                            }
                            DataType::Character => match self.read_char_at(skip_type(start_ref))? {
                                Some(c) => match std::char::from_u32(c as u32 + 1) {
                                    Some(value) => {
                                        let r = self.value_cursor;
                                        self.insert_character(value)?;
                                        self.ref_cursor -= 1;
                                        (r, 1)
                                    }
                                    None => {
                                        return Err(format!("Invalid char for range value").into())
                                    }
                                },
                                None => return Err(format!("Invalid char for range value").into()),
                            },
                            x => return Err(format!("Unsupported type for range {}.", x).into()),
                        },
                        None => unimplemented!(),
                    }
                } else {
                    match start_ref_opt {
                        Some(start_ref) => (start_ref, 0),
                        None => unimplemented!(),
                    }
                };

                let input_ref = self.push_iteration_input_list(start_ref, result_ref)?;

                self.iteration_stack.push(IterationData {
                    iteration_type: IterationType::Range,
                    single_result,
                    expression_start,
                    counter,
                    end: 0,
                    elements_start: left_ref,
                    last_value_ref: input_ref,
                    result_ref_cursor: self.ref_cursor,
                    call_stack_index: self.call_stack.len(),
                });

                self.push_iteration_frame(expression_start);
            }
            _ => {
                self.insert_unit()?;
                return Ok(());
            }
        }

        Ok(())
    }

    pub(crate) fn perform_reiterate(&mut self) -> Result {
        if self.iteration_stack.len() == 0 {
            return Err("Reiterate called with no active iteration.".into());
        }

        let data = self.update_iteration_data()?;

        let end = match data.iteration_type {
            IterationType::ReferenceList => {
                if data.counter == data.end {
                    true
                } else {
                    // set next input
                    let current_ref =
                        self.read_size_at(skip_sizes(data.elements_start, data.counter))?;

                    self.push_iteration_input_list_with_last_result(current_ref)?;

                    self.push_iteration_frame(data.expression_start);

                    false
                }
            }
            IterationType::Range => {
                let (flags, opt_start, opt_end, opt_step) =
                    self.get_range_values_at(data.elements_start)?;

                match (opt_start, opt_end, opt_step) {
                    (Some(start), Some(end), Some(step)) => {
                        match DataType::try_from(self.data[start])? {
                            DataType::Integer => {
                                let increment_value = match DataType::try_from(self.data[start])? {
                                    DataType::Integer => self.read_integer_at(skip_type(step))?,
                                    x => {
                                        return Err(format!(
                                            "Unsupported step value type {} for integer range.",
                                            x
                                        )
                                        .into())
                                    }
                                };

                                self.reiterate_range(
                                    ExpressionRuntime::read_integer_at,
                                    ExpressionRuntime::insert_integer,
                                    |x| x + increment_value,
                                    flags,
                                    start,
                                    end,
                                    &data,
                                )?
                            }
                            DataType::Float => {
                                let increment_value = match DataType::try_from(self.data[start])? {
                                    DataType::Float => self.read_float_at(skip_type(step))?,
                                    x => {
                                        return Err(format!(
                                            "Unsupported step value type {} for integer range.",
                                            x
                                        )
                                        .into())
                                    }
                                };

                                self.reiterate_range(
                                    ExpressionRuntime::read_float_at,
                                    ExpressionRuntime::insert_float,
                                    |x| x + increment_value,
                                    flags,
                                    start,
                                    end,
                                    &data,
                                )?
                            }
                            DataType::Character => {
                                let step_value = self.read_integer_at(skip_type(step))?;

                                self.reiterate_character_range(flags, end, &data, |c| {
                                    std::char::from_u32(c as u32 + step_value as u32)
                                })?
                            }
                            x => return Err(format!("Unsupported type {} for range.", x).into()),
                        }
                    }
                    (Some(start), Some(end), None) => match DataType::try_from(self.data[start])? {
                        DataType::Integer => self.reiterate_range(
                            ExpressionRuntime::read_integer_at,
                            ExpressionRuntime::insert_integer,
                            |x| x + 1,
                            flags,
                            start,
                            end,
                            &data,
                        )?,
                        DataType::Float => self.reiterate_range(
                            ExpressionRuntime::read_float_at,
                            ExpressionRuntime::insert_float,
                            |x| x + 1.0,
                            flags,
                            start,
                            end,
                            &data,
                        )?,
                        DataType::Character => {
                            self.reiterate_character_range(flags, end, &data, |c| {
                                std::char::from_u32(c as u32 + 1)
                            })?
                        }
                        x => return Err(format!("Unsupported type {} for range.", x).into()),
                    },
                    _ => unimplemented!(),
                }
            }
        };

        if end {
            self.end_iteration()?;
        }

        Ok(())
    }

    pub(crate) fn iteration_expression_end(&mut self, frame: CallFrame) -> Result {
        // insert list from expression results
        self.start_list();

        let result_range = frame.result_start..self.result_stack.len();
        let results: Vec<usize> = self.result_stack[result_range].iter().map(|r| *r).collect();

        for result in results {
            self.insert_reference_value(result)?;
        }

        let list_ref = self.value_cursor;
        self.make_list()?;
        // remove this list from registers
        self.ref_cursor -= 1;

        let list_length = self.read_size_at(skip_type(list_ref))?;
        let first_element = skip_type_and_2_sizes(list_ref);
        let first_value_ref = self.read_size_at(first_element)?;

        // last iteration stack should always line up with calls to this method
        let last = self.iteration_stack.len() - 1;

        // result ref is stored during initial iteration
        let last_value_ref = match self.iteration_stack.get(last) {
            None => return Err(format!("Iteration stack is empty.").into()),
            Some(data) => data.last_value_ref,
        };

        let input_ref = self.push_iteration_input_list(first_value_ref, last_value_ref)?;

        let expression_start = match self.iteration_stack.get_mut(last) {
            None => return Err(format!("Iteration stack is empty.").into()),
            Some(data) => {
                data.elements_start = first_element;
                data.last_value_ref = input_ref;
                data.end = list_length;
                data.result_ref_cursor = self.ref_cursor;
                data.call_stack_index = self.call_stack.len();

                data.expression_start
            }
        };

        self.push_iteration_frame(expression_start);

        self.start_list();

        Ok(())
    }

    fn update_iteration_data(&mut self) -> Result<IterationData> {
        let last = self.iteration_stack.len() - 1;
        match self.iteration_stack.get_mut(last) {
            None => {
                // TODO: Needs test coverage
                return Err("Reiterate called with no active iteration.".into());
            }
            Some(data) => {
                self.call_stack.pop();
                self.input_stack.pop();

                data.counter += 1;

                // push back ref cursor and ref in that register position
                // because only last value is implicitly used
                let iteration_value_ref = self.registers[self.ref_cursor - 1];
                self.registers[data.result_ref_cursor] = iteration_value_ref;
                self.ref_cursor = data.result_ref_cursor + 1;

                data.result_ref_cursor = self.ref_cursor;

                // clone so we may re-borrow 'self' later safely
                Ok(self.iteration_stack.get(last).unwrap().clone())
            }
        }
    }

    fn end_iteration(&mut self) -> Result {
        match self.iteration_stack.pop() {
            None => return Err("Tried to end iteration with no iteration data available.".into()),
            Some(data) => {
                if data.single_result {
                    // get most recent result from ref
                    let result_ref_index = self.ref_cursor - 1;
                    let result_ref = self.registers[result_ref_index];

                    // reset ref cursor to which is the most recent entry in list stack
                    self.ref_cursor = match self.list_stack.pop() {
                        Some(r) => r,
                        None => return Err("No lists being made at end of iteration.".into()),
                    };

                    self.insert_reference_value(result_ref)?;
                } else {
                    // make list containing each result
                    self.make_list()?;
                }
            }
        }

        Ok(())
    }

    fn reiterate_character_range<AFunc>(
        &mut self,
        flags: u8,
        end: usize,
        data: &IterationData,
        add_func: AFunc,
    ) -> Result<bool>
    where
        AFunc: Fn(char) -> Option<char>,
    {
        let last_value_pair_ref = self.read_size_at(skip_type_and_2_sizes(data.last_value_ref))?;
        let last_value_ref = self.read_size_at(skip_type_and_size(last_value_pair_ref))?;
        let last_value_opt = self.read_char_at(skip_type(last_value_ref))?;
        let end_value_opt = self.read_char_at(skip_type(end))?;

        let (_last_value, end_value, next_value) = match (last_value_opt, end_value_opt) {
            (Some(last_value), Some(end_value)) => {
                match add_func(last_value) {
                    Some(next_value) => (last_value, end_value, next_value),
                    // exited char bounds
                    // end iteration
                    None => return Ok(true),
                }
            }
            // exited char bounds
            // end iteration
            _ => return Ok(true),
        };

        self.insert_next_range_value(
            flags,
            next_value,
            end_value,
            ExpressionRuntime::insert_character,
            data,
        )
    }

    fn reiterate_range<T, AFunc>(
        &mut self,
        read_func: fn(&ExpressionRuntime, usize) -> Result<T>,
        insert_func: fn(&mut ExpressionRuntime, T) -> Result,
        add_func: AFunc,
        range_flags: u8,
        _start: usize,
        end: usize,
        data: &IterationData,
    ) -> Result<bool>
    where
        AFunc: FnOnce(T) -> T,
        T: PartialOrd<T>,
    {
        let end_value = read_func(self, skip_type(end))?;
        // current is set as the first value in input list
        // so skip type 2 sizes for length to get ref
        let last_value_pair_ref = self.read_size_at(skip_type_and_2_sizes(data.last_value_ref))?;
        let last_value_ref = self.read_size_at(skip_type_and_size(last_value_pair_ref))?;
        // get value by reading from ref
        let last_value = read_func(self, skip_type(last_value_ref))?;

        let next_value = add_func(last_value);

        self.insert_next_range_value(range_flags, next_value, end_value, insert_func, data)
    }

    fn insert_next_range_value<T>(
        &mut self,
        range_flags: u8,
        next_value: T,
        end_value: T,
        insert_func: fn(&mut ExpressionRuntime, T) -> Result,
        data: &IterationData,
    ) -> Result<bool>
    where
        T: PartialOrd<T>,
    {
        let end = if is_end_exclusive(range_flags) {
            next_value >= end_value
        } else {
            next_value > end_value
        };

        if end {
            Ok(true)
        } else {
            let next_ref = self.value_cursor;
            insert_func(self, next_value)?;
            self.ref_cursor -= 1; // TODO: make utility for inserting data without modifying registers

            let input_ref = self.push_iteration_input_list_with_last_result(next_ref)?;

            self.push_iteration_frame(data.expression_start);

            // update last ref on current iteration data
            // given data is a clone
            // so need to modify through self
            let last = self.iteration_stack.len() - 1;
            self.iteration_stack.get_mut(last).and_then(|data| {
                data.last_value_ref = input_ref;
                Some(())
            });

            Ok(false)
        }
    }

    fn push_iteration_input_list_with_last_result(&mut self, current_ref: usize) -> Result<usize> {
        // result from previous iteration will be in most recent register
        // which is 1 behind ref cursor
        let result_ref_index = self.ref_cursor - 1;
        let result_ref = self.registers[result_ref_index];

        self.push_iteration_input_list(current_ref, result_ref)
    }

    fn push_iteration_input_list(
        &mut self,
        current_ref: usize,
        result_ref: usize,
    ) -> Result<usize> {
        self.start_list();

        self.insert_symbol_value(self.current_value)?;
        self.insert_reference_value(current_ref)?;
        self.make_pair()?;

        self.insert_symbol_value(self.result_value)?;
        self.insert_reference_value(result_ref)?;
        self.make_pair()?;

        let input_ref = self.value_cursor;
        self.make_list()?;

        // remove this input from registers
        self.ref_cursor -= 1;

        self.input_stack.push(input_ref);

        Ok(input_ref)
    }
}

#[cfg(test)]
mod tests {
    use garnish_common::ExpressionValue;
    use garnish_instruction_set_builder::InstructionSetBuilder;

    use crate::runtime::ExpressionRuntime;

    #[test]
    fn iterates_list() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.put(ExpressionValue::integer(5)).unwrap();
        instructions.perform_addition();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.make_list();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 15);
        assert_eq!(result.get_list_item(1).unwrap().as_integer().unwrap(), 25);
        assert_eq!(result.get_list_item(2).unwrap().as_integer().unwrap(), 35);
    }

    #[test]
    fn iterates_character_list() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.put(ExpressionValue::integer(0)).unwrap();
        instructions.perform_type_cast();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::character_list("bears".into()))
            .unwrap();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 98);
        assert_eq!(result.get_list_item(1).unwrap().as_integer().unwrap(), 101);
        assert_eq!(result.get_list_item(2).unwrap().as_integer().unwrap(), 97);
        assert_eq!(result.get_list_item(3).unwrap().as_integer().unwrap(), 114);
        assert_eq!(result.get_list_item(4).unwrap().as_integer().unwrap(), 115);
    }

    #[test]
    fn iterates_integer_range() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.put(ExpressionValue::integer(5)).unwrap();
        instructions.perform_addition();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::integer_range(Some(0), Some(5)))
            .unwrap();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.list_len().unwrap(), 6);
        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 5);
        assert_eq!(result.get_list_item(1).unwrap().as_integer().unwrap(), 6);
        assert_eq!(result.get_list_item(2).unwrap().as_integer().unwrap(), 7);
        assert_eq!(result.get_list_item(3).unwrap().as_integer().unwrap(), 8);
        assert_eq!(result.get_list_item(4).unwrap().as_integer().unwrap(), 9);
        assert_eq!(result.get_list_item(5).unwrap().as_integer().unwrap(), 10);
    }

    #[test]
    fn iterates_float_range() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.put(ExpressionValue::integer(5)).unwrap();
        instructions.perform_addition();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::float_range(Some(0.0), Some(5.0)))
            .unwrap();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.list_len().unwrap(), 6);
        assert_eq!(result.get_list_item(0).unwrap().as_float().unwrap(), 5.0);
        assert_eq!(result.get_list_item(1).unwrap().as_float().unwrap(), 6.0);
        assert_eq!(result.get_list_item(2).unwrap().as_float().unwrap(), 7.0);
        assert_eq!(result.get_list_item(3).unwrap().as_float().unwrap(), 8.0);
        assert_eq!(result.get_list_item(4).unwrap().as_float().unwrap(), 9.0);
        assert_eq!(result.get_list_item(5).unwrap().as_float().unwrap(), 10.0);
    }

    #[test]
    fn iterates_character_range() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.put(ExpressionValue::integer(0)).unwrap();
        instructions.perform_type_cast();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::char_range(Some('a'), Some('f')))
            .unwrap();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.list_len().unwrap(), 6);
        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 97);
        assert_eq!(result.get_list_item(1).unwrap().as_integer().unwrap(), 98);
        assert_eq!(result.get_list_item(2).unwrap().as_integer().unwrap(), 99);
        assert_eq!(result.get_list_item(3).unwrap().as_integer().unwrap(), 100);
        assert_eq!(result.get_list_item(4).unwrap().as_integer().unwrap(), 101);
        assert_eq!(result.get_list_item(5).unwrap().as_integer().unwrap(), 102);
    }

    #[test]
    fn iterates_end_exclusive_integer_range() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.put(ExpressionValue::integer(5)).unwrap();
        instructions.perform_addition();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::integer_range(Some(0), Some(5)).exclude_end())
            .unwrap();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.list_len().unwrap(), 5);
        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 5);
        assert_eq!(result.get_list_item(1).unwrap().as_integer().unwrap(), 6);
        assert_eq!(result.get_list_item(2).unwrap().as_integer().unwrap(), 7);
        assert_eq!(result.get_list_item(3).unwrap().as_integer().unwrap(), 8);
        assert_eq!(result.get_list_item(4).unwrap().as_integer().unwrap(), 9);
    }

    #[test]
    fn iterates_start_exclusive_integer_range() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.put(ExpressionValue::integer(5)).unwrap();
        instructions.perform_addition();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::integer_range(Some(0), Some(5)).exclude_start())
            .unwrap();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.list_len().unwrap(), 5);
        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 6);
        assert_eq!(result.get_list_item(1).unwrap().as_integer().unwrap(), 7);
        assert_eq!(result.get_list_item(2).unwrap().as_integer().unwrap(), 8);
        assert_eq!(result.get_list_item(3).unwrap().as_integer().unwrap(), 9);
        assert_eq!(result.get_list_item(4).unwrap().as_integer().unwrap(), 10);
    }

    #[test]
    fn iterates_exclusive_integer_range() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.put(ExpressionValue::integer(5)).unwrap();
        instructions.perform_addition();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions
            .put(
                ExpressionValue::integer_range(Some(0), Some(5))
                    .exclude_start()
                    .exclude_end(),
            )
            .unwrap();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.list_len().unwrap(), 4);
        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 6);
        assert_eq!(result.get_list_item(1).unwrap().as_integer().unwrap(), 7);
        assert_eq!(result.get_list_item(2).unwrap().as_integer().unwrap(), 8);
        assert_eq!(result.get_list_item(3).unwrap().as_integer().unwrap(), 9);
    }

    #[test]
    fn iterates_end_exclusive_float_range() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.put(ExpressionValue::integer(5)).unwrap();
        instructions.perform_addition();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::float_range(Some(0.0), Some(5.0)).exclude_end())
            .unwrap();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.list_len().unwrap(), 5);
        assert_eq!(result.get_list_item(0).unwrap().as_float().unwrap(), 5.0);
        assert_eq!(result.get_list_item(1).unwrap().as_float().unwrap(), 6.0);
        assert_eq!(result.get_list_item(2).unwrap().as_float().unwrap(), 7.0);
        assert_eq!(result.get_list_item(3).unwrap().as_float().unwrap(), 8.0);
        assert_eq!(result.get_list_item(4).unwrap().as_float().unwrap(), 9.0);
    }

    #[test]
    fn iterates_start_exclusive_float_range() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.put(ExpressionValue::integer(5)).unwrap();
        instructions.perform_addition();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::float_range(Some(0.0), Some(5.0)).exclude_start())
            .unwrap();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.list_len().unwrap(), 5);
        assert_eq!(result.get_list_item(0).unwrap().as_float().unwrap(), 6.0);
        assert_eq!(result.get_list_item(1).unwrap().as_float().unwrap(), 7.0);
        assert_eq!(result.get_list_item(2).unwrap().as_float().unwrap(), 8.0);
        assert_eq!(result.get_list_item(3).unwrap().as_float().unwrap(), 9.0);
        assert_eq!(result.get_list_item(4).unwrap().as_float().unwrap(), 10.0);
    }

    #[test]
    fn iterates_exclusive_float_range() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.put(ExpressionValue::integer(5)).unwrap();
        instructions.perform_addition();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions
            .put(
                ExpressionValue::float_range(Some(0.0), Some(5.0))
                    .exclude_start()
                    .exclude_end(),
            )
            .unwrap();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.list_len().unwrap(), 4);
        assert_eq!(result.get_list_item(0).unwrap().as_float().unwrap(), 6.0);
        assert_eq!(result.get_list_item(1).unwrap().as_float().unwrap(), 7.0);
        assert_eq!(result.get_list_item(2).unwrap().as_float().unwrap(), 8.0);
        assert_eq!(result.get_list_item(3).unwrap().as_float().unwrap(), 9.0);
    }

    #[test]
    fn iterates_end_exclusive_char_range() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.put(ExpressionValue::integer(0)).unwrap();
        instructions.perform_type_cast();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::char_range(Some('a'), Some('f')).exclude_end())
            .unwrap();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.list_len().unwrap(), 5);
        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 97);
        assert_eq!(result.get_list_item(1).unwrap().as_integer().unwrap(), 98);
        assert_eq!(result.get_list_item(2).unwrap().as_integer().unwrap(), 99);
        assert_eq!(result.get_list_item(3).unwrap().as_integer().unwrap(), 100);
        assert_eq!(result.get_list_item(4).unwrap().as_integer().unwrap(), 101);
    }

    #[test]
    fn iterates_start_exclusive_char_range() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.put(ExpressionValue::integer(0)).unwrap();
        instructions.perform_type_cast();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::char_range(Some('a'), Some('f')).exclude_start())
            .unwrap();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.list_len().unwrap(), 5);
        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 98);
        assert_eq!(result.get_list_item(1).unwrap().as_integer().unwrap(), 99);
        assert_eq!(result.get_list_item(2).unwrap().as_integer().unwrap(), 100);
        assert_eq!(result.get_list_item(3).unwrap().as_integer().unwrap(), 101);
        assert_eq!(result.get_list_item(4).unwrap().as_integer().unwrap(), 102);
    }

    #[test]
    fn iterates_exclusive_char_range() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.put(ExpressionValue::integer(0)).unwrap();
        instructions.perform_type_cast();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions
            .put(
                ExpressionValue::char_range(Some('a'), Some('f'))
                    .exclude_start()
                    .exclude_end(),
            )
            .unwrap();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.list_len().unwrap(), 4);
        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 98);
        assert_eq!(result.get_list_item(1).unwrap().as_integer().unwrap(), 99);
        assert_eq!(result.get_list_item(2).unwrap().as_integer().unwrap(), 100);
        assert_eq!(result.get_list_item(3).unwrap().as_integer().unwrap(), 101);
    }

    #[test]
    fn iterates_integer_range_with_step() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.put(ExpressionValue::integer(5)).unwrap();
        instructions.perform_addition();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions
            .put(
                ExpressionValue::integer_range(Some(0), Some(5))
                    .with_step(ExpressionValue::integer(2)),
            )
            .unwrap();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.list_len().unwrap(), 3);
        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 5);
        assert_eq!(result.get_list_item(1).unwrap().as_integer().unwrap(), 7);
        assert_eq!(result.get_list_item(2).unwrap().as_integer().unwrap(), 9);
    }

    #[test]
    fn iterates_start_exclusive_integer_range_with_step() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.put(ExpressionValue::integer(5)).unwrap();
        instructions.perform_addition();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions
            .put(
                ExpressionValue::integer_range(Some(0), Some(5))
                    .exclude_start()
                    .with_step(ExpressionValue::integer(2)),
            )
            .unwrap();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.list_len().unwrap(), 3);
        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 6);
        assert_eq!(result.get_list_item(1).unwrap().as_integer().unwrap(), 8);
        assert_eq!(result.get_list_item(2).unwrap().as_integer().unwrap(), 10);
    }

    #[test]
    fn iterates_float_range_with_step() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.put(ExpressionValue::float(5.0)).unwrap();
        instructions.perform_addition();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions
            .put(
                ExpressionValue::float_range(Some(0.0), Some(5.0))
                    .with_step(ExpressionValue::float(2.0)),
            )
            .unwrap();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.list_len().unwrap(), 3);
        assert_eq!(result.get_list_item(0).unwrap().as_float().unwrap(), 5.0);
        assert_eq!(result.get_list_item(1).unwrap().as_float().unwrap(), 7.0);
        assert_eq!(result.get_list_item(2).unwrap().as_float().unwrap(), 9.0);
    }

    #[test]
    fn iterates_start_exclusive_float_range_with_step() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.put(ExpressionValue::float(5.0)).unwrap();
        instructions.perform_addition();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions
            .put(
                ExpressionValue::float_range(Some(0.0), Some(5.0))
                    .exclude_start()
                    .with_step(ExpressionValue::float(2.0)),
            )
            .unwrap();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.list_len().unwrap(), 3);
        assert_eq!(result.get_list_item(0).unwrap().as_float().unwrap(), 6.0);
        assert_eq!(result.get_list_item(1).unwrap().as_float().unwrap(), 8.0);
        assert_eq!(result.get_list_item(2).unwrap().as_float().unwrap(), 10.0);
    }

    #[test]
    fn iterates_char_range_with_step() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.put(ExpressionValue::integer(0)).unwrap();
        instructions.perform_type_cast();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::char_range(Some('a'), Some('f')))
            .unwrap();
        // ExpressionValue doesn't support integer steps on char range yet
        // perform with instructions
        instructions.put(ExpressionValue::integer(2)).unwrap();
        instructions.apply();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.list_len().unwrap(), 3);
        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 97);
        assert_eq!(result.get_list_item(1).unwrap().as_integer().unwrap(), 99);
        assert_eq!(result.get_list_item(2).unwrap().as_integer().unwrap(), 101);
    }

    #[test]
    fn iterates_start_exclusive_char_range_with_step() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.put(ExpressionValue::integer(0)).unwrap();
        instructions.perform_type_cast();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::char_range(Some('a'), Some('f')).exclude_start())
            .unwrap();
        // ExpressionValue doesn't support integer steps on char range yet
        // perform with instructions
        instructions.put(ExpressionValue::integer(2)).unwrap();
        instructions.apply();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.list_len().unwrap(), 3);
        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 98);
        assert_eq!(result.get_list_item(1).unwrap().as_integer().unwrap(), 100);
        assert_eq!(result.get_list_item(2).unwrap().as_integer().unwrap(), 102);
    }

    #[test]
    fn iterates_expression() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.put(ExpressionValue::integer(3)).unwrap();
        instructions.perform_addition();

        instructions.end_expression();

        instructions.start_expression("input_expr");

        instructions.put(ExpressionValue::integer(5)).unwrap();
        instructions.output_result();

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.output_result();

        instructions.put(ExpressionValue::integer(15)).unwrap();
        instructions.output_result();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::expression("input_expr"))
            .unwrap();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.list_len().unwrap(), 3);
        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 8);
        assert_eq!(result.get_list_item(1).unwrap().as_integer().unwrap(), 13);
        assert_eq!(result.get_list_item(2).unwrap().as_integer().unwrap(), 18);
    }

    #[test]
    fn iterates_list_using_result() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("put_result");

        instructions.put_input();
        instructions.put(ExpressionValue::symbol("result")).unwrap();
        instructions.perform_access();

        instructions.end_expression();

        instructions.start_expression("put_zero");
        instructions.put(ExpressionValue::integer(0)).unwrap();
        instructions.end_expression();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions.put(ExpressionValue::symbol("result")).unwrap();
        instructions.perform_access();

        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.perform_type_comparison();

        instructions.conditional_execute(Some("put_zero".into()), Some("put_result".into()));

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.perform_addition();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.make_list();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 10);
        assert_eq!(result.get_list_item(1).unwrap().as_integer().unwrap(), 30);
        assert_eq!(result.get_list_item(2).unwrap().as_integer().unwrap(), 60);
    }

    #[test]
    fn iterates_list_using_result_and_initial_result() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions.put(ExpressionValue::symbol("result")).unwrap();
        instructions.perform_access();

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.perform_addition();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.make_list();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();
        instructions.put(ExpressionValue::integer(0)).unwrap();
        instructions.partially_apply();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 10);
        assert_eq!(result.get_list_item(1).unwrap().as_integer().unwrap(), 30);
        assert_eq!(result.get_list_item(2).unwrap().as_integer().unwrap(), 60);
    }

    #[test]
    fn iterates_list_to_single_result() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions.put(ExpressionValue::symbol("result")).unwrap();
        instructions.perform_access();

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.perform_addition();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.make_list();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();
        instructions.put(ExpressionValue::integer(0)).unwrap();
        instructions.partially_apply();

        instructions.iterate_to_single_value();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert_eq!(result.as_integer().unwrap(), 60);
    }

    #[test]
    fn iterates_integer_range_using_result() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("put_result");

        instructions.put_input();
        instructions.put(ExpressionValue::symbol("result")).unwrap();
        instructions.perform_access();

        instructions.end_expression();

        instructions.start_expression("put_zero");
        instructions.put(ExpressionValue::integer(0)).unwrap();
        instructions.end_expression();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions.put(ExpressionValue::symbol("result")).unwrap();
        instructions.perform_access();

        instructions.put(ExpressionValue::unit()).unwrap();
        instructions.perform_type_comparison();

        instructions.conditional_execute(Some("put_zero".into()), Some("put_result".into()));

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.perform_addition();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::integer_range(Some(10), Some(15)))
            .unwrap();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 10); // 0 + 10 = 10
        assert_eq!(result.get_list_item(1).unwrap().as_integer().unwrap(), 21); // 10 + 11 = 21
        assert_eq!(result.get_list_item(2).unwrap().as_integer().unwrap(), 33); // 21 + 12 = 33
        assert_eq!(result.get_list_item(3).unwrap().as_integer().unwrap(), 46); // 33 + 13 = 46
        assert_eq!(result.get_list_item(4).unwrap().as_integer().unwrap(), 60); // 46 + 14 = 60
        assert_eq!(result.get_list_item(5).unwrap().as_integer().unwrap(), 75); // 60 + 15 = 75
    }

    #[test]
    fn iterates_integer_range_using_result_and_initial_result() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions.put(ExpressionValue::symbol("result")).unwrap();
        instructions.perform_access();

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.perform_addition();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::integer_range(Some(10), Some(15)))
            .unwrap();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();
        instructions.put(ExpressionValue::integer(0)).unwrap();
        instructions.partially_apply();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 10); // 0 + 10 = 10
        assert_eq!(result.get_list_item(1).unwrap().as_integer().unwrap(), 21); // 10 + 11 = 21
        assert_eq!(result.get_list_item(2).unwrap().as_integer().unwrap(), 33); // 21 + 12 = 33
        assert_eq!(result.get_list_item(3).unwrap().as_integer().unwrap(), 46); // 33 + 13 = 46
        assert_eq!(result.get_list_item(4).unwrap().as_integer().unwrap(), 60); // 46 + 14 = 60
        assert_eq!(result.get_list_item(5).unwrap().as_integer().unwrap(), 75); // 60 + 15 = 75
    }

    #[test]
    fn iterates_expression_with_initial_result() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions.put(ExpressionValue::symbol("result")).unwrap();
        instructions.perform_access();

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.perform_addition();

        instructions.end_expression();

        instructions.start_expression("input_expr");

        instructions.put(ExpressionValue::integer(5)).unwrap();
        instructions.output_result();

        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.output_result();

        instructions.put(ExpressionValue::integer(15)).unwrap();
        instructions.output_result();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions
            .put(ExpressionValue::expression("input_expr"))
            .unwrap();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();
        instructions.put(ExpressionValue::integer(0)).unwrap();
        instructions.partially_apply();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.list_len().unwrap(), 3);
        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 5);
        assert_eq!(result.get_list_item(1).unwrap().as_integer().unwrap(), 15);
        assert_eq!(result.get_list_item(2).unwrap().as_integer().unwrap(), 30);
    }

    #[test]
    fn iterates_list_ignores_implicit_result_outputs() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put(ExpressionValue::integer(5)).unwrap();

        instructions.put(ExpressionValue::integer(10)).unwrap();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.make_list();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.list_len().unwrap(), 3);
        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 10);
        assert_eq!(result.get_list_item(1).unwrap().as_integer().unwrap(), 10);
        assert_eq!(result.get_list_item(2).unwrap().as_integer().unwrap(), 10);
    }

    #[test]
    fn iterates_list_uses_explicit_result_outputs() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put(ExpressionValue::integer(5)).unwrap();
        instructions.iteration_output();

        instructions.put(ExpressionValue::integer(10)).unwrap();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.make_list();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.list_len().unwrap(), 6);
        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 5);
        assert_eq!(result.get_list_item(1).unwrap().as_integer().unwrap(), 10);
        assert_eq!(result.get_list_item(2).unwrap().as_integer().unwrap(), 5);
        assert_eq!(result.get_list_item(3).unwrap().as_integer().unwrap(), 10);
        assert_eq!(result.get_list_item(4).unwrap().as_integer().unwrap(), 5);
        assert_eq!(result.get_list_item(5).unwrap().as_integer().unwrap(), 10);
    }

    #[test]
    fn iterates_list_continue() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put(ExpressionValue::integer(5)).unwrap();
        instructions.iteration_continue();

        instructions.put(ExpressionValue::integer(10)).unwrap();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.make_list();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.list_len().unwrap(), 3);
        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 5);
        assert_eq!(result.get_list_item(1).unwrap().as_integer().unwrap(), 5);
        assert_eq!(result.get_list_item(2).unwrap().as_integer().unwrap(), 5);
    }

    #[test]
    fn iterates_list_with_skip() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("skip");
        instructions.iteration_skip();
        instructions.end_expression();

        instructions.start_expression("iteration_expr");

        instructions.put(ExpressionValue::integer(20)).unwrap();

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.perform_equality_comparison();

        instructions.conditional_execute(Some("skip".into()), None);

        instructions.put(ExpressionValue::integer(5)).unwrap();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.make_list();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.list_len().unwrap(), 2);
        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 5);
        assert_eq!(result.get_list_item(1).unwrap().as_integer().unwrap(), 5);
    }

    #[test]
    fn iterates_list_with_complete() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("complete");
        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();
        instructions.iteration_complete();
        instructions.end_expression();

        instructions.start_expression("iteration_expr");

        instructions.put(ExpressionValue::integer(20)).unwrap();

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.perform_equality_comparison();

        instructions.conditional_execute(Some("complete".into()), None);

        instructions.put(ExpressionValue::integer(5)).unwrap();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions.start_list();
        instructions.put(ExpressionValue::integer(10)).unwrap();
        instructions.put(ExpressionValue::integer(20)).unwrap();
        instructions.put(ExpressionValue::integer(30)).unwrap();
        instructions.make_list();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_list());
        assert_eq!(result.list_len().unwrap(), 2);
        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 5);
        assert_eq!(result.get_list_item(1).unwrap().as_integer().unwrap(), 20);
    }

    #[test]
    fn iterates_number() {
        let mut instructions = InstructionSetBuilder::new();

        instructions.start_expression("iteration_expr");

        instructions.put_input();
        instructions
            .put(ExpressionValue::symbol("current"))
            .unwrap();
        instructions.perform_access();

        instructions.put(ExpressionValue::integer(5)).unwrap();
        instructions.perform_addition();

        instructions.end_expression();

        instructions.start_expression("main");

        instructions.put(ExpressionValue::integer(10)).unwrap();

        instructions
            .put(ExpressionValue::expression("iteration_expr"))
            .unwrap();

        instructions.iterate();

        instructions.end_expression();

        let mut expression_runtime = ExpressionRuntime::new(&instructions);

        let result = expression_runtime.execute("main").unwrap();

        assert!(result.is_unit());
    }
}
