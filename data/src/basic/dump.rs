use crate::{BasicDataCustom, BasicGarnishData};

impl<T> BasicGarnishData<T> where T: BasicDataCustom {
    pub fn dump_data_block(&self) -> String {
        let mut s = vec![];
        for i in 0..self.data_block.cursor {
            s.push(format!("{}: {:?}", i, self.get_basic_data(i).unwrap()));
        }
        s.join("\n")
    }
}