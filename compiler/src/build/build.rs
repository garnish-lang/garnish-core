use crate::build::InstructionMetadata;
use crate::error::CompilerError;
use crate::parse::{Definition, ParseNode};
use garnish_lang_traits::{GarnishData, Instruction, TypeConstants};

trait GetError<T, Data: GarnishData> {
    fn get_mut_or_error(&mut self, index: usize) -> Result<&mut T, CompilerError<Data::Error>>;
}

impl<Data: GarnishData> GetError<BuildNode<Data>, Data> for Vec<Option<BuildNode<Data>>> {
    fn get_mut_or_error(&mut self, index: usize) -> Result<&mut BuildNode<Data>, CompilerError<Data::Error>> {
        match self.get_mut(index) {
            Some(Some(node)) => Ok(node),
            _ => Err(CompilerError::new_message(format!("No node at index {}", index)))?,
        }
    }
}

pub struct BuildData {
    parse_root: usize,
    parse_tree: Vec<ParseNode>,
    instruction_metadata: Vec<InstructionMetadata>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum BuildNodeState {
    Uninitialized,
    Initialized,
}

#[derive(Debug, PartialEq, Eq)]
struct BuildNode<Data: GarnishData> {
    state: BuildNodeState,
    parse_node_index: usize,
    list_parent: Option<usize>,
    child_count: Data::SizeIterator,
    contributes_to_list: bool,
}

impl<Data: GarnishData> BuildNode<Data> {
    fn new(parse_node_index: usize) -> Self {
        Self {
            state: BuildNodeState::Uninitialized,
            parse_node_index,
            list_parent: None,
            child_count: Data::make_size_iterator_range(Data::Size::zero(), Data::Size::max_value()),
            contributes_to_list: true,
        }
    }

    fn new_with_list(parse_node_index: usize, list_parent: usize) -> Self {
        Self {
            state: BuildNodeState::Uninitialized,
            parse_node_index,
            list_parent: Some(list_parent),
            child_count: Data::make_size_iterator_range(Data::Size::zero(), Data::Size::max_value()),
            contributes_to_list: true,
        }
    }
}

pub fn build<Data: GarnishData>(parse_root: usize, parse_tree: Vec<ParseNode>, data: &mut Data) -> Result<BuildData, CompilerError<Data::Error>> {
    if parse_tree.is_empty() {
        data.push_instruction(Instruction::EndExpression, None)?;
        return Ok(BuildData {
            parse_root,
            parse_tree,
            instruction_metadata: vec![InstructionMetadata::new(None)],
        });
    }

    let mut nodes: Vec<Option<BuildNode<Data>>> = Vec::with_capacity(parse_tree.len());
    for _ in 0..parse_tree.len() {
        nodes.push(None);
    }
    nodes[parse_root] = Some(BuildNode::new(parse_root));

    let mut instruction_metadata = vec![];

    let mut stack = vec![parse_root];

    while let Some(node_index) = stack.pop() {
        let parse_node = match parse_tree.get(node_index) {
            Some(node) => node,
            None => Err(CompilerError::new_message(format!("No parse node at index {}", node_index)))?,
        };

        match parse_node.get_definition() {
            Definition::Unit => {
                let addr = data.add_unit()?;
                data.push_instruction(Instruction::Put, Some(addr))?;
                instruction_metadata.push(InstructionMetadata::new(Some(node_index)));
            }
            Definition::False => {
                let addr = data.add_false()?;
                data.push_instruction(Instruction::Put, Some(addr))?;
                instruction_metadata.push(InstructionMetadata::new(Some(node_index)));
            }
            Definition::True => {
                let addr = data.add_true()?;
                data.push_instruction(Instruction::Put, Some(addr))?;
                instruction_metadata.push(InstructionMetadata::new(Some(node_index)));
            }
            Definition::Number => {
                let addr = data.parse_add_number(parse_node.text())?;
                data.push_instruction(Instruction::Put, Some(addr))?;
                instruction_metadata.push(InstructionMetadata::new(Some(node_index)));
            }
            Definition::CharList => {
                let addr = data.parse_add_char_list(parse_node.text())?;
                data.push_instruction(Instruction::Put, Some(addr))?;
                instruction_metadata.push(InstructionMetadata::new(Some(node_index)));
            }
            Definition::ByteList => {
                let addr = data.parse_add_byte_list(parse_node.text())?;
                data.push_instruction(Instruction::Put, Some(addr))?;
                instruction_metadata.push(InstructionMetadata::new(Some(node_index)));
            }
            Definition::Symbol => {
                let addr = data.parse_add_symbol(&parse_node.text()[1..])?;
                data.push_instruction(Instruction::Put, Some(addr))?;
                instruction_metadata.push(InstructionMetadata::new(Some(node_index)));
            }
            Definition::Value => {
                data.push_instruction(Instruction::PutValue, None)?;
                instruction_metadata.push(InstructionMetadata::new(Some(node_index)));
            }
            Definition::Identifier => {
                let addr = data.parse_add_symbol(parse_node.text())?;
                data.push_instruction(Instruction::Resolve, Some(addr))?;
                instruction_metadata.push(InstructionMetadata::new(Some(node_index)));
            }
            Definition::ExpressionTerminator => {
                data.push_instruction(Instruction::EndExpression, None)?;
                instruction_metadata.push(InstructionMetadata::new(Some(node_index)));
            }
            Definition::Addition => {
                let node = match nodes.get(node_index) {
                    Some(Some(node)) => node,
                    _ => Err(CompilerError::new_message(format!("No build node at index {}", node_index)))?,
                };
                match node.state {
                    BuildNodeState::Uninitialized => {
                        stack.push(node.parse_node_index);
                        let right = parse_node
                            .get_right()
                            .ok_or(CompilerError::new_message("No right on Addition definition".to_string()))?;
                        let left = parse_node
                            .get_left()
                            .ok_or(CompilerError::new_message("No left on Addition definition".to_string()))?;
                        stack.push(right);
                        stack.push(left);

                        nodes[right] = Some(BuildNode::new(right));
                        nodes[left] = Some(BuildNode::new(left));

                        let node = nodes.get_mut_or_error(node_index)?;

                        node.state = BuildNodeState::Initialized
                    }
                    BuildNodeState::Initialized => {
                        data.push_instruction(Instruction::Add, None)?;
                        instruction_metadata.push(InstructionMetadata::new(Some(node.parse_node_index)));
                    }
                }
            }
            Definition::List => {
                let node = match nodes.get_mut(node_index) {
                    Some(Some(node)) => node,
                    _ => Err(CompilerError::new_message(format!("No build node at index {}", node_index)))?,
                };

                match node.state {
                    BuildNodeState::Uninitialized => {
                        node.contributes_to_list = false;

                        stack.push(node.parse_node_index);
                        let right = parse_node
                            .get_right()
                            .ok_or(CompilerError::new_message("No right on Addition definition".to_string()))?;
                        let left = parse_node
                            .get_left()
                            .ok_or(CompilerError::new_message("No left on Addition definition".to_string()))?;
                        stack.push(right);
                        stack.push(left);

                        match node.list_parent {
                            Some(parent) => {
                                nodes[right] = Some(BuildNode::new_with_list(right, parent));
                                nodes[left] = Some(BuildNode::new_with_list(left, parent));
                            }
                            None => {
                                nodes[right] = Some(BuildNode::new_with_list(right, node_index));
                                nodes[left] = Some(BuildNode::new_with_list(left, node_index));
                            }
                        }

                        let node = nodes.get_mut_or_error(node_index)?;

                        node.state = BuildNodeState::Initialized
                    }
                    BuildNodeState::Initialized => match node.list_parent {
                        Some(_) => {}
                        None => {
                            let node = nodes.get_mut_or_error(node_index)?;

                            let count = node
                                .child_count
                                .next()
                                .ok_or(CompilerError::new_message("Failed to increment child count for List".to_string()))?;

                            data.push_instruction(Instruction::MakeList, Some(count))?;
                            instruction_metadata.push(InstructionMetadata::new(Some(node.parse_node_index)));
                        }
                    },
                }
            }
            _ => unimplemented!(),
        }

        match nodes.get(node_index) {
            Some(Some(node)) if node.contributes_to_list => match node.list_parent {
                Some(parent) => {
                    let node = nodes.get_mut_or_error(parent)?;
                    node.child_count.next();
                }
                None => {}
            },
            _ => {}
        }
    }

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
    use garnish_lang_simple_data::{SimpleDataList, SimpleGarnishData, SimpleInstruction};
    use garnish_lang_traits::Instruction;

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
        let result = build(0, vec![], &mut data).unwrap();

        assert_eq!(data.get_instructions(), &vec![SimpleInstruction::new(Instruction::EndExpression, None)]);

        assert_eq!(data.get_data(), &SimpleDataList::default());
        assert_eq!(result.instruction_metadata, vec![InstructionMetadata::new(None)])
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

    #[test]
    fn build_identifier() {
        let (data, metadata) = build_input("my_value");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Resolve, Some(3)),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );
        assert_eq!(data.get_data(), &SimpleDataList::default().append_symbol("my_value"));
        assert_eq!(metadata, vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)])
    }
}

