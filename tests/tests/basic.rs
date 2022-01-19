#[cfg(test)]
mod tests {
    use garnish_lang_compiler::{build_with_data, lex, parse};
    use garnish_lang_runtime::{GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn basic_addition() {
        let input = "5 + 5";

        let lexed = lex(input).unwrap();
        let parsed = parse(lexed).unwrap();

        let mut data = SimpleRuntimeData::new();

        build_with_data(parsed.get_root(), parsed.get_nodes().clone(), &mut data).unwrap();

        data.set_end_of_constant(data.get_data_len()).unwrap();

        data.set_instruction_cursor(data.get_jump_point(0).unwrap()).unwrap();

        data.execute_all_instructions().unwrap();

        assert_eq!(data.get_number(data.get_current_value().unwrap()).unwrap(), 10)
    }
}
