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

#[derive(Debug, PartialEq, Eq)]
struct ConditionItem<Data: GarnishData> {
    node_index: usize,
    jump_index_to_update: Data::Size,
    root_end_instruction: (Instruction, Option<Data::Size>),
}

pub struct BuildData<Data: GarnishData> {
    parse_root: usize,
    parse_tree: Vec<ParseNode>,
    instruction_metadata: Vec<InstructionMetadata>,
    jump_index: Data::Size,
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
    jump_index_to_update: Option<Data::Size>,
    root_end_instruction: Option<Vec<(Instruction, Option<Data::Size>)>>,
    conditional_parent: Option<usize>,
    conditional_items: Vec<ConditionItem<Data>>,
}

impl<Data: GarnishData> BuildNode<Data> {
    fn new(parse_node_index: usize) -> Self {
        Self {
            state: BuildNodeState::Uninitialized,
            parse_node_index,
            list_parent: None,
            child_count: Data::make_size_iterator_range(Data::Size::zero(), Data::Size::max_value()),
            contributes_to_list: true,
            jump_index_to_update: None,
            root_end_instruction: None,
            conditional_parent: None,
            conditional_items: vec![],
        }
    }

    fn new_with_list(parse_node_index: usize, list_parent: usize) -> Self {
        Self {
            state: BuildNodeState::Uninitialized,
            parse_node_index,
            list_parent: Some(list_parent),
            child_count: Data::make_size_iterator_range(Data::Size::zero(), Data::Size::max_value()),
            contributes_to_list: true,
            jump_index_to_update: None,
            root_end_instruction: None,
            conditional_parent: None,
            conditional_items: vec![],
        }
    }

    fn new_with_conditional(parse_node_index: usize, conditional_parent: usize) -> Self {
        Self {
            state: BuildNodeState::Uninitialized,
            parse_node_index,
            list_parent: None,
            child_count: Data::make_size_iterator_range(Data::Size::zero(), Data::Size::max_value()),
            contributes_to_list: true,
            jump_index_to_update: None,
            root_end_instruction: None,
            conditional_parent: Some(conditional_parent),
            conditional_items: vec![],
        }
    }

    fn new_with_jump(parse_node_index: usize, jump_index: Data::Size) -> Self {
        Self {
            state: BuildNodeState::Uninitialized,
            parse_node_index,
            list_parent: None,
            child_count: Data::make_size_iterator_range(Data::Size::zero(), Data::Size::max_value()),
            contributes_to_list: true,
            jump_index_to_update: Some(jump_index),
            root_end_instruction: None,
            conditional_parent: None,
            conditional_items: vec![],
        }
    }

    fn new_with_jump_and_end(parse_node_index: usize, jump_index: Data::Size, end_instruction: Vec<(Instruction, Option<Data::Size>)>) -> Self {
        Self {
            state: BuildNodeState::Uninitialized,
            parse_node_index,
            list_parent: None,
            child_count: Data::make_size_iterator_range(Data::Size::zero(), Data::Size::max_value()),
            contributes_to_list: true,
            jump_index_to_update: Some(jump_index),
            root_end_instruction: Some(end_instruction),
            conditional_parent: None,
            conditional_items: vec![],
        }
    }
}

