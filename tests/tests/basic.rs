#[cfg(test)]
mod tests {
    use garnish_lang_compiler::{instructions_from_ast, lex, parse};
    use garnish_lang_runtime::{GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn basic_addition() {
        let input = "5 + 5";

        let lexed = lex(&input.into()).unwrap();
        let parsed = parse(lexed).unwrap();

        let mut data = SimpleRuntimeData::new();

        instructions_from_ast(parsed.get_root(), parsed.get_nodes().clone(), &mut data).unwrap();

        data.set_end_of_constant(data.get_data_len()).unwrap();

        data.set_instruction_cursor(1).unwrap();

        data.execute_all_instructions().unwrap();

        assert_eq!(data.get_integer(data.get_result().unwrap()).unwrap(), 10)
    }
}
