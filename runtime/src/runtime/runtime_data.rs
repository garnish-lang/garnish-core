use crate::{ExpressionData, ExpressionDataType, Instruction, InstructionData};

pub trait GarnishLangRuntimeData {
    type Error;

    fn create_symbol(&self, sym: &str) -> u64;

    fn set_end_of_constant(&mut self, addr: usize) -> Result<(), Self::Error>;
    fn get_end_of_constant_data(&self) -> usize;

    fn remove_non_constant_data(&mut self) -> Result<(), Self::Error>;

    fn get_data_len(&self) -> usize;

    fn set_result(&mut self, result: Option<usize>) -> Result<(), Self::Error>;
    fn get_result(&self) -> Option<usize>;

    fn push_input_stack(&mut self, addr: usize) -> Result<(), Self::Error>;
    fn pop_input_stack(&mut self) -> Option<usize>;
    fn get_input(&self, index: usize) -> Option<usize>;
    fn get_input_count(&self) -> usize;
    fn get_current_input(&self) -> Option<usize>;

    fn get_data_type(&self, index: usize) -> Result<ExpressionDataType, Self::Error>;
    fn get_integer(&self, index: usize) -> Result<i64, Self::Error>;
    fn get_reference(&self, index: usize) -> Result<usize, Self::Error>;
    fn get_symbol(&self, index: usize) -> Result<u64, Self::Error>;
    fn get_expression(&self, index: usize) -> Result<usize, Self::Error>;
    fn get_external(&self, index: usize) -> Result<usize, Self::Error>;
    fn get_pair(&self, index: usize) -> Result<(usize, usize), Self::Error>;
    fn get_list_len(&self, index: usize) -> Result<usize, Self::Error>;
    fn get_list_item(&self, list_index: usize, item_index: usize) -> Result<usize, Self::Error>;
    fn get_list_associations_len(&self, index: usize) -> Result<usize, Self::Error>;
    fn get_list_association(&self, list_index: usize, item_index: usize) -> Result<usize, Self::Error>;

    fn add_integer(&mut self, value: i64) -> Result<usize, Self::Error>;
    fn add_symbol(&mut self, value: u64) -> Result<usize, Self::Error>;
    fn add_expression(&mut self, value: usize) -> Result<usize, Self::Error>;
    fn add_external(&mut self, value: usize) -> Result<usize, Self::Error>;
    fn add_pair(&mut self, value: (usize, usize)) -> Result<usize, Self::Error>;
    fn add_unit(&mut self) -> Result<usize, Self::Error>;
    fn add_true(&mut self) -> Result<usize, Self::Error>;
    fn add_false(&mut self) -> Result<usize, Self::Error>;

    fn add_list(&mut self, value: Vec<usize>, associations: Vec<usize>) -> Result<usize, Self::Error>;
    fn start_list(&mut self, len: usize) -> Result<(), Self::Error>;
    fn add_to_list(&mut self, addr: usize, is_associative: bool) -> Result<(), Self::Error>;
    fn end_list(&mut self) -> Result<usize, Self::Error>;
    fn get_list_item_with_symbol(&self, list_addr: usize, sym: u64) -> Result<Option<usize>, Self::Error>;

    fn push_register(&mut self, addr: usize) -> Result<(), Self::Error>;
    fn pop_register(&mut self) -> Option<usize>;

    fn push_instruction(&mut self, instruction: Instruction, data: Option<usize>) -> Result<(), Self::Error>;
    fn get_instruction(&self, index: usize) -> Option<&InstructionData>;
    fn set_instruction_cursor(&mut self, index: usize) -> Result<(), Self::Error>;
    fn advance_instruction_cursor(&mut self) -> Result<(), Self::Error>;
    fn get_current_instruction(&self) -> Option<&InstructionData>;
    fn get_instruction_cursor(&self) -> usize;
    fn get_instruction_len(&self) -> usize;

    fn push_jump_point(&mut self, index: usize) -> Result<(), Self::Error>;
    fn get_jump_point(&self, index: usize) -> Option<usize>;
    fn get_jump_point_mut(&mut self, index: usize) -> Option<&mut usize>;
    fn get_jump_point_count(&self) -> usize;

    fn push_jump_path(&mut self, index: usize) -> Result<(), Self::Error>;
    fn pop_jump_path(&mut self) -> Option<usize>;
    fn get_jump_path(&self, index: usize) -> Option<usize>;

    // maybe temporary
    fn add_data(&mut self, data: ExpressionData) -> Result<usize, Self::Error>;
}