pub fn build<Data: GarnishData>(
    parse_root: usize,
    parse_tree: Vec<ParseNode>,
    data: &mut Data,
) -> Result<BuildData<Data>, CompilerError<Data::Error>> {
    if parse_tree.is_empty() {
        data.push_instruction(Instruction::EndExpression, None)?;
        return Ok(BuildData {
            parse_root,
            parse_tree,
            instruction_metadata: vec![InstructionMetadata::new(None)],
            jump_index: Data::Size::zero(),
        });
    }

    let mut nodes: Vec<Option<BuildNode<Data>>> = Vec::with_capacity(parse_tree.len());
    for _ in 0..parse_tree.len() {
        nodes.push(None);
    }
    nodes[parse_root] = Some(BuildNode::new(parse_root));

    let mut instruction_metadata = vec![];

    let mut root_stack = vec![parse_root];
    // same as root jump index but this one needs to be returned
    let tree_root_jump = data.get_jump_table_len();

    while let Some(root_index) = root_stack.pop() {
        match nodes.get(root_index) {
            Some(Some(node)) => match &node.jump_index_to_update {
                Some(index) => {
                    let jump_index = data.get_instruction_len();
                    match data.get_jump_point_mut(index.clone()) {
                        Some(item) => *item = jump_index,
                        None => todo!(),
                    }
                }
                None => data.push_jump_point(data.get_instruction_len())?,
            },
            _ => {
                data.push_jump_point(data.get_instruction_len())?;
            }
        }
        let mut stack = vec![root_index];

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
                    let node = match nodes.get_mut(node_index) {
                        Some(Some(node)) => node,
                        _ => Err(CompilerError::new_message(format!("No build node at index {}", node_index)))?,
                    };

                    match node.state {
                        BuildNodeState::Uninitialized => {
                            node.state = BuildNodeState::Initialized;

                            match parse_node.get_right() {
                                None => {},
                                Some(right) => {
                                    stack.push(right);
                                    nodes[right] = Some(BuildNode::new(right));
                                },
                            }
                            
                            stack.push(node_index);

                            match parse_node.get_left() {
                                None => {},
                                Some(left) => {
                                    stack.push(left);
                                    nodes[left] = Some(BuildNode::new(left));
                                },
                            }
                        }
                        BuildNodeState::Initialized => {
                            node.contributes_to_list = false;
                            
                            let addr = data.parse_add_number(parse_node.text())?;
                            data.push_instruction(Instruction::Put, Some(addr))?;
                            instruction_metadata.push(InstructionMetadata::new(Some(node_index)));
                        }
                    }
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
                Definition::Group => {
                    let right = parse_node
                        .get_right()
                        .ok_or(CompilerError::new_message("No right on NestedExpression definition".to_string()))?;

                    nodes[right] = Some(BuildNode::new(right));

                    stack.push(right);
                }
                Definition::SideEffect => {
                    let node = match nodes.get_mut(node_index) {
                        Some(Some(node)) => node,
                        _ => Err(CompilerError::new_message(format!("No build node at index {}", node_index)))?,
                    };

                    match node.state {
                        BuildNodeState::Uninitialized => {
                            node.state = BuildNodeState::Initialized;

                            data.push_instruction(Instruction::StartSideEffect, None)?;
                            instruction_metadata.push(InstructionMetadata::new(Some(node_index)));

                            let right = parse_node
                                .get_right()
                                .ok_or(CompilerError::new_message("No right on SideEffect definition".to_string()))?;

                            nodes[right] = Some(BuildNode::new(right));

                            stack.push(node_index);
                            stack.push(right);
                        }
                        BuildNodeState::Initialized => {
                            data.push_instruction(Instruction::EndSideEffect, None)?;
                            instruction_metadata.push(InstructionMetadata::new(Some(node_index)));
                        }
                    }
                }
                Definition::NestedExpression => {
                    let jump_index = data.get_jump_table_len();
                    data.push_jump_point(Data::Size::zero())?;
                    let addr = data.add_expression(jump_index.clone())?;
                    data.push_instruction(Instruction::Put, Some(addr))?;
                    instruction_metadata.push(InstructionMetadata::new(Some(node_index)));

                    let right = parse_node
                        .get_right()
                        .ok_or(CompilerError::new_message("No right on NestedExpression definition".to_string()))?;

                    nodes[right] = Some(BuildNode::new_with_jump(right, jump_index));

                    root_stack.push(right);
                }
                Definition::JumpIfTrue => {
                    let node = match nodes.get_mut(node_index) {
                        Some(Some(node)) => node,
                        _ => Err(CompilerError::new_message(format!("No build node at index {}", node_index)))?,
                    };

                    match node.state {
                        BuildNodeState::Uninitialized => {
                            node.state = BuildNodeState::Initialized;

                            stack.push(node.parse_node_index);
                            let left = parse_node
                                .get_left()
                                .ok_or(CompilerError::new_message("No left on JumpIfTrue definition".to_string()))?;
                            stack.push(left);
                        }
                        BuildNodeState::Initialized => {
                            let jump_index = data.get_jump_table_len();
                            data.push_jump_point(Data::Size::zero())?;
                            data.push_instruction(Instruction::JumpIfTrue, Some(jump_index.clone()))?;
                            instruction_metadata.push(InstructionMetadata::new(Some(node_index)));

                            let right = parse_node
                                .get_right()
                                .ok_or(CompilerError::new_message("No right on JumpIfTrue definition".to_string()))?;
                            match node.conditional_parent {
                                Some(conditional_parent) => match nodes.get_mut(conditional_parent) {
                                    Some(Some(parent)) => parent.conditional_items.push(ConditionItem {
                                        node_index: right,
                                        jump_index_to_update: jump_index,
                                        root_end_instruction: (Instruction::Invalid, None),
                                    }),
                                    _ => {}
                                },
                                None => {
                                    root_stack.push(right);

                                    let jump_to_index = data.get_jump_table_len();
                                    data.push_jump_point(data.get_instruction_len())?;

                                    nodes[right] = Some(BuildNode::new_with_jump_and_end(
                                        right,
                                        jump_index.clone(),
                                        vec![(Instruction::JumpTo, Some(jump_to_index))],
                                    ));
                                }
                            }
                        }
                    }
                }
                Definition::ElseJump => {
                    let node = match nodes.get_mut(node_index) {
                        Some(Some(node)) => node,
                        _ => Err(CompilerError::new_message(format!("No build node at index {}", node_index)))?,
                    };

                    match node.state {
                        BuildNodeState::Uninitialized => {
                            node.state = BuildNodeState::Initialized;

                            stack.push(node.parse_node_index);
                            let right = parse_node
                                .get_right()
                                .ok_or(CompilerError::new_message("No right on ElseJump definition".to_string()))?;
                            let left = parse_node
                                .get_left()
                                .ok_or(CompilerError::new_message("No left on ElseJump definition".to_string()))?;
                            stack.push(right);
                            stack.push(left);

                            match node.conditional_parent {
                                Some(parent) => {
                                    nodes[right] = Some(BuildNode::new_with_conditional(right, parent));
                                    nodes[left] = Some(BuildNode::new_with_conditional(left, parent));
                                }
                                None => {
                                    nodes[right] = Some(BuildNode::new_with_conditional(right, node_index));
                                    nodes[left] = Some(BuildNode::new_with_conditional(left, node_index));
                                }
                            }
                        }
                        BuildNodeState::Initialized => match node.conditional_parent {
                            Some(_) => {}
                            None => {
                                let mut new_items: Vec<(usize, BuildNode<Data>)> = vec![];

                                let jump_to_index = data.get_jump_table_len();
                                data.push_jump_point(data.get_instruction_len())?;

                                for condition in &node.conditional_items {
                                    root_stack.push(condition.node_index);

                                    new_items.push((
                                        condition.node_index,
                                        BuildNode::new_with_jump_and_end(
                                            condition.node_index,
                                            condition.jump_index_to_update.clone(),
                                            vec![(Instruction::JumpTo, Some(jump_to_index.clone()))],
                                        ),
                                    ));
                                }

                                for (index, data) in new_items {
                                    nodes[index] = Some(data);
                                }
                            }
                        },
                    }
                }
                Definition::And => {
                    let node = match nodes.get_mut(node_index) {
                        Some(Some(node)) => node,
                        _ => Err(CompilerError::new_message(format!("No build node at index {}", node_index)))?,
                    };

                    match node.state {
                        BuildNodeState::Uninitialized => {
                            node.state = BuildNodeState::Initialized;

                            stack.push(node.parse_node_index);
                            let left = parse_node
                                .get_left()
                                .ok_or(CompilerError::new_message("No left on And definition".to_string()))?;
                            stack.push(left);

                            nodes[left] = Some(BuildNode::new_with_conditional(left, node_index));
                        }
                        BuildNodeState::Initialized => {
                            let jump_index = data.get_jump_table_len();
                            data.push_jump_point(Data::Size::zero())?;
                            data.push_instruction(Instruction::And, Some(jump_index.clone()))?;
                            instruction_metadata.push(InstructionMetadata::new(Some(node_index)));

                            let right = parse_node
                                .get_right()
                                .ok_or(CompilerError::new_message("No right on And definition".to_string()))?;

                            root_stack.push(right);

                            let jump_to_index = data.get_jump_table_len();
                            data.push_jump_point(data.get_instruction_len())?;

                            nodes[right] = Some(BuildNode::new_with_jump_and_end(
                                right,
                                jump_index.clone(),
                                vec![(Instruction::Tis, None), (Instruction::JumpTo, Some(jump_to_index))],
                            ));
                        }
                    }
                }
                Definition::Reapply => {
                    let node = match nodes.get_mut(node_index) {
                        Some(Some(node)) => node,
                        _ => Err(CompilerError::new_message(format!("No build node at index {}", node_index)))?,
                    };

                    match node.state {
                        BuildNodeState::Uninitialized => {
                            node.state = BuildNodeState::Initialized;

                            stack.push(node.parse_node_index);
                            let left = parse_node
                                .get_left()
                                .ok_or(CompilerError::new_message("No left on Reapply definition".to_string()))?;
                            stack.push(left);

                            nodes[left] = Some(BuildNode::new_with_conditional(left, node_index));
                        }
                        BuildNodeState::Initialized => {
                            let jump_index = data.get_jump_table_len();
                            data.push_jump_point(Data::Size::zero())?;
                            data.push_instruction(Instruction::JumpIfTrue, Some(jump_index.clone()))?;
                            instruction_metadata.push(InstructionMetadata::new(Some(node_index)));
                            data.push_instruction(Instruction::PutValue, None)?;
                            instruction_metadata.push(InstructionMetadata::new(None));

                            let right = parse_node
                                .get_right()
                                .ok_or(CompilerError::new_message("No right on Reapply definition".to_string()))?;

                            root_stack.push(right);

                            let jump_to_index = data.get_jump_table_len();
                            data.push_jump_point(data.get_instruction_len())?;

                            nodes[right] = Some(BuildNode::new_with_jump_and_end(
                                right,
                                jump_index.clone(),
                                vec![(Instruction::UpdateValue, None), (Instruction::JumpTo, Some(jump_to_index))],
                            ));
                        }
                    }
                }
                Definition::Subexpression => {
                    let node = match nodes.get_mut(node_index) {
                        Some(Some(node)) => node,
                        _ => Err(CompilerError::new_message(format!("No build node at index {}", node_index)))?,
                    };
                    match node.state {
                        BuildNodeState::Uninitialized => {
                            node.state = BuildNodeState::Initialized;

                            let right = parse_node
                                .get_right()
                                .ok_or(CompilerError::new_message("No right on Subexpression definition".to_string()))?;
                            let left = parse_node
                                .get_left()
                                .ok_or(CompilerError::new_message("No left on Subexpression definition".to_string()))?;

                            stack.push(right);
                            stack.push(node_index);
                            stack.push(left);

                            nodes[right] = Some(BuildNode::new(right));
                            nodes[left] = Some(BuildNode::new(left));
                        }
                        BuildNodeState::Initialized => {
                            data.push_instruction(Instruction::UpdateValue, None)?;
                            instruction_metadata.push(InstructionMetadata::new(Some(node_index)));
                        }
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
        let end_instructions = match nodes.get(root_index) {
            Some(Some(node)) => match &node.root_end_instruction {
                Some(end_instruction) => end_instruction.clone(),
                None => vec![(Instruction::EndExpression, None)],
            },
            _ => vec![(Instruction::EndExpression, None)],
        };

        for end_instruction in end_instructions {
            match last_instruction.clone().and_then(|i| data.get_instruction(i)) {
                Some(instruction) if instruction == end_instruction => {}
                _ => {
                    data.push_instruction(end_instruction.0, end_instruction.1)?;
                    instruction_metadata.push(InstructionMetadata::new(None));
                }
            }
        }
    }

    Ok(BuildData {
        parse_root,
        parse_tree,
        instruction_metadata,
        jump_index: tree_root_jump,
    })
}

#[cfg(test)]
mod tests {
    use crate::build::build::build;
    use crate::build::{BuildData, InstructionMetadata};
    use crate::lex::lex;
    use crate::parse::parse;
    use garnish_lang_simple_data::{SimpleDataList, SimpleGarnishData, SimpleInstruction};
    use garnish_lang_traits::Instruction;

    pub fn build_input(input: &str) -> (SimpleGarnishData, BuildData<SimpleGarnishData>) {
        let tokens = lex(input).unwrap();
        let parsed = parse(&tokens).unwrap();
        let mut data = SimpleGarnishData::new();
        let result = build(parsed.get_root(), parsed.get_nodes_owned(), &mut data).unwrap();
        (data, result)
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
        let (data, build_data) = build_input(";;");

        assert_eq!(data.get_instructions(), &vec![SimpleInstruction::new(Instruction::EndExpression, None)]);

        assert_eq!(data.get_data(), &SimpleDataList::default());
        assert_eq!(build_data.instruction_metadata, vec![InstructionMetadata::new(Some(0))])
    }

    #[test]
    fn build_unit() {
        let (data, build_data) = build_input("()");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(0)),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );

        assert_eq!(data.get_data(), &SimpleDataList::default());
        assert_eq!(
            build_data.instruction_metadata,
            vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)]
        )
    }

    #[test]
    fn build_false() {
        let (data, build_data) = build_input("$!");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(1)),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );

        assert_eq!(data.get_data(), &SimpleDataList::default());
        assert_eq!(
            build_data.instruction_metadata,
            vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)]
        )
    }

    #[test]
    fn build_true() {
        let (data, build_data) = build_input("$?");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(2)),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );

        assert_eq!(data.get_data(), &SimpleDataList::default());
        assert_eq!(
            build_data.instruction_metadata,
            vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)]
        )
    }

    #[test]
    fn build_number() {
        let (data, build_data) = build_input("5");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );

        assert_eq!(data.get_data(), &SimpleDataList::default().append(SimpleData::Number(5.into())));
        assert_eq!(
            build_data.instruction_metadata,
            vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)]
        )
    }

    #[test]
    fn build_character_list() {
        let (data, build_data) = build_input("\"characters\"");

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
        assert_eq!(
            build_data.instruction_metadata,
            vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)]
        )
    }

    #[test]
    fn build_symbol() {
        let (data, build_data) = build_input(":my_symbol");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );

        assert_eq!(data.get_data(), &SimpleDataList::default().append_symbol("my_symbol"));
        assert_eq!(
            build_data.instruction_metadata,
            vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)]
        )
    }

    #[test]
    fn build_empty_symbol() {
        let (data, build_data) = build_input(":");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );

        assert_eq!(data.get_data(), &SimpleDataList::default().append_symbol(""));
        assert_eq!(
            build_data.instruction_metadata,
            vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)]
        )
    }

    #[test]
    fn build_byte_list() {
        let (data, build_data) = build_input("'abc'");

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
        assert_eq!(
            build_data.instruction_metadata,
            vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)]
        )
    }

    #[test]
    fn build_value() {
        let (data, build_data) = build_input("$");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::PutValue, None),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );
        assert_eq!(data.get_data(), &SimpleDataList::default());
        assert_eq!(
            build_data.instruction_metadata,
            vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)]
        )
    }

    #[test]
    fn build_identifier() {
        let (data, build_data) = build_input("my_value");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Resolve, Some(3)),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );
        assert_eq!(data.get_data(), &SimpleDataList::default().append_symbol("my_value"));
        assert_eq!(
            build_data.instruction_metadata,
            vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)]
        )
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
        let (data, build_data) = build_input("5 + 10");

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
            build_data.instruction_metadata,
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
        let (data, build_data) = build_input("5 10 15 20 25");

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
            build_data.instruction_metadata,
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

