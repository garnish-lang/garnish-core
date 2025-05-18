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
        None => Err(CompilerError::new_message(format!("No node at given parse root {}", parse_root)))?
    };
    
    match root_node.get_definition() {
        Definition::Number => {
            let addr = data.parse_add_number(root_node.get_lex_token().get_text())?;
            data.push_instruction(Instruction::Put, Some(addr))?;
        }
        _ => unimplemented!()
    }
    
    Ok(BuildData { parse_root, parse_tree })
}

#[cfg(test)]
mod tests {
    use crate::build::build::build;
    use crate::lex::lex;
    use crate::parse::parse;
    use garnish_lang_simple_data::{SimpleData, SimpleDataList, SimpleGarnishData, SimpleInstruction};
    use garnish_lang_traits::Instruction;

    #[test]
    fn build_empty() {
        let mut data = SimpleGarnishData::new();
        build(0, vec![], &mut data).unwrap();

        assert!(data.get_instructions().is_empty());
    }

    #[test]
    fn build_number() {
        let tokens = lex("5").unwrap();
        let parsed = parse(&tokens).unwrap();
        let mut data = SimpleGarnishData::new();
        build(parsed.get_root(), parsed.get_nodes_owned(), &mut data).unwrap();

        assert_eq!(data.get_instructions(), &vec![SimpleInstruction::new(Instruction::Put, Some(3))]);

        assert_eq!(data.get_data(), &SimpleDataList::default().append(SimpleData::Number(5.into())));
    }
}
