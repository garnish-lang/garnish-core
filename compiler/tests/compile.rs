#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use garnish_lang_compiler::build::build_with_data;
    use garnish_lang_compiler::lex::lex;
    use garnish_lang_compiler::parse::parse;
    use garnish_lang_simple_data::{symbol_value, SimpleData, SimpleDataList, SimpleGarnishData, SimpleNumber};
    use garnish_lang_traits::{GarnishData, Instruction};

    fn read_and_build(
        file: &str,
        expected_instructions: Vec<(Instruction, Option<usize>)>,
        expected_data: SimpleDataList,
    ) {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/resources");
        path.push(file);

        let source = fs::read_to_string(path).unwrap();

        let lexed = lex(&source).unwrap();

        // Not showing in debugger on Windows, uncomment to inspect in output
        // for token in lexed.iter() {
        //     println!("{:?}", token);
        // }

        let parsed = parse(&lexed).unwrap();
        let mut data = SimpleGarnishData::new();
        build_with_data(
            parsed.get_root(),
            parsed.get_nodes().clone(),
            &mut data,
        ).unwrap();

        for i in 0..expected_instructions.len() {
            assert_eq!(
                data.get_instruction(i),
                expected_instructions.get(i).cloned()
            )
        }

        for i in 0..expected_data.len() {
            assert_eq!(
                data.get_data().get(i),
                expected_data.get(i)
            )
        }
    }

    #[test]
    fn simple_list() {
        read_and_build(
            "simple_list",
            vec![
                (Instruction::Put, Some(3)),
                (Instruction::Put, Some(4)),
                (Instruction::MakePair, None),
                (Instruction::Put, Some(5)),
                (Instruction::Put, Some(6)),
                (Instruction::MakePair, None),
                (Instruction::Put, Some(7)),
                (Instruction::Put, Some(8)),
                (Instruction::MakePair, None),
                (Instruction::MakeList, Some(3)),
                (Instruction::EndExpression, None)
            ],
            SimpleDataList::default()
                .append(SimpleData::Symbol(symbol_value("item_one")))
                .append(SimpleData::Number(SimpleNumber::Integer(1)))
                .append(SimpleData::Symbol(symbol_value("item_two")))
                .append(SimpleData::Number(SimpleNumber::Integer(2)))
                .append(SimpleData::Symbol(symbol_value("item_three")))
                .append(SimpleData::Number(SimpleNumber::Integer(3)))
        )
    }
}