#[cfg(test)]
mod expressions {
    use crate::build::InstructionMetadata;
    use crate::build::build::tests::build_input;
    use garnish_lang_simple_data::{SimpleData, SimpleDataList, SimpleInstruction};
    use garnish_lang_traits::Instruction;

    #[test]
    fn build_expression() {
        let (data, build_data) = build_input("{ 5 + 10 }");

        assert_eq!(build_data.jump_index, 0);
        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::EndExpression, None),
                SimpleInstruction::new(Instruction::Put, Some(4)),
                SimpleInstruction::new(Instruction::Put, Some(5)),
                SimpleInstruction::new(Instruction::Add, None),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );
        assert_eq!(data.get_jump_points(), &vec![0, 2]);
        assert_eq!(
            data.get_data(),
            &SimpleDataList::default()
                .append(SimpleData::Expression(1))
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into()))
        );
        assert_eq!(
            build_data.instruction_metadata,
            vec![
                InstructionMetadata::new(Some(0)),
                InstructionMetadata::new(None),
                InstructionMetadata::new(Some(1)),
                InstructionMetadata::new(Some(3)),
                InstructionMetadata::new(Some(2)),
                InstructionMetadata::new(None),
            ]
        )
    }
}

#[cfg(test)]
mod jumps {
    use crate::build::InstructionMetadata;
    use crate::build::build::tests::build_input;
    use garnish_lang_simple_data::{SimpleData, SimpleDataList, SimpleInstruction};
    use garnish_lang_traits::Instruction;

