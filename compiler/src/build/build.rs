use crate::error::CompilerError;
use crate::parse::{Definition, ParseNode};
use garnish_lang_traits::{GarnishData, Instruction};

pub struct BuildData {
    parse_root: usize,
    parse_tree: Vec<ParseNode>,
}

pub fn build<Data: GarnishData>(parse_root: usize, parse_tree: Vec<ParseNode>, data: &mut Data) -> Result<BuildData, CompilerError<Data::Error>> {
    if parse_tree.is_empty() {
        return Ok(BuildData { parse_root, parse_tree });
    }

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
        _ => unimplemented!(),
    }
    
    data.push_instruction(Instruction::EndExpression, None)?;

    Ok(BuildData { parse_root, parse_tree })
}

#[cfg(test)]
mod tests {
    use crate::build::build::build;
    use crate::lex::lex;
    use crate::parse::parse;
    use garnish_lang_simple_data::SimpleGarnishData;

    pub fn build_input(input: &str) -> SimpleGarnishData {
        let tokens = lex(input).unwrap();
        let parsed = parse(&tokens).unwrap();
        let mut data = SimpleGarnishData::new();
        build(parsed.get_root(), parsed.get_nodes_owned(), &mut data).unwrap();
        data
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
    use garnish_lang_simple_data::{SimpleData, SimpleDataList, SimpleInstruction};
    use garnish_lang_traits::Instruction;
    use crate::build::build::tests::build_input;

    #[test]
    fn build_unit() {
        let data = build_input("()");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(0)),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );

        assert_eq!(data.get_data(), &SimpleDataList::default());
    }

    #[test]
    fn build_false() {
        let data = build_input("$!");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(1)),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );

        assert_eq!(data.get_data(), &SimpleDataList::default());
    }

    #[test]
    fn build_true() {
        let data = build_input("$?");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(2)),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );

        assert_eq!(data.get_data(), &SimpleDataList::default());
    }

    #[test]
    fn build_number() {
        let data = build_input("5");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );

        assert_eq!(data.get_data(), &SimpleDataList::default().append(SimpleData::Number(5.into())));
    }

    #[test]
    fn build_character_list() {
        let data = build_input("\"characters\"");

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
    }

    #[test]
    fn build_symbol() {
        let data = build_input(":my_symbol");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );

        assert_eq!(data.get_data(), &SimpleDataList::default().append_symbol("my_symbol"));
    }

    #[test]
    fn build_byte_list() {
        let data = build_input("'abc'");

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
    }

    #[test]
    fn build_value() {
        let data = build_input("$");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::PutValue, None),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );
        assert_eq!(data.get_data(), &SimpleDataList::default());
    }
}
