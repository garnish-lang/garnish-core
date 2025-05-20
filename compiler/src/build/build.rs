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

#[derive(Debug, Clone)]
pub struct BuildData<Data: GarnishData> {
    parse_root: usize,
    parse_tree: Vec<ParseNode>,
    instruction_metadata: Vec<InstructionMetadata>,
    jump_index: Data::Size,
}

impl<Data: GarnishData> BuildData<Data> {
    pub fn new(parse_root: usize, parse_tree: Vec<ParseNode>, jump_index: Data::Size, instruction_metadata: Vec<InstructionMetadata>) -> Self {
        Self {
            parse_root,
            parse_tree,
            instruction_metadata,
            jump_index,
        }
    }
    
    pub fn parse_root(&self) -> usize {
        self.parse_root
    }
    
    pub fn parse_tree(&self) -> &Vec<ParseNode> {
        &self.parse_tree
    }
    
    pub fn instruction_metadata(&self) -> &Vec<InstructionMetadata> {
        &self.instruction_metadata
    }
    
    pub fn jump_index(&self) -> &Data::Size {
        &self.jump_index
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ConditionItem<Data: GarnishData> {
    node_index: usize,
    jump_index_to_update: Data::Size,
    root_end_instruction: (Instruction, Option<Data::Size>),
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
    list_parent: Option<(usize, Definition)>,
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

    fn new_with_list(parse_node_index: usize, list_parent: usize, list_parent_definition: Definition) -> Self {
        Self {
            state: BuildNodeState::Uninitialized,
            parse_node_index,
            list_parent: Some((list_parent, list_parent_definition)),
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

pub fn build<Data: GarnishData>(parse_root: usize, parse_tree: Vec<ParseNode>, data: &mut Data) -> Result<BuildData<Data>, CompilerError<Data::Error>> {
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
        let current_root_jump = match nodes.get(root_index) {
            Some(Some(node)) => match &node.jump_index_to_update {
                Some(index) => {
                    let jump_index = data.get_instruction_len();
                    match data.get_jump_point_mut(index.clone()) {
                        Some(item) => *item = jump_index,
                        None => Err(CompilerError::new_message(format!("No jump point at {} when pulling from root stack", index)))?,
                    }

                    index.clone()
                }
                None => {
                    let index = data.get_jump_table_len();
                    data.push_jump_point(data.get_instruction_len())?;
                    index
                }
            },
            _ => {
                let index = data.get_jump_table_len();
                data.push_jump_point(data.get_instruction_len())?;
                index
            }
        };
        let mut stack = vec![root_index];

        while let Some(node_index) = stack.pop() {
            let parse_node = match parse_tree.get(node_index) {
                Some(node) => node,
                None => Err(CompilerError::new_message(format!("No parse node at index {}", node_index)))?,
            };

            handle_parse_node(
                data,
                &mut nodes,
                &mut instruction_metadata,
                &mut root_stack,
                current_root_jump.clone(),
                &mut stack,
                node_index,
                parse_node,
            )?;

            match nodes.get_mut(node_index) {
                Some(Some(node)) if node.contributes_to_list => match node.list_parent {
                    Some((parent, _)) => {
                        node.contributes_to_list = false;

                        let parent_node = nodes.get_mut_or_error(parent)?;
                        parent_node.child_count.next();
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

    Ok(BuildData::new(parse_root, parse_tree, tree_root_jump, instruction_metadata))
}

fn handle_parse_node<Data: GarnishData>(
    data: &mut Data,
    mut nodes: &mut Vec<Option<BuildNode<Data>>>,
    mut instruction_metadata: &mut Vec<InstructionMetadata>,
    mut root_stack: &mut Vec<usize>,
    current_root_jump: <Data as GarnishData>::Size,
    mut stack: &mut Vec<usize>,
    node_index: usize,
    parse_node: &ParseNode,
) -> Result<(), CompilerError<<Data as GarnishData>::Error>> {
    match parse_node.get_definition() {
        Definition::Unit => handle_value_primitive(|data, _| data.add_unit(), &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::False => handle_value_primitive(|data, _| data.add_false(), &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::True => handle_value_primitive(|data, _| data.add_true(), &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::Number => handle_value_primitive(
            |data, node| data.parse_add_number(node.text()),
            &mut nodes,
            node_index,
            &mut stack,
            parse_node,
            data,
            &mut instruction_metadata,
        )?,
        Definition::CharList => handle_value_primitive(
            |data, node| data.parse_add_char_list(node.text()),
            &mut nodes,
            node_index,
            &mut stack,
            parse_node,
            data,
            &mut instruction_metadata,
        )?,
        Definition::ByteList => handle_value_primitive(
            |data, node| data.parse_add_byte_list(node.text()),
            &mut nodes,
            node_index,
            &mut stack,
            parse_node,
            data,
            &mut instruction_metadata,
        )?,
        Definition::Symbol => handle_value_primitive(
            |data, node| data.parse_add_symbol(&node.text()[1..]),
            &mut nodes,
            node_index,
            &mut stack,
            parse_node,
            data,
            &mut instruction_metadata,
        )?,
        Definition::Value => handle_value_like(|_, _| Ok(None), Instruction::PutValue, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::Identifier => handle_value_like(
            |data, node| Ok(Some(data.parse_add_symbol(node.text())?)),
            Instruction::Resolve,
            &mut nodes,
            node_index,
            &mut stack,
            parse_node,
            data,
            &mut instruction_metadata,
        )?,
        Definition::Property => handle_value_like(
            |data, node| Ok(Some(data.parse_add_symbol(node.text())?)),
            Instruction::Put,
            &mut nodes,
            node_index,
            &mut stack,
            parse_node,
            data,
            &mut instruction_metadata,
        )?,
        Definition::ExpressionTerminator => handle_value_like(
            |_, _| Ok(None),
            Instruction::EndExpression,
            &mut nodes,
            node_index,
            &mut stack,
            parse_node,
            data,
            &mut instruction_metadata,
        )?,
        Definition::AbsoluteValue => handle_unary_prefix(Instruction::AbsoluteValue, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::Opposite => handle_unary_prefix(Instruction::Opposite, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::BitwiseNot => handle_unary_prefix(Instruction::BitwiseNot, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::Not => handle_unary_prefix(Instruction::Not, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::Tis => handle_unary_prefix(Instruction::Tis, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::TypeOf => handle_unary_prefix(Instruction::TypeOf, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::AccessLeftInternal => handle_unary_prefix(Instruction::AccessLeftInternal, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::EmptyApply => handle_unary_suffix(Instruction::EmptyApply, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::AccessRightInternal => handle_unary_suffix(Instruction::AccessRightInternal, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::AccessLengthInternal => handle_unary_suffix(Instruction::AccessLengthInternal, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::Addition => handle_binary_operation(Instruction::Add, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::Subtraction => handle_binary_operation(Instruction::Subtract, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::MultiplicationSign => handle_binary_operation(Instruction::Multiply, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::Division => handle_binary_operation(Instruction::Divide, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::Access => handle_binary_operation(Instruction::Access, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::Pair => handle_binary_operation(Instruction::MakePair, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::Range => handle_binary_operation(Instruction::MakeRange, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::StartExclusiveRange => handle_binary_operation(Instruction::MakeStartExclusiveRange, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::EndExclusiveRange => handle_binary_operation(Instruction::MakeEndExclusiveRange, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::ExclusiveRange => handle_binary_operation(Instruction::MakeExclusiveRange, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::ExponentialSign => handle_binary_operation(Instruction::Power, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::Remainder => handle_binary_operation(Instruction::Remainder, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::IntegerDivision => handle_binary_operation(Instruction::IntegerDivide, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::BitwiseAnd => handle_binary_operation(Instruction::BitwiseAnd, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::BitwiseOr => handle_binary_operation(Instruction::BitwiseOr, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::BitwiseXor => handle_binary_operation(Instruction::BitwiseXor, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::BitwiseRightShift => handle_binary_operation(Instruction::BitwiseShiftRight, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::BitwiseLeftShift => handle_binary_operation(Instruction::BitwiseShiftLeft, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::Xor => handle_binary_operation(Instruction::Xor, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::TypeEqual => handle_binary_operation(Instruction::TypeEqual, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::TypeCast => handle_binary_operation(Instruction::ApplyType, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::Equality => handle_binary_operation(Instruction::Equal, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::Inequality => handle_binary_operation(Instruction::NotEqual, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::LessThan => handle_binary_operation(Instruction::LessThan, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::LessThanOrEqual => handle_binary_operation(Instruction::LessThanOrEqual, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::GreaterThan => handle_binary_operation(Instruction::GreaterThan, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::GreaterThanOrEqual => handle_binary_operation(Instruction::GreaterThanOrEqual, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::Apply => handle_binary_operation(Instruction::Apply, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::Concatenation => handle_binary_operation(Instruction::Concat, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::ApplyTo => handle_binary_operation_with_push(Instruction::Apply, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata, |left, right| {
            (left, right)
        })?,
        Definition::CommaList => handle_list(Definition::CommaList, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::List => handle_list(Definition::List, &mut nodes, node_index, &mut stack, parse_node, data, &mut instruction_metadata)?,
        Definition::Or => handle_logical_binary(Instruction::Or, &mut nodes, node_index, &mut stack, &mut root_stack, parse_node, data, &mut instruction_metadata)?,
        Definition::And => handle_logical_binary(Instruction::And, &mut nodes, node_index, &mut stack, &mut root_stack, parse_node, data, &mut instruction_metadata)?,
        Definition::Group => {
            let right = parse_node.get_right().ok_or(CompilerError::new_message("No right on NestedExpression definition".to_string()))?;

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

                    let right = parse_node.get_right().ok_or(CompilerError::new_message("No right on SideEffect definition".to_string()))?;

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

            let right = parse_node.get_right().ok_or(CompilerError::new_message("No right on NestedExpression definition".to_string()))?;

            nodes[right] = Some(BuildNode::new_with_jump(right, jump_index));

            root_stack.push(right);
        }
        Definition::JumpIfFalse => handle_jump_if(
            Instruction::JumpIfFalse,
            &mut nodes,
            node_index,
            &mut stack,
            &mut root_stack,
            parse_node,
            data,
            &mut instruction_metadata,
        )?,
        Definition::JumpIfTrue => handle_jump_if(
            Instruction::JumpIfTrue,
            &mut nodes,
            node_index,
            &mut stack,
            &mut root_stack,
            parse_node,
            data,
            &mut instruction_metadata,
        )?,
        Definition::ElseJump => {
            let node = match nodes.get_mut(node_index) {
                Some(Some(node)) => node,
                _ => Err(CompilerError::new_message(format!("No build node at index {}", node_index)))?,
            };

            match node.state {
                BuildNodeState::Uninitialized => {
                    node.state = BuildNodeState::Initialized;

                    stack.push(node.parse_node_index);
                    let right = parse_node.get_right().ok_or(CompilerError::new_message("No right on ElseJump definition".to_string()))?;
                    let left = parse_node.get_left().ok_or(CompilerError::new_message("No left on ElseJump definition".to_string()))?;
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
                        if node.conditional_items.len() > 0 {
                            let mut new_items: Vec<(usize, BuildNode<Data>)> = vec![];

                            let jump_to_index = data.get_jump_table_len();
                            data.push_jump_point(data.get_instruction_len())?;

                            for condition in &node.conditional_items {
                                root_stack.push(condition.node_index);

                                new_items.push((
                                    condition.node_index,
                                    BuildNode::new_with_jump_and_end(condition.node_index, condition.jump_index_to_update.clone(), vec![(Instruction::JumpTo, Some(jump_to_index.clone()))]),
                                ));
                            }

                            for (index, data) in new_items {
                                nodes[index] = Some(data);
                            }
                        }
                    }
                },
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
                    let left = parse_node.get_left().ok_or(CompilerError::new_message("No left on Reapply definition".to_string()))?;
                    stack.push(left);

                    nodes[left] = Some(BuildNode::new_with_conditional(left, node_index));
                }
                BuildNodeState::Initialized => {
                    let jump_index = data.get_jump_table_len();
                    data.push_jump_point(Data::Size::zero())?;
                    data.push_instruction(Instruction::JumpIfTrue, Some(jump_index.clone()))?;
                    instruction_metadata.push(InstructionMetadata::new(Some(node_index)));

                    match node.conditional_parent {
                        Some(_) => {}
                        None => {
                            data.push_instruction(Instruction::PutValue, None)?;
                            instruction_metadata.push(InstructionMetadata::new(None));
                        }
                    }

                    let right = parse_node.get_right().ok_or(CompilerError::new_message("No right on Reapply definition".to_string()))?;

                    root_stack.push(right);

                    nodes[right] = Some(BuildNode::new_with_jump_and_end(
                        right,
                        jump_index.clone(),
                        vec![(Instruction::UpdateValue, None), (Instruction::JumpTo, Some(current_root_jump.clone()))],
                    ));
                }
            }
        }
        Definition::Subexpression | Definition::ExpressionSeparator => {
            let node = match nodes.get_mut(node_index) {
                Some(Some(node)) => node,
                _ => Err(CompilerError::new_message(format!("No build node at index {}", node_index)))?,
            };
            match node.state {
                BuildNodeState::Uninitialized => {
                    node.state = BuildNodeState::Initialized;

                    let right = parse_node.get_right().ok_or(CompilerError::new_message("No right on Subexpression definition".to_string()))?;
                    let left = parse_node.get_left().ok_or(CompilerError::new_message("No left on Subexpression definition".to_string()))?;

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
        Definition::SuffixApply => handle_unary_fix_apply(
            parse_node.get_left(),
            Definition::SuffixApply,
            &mut nodes,
            node_index,
            &mut stack,
            parse_node,
            data,
            &mut instruction_metadata,
        )?,
        Definition::PrefixApply => handle_unary_fix_apply(
            parse_node.get_right(),
            Definition::PrefixApply,
            &mut nodes,
            node_index,
            &mut stack,
            parse_node,
            data,
            &mut instruction_metadata,
        )?,
        Definition::InfixApply => {
            let node = match nodes.get_mut(node_index) {
                Some(Some(node)) => node,
                _ => Err(CompilerError::new_message(format!("No build node at index {}", node_index)))?,
            };
            match node.state {
                BuildNodeState::Uninitialized => {
                    node.state = BuildNodeState::Initialized;

                    let addr = data.parse_add_symbol(parse_node.text().trim_matches('`'))?;

                    data.push_instruction(Instruction::Resolve, Some(addr))?;
                    instruction_metadata.push(InstructionMetadata::new(None));

                    let right = parse_node.get_right().ok_or(CompilerError::new_message("No right on InfixApply definition".to_string()))?;
                    let left = parse_node.get_left().ok_or(CompilerError::new_message("No left on InfixApply definition".to_string()))?;

                    stack.push(node_index);
                    stack.push(right);
                    stack.push(left);

                    nodes[right] = Some(BuildNode::new(right));
                    nodes[left] = Some(BuildNode::new(left));
                }
                BuildNodeState::Initialized => {
                    data.push_instruction(Instruction::MakeList, Some(Data::Size::one() + Data::Size::one()))?;
                    instruction_metadata.push(InstructionMetadata::new(None));
                    data.push_instruction(Instruction::Apply, None)?;
                    instruction_metadata.push(InstructionMetadata::new(Some(node_index)));
                }
            }
        }
        Definition::Drop => Err(CompilerError::new_message("Cannot build a Drop definition".to_string()))?,
    }
    Ok(())
}

fn handle_unary_fix_apply<Data: GarnishData>(
    child: Option<usize>,
    definition: Definition,
    nodes: &mut Vec<Option<BuildNode<Data>>>,
    node_index: usize,
    stack: &mut Vec<usize>,
    parse_node: &ParseNode,
    data: &mut Data,
    instruction_metadata: &mut Vec<InstructionMetadata>,
) -> Result<(), CompilerError<Data::Error>> {
    {
        let node = match nodes.get_mut(node_index) {
            Some(Some(node)) => node,
            _ => Err(CompilerError::new_message(format!("No build node at index {}", node_index)))?,
        };
        match node.state {
            BuildNodeState::Uninitialized => {
                node.state = BuildNodeState::Initialized;

                let addr = data.parse_add_symbol(parse_node.text().trim_matches('`'))?;

                data.push_instruction(Instruction::Resolve, Some(addr))?;
                instruction_metadata.push(InstructionMetadata::new(None));

                let right = child.ok_or(CompilerError::new_message(format!("No right on {:?} definition", definition)))?;

                stack.push(node_index);
                stack.push(right);

                nodes[right] = Some(BuildNode::new(right));
            }
            BuildNodeState::Initialized => {
                data.push_instruction(Instruction::Apply, None)?;
                instruction_metadata.push(InstructionMetadata::new(Some(node_index)));
            }
        }
    }

    Ok(())
}

fn handle_jump_if<Data: GarnishData>(
    instruction: Instruction,
    nodes: &mut Vec<Option<BuildNode<Data>>>,
    node_index: usize,
    stack: &mut Vec<usize>,
    root_stack: &mut Vec<usize>,
    parse_node: &ParseNode,
    data: &mut Data,
    instruction_metadata: &mut Vec<InstructionMetadata>,
) -> Result<(), CompilerError<Data::Error>> {
    let node = match nodes.get_mut(node_index) {
        Some(Some(node)) => node,
        _ => Err(CompilerError::new_message(format!("No build node at index {}", node_index)))?,
    };

    match node.state {
        BuildNodeState::Uninitialized => {
            node.state = BuildNodeState::Initialized;

            stack.push(node.parse_node_index);
            let left = parse_node.get_left().ok_or(CompilerError::new_message(format!("No left on {:?} definition", instruction)))?;
            stack.push(left);

            nodes[left] = Some(BuildNode::new(left));
        }
        BuildNodeState::Initialized => {
            let jump_index = data.get_jump_table_len();
            data.push_jump_point(Data::Size::zero())?;

            let right = parse_node.get_right().ok_or(CompilerError::new_message(format!("No right on {:?} definition", instruction)))?;
            match node.conditional_parent {
                Some(conditional_parent) => match nodes.get_mut(conditional_parent) {
                    Some(Some(parent)) => {
                        parent.conditional_items.push(ConditionItem {
                            node_index: right,
                            jump_index_to_update: jump_index.clone(),
                            root_end_instruction: (Instruction::Invalid, None),
                        });
                        data.push_instruction(instruction, Some(jump_index.clone()))?;
                        instruction_metadata.push(InstructionMetadata::new(Some(node_index)));
                    }
                    _ => {}
                },
                None => {
                    data.push_instruction(instruction, Some(jump_index.clone()))?;
                    instruction_metadata.push(InstructionMetadata::new(Some(node_index)));
                    data.push_instruction(Instruction::PutValue, None)?;
                    instruction_metadata.push(InstructionMetadata::new(None));

                    root_stack.push(right);

                    let jump_to_index = data.get_jump_table_len();
                    data.push_jump_point(data.get_instruction_len())?;

                    nodes[right] = Some(BuildNode::new_with_jump_and_end(right, jump_index.clone(), vec![(Instruction::JumpTo, Some(jump_to_index))]));
                }
            }
        }
    }

    Ok(())
}

fn handle_logical_binary<Data: GarnishData>(
    instruction: Instruction,
    nodes: &mut Vec<Option<BuildNode<Data>>>,
    node_index: usize,
    stack: &mut Vec<usize>,
    root_stack: &mut Vec<usize>,
    parse_node: &ParseNode,
    data: &mut Data,
    instruction_metadata: &mut Vec<InstructionMetadata>,
) -> Result<(), CompilerError<Data::Error>> {
    let node = match nodes.get_mut(node_index) {
        Some(Some(node)) => node,
        _ => Err(CompilerError::new_message(format!("No build node at index {}", node_index)))?,
    };

    match node.state {
        BuildNodeState::Uninitialized => {
            node.state = BuildNodeState::Initialized;

            stack.push(node.parse_node_index);
            let left = parse_node.get_left().ok_or(CompilerError::new_message(format!("No left on {:?} definition", instruction)))?;
            stack.push(left);

            nodes[left] = Some(BuildNode::new_with_conditional(left, node_index));
        }
        BuildNodeState::Initialized => {
            let jump_index = data.get_jump_table_len();
            data.push_jump_point(Data::Size::zero())?;
            data.push_instruction(instruction, Some(jump_index.clone()))?;
            instruction_metadata.push(InstructionMetadata::new(Some(node_index)));

            let right = parse_node.get_right().ok_or(CompilerError::new_message(format!("No right on {:?} definition", instruction)))?;

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

    Ok(())
}

fn handle_value_primitive<Data: GarnishData, Fn>(
    add_fn: Fn,
    nodes: &mut Vec<Option<BuildNode<Data>>>,
    node_index: usize,
    stack: &mut Vec<usize>,
    parse_node: &ParseNode,
    data: &mut Data,
    instruction_metadata: &mut Vec<InstructionMetadata>,
) -> Result<(), CompilerError<Data::Error>>
where
    Fn: FnOnce(&mut Data, &ParseNode) -> Result<Data::Size, Data::Error>,
{
    handle_value_like(
        |data, node| Ok(Some(add_fn(data, node)?)),
        Instruction::Put,
        nodes,
        node_index,
        stack,
        parse_node,
        data,
        instruction_metadata,
    )
}

fn handle_value_like<Data: GarnishData, Fn>(
    add_fn: Fn,
    instruction: Instruction,
    nodes: &mut Vec<Option<BuildNode<Data>>>,
    node_index: usize,
    stack: &mut Vec<usize>,
    parse_node: &ParseNode,
    data: &mut Data,
    instruction_metadata: &mut Vec<InstructionMetadata>,
) -> Result<(), CompilerError<Data::Error>>
where
    Fn: FnOnce(&mut Data, &ParseNode) -> Result<Option<Data::Size>, Data::Error>,
{
    let node = match nodes.get_mut(node_index) {
        Some(Some(node)) => node,
        _ => Err(CompilerError::new_message(format!("No build node at index {}", node_index)))?,
    };

    match node.state {
        BuildNodeState::Uninitialized => {
            node.state = BuildNodeState::Initialized;

            match parse_node.get_right() {
                None => {}
                Some(right) => {
                    stack.push(right);
                    nodes[right] = Some(BuildNode::new(right));
                }
            }

            stack.push(node_index);

            match parse_node.get_left() {
                None => {}
                Some(left) => {
                    stack.push(left);
                    nodes[left] = Some(BuildNode::new(left));
                }
            }
        }
        BuildNodeState::Initialized => {
            let instruction_data = add_fn(data, parse_node)?;
            data.push_instruction(instruction, instruction_data)?;
            instruction_metadata.push(InstructionMetadata::new(Some(node_index)));
        }
    }

    Ok(())
}

fn handle_unary_suffix<Data: GarnishData>(
    instruction: Instruction,
    nodes: &mut Vec<Option<BuildNode<Data>>>,
    node_index: usize,
    stack: &mut Vec<usize>,
    parse_node: &ParseNode,
    data: &mut Data,
    instruction_metadata: &mut Vec<InstructionMetadata>,
) -> Result<(), CompilerError<Data::Error>> {
    let node = nodes.get_mut_or_error(node_index)?;

    match node.state {
        BuildNodeState::Uninitialized => {
            node.state = BuildNodeState::Initialized;

            stack.push(node.parse_node_index);
            let left = parse_node.get_left().ok_or(CompilerError::new_message(format!("No left on {:?} definition", instruction)))?;
            stack.push(left);

            nodes[left] = Some(BuildNode::new(left));
        }
        BuildNodeState::Initialized => {
            data.push_instruction(instruction, None)?;
            instruction_metadata.push(InstructionMetadata::new(Some(node.parse_node_index)));
        }
    }

    Ok(())
}

fn handle_unary_prefix<Data: GarnishData>(
    instruction: Instruction,
    nodes: &mut Vec<Option<BuildNode<Data>>>,
    node_index: usize,
    stack: &mut Vec<usize>,
    parse_node: &ParseNode,
    data: &mut Data,
    instruction_metadata: &mut Vec<InstructionMetadata>,
) -> Result<(), CompilerError<Data::Error>> {
    let node = nodes.get_mut_or_error(node_index)?;

    match node.state {
        BuildNodeState::Uninitialized => {
            node.state = BuildNodeState::Initialized;

            stack.push(node.parse_node_index);
            let right = parse_node.get_right().ok_or(CompilerError::new_message(format!("No right on {:?} definition", instruction)))?;
            stack.push(right);

            nodes[right] = Some(BuildNode::new(right));
        }
        BuildNodeState::Initialized => {
            data.push_instruction(instruction, None)?;
            instruction_metadata.push(InstructionMetadata::new(Some(node.parse_node_index)));
        }
    }

    Ok(())
}

fn handle_binary_operation<Data: GarnishData>(
    instruction: Instruction,
    nodes: &mut Vec<Option<BuildNode<Data>>>,
    node_index: usize,
    stack: &mut Vec<usize>,
    parse_node: &ParseNode,
    data: &mut Data,
    instruction_metadata: &mut Vec<InstructionMetadata>,
) -> Result<(), CompilerError<Data::Error>> {
    handle_binary_operation_with_push(instruction, nodes, node_index, stack, parse_node, data, instruction_metadata, |left, right| (right, left))
}

fn handle_binary_operation_with_push<Data: GarnishData, Fn>(
    instruction: Instruction,
    nodes: &mut Vec<Option<BuildNode<Data>>>,
    node_index: usize,
    stack: &mut Vec<usize>,
    parse_node: &ParseNode,
    data: &mut Data,
    instruction_metadata: &mut Vec<InstructionMetadata>,
    order_fn: Fn,
) -> Result<(), CompilerError<Data::Error>>
where
    Fn: FnOnce(usize, usize) -> (usize, usize),
{
    let node = match nodes.get_mut(node_index) {
        Some(Some(node)) => node,
        _ => Err(CompilerError::new_message(format!("No build node at index {}", node_index)))?,
    };

    match node.state {
        BuildNodeState::Uninitialized => {
            node.state = BuildNodeState::Initialized;

            stack.push(node.parse_node_index);
            let right = parse_node.get_right().ok_or(CompilerError::new_message(format!("No right on {:?} definition", instruction)))?;
            let left = parse_node.get_left().ok_or(CompilerError::new_message(format!("No left on {:?} definition", instruction)))?;

            let (first, second) = order_fn(left, right);
            stack.push(first);
            stack.push(second);

            nodes[right] = Some(BuildNode::new(right));
            nodes[left] = Some(BuildNode::new(left));
        }
        BuildNodeState::Initialized => {
            data.push_instruction(instruction, None)?;
            instruction_metadata.push(InstructionMetadata::new(Some(node.parse_node_index)));
        }
    }

    Ok(())
}

fn handle_list<Data: GarnishData>(
    definition: Definition,
    nodes: &mut Vec<Option<BuildNode<Data>>>,
    node_index: usize,
    stack: &mut Vec<usize>,
    parse_node: &ParseNode,
    data: &mut Data,
    instruction_metadata: &mut Vec<InstructionMetadata>,
) -> Result<(), CompilerError<Data::Error>> {
    let node = match nodes.get_mut(node_index) {
        Some(Some(node)) => node,
        _ => Err(CompilerError::new_message(format!("No build node at index {}", node_index)))?,
    };

    match node.state {
        BuildNodeState::Uninitialized => {
            node.state = BuildNodeState::Initialized;

            stack.push(node.parse_node_index);

            let (parent, definition, contributes_to_list) = match node.list_parent {
                Some((parent, definition)) if definition == parse_node.get_definition() => (parent, definition, false),
                _ => (node_index, parse_node.get_definition(), true),
            };
            node.contributes_to_list = contributes_to_list;

            match parse_node.get_right() {
                None => {}
                Some(right) => {
                    stack.push(right);
                    nodes[right] = Some(BuildNode::new_with_list(right, parent, definition));
                }
            }
            match parse_node.get_left() {
                None => {}
                Some(left) => {
                    stack.push(left);
                    nodes[left] = Some(BuildNode::new_with_list(left, parent, definition));
                }
            }
        }
        BuildNodeState::Initialized => match node.list_parent {
            Some((_parent, definition)) if definition == parse_node.get_definition() => {}
            _ => {
                let node = nodes.get_mut_or_error(node_index)?;

                let count = node
                    .child_count
                    .next()
                    .ok_or(CompilerError::new_message(format!("Failed to increment child count for {:?}", definition)))?;

                data.push_instruction(Instruction::MakeList, Some(count))?;
                instruction_metadata.push(InstructionMetadata::new(Some(node.parse_node_index)));
            }
        },
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::build::build::build;
    use crate::build::{BuildData, InstructionMetadata};
    use crate::lex::{LexerToken, TokenType, lex};
    use crate::parse::{Definition, ParseNode, SecondaryDefinition, parse};
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

    #[test]
    fn build_drop_is_error() {
        let mut data = SimpleGarnishData::new();
        let result = build(
            0,
            vec![ParseNode::new(
                Definition::Drop,
                SecondaryDefinition::None,
                None,
                None,
                None,
                LexerToken::new("".to_string(), TokenType::Unknown, 0, 0),
            )],
            &mut data,
        );

        assert!(result.is_err());
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
            &vec![SimpleInstruction::new(Instruction::Put, Some(0)), SimpleInstruction::new(Instruction::EndExpression, None)]
        );

        assert_eq!(data.get_data(), &SimpleDataList::default());
        assert_eq!(build_data.instruction_metadata, vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)])
    }

    #[test]
    fn build_false() {
        let (data, build_data) = build_input("$!");

        assert_eq!(
            data.get_instructions(),
            &vec![SimpleInstruction::new(Instruction::Put, Some(1)), SimpleInstruction::new(Instruction::EndExpression, None)]
        );

        assert_eq!(data.get_data(), &SimpleDataList::default());
        assert_eq!(build_data.instruction_metadata, vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)])
    }

    #[test]
    fn build_true() {
        let (data, build_data) = build_input("$?");

        assert_eq!(
            data.get_instructions(),
            &vec![SimpleInstruction::new(Instruction::Put, Some(2)), SimpleInstruction::new(Instruction::EndExpression, None)]
        );

        assert_eq!(data.get_data(), &SimpleDataList::default());
        assert_eq!(build_data.instruction_metadata, vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)])
    }

    #[test]
    fn build_number() {
        let (data, build_data) = build_input("5");

        assert_eq!(
            data.get_instructions(),
            &vec![SimpleInstruction::new(Instruction::Put, Some(3)), SimpleInstruction::new(Instruction::EndExpression, None)]
        );

        assert_eq!(data.get_data(), &SimpleDataList::default().append(SimpleData::Number(5.into())));
        assert_eq!(build_data.instruction_metadata, vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)])
    }

    #[test]
    fn build_character_list() {
        let (data, build_data) = build_input("\"characters\"");

        assert_eq!(
            data.get_instructions(),
            &vec![SimpleInstruction::new(Instruction::Put, Some(3)), SimpleInstruction::new(Instruction::EndExpression, None)]
        );

        assert_eq!(data.get_data(), &SimpleDataList::default().append(SimpleData::CharList("characters".into())));
        assert_eq!(build_data.instruction_metadata, vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)])
    }

    #[test]
    fn build_symbol() {
        let (data, build_data) = build_input(":my_symbol");

        assert_eq!(
            data.get_instructions(),
            &vec![SimpleInstruction::new(Instruction::Put, Some(3)), SimpleInstruction::new(Instruction::EndExpression, None)]
        );

        assert_eq!(data.get_data(), &SimpleDataList::default().append_symbol("my_symbol"));
        assert_eq!(build_data.instruction_metadata, vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)])
    }

    #[test]
    fn build_empty_symbol() {
        let (data, build_data) = build_input(":");

        assert_eq!(
            data.get_instructions(),
            &vec![SimpleInstruction::new(Instruction::Put, Some(3)), SimpleInstruction::new(Instruction::EndExpression, None)]
        );

        assert_eq!(data.get_data(), &SimpleDataList::default().append_symbol(""));
        assert_eq!(build_data.instruction_metadata, vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)])
    }

    #[test]
    fn build_byte_list() {
        let (data, build_data) = build_input("'abc'");

        assert_eq!(
            data.get_instructions(),
            &vec![SimpleInstruction::new(Instruction::Put, Some(3)), SimpleInstruction::new(Instruction::EndExpression, None)]
        );

        assert_eq!(data.get_data(), &SimpleDataList::default().append(SimpleData::ByteList(vec!['a' as u8, 'b' as u8, 'c' as u8])));
        assert_eq!(build_data.instruction_metadata, vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)])
    }

    #[test]
    fn build_value() {
        let (data, build_data) = build_input("$");

        assert_eq!(
            data.get_instructions(),
            &vec![SimpleInstruction::new(Instruction::PutValue, None), SimpleInstruction::new(Instruction::EndExpression, None)]
        );
        assert_eq!(data.get_data(), &SimpleDataList::default());
        assert_eq!(build_data.instruction_metadata, vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)])
    }

    #[test]
    fn build_identifier() {
        let (data, build_data) = build_input("my_value");

        assert_eq!(
            data.get_instructions(),
            &vec![SimpleInstruction::new(Instruction::Resolve, Some(3)), SimpleInstruction::new(Instruction::EndExpression, None)]
        );
        assert_eq!(data.get_data(), &SimpleDataList::default().append_symbol("my_value"));
        assert_eq!(build_data.instruction_metadata, vec![InstructionMetadata::new(Some(0)), InstructionMetadata::new(None)])
    }
}

#[cfg(test)]
mod binary_operations {
    use crate::build::build::tests::build_input;
    use garnish_lang_simple_data::{SimpleData, SimpleDataList, SimpleInstruction};
    use garnish_lang_traits::Instruction;

    macro_rules! binary_tests {
        ($($name:ident: $input:expr, $instruction:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (data, _build_data) = build_input($input);

                    assert_eq!(
                        data.get_instructions(),
                        &vec![
                            SimpleInstruction::new(Instruction::Resolve, Some(3)),
                            SimpleInstruction::new(Instruction::Resolve, Some(4)),
                            SimpleInstruction::new($instruction, None),
                            SimpleInstruction::new(Instruction::EndExpression, None)
                        ]
                    );
                    assert_eq!(data.get_data(), &SimpleDataList::default().append_symbol("value1").append_symbol("value2"));
                }
            )*
        }
    }

    binary_tests! {
        addition: "value1 + value2", Instruction::Add,
        subtraction: "value1 - value2", Instruction::Subtract,
        multiplication: "value1 * value2", Instruction::Multiply,
        division: "value1 / value2", Instruction::Divide,
        pair: "value1 = value2", Instruction::MakePair,
        range: "value1..value2", Instruction::MakeRange,
        start_exclusive_range: "value1>..value2", Instruction::MakeStartExclusiveRange,
        end_exclusive_range: "value1..<value2", Instruction::MakeEndExclusiveRange,
        exclusive_range: "value1>..<value2", Instruction::MakeExclusiveRange,
        exponential: "value1 ** value2", Instruction::Power,
        remainder: "value1 % value2", Instruction::Remainder,
        integer_division: "value1 // value2", Instruction::IntegerDivide,
        bitwise_and: "value1 & value2", Instruction::BitwiseAnd,
        bitwise_or: "value1 | value2", Instruction::BitwiseOr,
        bitwise_xor: "value1 ^ value2", Instruction::BitwiseXor,
        bitwise_shift_right: "value1 >> value2", Instruction::BitwiseShiftRight,
        bitwise_shift_left: "value1 << value2", Instruction::BitwiseShiftLeft,
        logical_xor: "value1 ^^ value2", Instruction::Xor,
        type_cast: "value1 ~# value2", Instruction::ApplyType,
        type_equal: "value1 #= value2", Instruction::TypeEqual,
        equality: "value1 == value2", Instruction::Equal,
        inequality: "value1 != value2", Instruction::NotEqual,
        less_than: "value1 < value2", Instruction::LessThan,
        less_than_or_equal: "value1 <= value2", Instruction::LessThanOrEqual,
        greater_than: "value1 > value2", Instruction::GreaterThan,
        greater_than_or_equal: "value1 >= value2", Instruction::GreaterThanOrEqual,
        apply: "value1 ~ value2", Instruction::Apply,
        concatenation: "value1 <> value2", Instruction::Concat,
    }

    #[test]
    fn apply_to() {
        let (data, _build_data) = build_input("5 ~> 10");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::Put, Some(4)),
                SimpleInstruction::new(Instruction::Apply, None),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );
        assert_eq!(data.get_data(), &SimpleDataList::default().append(SimpleData::Number(10.into())).append(SimpleData::Number(5.into())));
    }

    #[test]
    fn access() {
        let (data, _build_data) = build_input("my_value.my_property");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Resolve, Some(3)),
                SimpleInstruction::new(Instruction::Put, Some(4)),
                SimpleInstruction::new(Instruction::Access, None),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );
        assert_eq!(data.get_data(), &SimpleDataList::default().append_symbol("my_value").append_symbol("my_property"));
    }

    #[test]
    fn infix_apply() {
        let (data, _build_data) = build_input("5`value`10");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Resolve, Some(3)),
                SimpleInstruction::new(Instruction::Put, Some(4)),
                SimpleInstruction::new(Instruction::Put, Some(5)),
                SimpleInstruction::new(Instruction::MakeList, Some(2)),
                SimpleInstruction::new(Instruction::Apply, None),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );
        assert_eq!(
            data.get_data(),
            &SimpleDataList::default()
                .append_symbol("value")
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into()))
        );
    }
}

#[cfg(test)]
mod unary_operations {
    use crate::build::build::tests::build_input;
    use garnish_lang_simple_data::{SimpleData, SimpleDataList, SimpleInstruction};
    use garnish_lang_traits::Instruction;

    macro_rules! unary_tests {
        ($($name:ident: $input:expr, $instruction:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (data, _build_data) = build_input($input);

                    assert_eq!(
                        data.get_instructions(),
                        &vec![
                            SimpleInstruction::new(Instruction::Resolve, Some(3)),
                            SimpleInstruction::new($instruction, None),
                            SimpleInstruction::new(Instruction::EndExpression, None)
                        ],
                    );
                    assert_eq!(data.get_data(), &SimpleDataList::default().append_symbol("value"));
                }
            )*
        };
    }

    unary_tests! {
        absolute_value: "++value", Instruction::AbsoluteValue,
        opposite: "--value", Instruction::Opposite,
        bitwise_not: "!value", Instruction::BitwiseNot,
        not: "!!value", Instruction::Not,
        tis: "??value", Instruction::Tis,
        type_of: "#value", Instruction::TypeOf,
        left_internal: "_.value", Instruction::AccessLeftInternal,
        empty_apply: "value~~", Instruction::EmptyApply,
        right_internal: "value._", Instruction::AccessRightInternal,
        length_internal: "value.|", Instruction::AccessLengthInternal,
    }

    #[test]
    fn suffix_apply() {
        let (data, _build_data) = build_input("5`value");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Resolve, Some(3)),
                SimpleInstruction::new(Instruction::Put, Some(4)),
                SimpleInstruction::new(Instruction::Apply, None),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );
        assert_eq!(data.get_data(), &SimpleDataList::default().append_symbol("value").append(SimpleData::Number(5.into())));
    }

    #[test]
    fn prefix_apply() {
        let (data, _build_data) = build_input("value`5");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Resolve, Some(3)),
                SimpleInstruction::new(Instruction::Put, Some(4)),
                SimpleInstruction::new(Instruction::Apply, None),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );
        assert_eq!(data.get_data(), &SimpleDataList::default().append_symbol("value").append(SimpleData::Number(5.into())));
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

    #[test]
    fn list_with_operations() {
        let (data, _build_data) = build_input("5 + 10 20 / 30");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::Put, Some(4)),
                SimpleInstruction::new(Instruction::Add, None),
                SimpleInstruction::new(Instruction::Put, Some(5)),
                SimpleInstruction::new(Instruction::Put, Some(6)),
                SimpleInstruction::new(Instruction::Divide, None),
                SimpleInstruction::new(Instruction::MakeList, Some(2)),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );
        assert_eq!(
            data.get_data(),
            &SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(10.into()))
                .append(SimpleData::Number(20.into()))
                .append(SimpleData::Number(30.into()))
        );
    }

    #[test]
    fn empty_list() {
        let (data, _build_data) = build_input("(,)");

        assert_eq!(
            data.get_instructions(),
            &vec![SimpleInstruction::new(Instruction::MakeList, Some(0)), SimpleInstruction::new(Instruction::EndExpression, None)]
        );
        assert_eq!(data.get_data(), &SimpleDataList::default());
    }

    #[test]
    fn empty_left() {
        let (data, _build_data) = build_input("(,5)");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::MakeList, Some(1)),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );
        assert_eq!(data.get_data(), &SimpleDataList::default().append(SimpleData::Number(5.into())));
    }

    #[test]
    fn empty_right() {
        let (data, _build_data) = build_input("(5,)");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::MakeList, Some(1)),
                SimpleInstruction::new(Instruction::EndExpression, None)
            ]
        );
        assert_eq!(data.get_data(), &SimpleDataList::default().append(SimpleData::Number(5.into())));
    }

    #[test]
    fn build_comma_list() {
        let (data, build_data) = build_input("5, 10, 15, 20, 25");

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

    #[test]
    fn nested_lists() {
        let (data, _build_data) = build_input("5, 10 15 20, 25");

        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::Put, Some(4)),
                SimpleInstruction::new(Instruction::Put, Some(5)),
                SimpleInstruction::new(Instruction::Put, Some(6)),
                SimpleInstruction::new(Instruction::MakeList, Some(3)),
                SimpleInstruction::new(Instruction::Put, Some(7)),
                SimpleInstruction::new(Instruction::MakeList, Some(3)),
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
                SimpleInstruction::new(Instruction::PutValue, None),
                SimpleInstruction::new(Instruction::EndExpression, None),
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::JumpTo, Some(2)),
            ]
        );
        assert_eq!(data.get_jump_points(), &vec![0, 4, 3]);
        assert_eq!(data.get_data(), &SimpleDataList::default().append(SimpleData::Number(10.into())));
        assert_eq!(
            build_data.instruction_metadata,
            vec![
                InstructionMetadata::new(Some(0)),
                InstructionMetadata::new(Some(1)),
                InstructionMetadata::new(None),
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
        assert_eq!(data.get_data(), &SimpleDataList::default().append(SimpleData::Number(20.into())).append(SimpleData::Number(10.into())));
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

    #[test]
    fn jump_if_false() {
        let (data, build_data) = build_input("$? !> 10");

        assert_eq!(build_data.jump_index, 0);
        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(2)),
                SimpleInstruction::new(Instruction::JumpIfFalse, Some(1)),
                SimpleInstruction::new(Instruction::PutValue, None),
                SimpleInstruction::new(Instruction::EndExpression, None),
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::JumpTo, Some(2)),
            ]
        );
        assert_eq!(data.get_jump_points(), &vec![0, 4, 3]);
        assert_eq!(data.get_data(), &SimpleDataList::default().append(SimpleData::Number(10.into())));
        assert_eq!(
            build_data.instruction_metadata,
            vec![
                InstructionMetadata::new(Some(0)),
                InstructionMetadata::new(Some(1)),
                InstructionMetadata::new(None),
                InstructionMetadata::new(None),
                InstructionMetadata::new(Some(2)),
                InstructionMetadata::new(None),
            ]
        )
    }

    #[test]
    fn jump_if_false_with_else() {
        let (data, build_data) = build_input("$? !> 10 |> 20");

        assert_eq!(build_data.jump_index, 0);
        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(2)),
                SimpleInstruction::new(Instruction::JumpIfFalse, Some(1)),
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::EndExpression, None),
                SimpleInstruction::new(Instruction::Put, Some(4)),
                SimpleInstruction::new(Instruction::JumpTo, Some(2)),
            ]
        );
        assert_eq!(data.get_jump_points(), &vec![0, 4, 3]);
        assert_eq!(data.get_data(), &SimpleDataList::default().append(SimpleData::Number(20.into())).append(SimpleData::Number(10.into())));
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
    fn triple_jump_if_false_with_else() {
        let (data, build_data) = build_input("$? !> 10 |> $? !> 20 |> $? !> 30");

        assert_eq!(build_data.jump_index, 0);
        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(2)),
                SimpleInstruction::new(Instruction::JumpIfFalse, Some(1)),
                SimpleInstruction::new(Instruction::Put, Some(2)),
                SimpleInstruction::new(Instruction::JumpIfFalse, Some(2)),
                SimpleInstruction::new(Instruction::Put, Some(2)),
                SimpleInstruction::new(Instruction::JumpIfFalse, Some(3)),
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
        assert_eq!(data.get_data(), &SimpleDataList::default().append(SimpleData::Number(5.into())).append(SimpleData::Number(10.into())));
    }

    #[test]
    fn double_and() {
        let (data, build_data) = build_input("5 && 10 && 20");

        assert_eq!(build_data.jump_index, 0);
        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::And, Some(1)),
                SimpleInstruction::new(Instruction::And, Some(3)),        // 2
                SimpleInstruction::new(Instruction::EndExpression, None), // 3
                SimpleInstruction::new(Instruction::Put, Some(4)),        // 4
                SimpleInstruction::new(Instruction::Tis, None),
                SimpleInstruction::new(Instruction::JumpTo, Some(4)),
                SimpleInstruction::new(Instruction::Put, Some(5)), // 7
                SimpleInstruction::new(Instruction::Tis, None),
                SimpleInstruction::new(Instruction::JumpTo, Some(2)),
            ]
        );
        assert_eq!(data.get_jump_points(), &vec![0, 7, 2, 4, 3]);
        assert_eq!(
            data.get_data(),
            &SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(20.into()))
                .append(SimpleData::Number(10.into()))
        );
    }

    #[test]
    fn or() {
        let (data, build_data) = build_input("5 || 10");

        assert_eq!(build_data.jump_index, 0);
        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::Or, Some(1)),
                SimpleInstruction::new(Instruction::EndExpression, None),
                SimpleInstruction::new(Instruction::Put, Some(4)),
                SimpleInstruction::new(Instruction::Tis, None),
                SimpleInstruction::new(Instruction::JumpTo, Some(2)),
            ]
        );
        assert_eq!(data.get_jump_points(), &vec![0, 3, 2]);
        assert_eq!(data.get_data(), &SimpleDataList::default().append(SimpleData::Number(5.into())).append(SimpleData::Number(10.into())));
    }

    #[test]
    fn double_or() {
        let (data, build_data) = build_input("5 || 10 || 20");

        assert_eq!(build_data.jump_index, 0);
        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::Or, Some(1)),
                SimpleInstruction::new(Instruction::Or, Some(3)),         // 2
                SimpleInstruction::new(Instruction::EndExpression, None), // 3
                SimpleInstruction::new(Instruction::Put, Some(4)),        // 4
                SimpleInstruction::new(Instruction::Tis, None),
                SimpleInstruction::new(Instruction::JumpTo, Some(4)),
                SimpleInstruction::new(Instruction::Put, Some(5)), // 7
                SimpleInstruction::new(Instruction::Tis, None),
                SimpleInstruction::new(Instruction::JumpTo, Some(2)),
            ]
        );
        assert_eq!(data.get_jump_points(), &vec![0, 7, 2, 4, 3]);
        assert_eq!(
            data.get_data(),
            &SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Number(20.into()))
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
                SimpleInstruction::new(Instruction::JumpTo, Some(0)),
            ]
        );
        assert_eq!(data.get_jump_points(), &vec![0, 4]);
        assert_eq!(data.get_data(), &SimpleDataList::default().append(SimpleData::Number(40.into())));
    }

    #[test]
    fn reapply_with_else() {
        let (data, build_data) = build_input("$! ^~ 40 |> 50");

        assert_eq!(build_data.jump_index, 0);
        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(1)),
                SimpleInstruction::new(Instruction::JumpIfTrue, Some(1)),
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::EndExpression, None),
                SimpleInstruction::new(Instruction::Put, Some(4)),
                SimpleInstruction::new(Instruction::UpdateValue, None),
                SimpleInstruction::new(Instruction::JumpTo, Some(0)),
            ]
        );
        assert_eq!(data.get_jump_points(), &vec![0, 4]);
        assert_eq!(data.get_data(), &SimpleDataList::default().append(SimpleData::Number(50.into())).append(SimpleData::Number(40.into())));
    }

    #[test]
    fn nested_reapply() {
        let (data, build_data) = build_input("5 {$! ^~ 40 |> 50}");

        assert_eq!(build_data.jump_index, 0);
        assert_eq!(
            data.get_instructions(),
            &vec![
                SimpleInstruction::new(Instruction::Put, Some(3)),
                SimpleInstruction::new(Instruction::Put, Some(4)),
                SimpleInstruction::new(Instruction::MakeList, Some(2)),
                SimpleInstruction::new(Instruction::EndExpression, None),
                SimpleInstruction::new(Instruction::Put, Some(1)),
                SimpleInstruction::new(Instruction::JumpIfTrue, Some(2)),
                SimpleInstruction::new(Instruction::Put, Some(5)),
                SimpleInstruction::new(Instruction::EndExpression, None),
                SimpleInstruction::new(Instruction::Put, Some(6)),
                SimpleInstruction::new(Instruction::UpdateValue, None),
                SimpleInstruction::new(Instruction::JumpTo, Some(1)),
            ]
        );
        assert_eq!(data.get_jump_points(), &vec![0, 4, 8]);
        assert_eq!(
            data.get_data(),
            &SimpleDataList::default()
                .append(SimpleData::Number(5.into()))
                .append(SimpleData::Expression(1))
                .append(SimpleData::Number(50.into()))
                .append(SimpleData::Number(40.into()))
        );
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
        assert_eq!(data.get_data(), &SimpleDataList::default().append(SimpleData::Number(10.into())).append(SimpleData::Number(20.into())));
    }

    #[test]
    fn expression_separator() {
        let (data, build_data) = build_input("10 ; 20");

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
        assert_eq!(data.get_data(), &SimpleDataList::default().append(SimpleData::Number(10.into())).append(SimpleData::Number(20.into())));
    }
}

#[cfg(test)]
mod groups {
    use crate::build::build::tests::build_input;
    use garnish_lang_simple_data::{SimpleData, SimpleDataList, SimpleInstruction};
    use garnish_lang_traits::Instruction;

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
    use crate::build::build::tests::build_input;
    use garnish_lang_simple_data::{SimpleData, SimpleDataList, SimpleInstruction};
    use garnish_lang_traits::Instruction;

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
