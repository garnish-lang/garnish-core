use crate::{BasicDataCustom, BasicGarnishData, basic::storage::StorageBlock};

impl<T> BasicGarnishData<T> where T: BasicDataCustom {
    pub fn dump_instruction_block(&self) -> String {
        self.dump_block(&self.instruction_block)
    }

    pub fn dump_jump_table_block(&self) -> String {
        self.dump_block(&self.jump_table_block)
    }

    pub fn dump_data_block(&self) -> String {
        self.dump_block(&self.data_block)
    }

    pub fn dump_symbol_table_block(&self) -> String {
        self.dump_block(&self.symbol_table_block)
    }

    fn dump_block(&self, block: &StorageBlock) -> String {
        let mut s = vec![];
        for i in block.start..block.start + block.cursor {
            s.push(format!("{}: {}", i, match self.get_basic_data(i) {
                Some(d) => format!("{:?}", d),
                None => "[No Data]".to_string()
            }));
        }

        s.join("\n")
    }
}