use crate::{BasicData, BasicDataCustom, BasicGarnishData, DataError};

impl<T> BasicGarnishData<T>
where
    T: BasicDataCustom,
{
    pub(crate) fn optimize_data_block(&mut self) -> Result<(), DataError> {
        let current_data_end = self.data_block().start + self.data_block().cursor;
        let true_start = self.data_block().start + self.data_retention_count();

        let offset = current_data_end - true_start;

        let mut registers = vec![];
        let mut current = self.current_register();
        while let Some(index) = current {
            let (previous, value) = self.get_from_data_block_ensure_index(index)?.as_register()?;
            registers.push(value);
            current = previous;
        }
        registers.reverse();

        let mut previous = None;
        for value in registers.iter() {
            let new_value = self.get_from_data_block_ensure_index(*value)?.clone();
            let new_value_index = self.push_to_data_block(new_value)?;
            let index = self.push_to_data_block(BasicData::Register(previous, new_value_index - offset))?;
            previous = Some(index - offset);
        }

        println!("{}", self.dump_data_block());

        let new_data_end = self.data_block().start + self.data_block().cursor;
        let from_range = current_data_end..new_data_end;
        let mut current = true_start;
        dbg!(current);

        for i in from_range.clone() {
            self.data_mut()[current] = self.data_mut()[i].clone();
            current += 1;
        }

        self.data_block_mut().cursor = current - self.data_block().start;
        dbg!(current);

        let clear_range = current..new_data_end;
        for i in clear_range.clone() {
            self.data_mut()[i] = BasicData::Empty;
        }

        dbg!(current_data_end, true_start, offset, registers, from_range, clear_range, current);

        Ok(())
    }

    pub(crate) fn push_clone_data_from(&mut self, from: usize) -> Result<usize, DataError> {
        match self.get_from_data_block_ensure_index(from)? {
            BasicData::Unit => {
                let data = self.get_from_data_block_ensure_index(from)?.clone();
                self.push_to_data_block(data)
            }
            _ => todo!(),
        }
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

    #[test]
    fn data_referenced_by_registers_is_kept() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        let register_index = data.push_to_data_block(BasicData::Register(None, index)).unwrap();
        data.push_object_to_data_block(basic_object!((Number 1234), (CharList "world"))).unwrap();
        let index = data.push_object_to_data_block(basic_object!((Number 1234) = (Char 'a'))).unwrap();
        let register_index = data.push_to_data_block(BasicData::Register(Some(register_index), index)).unwrap();
        data.set_current_register(Some(register_index));

        println!("{}", data.dump_data_block());

        data.optimize_data_block().unwrap();
    
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().resize(70, BasicData::Empty);
        expected_data.data_mut().splice(30..36, vec![
            BasicData::Number(100.into()),
            BasicData::Register(None, 0),
            BasicData::Number(1234.into()),
            BasicData::Char('a'),
            BasicData::Pair(2, 3),
            BasicData::Register(Some(1), 4),
        ]);

        expected_data.data_block_mut().cursor = 6;
        expected_data.set_current_register(Some(5));

        println!("{}", data.dump_data_block());

        assert_eq!(data, expected_data);
    }
}

#[cfg(test)]
mod clone {
    use crate::{BasicData, BasicGarnishData, basic_object};

    #[test]
    fn unit() {
        let mut data = BasicGarnishData::<()>::new().unwrap();
        let index = data.push_object_to_data_block(basic_object!(Unit)).unwrap();
        data.push_clone_data_from(index).unwrap();
        
        let mut expected_data = BasicGarnishData::<()>::new().unwrap();
        expected_data.data_mut().splice(30..32, vec![
            BasicData::Unit,
            BasicData::Unit,
        ]);
        expected_data.data_block_mut().cursor = 2;
        
        assert_eq!(data, expected_data);
    }
}