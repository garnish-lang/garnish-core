use crate::{BasicData, BasicDataCustom, BasicGarnishData};

impl<T> BasicGarnishData<T>
where
    T: BasicDataCustom,
{
    pub fn optimize_data_block(&mut self) {
        let range = self.data_block().range();
        let copy_cursor = self.data_retention_count();
        let true_start = range.start + copy_cursor;

        dbg!(true_start, range.end);
        for i in true_start..range.end {
            self.data_mut()[i] = BasicData::Empty;
        }
        self.data_block_mut().cursor = copy_cursor;
    }
}

#[cfg(test)]
mod tests {
    use crate::{BasicData, basic_object};

    use super::*;

    #[test]
    fn default_with_data_all_data_removed() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        data.push_object_to_data_block(basic_object!(Unit, (Number 100), (CharList "hello"), ((ByteList 1, 2, 3) = (Symbol "my_symbol"))))
            .unwrap();
        data.optimize_data_block();
        assert_eq!(data.data_size(), 0);
    }

    #[test]
    fn data_within_retention_count_is_kept() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        data.push_object_to_data_block(basic_object!(Unit, (Number 100), (CharList "hello"), ((ByteList 1, 2, 3) = (Symbol "my_symbol"))))
            .unwrap();
        data.retain_all_current_data();
        data.push_object_to_data_block(basic_object!((Number 1234), (CharList "world"))).unwrap();
        data.optimize_data_block();

        dbg!(data.total_allocated_size());
        
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().resize(80, BasicData::Empty);
        expected_data.data_mut().splice(30..49, vec![
            BasicData::Unit,
            BasicData::Number(100.into()),
            BasicData::CharList(5),
            BasicData::Char('h'),
            BasicData::Char('e'),
            BasicData::Char('l'),
            BasicData::Char('l'),
            BasicData::Char('o'),
            BasicData::ByteList(3),
            BasicData::Byte(1),
            BasicData::Byte(2),
            BasicData::Byte(3),
            BasicData::Symbol(8904929874702161741),
            BasicData::Pair(8, 12),
            BasicData::List(4, 0),
            BasicData::ListItem(0),
            BasicData::ListItem(1),
            BasicData::ListItem(2),
            BasicData::ListItem(13),
        ]);

        expected_data.set_data_retention_count(23);
        expected_data.data_block_mut().cursor = 23;
        expected_data.data_block_mut().size = 40;
        expected_data.data_block_mut().start = 30;
        expected_data.custom_data_block_mut().start = 70;
        expected_data.custom_data_block_mut().size = 10;

        assert_eq!(data, expected_data);
    }
}
