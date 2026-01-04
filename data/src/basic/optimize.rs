use crate::{BasicData, BasicDataCustom, BasicGarnishData, DataError, basic::companion::BasicDataCompanion};

impl<T, Companion> BasicGarnishData<T, Companion>
where
    T: BasicDataCustom,
    Companion: BasicDataCompanion<T>,
{   
    pub(crate) fn optimize_data_block_and_retain(&mut self, additional_data_retentions: &[usize]) -> Result<Vec<usize>, DataError> {
        let current_data_end = self.data_block().start + self.data_block().cursor;
        let retained_data_end = self.data_block().start + self.data_retention_count();
        
        let original_register = self.current_register();
        let original_value = self.current_value();
        let original_frame = self.current_frame();

        let index_list_start = self.data_block().cursor;
        
        if let Some(index) = original_register {
            self.create_index_stack(index)?;
        }

        if let Some(index) = original_value {
            self.create_index_stack(index)?;
        }

        if let Some(index) = original_frame {
            self.create_index_stack(index)?;
        }

        for additional_data_retention in additional_data_retentions {
            self.create_index_stack(*additional_data_retention)?;
        }

        let index_list_end = self.data_block().start + self.data_block().cursor;

        let offset = self.data_block().start + self.data_block().cursor - retained_data_end;

        if index_list_end != current_data_end {
            self.clone_index_stack(index_list_start, offset)?;
        }
        
        if let Some(original_register) = original_register {
            let mapped_index = self.lookup_in_data_slice(index_list_start, index_list_end, original_register)?;
            self.set_current_register(Some(mapped_index));
        }

        if let Some(original_value) = original_value {
            let mapped_index = self.lookup_in_data_slice(index_list_start, index_list_end, original_value)?;
            self.set_current_value(Some(mapped_index));
        }

        if let Some(original_frame) = original_frame {
            let mapped_index = self.lookup_in_data_slice(index_list_start, index_list_end, original_frame)?;
            self.set_current_frame(Some(mapped_index));
        }

        let mut mapped_indexes = vec![0; additional_data_retentions.len()];
        for (i, additional_data_retention) in additional_data_retentions.iter().enumerate() {
            mapped_indexes[i] = self.lookup_in_data_slice(index_list_start, index_list_end, *additional_data_retention)?;
        }

        let new_data_end = self.data_block().start + self.data_block().cursor;
        let from_range = index_list_end..new_data_end;
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

        Ok(mapped_indexes)
    }
}

#[cfg(test)]
mod optimize {
    use crate::{BasicData, NoOpCompanion, basic_object};

    use super::*;

    #[test]
    fn default_with_data_all_data_removed() {
        let mut data = BasicGarnishData::<(), NoOpCompanion>::new(NoOpCompanion::new()).unwrap();
        data.push_object_to_data_block(basic_object!(Unit, (Number 100), (CharList "hello"), ((ByteList 1, 2, 3) = (Symbol "my_symbol"))))
            .unwrap();
        data.optimize_data_block_and_retain(&[]).unwrap();
        assert_eq!(data.data_size(), 0);
    }

