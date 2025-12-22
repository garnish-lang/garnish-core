use crate::{BasicData, BasicDataCustom, BasicGarnishData};

impl<T> BasicGarnishData<T>
where
    T: BasicDataCustom,
{
    pub fn optimize_data_block(&mut self) {
        let range = self.data_block().range();
        for i in range {
            self.data_mut()[i] = BasicData::Empty;
        }
        self.data_block_mut().cursor = 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::basic_object;

    use super::*;

    #[test]
    fn default_with_dataall_data_removed() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        data.push_object_to_data_block(basic_object!(Unit, (Number 100), (CharList "hello"), ((ByteList 1, 2, 3) = (Symbol "my_symbol"))))
            .unwrap();
        data.optimize_data_block();
        assert_eq!(data.data_size(), 0);
    }
}
