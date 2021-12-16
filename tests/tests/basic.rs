#[cfg(test)]
mod tests {
    use garnish_lang_compiler::{instructions_from_ast, lex, parse};
    use garnish_lang_runtime::{EmptyContext, GarnishLangRuntime, GarnishLangRuntimeData, GarnishLangRuntimeState, SimpleRuntimeData};

    #[test]
    fn basic_addition() {
        let input = "5 + 5";

        let lexed = lex(&input.into()).unwrap();
        let parsed = parse(lexed).unwrap();

        let mut data = SimpleRuntimeData::new();

        instructions_from_ast(parsed.get_root(), parsed.get_nodes().clone(), &mut data).unwrap();

        data.set_end_of_constant(data.get_data_len()).unwrap();

        data.set_instruction_cursor(1).unwrap();

        let mut runtime = GarnishLangRuntime::new_with_data(data);

        loop {
            match runtime.execute_current_instruction::<EmptyContext>(None) {
                Err(e) => {
                    println!("{:?}", e);
                    break;
                }
                Ok(data) => match data.get_state() {
                    GarnishLangRuntimeState::Running => (),
                    GarnishLangRuntimeState::End => break,
                },
            }
        }

        assert_eq!(
            runtime
                .get_data_pool()
                .get_integer(runtime.get_data_pool().get_result().unwrap())
                .unwrap(),
            10
        )
    }
}