    #[test]
    fn jump_if_true() {
        let (data, build_data) = build_input("$? ?> 10");

        assert_eq!(build_data.jump_index, 0);
        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(2)),
                SimpleInstruction::new(Instruction::JumpIfTrue, Some(1)),
                SimpleInstruction::new(Instruction::EndExpression, None),
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::JumpTo, Some(2)),
            ]
        );
        assert_eq!(data.get_jump_points(), &vec![0, 3, 2]);
        assert_eq!(data.get_data(), &SimpleDataList::default().append(SimpleData::Number(10.into())));
        assert_eq!(
            build_data.instruction_metadata,
            vec![
                InstructionMetadata::new(Some(0)),
                InstructionMetadata::new(Some(1)),
                InstructionMetadata::new(None),
                InstructionMetadata::new(Some(2)),
                InstructionMetadata::new(None),
            ]
        )
    }

    #[test]
    fn jump_if_true_with_else() {
        let (data, build_data) = build_input("$? ?> 10 |> 20");

        assert_eq!(build_data.jump_index, 0);
        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(2)),
                SimpleInstruction::new(Instruction::JumpIfTrue, Some(1)),
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::EndExpression, None),
                SimpleInstruction::new(Instruction::Put, Some(4)),
                SimpleInstruction::new(Instruction::JumpTo, Some(2)),
            ]
        );
        assert_eq!(data.get_jump_points(), &vec![0, 4, 3]);
        assert_eq!(
            data.get_data(),
            &SimpleDataList::default()
                .append(SimpleData::Number(20.into()))
                .append(SimpleData::Number(10.into()))
        );
        assert_eq!(
            build_data.instruction_metadata,
            vec![
                InstructionMetadata::new(Some(0)),
                InstructionMetadata::new(Some(1)),
                InstructionMetadata::new(Some(4)),
                InstructionMetadata::new(None),
                InstructionMetadata::new(Some(2)),
                InstructionMetadata::new(None),
            ]
        )
    }

    #[test]
    fn triple_jump_if_true_with_else() {
        let (data, build_data) = build_input("$? ?> 10 |> $? ?> 20 |> $? ?> 30");

        assert_eq!(build_data.jump_index, 0);
        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(2)),
                SimpleInstruction::new(Instruction::JumpIfTrue, Some(1)),
                SimpleInstruction::new(Instruction::Put, Some(2)),
                SimpleInstruction::new(Instruction::JumpIfTrue, Some(2)),
                SimpleInstruction::new(Instruction::Put, Some(2)),
                SimpleInstruction::new(Instruction::JumpIfTrue, Some(3)),
                SimpleInstruction::new(Instruction::EndExpression, None),
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::JumpTo, Some(4)),
                SimpleInstruction::new(Instruction::Put, Some(4)),
                SimpleInstruction::new(Instruction::JumpTo, Some(4)),
                SimpleInstruction::new(Instruction::Put, Some(5)),
                SimpleInstruction::new(Instruction::JumpTo, Some(4)),
            ]
        );
        assert_eq!(data.get_jump_points(), &vec![0, 11, 9, 7, 6]);
        assert_eq!(
            data.get_data(),
            &SimpleDataList::default()
                .append(SimpleData::Number(30.into()))
                .append(SimpleData::Number(20.into()))
                .append(SimpleData::Number(10.into()))
        );
    }
}

