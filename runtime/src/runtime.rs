use std::collections::HashMap;

use garnish_lang_common::{Result, InstructionSet};

use crate::iterate::IterationData;
use crate::utils::insert_if_not_present;

#[derive(Clone, PartialOrd, PartialEq)]
pub(crate) enum CallType {
    Normal,
    Iteration,
    ExpressionIteration,
    Conditional,
}

#[derive(Clone)]
pub(crate) struct CallFrame {
    pub(crate) result_start: usize,
    pub(crate) cursor: usize,
    pub(crate) call_type: CallType,
}

#[derive(Clone)]
pub struct ExpressionRuntime {
    pub(crate) value_cursor: usize,
    pub(crate) ref_cursor: usize,
    pub(crate) constant_data_size: usize,
    pub(crate) input_size: usize,
    pub(crate) call_stack: Vec<CallFrame>,
    pub(crate) registers: Vec<usize>,
    pub(crate) data: Vec<u8>,
    pub(crate) instructions: Vec<u8>,
    pub(crate) symbol_table: HashMap<String, usize>,
    pub(crate) expression_table: Vec<usize>,
    pub(crate) expression_map: HashMap<String, usize>,
    pub(crate) input_stack: Vec<usize>,
    pub(crate) result_stack: Vec<usize>,
    pub(crate) list_stack: Vec<usize>,
    pub(crate) iteration_stack: Vec<IterationData>,

    // Known symbols
    pub(crate) true_value: usize,
    pub(crate) false_value: usize,
    pub(crate) length_value: usize,
    pub(crate) key_count_value: usize,
    pub(crate) left_value: usize,
    pub(crate) right_value: usize,
    pub(crate) base_value: usize,
    pub(crate) value_value: usize,
    pub(crate) start_value: usize,
    pub(crate) end_value: usize,
    pub(crate) step_value: usize,
    pub(crate) is_start_exclusive_value: usize,
    pub(crate) is_end_exclusive_value: usize,
    pub(crate) is_start_open_value: usize,
    pub(crate) is_end_open_value: usize,
    pub(crate) infinite_value: usize,
    pub(crate) current_value: usize,
    pub(crate) result_value: usize,
}

impl ExpressionRuntime {
    pub fn new<T: InstructionSet>(instructions: &T) -> ExpressionRuntime {
        let mut d: Vec<u8> = vec![0; 1024 * 5];

        let constant_data_size = instructions.get_data().len();
        for (i, v) in instructions.get_data().iter().enumerate() {
            // one offset for unit value
            d[i] = *v;
        }

        // store true and false symbol values for now
        // should make them fixed values
        let mut symbol_table = instructions.get_symbol_table().clone();
        symbol_table.insert("".to_string(), 0);

        // insert known symbols
        let true_value = insert_if_not_present("true", &mut symbol_table);
        let false_value = insert_if_not_present("false", &mut symbol_table);
        let length_value = insert_if_not_present("length", &mut symbol_table);
        let key_count_value = insert_if_not_present("key_count", &mut symbol_table);
        let left_value = insert_if_not_present("left", &mut symbol_table);
        let right_value = insert_if_not_present("right", &mut symbol_table);
        let base_value = insert_if_not_present("base", &mut symbol_table);
        let value_value = insert_if_not_present("value", &mut symbol_table);
        let start_value = insert_if_not_present("start", &mut symbol_table);
        let end_value = insert_if_not_present("end", &mut symbol_table);
        let step_value = insert_if_not_present("step", &mut symbol_table);
        let is_start_exclusive_value =
            insert_if_not_present("is_start_exclusive", &mut symbol_table);
        let is_end_exclusive_value = insert_if_not_present("is_end_exclusive", &mut symbol_table);
        let is_start_open_value = insert_if_not_present("is_start_open", &mut symbol_table);
        let is_end_open_value = insert_if_not_present("is_end_open", &mut symbol_table);
        let infinite_value = insert_if_not_present("infinite", &mut symbol_table);
        let current_value = insert_if_not_present("current", &mut symbol_table);
        let result_value = insert_if_not_present("result", &mut symbol_table);

        return ExpressionRuntime {
            value_cursor: constant_data_size,
            ref_cursor: 0,
            call_stack: vec![],
            registers: vec![0; 1024],
            data: d,
            instructions: instructions.get_instructions().clone(),
            symbol_table,
            expression_table: instructions.get_expression_table().clone(),
            expression_map: instructions.get_expression_map().clone(),
            input_stack: vec![],
            result_stack: vec![],
            list_stack: vec![],
            iteration_stack: vec![],
            constant_data_size,
            input_size: 0,
            true_value,
            false_value,
            length_value,
            key_count_value,
            left_value,
            right_value,
            base_value,
            value_value,
            start_value,
            end_value,
            step_value,
            is_start_exclusive_value,
            is_end_exclusive_value,
            is_start_open_value,
            is_end_open_value,
            infinite_value,
            current_value,
            result_value,
        };
    }

    pub fn set_mem(&mut self, bytes: usize) -> Result {
        self.data.resize(bytes, 0);
        self.registers.resize(bytes / 5, 0);
        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::ExpressionRuntime;

    pub fn data_slice(expression_runtime: &ExpressionRuntime, size: usize) -> Vec<u8> {
        let start = expression_runtime.constant_data_size;
        let end = start + size;
        Vec::from(&expression_runtime.data[start..end])
    }
}
