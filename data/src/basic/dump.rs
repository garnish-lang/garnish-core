use crate::{BasicDataCustom, BasicGarnishData, basic::storage::StorageBlock, basic::companion::BasicDataCompanion};

impl<T, Companion> BasicGarnishData<T, Companion> 
where 
    T: BasicDataCustom,
    Companion: BasicDataCompanion<T>,
{
    pub fn dump_instruction_block(&self) -> String {
        self.dump_block(self.instruction_block())
    }

    pub fn dump_jump_table_block(&self) -> String {
        self.dump_block(self.jump_table_block())
    }

    pub fn dump_data_block(&self) -> String {
        self.dump_block(self.data_block())
    }

    pub fn dump_symbol_table_block(&self) -> String {
        self.dump_block(self.symbol_table_block())
    }

    pub fn dump_expression_symbol_block(&self) -> String {
        self.dump_block(self.expression_symbol_block())
    }

    pub fn dump_custom_block(&self) -> String {
        self.dump_block(self.custom_data_block())
    }

    pub fn dump_all_blocks(&self) -> String {
        let mut s = vec![];
        s.push(format!("Instruction Block ({}):", self.instruction_block().cursor));
        s.push(self.dump_instruction_block());
        s.push(format!("Jump Table Block ({}):", self.jump_table_block().cursor));
        s.push(self.dump_jump_table_block());
        s.push(format!("Symbol Table Block ({}):", self.symbol_table_block().cursor));
        s.push(self.dump_symbol_table_block());
        s.push(format!("Expression Symbol Block ({}):", self.expression_symbol_block().cursor));
        s.push(self.dump_expression_symbol_block());
        s.push(format!("Data Block ({}):", self.data_block().cursor));
        s.push(self.dump_data_block());
        s.push(format!("Custom Data Block ({}):", self.custom_data_block().cursor));
        s.push(self.dump_custom_block());
        s.join("\n")
    }

    fn dump_block(&self, block: &StorageBlock) -> String {
        let mut s = vec![];
        for i in block.start..block.start + block.cursor {
            let relative_index = i - block.start;
            s.push(format!("{} | {}: {}", i, relative_index,match self.get_basic_data(i) {
                Some(d) => format!("{}", d),
                None => "[No Data]".to_string()
            }));
        }

        s.join("\n")
    }
}