#[cfg(test)]
mod logical {
    use crate::build::build::tests::build_input;
    use garnish_lang_simple_data::{SimpleData, SimpleDataList, SimpleInstruction};
    use garnish_lang_traits::Instruction;

    #[test]
    fn and() {
        let (data, build_data) = build_input("5 && 10");

        assert_eq!(build_data.jump_index, 0);
        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::And, Some(1)),
                SimpleInstruction::new(Instruction::EndExpression, None),
                SimpleInstruction::new(Instruction::Put, Some(4)),
                SimpleInstruction::new(Instruction::Tis, None),
                SimpleInstruction::new(Instruction::JumpTo, Some(2)),
            ]
        );
        assert_eq!(data.get_jump_points(), &vec![0, 3, 2]);
        assert_eq!(
            data.get_data(),
            &SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into()))
        );
    }
}

#[cfg(test)]
mod reapply {
    use crate::build::build::tests::build_input;
    use garnish_lang_simple_data::{SimpleData, SimpleDataList, SimpleInstruction};
    use garnish_lang_traits::Instruction;

    #[test]
    fn reapply() {
        let (data, build_data) = build_input("$! ^~ 40");

        assert_eq!(build_data.jump_index, 0);
        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(1)),
                SimpleInstruction::new(Instruction::JumpIfTrue, Some(1)),
                SimpleInstruction::new(Instruction::PutValue, None),
                SimpleInstruction::new(Instruction::EndExpression, None),
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::UpdateValue, None),
                SimpleInstruction::new(Instruction::JumpTo, Some(2)),
            ]
        );
        assert_eq!(data.get_jump_points(), &vec![0, 4, 3]);
        assert_eq!(data.get_data(), &SimpleDataList::default().append(SimpleData::Number(40.into())));
    }
}