#[cfg(test)]
mod binary_operations {
    use crate::build::InstructionMetadata;
    use crate::build::build::tests::build_input;
    use garnish_lang_simple_data::{SimpleData, SimpleDataList, SimpleInstruction};
    use garnish_lang_traits::Instruction;

    #[test]
    fn build_addition() {
        let (data, metadata) = build_input("5 + 10");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::Put, Some(4)),
                SimpleInstruction::new(Instruction::Add, None),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );
        assert_eq!(
            data.get_data(),
            &SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into()))
        );
        assert_eq!(
            metadata,
            vec![
                InstructionMetadata::new(Some(0)),
                InstructionMetadata::new(Some(2)),
                InstructionMetadata::new(Some(1)),
                InstructionMetadata::new(None)
            ]
        )
    }
}

#[cfg(test)]
mod lists {
    use crate::build::InstructionMetadata;
    use crate::build::build::tests::build_input;
    use garnish_lang_simple_data::{SimpleData, SimpleDataList, SimpleInstruction};
    use garnish_lang_traits::Instruction;

    #[test]
    fn build_list() {
        let (data, metadata) = build_input("5 10 15 20 25");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::Put, Some(4)),
                SimpleInstruction::new(Instruction::Put, Some(5)),
                SimpleInstruction::new(Instruction::Put, Some(6)),
                SimpleInstruction::new(Instruction::Put, Some(7)),
                SimpleInstruction::new(Instruction::MakeList, Some(5)),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );
        assert_eq!(
            data.get_data(),
            &SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into()))
                .append(SimpleData::Number(15.into()))
                .append(SimpleData::Number(20.into()))
                .append(SimpleData::Number(25.into()))
        );
        assert_eq!(
            metadata,
            vec![
                InstructionMetadata::new(Some(0)),
                InstructionMetadata::new(Some(2)),
                InstructionMetadata::new(Some(4)),
                InstructionMetadata::new(Some(6)),
                InstructionMetadata::new(Some(8)),
                InstructionMetadata::new(Some(7)),
                InstructionMetadata::new(None)
            ]
        )
    }
}
