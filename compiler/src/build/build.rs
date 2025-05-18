use crate::build::InstructionMetadata;
use crate::error::CompilerError;
use crate::parse::{Definition, ParseNode};
use garnish_lang_traits::{GarnishData, Instruction};

pub struct BuildData {
    parse_root: usize,
    parse_tree: Vec<ParseNode>,
    instruction_metadata: Vec<InstructionMetadata>,
}

pub fn build<Data: GarnishData>(parse_root: usize, parse_tree: Vec<ParseNode>, data: &mut Data) -> Result<BuildData, CompilerError<Data::Error>> {
    if parse_tree.is_empty() {
        return Ok(BuildData {
            parse_root,
            parse_tree,
            instruction_metadata: vec![],
        });
    }

    let mut instruction_metadata = vec![];

    let root_node = match parse_tree.get(parse_root) {
        Some(node) => node,
        None => Err(CompilerError::new_message(format!("No node at given parse root {}", parse_root)))?,
    };

    match root_node.get_definition() {
        Definition::Unit => {
            let addr = data.add_unit()?;
            data.push_instruction(Instruction::Put, Some(addr))?;
        }
        Definition::False => {
            let addr = data.add_false()?;
            data.push_instruction(Instruction::Put, Some(addr))?;
        }
        Definition::True => {
            let addr = data.add_true()?;
            data.push_instruction(Instruction::Put, Some(addr))?;
        }
        Definition::Number => {
            let addr = data.parse_add_number(root_node.get_lex_token().get_text())?;
            data.push_instruction(Instruction::Put, Some(addr))?;
        }
        Definition::CharList => {
            let addr = data.parse_add_char_list(root_node.get_lex_token().get_text())?;
            data.push_instruction(Instruction::Put, Some(addr))?;
        }
        Definition::ByteList => {
            let addr = data.parse_add_byte_list(root_node.get_lex_token().get_text())?;
            data.push_instruction(Instruction::Put, Some(addr))?;
        }
        Definition::Symbol => {
            let addr = data.parse_add_symbol(&root_node.get_lex_token().get_text()[1..])?;
            data.push_instruction(Instruction::Put, Some(addr))?;
        }
        Definition::Value => {
            data.push_instruction(Instruction::PutValue, None)?;
        }
        Definition::ExpressionTerminator => {
            data.push_instruction(Instruction::EndExpression, None)?;
        }
        _ => unimplemented!(),
    }

    instruction_metadata.push(InstructionMetadata::new(Some(parse_root)));

    let last_instruction = data.get_instruction_iter().last();
    match last_instruction.and_then(|i| data.get_instruction(i)) {
        Some((Instruction::EndExpression, _)) => {}
        _ => {
            data.push_instruction(Instruction::EndExpression, None)?;
            instruction_metadata.push(InstructionMetadata::new(None));
        }
    }

    Ok(BuildData {
        parse_root,
        parse_tree,
        instruction_metadata,
    })
}

#[cfg(test)]
mod tests {
    use crate::build::InstructionMetadata;
    use crate::build::build::build;
    use crate::lex::lex;
    use crate::parse::parse;
    use garnish_lang_simple_data::SimpleGarnishData;

    pub fn build_input(input: &str) -> (SimpleGarnishData, Vec<InstructionMetadata>) {
        let tokens = lex(input).unwrap();
        let parsed = parse(&tokens).unwrap();
        let mut data = SimpleGarnishData::new();
        let result = build(parsed.get_root(), parsed.get_nodes_owned(), &mut data).unwrap();
        (data, result.instruction_metadata)
    }

    #[test]
    fn build_empty() {
        let mut data = SimpleGarnishData::new();
        build(0, vec![], &mut data).unwrap();

        assert!(data.get_instructions().is_empty());
    }
}

#[cfg(test)]
mod put_values {
    use crate::build::InstructionMetadata;
    use crate::build::build::tests::build_input;
    use garnish_lang_simple_data::{SimpleData, SimpleDataList, SimpleInstruction};
    use garnish_lang_traits::Instruction;

    #[test]
    fn build_expression_terminator() {
        let (data, metadata) = build_input(";;");

        assert_eq!(data.get_instructions(), &vec![SimpleInstruction::new(Instruction::EndExpression, None)]);

        assert_eq!(data.get_data(), &SimpleDataList::default());
        assert_eq!(metadata, vec![InstructionMetadata::new(Some(0))])
    }

    #[test]
    fn build_unit() {
        let (data, metadata) = build_input("()");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(0)),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );

        assert_eq!(data.get_data(), &SimpleDataList::default());
        assert_eq!(metadata, vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)])
    }

    #[test]
    fn build_false() {
        let (data, metadata) = build_input("$!");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(1)),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );

        assert_eq!(data.get_data(), &SimpleDataList::default());
        assert_eq!(metadata, vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)])
    }

    #[test]
    fn build_true() {
        let (data, metadata) = build_input("$?");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(2)),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );

        assert_eq!(data.get_data(), &SimpleDataList::default());
        assert_eq!(metadata, vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)])
    }

    #[test]
    fn build_number() {
        let (data, metadata) = build_input("5");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );

        assert_eq!(data.get_data(), &SimpleDataList::default().append(SimpleData::Number(5.into())));
        assert_eq!(metadata, vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)])
    }

    #[test]
    fn build_character_list() {
        let (data, metadata) = build_input("\"characters\"");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );

        assert_eq!(
            data.get_data(),
            &SimpleDataList::default().append(SimpleData::CharList("characters".into()))
        );
        assert_eq!(metadata, vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)])
    }

    #[test]
    fn build_symbol() {
        let (data, metadata) = build_input(":my_symbol");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );

        assert_eq!(data.get_data(), &SimpleDataList::default().append_symbol("my_symbol"));
        assert_eq!(metadata, vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)])
    }

    #[test]
    fn build_empty_symbol() {
        let (data, metadata) = build_input(":");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );

        assert_eq!(data.get_data(), &SimpleDataList::default().append_symbol(""));
        assert_eq!(metadata, vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)])
    }

    #[test]
    fn build_byte_list() {
        let (data, metadata) = build_input("'abc'");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );

        assert_eq!(
            data.get_data(),
            &SimpleDataList::default().append(SimpleData::ByteList(vec!['a' as u8, 'b' as u8, 'c' as u8]))
        );
        assert_eq!(metadata, vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)])
    }

    #[test]
    fn build_value() {
        let (data, metadata) = build_input("$");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::PutValue, None),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );
        assert_eq!(data.get_data(), &SimpleDataList::default());
        assert_eq!(metadata, vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)])
    }
}