#[cfg(test)]
mod subexpression {
    use crate::build::build::tests::build_input;
    use garnish_lang_simple_data::{SimpleData, SimpleDataList, SimpleInstruction};
    use garnish_lang_traits::Instruction;

    #[test]
    fn subexpression() {
        let (data, build_data) = build_input("10\n\n20");

        assert_eq!(build_data.jump_index, 0);
        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::UpdateValue, None),
                SimpleInstruction::new(Instruction::Put, Some(4)),
                SimpleInstruction::new(Instruction::EndExpression, None),
            ]
        );
        assert_eq!(data.get_jump_points(), &vec![0]);
        assert_eq!(
            data.get_data(),
            &SimpleDataList::default()
                .append(SimpleData::Number(10.into()))
                .append(SimpleData::Number(20.into()))
        );
    }
}

#[cfg(test)]
mod groups {
    use garnish_lang_simple_data::{SimpleData, SimpleDataList, SimpleInstruction};
    use garnish_lang_traits::Instruction;
    use crate::build::build::tests::build_input;

    #[test]
    fn groups() {
        let (data, build_data) = build_input("10 (5 + 20)");

        assert_eq!(build_data.jump_index, 0);
        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::Put, Some(4)),
                SimpleInstruction::new(Instruction::Put, Some(5)),
                SimpleInstruction::new(Instruction::Add, None),
                SimpleInstruction::new(Instruction::MakeList, Some(2)),
                SimpleInstruction::new(Instruction::EndExpression, None),
            ]
        );
        assert_eq!(data.get_jump_points(), &vec![0]);
        assert_eq!(
            data.get_data(),
            &SimpleDataList::default()
                .append(SimpleData::Number(10.into()))
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(20.into()))
        );
    }
}

#[cfg(test)]
mod side_effects {
    use garnish_lang_simple_data::{SimpleData, SimpleDataList, SimpleInstruction};
    use garnish_lang_traits::Instruction;
    use crate::build::build::tests::build_input;

    #[test]
    fn side_effect() {
        let (data, build_data) = build_input("10 [ 5 + 15 ] 20");

        assert_eq!(build_data.jump_index, 0);
        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::StartSideEffect, None),
                SimpleInstruction::new(Instruction::Put, Some(4)),
                SimpleInstruction::new(Instruction::Put, Some(5)),
                SimpleInstruction::new(Instruction::Add, None),
                SimpleInstruction::new(Instruction::EndSideEffect, None),
                SimpleInstruction::new(Instruction::Put, Some(6)),
                SimpleInstruction::new(Instruction::MakeList, Some(2)),
                SimpleInstruction::new(Instruction::EndExpression, None),
            ]
        );
        assert_eq!(data.get_jump_points(), &vec![0]);
        assert_eq!(
            data.get_data(),
            &SimpleDataList::default()
                .append(SimpleData::Number(10.into()))
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(15.into()))
                .append(SimpleData::Number(20.into()))
        );
    }
}