    #[test]
    fn data_within_retention_count_is_kept() {
        let mut data = BasicGarnishData::<(), NoOpCompanion>::new(NoOpCompanion::new()).unwrap();
        data.push_object_to_data_block(basic_object!(Unit, (Number 100), (CharList "hello"), ((ByteList 1, 2, 3) = (Symbol "my_symbol"))))
            .unwrap();
        data.retain_all_current_data();
        data.push_object_to_data_block(basic_object!((Number 1234), (CharList "world"))).unwrap();
        data.optimize_data_block_and_retain(&[]).unwrap();

        let mut expected_data = BasicGarnishData::<(), NoOpCompanion>::new(NoOpCompanion::new()).unwrap();
        expected_data.data_mut().resize(90, BasicData::Empty);
        expected_data.data_mut().splice(
            40..59,
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
        expected_data.custom_data_block_mut().start = 80;
        expected_data.custom_data_block_mut().size = 10;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn data_referenced_by_registers_is_kept() {
        let mut data = BasicGarnishData::<(), NoOpCompanion>::new(NoOpCompanion::new()).unwrap();
        let index = data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        let register_index = data.push_to_data_block(BasicData::RegisterRoot(index)).unwrap();
        data.push_object_to_data_block(basic_object!((Number 1234), (CharList "world"))).unwrap();
        let index = data.push_object_to_data_block(basic_object!((Number 1234) = (Char 'a'))).unwrap();
        let register_index = data.push_to_data_block(BasicData::Register(register_index, index)).unwrap();
        data.set_current_register(Some(register_index));

        data.optimize_data_block_and_retain(&[]).unwrap();

        let mut expected_data = BasicGarnishData::<(), NoOpCompanion>::new(NoOpCompanion::new()).unwrap();
        expected_data.data_mut().resize(80, BasicData::Empty);
        expected_data.data_mut().splice(40..46, vec![
            BasicData::Number(1234.into()),
            BasicData::Char('a'),
            BasicData::Number(100.into()),
            BasicData::Pair(0, 1),
            BasicData::RegisterRoot(2),
            BasicData::Register(4, 3),
        ]);

        expected_data.data_block_mut().cursor = 6;
        expected_data.data_block_mut().size = 30;
        expected_data.custom_data_block_mut().start = 70;
        expected_data.set_current_register(Some(5));

        assert_eq!(data, expected_data);
    }

    #[test]
    fn data_referenced_by_values_is_kept() {
        let mut data = BasicGarnishData::<(), NoOpCompanion>::new(NoOpCompanion::new()).unwrap();
        let index = data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        let value_index = data.push_to_data_block(BasicData::ValueRoot(index)).unwrap();
        data.push_object_to_data_block(basic_object!((Number 1234), (CharList "world"))).unwrap();
        let index = data.push_object_to_data_block(basic_object!((Number 1234) = (Char 'a'))).unwrap();
        let value_index = data.push_to_data_block(BasicData::Value(value_index, index)).unwrap();
        data.set_current_value(Some(value_index));

        data.optimize_data_block_and_retain(&[]).unwrap();

        let mut expected_data = BasicGarnishData::<(), NoOpCompanion>::new(NoOpCompanion::new()).unwrap();
        expected_data.data_mut().resize(80, BasicData::Empty);
        expected_data.data_mut().splice(40..46, vec![
            BasicData::Number(1234.into()),
            BasicData::Char('a'),
            BasicData::Number(100.into()),
            BasicData::Pair(0, 1),
            BasicData::ValueRoot(2),
            BasicData::Value(4, 3),
        ]);

        expected_data.data_block_mut().cursor = 6;
        expected_data.data_block_mut().size = 30;
        expected_data.custom_data_block_mut().start = 70;
        expected_data.set_current_value(Some(5));

        assert_eq!(data, expected_data);
    }

    #[test]
    fn data_referenced_by_frames_is_kept() {
        let mut data = BasicGarnishData::<(), NoOpCompanion>::new(NoOpCompanion::new()).unwrap();
        data.push_to_data_block(BasicData::JumpPoint(10)).unwrap();
        let frame_index = data.push_to_data_block(BasicData::FrameRoot).unwrap();
        data.push_object_to_data_block(basic_object!((Number 1234), (CharList "world"))).unwrap();
        let index = data.push_object_to_data_block(basic_object!((Number 1234) = (Char 'a'))).unwrap();
        data.push_to_data_block(BasicData::JumpPoint(20)).unwrap();
        let frame_index = data.push_to_data_block(BasicData::Frame(frame_index, index)).unwrap();
        data.set_current_frame(Some(frame_index));

        data.optimize_data_block_and_retain(&[]).unwrap();

        let mut expected_data = BasicGarnishData::<(), NoOpCompanion>::new(NoOpCompanion::new()).unwrap();
        expected_data.data_mut().resize(90, BasicData::Empty);
        expected_data.data_mut().splice(40..47, vec![
            BasicData::Number(1234.into()),
            BasicData::Char('a'),
            BasicData::Pair(0, 1),
            BasicData::JumpPoint(10),
            BasicData::FrameRoot,
            BasicData::JumpPoint(20),
            BasicData::Frame(4, 2),
        ]);

        expected_data.data_block_mut().cursor = 7;
        expected_data.data_block_mut().size = 40;
        expected_data.custom_data_block_mut().start = 80;
        expected_data.set_current_frame(Some(6));
        
        assert_eq!(data, expected_data);
    }

    #[test]
    fn with_retained_keeps_data() {
        let mut data = BasicGarnishData::<(), NoOpCompanion>::new(NoOpCompanion::new()).unwrap();

        data.push_object_to_data_block(basic_object!((Number 1234), (CharList "world"))).unwrap();
        let index = data.push_object_to_data_block(basic_object!((Number 1234) = (Char 'a'))).unwrap();

        let result = data.optimize_data_block_and_retain(&[index]).unwrap();

        let mut expected_data = BasicGarnishData::<(), NoOpCompanion>::new(NoOpCompanion::new()).unwrap();
        expected_data.data_mut().resize(80, BasicData::Empty);
        expected_data.data_mut().splice(40..43, vec![
            BasicData::Number(1234.into()),
            BasicData::Char('a'),
            BasicData::Pair(0, 1),
        ]);

        expected_data.data_block_mut().cursor = 3;
        expected_data.data_block_mut().size = 30;
        expected_data.custom_data_block_mut().start = 70;
        
        assert_eq!(result, vec![2]);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn retain_all_options() {
        let mut data = BasicGarnishData::<(), NoOpCompanion>::new(NoOpCompanion::new()).unwrap();
        
        let retained_value_index = data.push_object_to_data_block(basic_object!(Number 100)).unwrap();
        data.retain_all_current_data();
        
        data.push_object_to_data_block(basic_object!((Number 200), (CharList "extra"))).unwrap();
        let later_value_index = data.push_object_to_data_block(basic_object!(Number 300)).unwrap();
        let later_pair_index = data.push_object_to_data_block(basic_object!((Number 400) = (Number 500))).unwrap();
        let additional_retained_index = data.push_object_to_data_block(basic_object!(Number 600)).unwrap();
        let additional_retained_pair_index = data.push_object_to_data_block(basic_object!((Number 700) = (Number 800))).unwrap();
        
        let value_index = data.push_to_data_block(BasicData::ValueRoot(retained_value_index)).unwrap();
        let value_index = data.push_to_data_block(BasicData::Value(value_index, later_value_index)).unwrap();
        data.set_current_value(Some(value_index));
        
        let register_index = data.push_to_data_block(BasicData::RegisterRoot(later_pair_index)).unwrap();
        data.set_current_register(Some(register_index));
        
        data.push_to_data_block(BasicData::JumpPoint(10)).unwrap();
        let frame_index = data.push_to_data_block(BasicData::FrameRoot).unwrap();
        data.push_to_data_block(BasicData::JumpPoint(20)).unwrap();
        let frame_index = data.push_to_data_block(BasicData::Frame(frame_index, later_value_index)).unwrap();
        data.set_current_frame(Some(frame_index));
        
        let result = data.optimize_data_block_and_retain(&[additional_retained_index, additional_retained_pair_index]).unwrap();

        let mut expected_data = BasicGarnishData::<(), NoOpCompanion>::new(NoOpCompanion::new()).unwrap();
        expected_data.data_mut().resize(110, BasicData::Empty);
        
        expected_data.data_mut().splice(40..56, vec![
            BasicData::Number(100.into()),
            BasicData::Number(700.into()),
            BasicData::Number(800.into()),
            BasicData::Pair(1, 2),
            BasicData::Number(600.into()),
            BasicData::Number(300.into()),
            BasicData::JumpPoint(10),
            BasicData::FrameRoot,
            BasicData::JumpPoint(20),
            BasicData::Frame(7, 5),
            BasicData::ValueRoot(0),
            BasicData::Value(10, 5),
            BasicData::Number(400.into()),
            BasicData::Number(500.into()),
            BasicData::Pair(12, 13),
            BasicData::RegisterRoot(14),
        ]);

        expected_data.set_data_retention_count(1);
        expected_data.data_block_mut().cursor = 16;
        expected_data.data_block_mut().size = 60;
        expected_data.custom_data_block_mut().start = 100;
        expected_data.custom_data_block_mut().size = 10;
        expected_data.set_current_value(Some(11));
        expected_data.set_current_register(Some(15));
        expected_data.set_current_frame(Some(9));

        assert_eq!(result, vec![4, 3]);
        assert_eq!(data, expected_data);
    }
}
