use crate::{BasicData, BasicDataCustom, BasicGarnishData, DataError};

impl<T> BasicGarnishData<T>
where
    T: BasicDataCustom,
{
    pub(crate) fn optimize_data_block(&mut self) -> Result<(), DataError> {
        let current_data_end = self.data_block().start + self.data_block().cursor;
        let retained_data_end = self.data_block().start + self.data_retention_count();
        
        let original_register = self.current_register();
        let index_list_start = match original_register {
            Some(index) => self.create_index_stack(index)?,
            None => current_data_end,
        };

        let index_list_end = self.data_block().start + self.data_block().cursor;

        let clone_start = self.data_block().start + self.data_block().cursor;

        let offset = self.data_block().start + self.data_block().cursor - retained_data_end;

        if index_list_start != current_data_end {
            self.clone_index_stack(index_list_start, offset)?;
        }
        
        if let Some(original_register) = original_register {
            let mapped_index = self.lookup_in_data_slice(index_list_start, index_list_end, original_register)?;
            self.set_current_register(Some(mapped_index));
        }

        let new_data_end = self.data_block().start + self.data_block().cursor;
        let from_range = clone_start..new_data_end;
        let mut current = retained_data_end;

        for i in from_range.clone() {
            self.data_mut()[current] = self.data_mut()[i].clone();
            current += 1;
        }

        self.data_block_mut().cursor = current - self.data_block().start;

        let clear_range = current..new_data_end;
        for i in clear_range.clone() {
            self.data_mut()[i] = BasicData::Empty;
        }

        Ok(())
    }
}

#[cfg(test)]
mod optimize {
    use crate::{BasicData, basic_object};

    use super::*;

    #[test]
    fn default_with_data_all_data_removed() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        data.push_object_to_data_block(basic_object!(Unit, (Number 100), (CharList "hello"), ((ByteList 1, 2, 3) = (Symbol "my_symbol"))))
            .unwrap();
        data.optimize_data_block().unwrap();
        assert_eq!(data.data_size(), 0);
    }

    #[test]
    fn data_within_retention_count_is_kept() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        data.push_object_to_data_block(basic_object!(Unit, (Number 100), (CharList "hello"), ((ByteList 1, 2, 3) = (Symbol "my_symbol"))))
            .unwrap();
        data.retain_all_current_data();
        data.push_object_to_data_block(basic_object!((Number 1234), (CharList "world"))).unwrap();
        data.optimize_data_block().unwrap();

        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().resize(80, BasicData::Empty);
        expected_data.data_mut().splice(
            30..49,
            vec![
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
            ],
        );

        expected_data.set_data_retention_count(23);
        expected_data.data_block_mut().cursor = 23;
        expected_data.data_block_mut().size = 40;
        expected_data.data_block_mut().start = 30;
        expected_data.custom_data_block_mut().start = 70;
        expected_data.custom_data_block_mut().size = 10;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn data_referenced_by_registers_is_kept() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        let register_index = data.push_to_data_block(BasicData::RegisterRoot(index)).unwrap();
        data.push_object_to_data_block(basic_object!((Number 1234), (CharList "world"))).unwrap();
        let index = data.push_object_to_data_block(basic_object!((Number 1234) = (Char 'a'))).unwrap();
        let register_index = data.push_to_data_block(BasicData::Register(register_index, index)).unwrap();
        data.set_current_register(Some(register_index));

        println!("{}", data.dump_data_block());

        data.optimize_data_block().unwrap();

        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().resize(70, BasicData::Empty);
        expected_data.data_mut().splice(30..36, vec![
            BasicData::Number(1234.into()),
            BasicData::Char('a'),
            BasicData::Number(100.into()),
            BasicData::Pair(0, 1),
            BasicData::RegisterRoot(2),
            BasicData::Register(4, 3),
        ]);

        expected_data.data_block_mut().cursor = 6;
        expected_data.data_block_mut().size = 30;
        expected_data.custom_data_block_mut().start = 60;
        expected_data.set_current_register(Some(5));

        println!("===================");
        println!("{}", data.dump_data_block());

        assert_eq!(data, expected_data);
    }
}
