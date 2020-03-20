use std::collections::HashMap;
use std::convert::TryFrom;
use std::ops::Range;

use crate::byte_vec::{DataVecReader, DataVecWriter};
use crate::data_type::DataType;
use crate::{
    has_end, has_start, has_step, skip_byte_sizes, skip_size, skip_sizes, skip_type,
    skip_type_and_byte_size, skip_type_and_size, ExpressionValue, Result, BYTE_SIZE_LENGTH,
    DATA_TYPE_LENGTH, FLOAT_LENGTH, INTEGER_LENGTH, SIZE_LENGTH,
};

pub trait With<T, U> {
    fn with(self, other: Option<U>) -> Option<(T, U)>;
}

impl<T, U> With<T, U> for Option<T> {
    fn with(self, other: Option<U>) -> Option<(T, U)> {
        self.and_then(|val| other.map(|val2| (val, val2)))
    }
}

pub fn hash_byte_slice(bytes: &[u8]) -> usize {
    let mut hash = 1;
    for i in bytes {
        hash ^= *i as usize;
    }

    hash
}

pub fn hash_of_character_list(start: usize, data: &[u8]) -> usize {
    let mut hash = 1;
    let reader = DataVecReader::new_from_slice(data);
    let char_count = reader.read_size(skip_type(start));

    let characters_start = skip_type_and_size(start);
    for i in 0..char_count {
        let character_ref_start = skip_sizes(characters_start, i);
        let character_ref = reader.read_size(character_ref_start);
        let character_length = reader.read_byte_size(skip_type(character_ref));

        let code_points_start = skip_type_and_byte_size(character_ref);
        for j in 0..character_length {
            let code_point = reader.read_byte_size(skip_byte_sizes(code_points_start, j));

            hash ^= code_point;
        }
    }

    hash
}

pub fn hash_expression_string_slice(value_start: usize, vec: &[u8]) -> usize {
    let length = DataVecReader::new_from_slice(vec).read_size(value_start + 1);
    let string_range = (value_start + 1 + SIZE_LENGTH)
        ..(value_start + 1 + SIZE_LENGTH + length * SIZE_LENGTH as usize);
    hash_byte_slice(&vec[string_range])
}

pub fn hash_expression_string(value_start: usize, vec: &Vec<u8>) -> usize {
    hash_expression_string_slice(value_start, &vec)
}

pub fn get_value_with_hash(
    target: usize,
    data: &[u8],
    key_count: usize,
    key_area_range: &Range<usize>,
) -> Result<Option<usize>> {
    let list_index = target % key_count;
    let mut key_index = skip_sizes(key_area_range.start, list_index);

    let reader = DataVecReader::new_from_slice(data);

    let mut pair_ref = reader.read_size(key_index);
    let mut key_ref = reader.read_size(skip_type(pair_ref));
    let mut key_type = DataType::try_from(data[key_ref])?;
    let mut key_value = match key_type {
        DataType::Symbol => reader.read_size(skip_type(key_ref)),
        DataType::CharacterList => hash_of_character_list(key_ref, data),
        _ => 0,
    };

    let mut found = false;

    if key_value != target {
        let max_it = key_count;
        let mut it_count = 0;
        while it_count < max_it {
            key_index = skip_size(key_index);
            if key_index >= key_area_range.end {
                key_index = key_area_range.start;
            }

            pair_ref = reader.read_size(key_index);
            key_ref = reader.read_size(skip_type(pair_ref));
            key_type = DataType::try_from(data[key_ref])?;
            key_value = match key_type {
                DataType::Symbol => reader.read_size(skip_type(key_ref)),
                DataType::CharacterList => hash_of_character_list(key_ref, data),
                _ => 0,
            };

            if key_value == target {
                found = true;
                break;
            }

            it_count += 1;
        }
    } else {
        found = true;
    }

    if found {
        Ok(Some(pair_ref))
    } else {
        Ok(None)
    }
}

pub fn insert_associative_list_keys(
    data: &mut Vec<u8>,
    key_area_range: Range<usize>,
    key_refs: &Vec<usize>,
) -> Result {
    // make sure values are zeroed
    for i in key_area_range.clone() {
        data[i] = 0;
    }

    for key in key_refs.iter() {
        let pair_ref = *key;
        let key_start = pair_ref + 1;
        // convert key data to usize
        let key_ref = DataVecReader::new(data).read_size(key_start);

        let key_type = DataType::try_from(data[key_ref])?;
        let symbol_value = match key_type {
            DataType::Symbol => DataVecReader::new(data).read_size(key_ref + 1),
            DataType::CharacterList => hash_of_character_list(key_ref, &data) as usize,
            _ => 0,
        };

        let mut key_index = (symbol_value % key_refs.len()) * SIZE_LENGTH + key_area_range.start;

        let mut pair_ref_data: Vec<u8> = vec![];
        DataVecWriter::new(&mut pair_ref_data).push_size(pair_ref);

        let existing = DataVecReader::new(data).read_size(key_index);

        if existing != 0 {
            // probe
            let max_it = key_refs.len();
            let mut it_count = 0;
            while it_count < max_it {
                key_index += SIZE_LENGTH;
                if key_index >= key_area_range.end {
                    key_index = key_area_range.start;
                }

                let existing = DataVecReader::new(data).read_size(key_index);
                if existing == 0 {
                    // make ref relative
                    break;
                }

                // fail safe
                it_count += 1;
            }
        }

        DataVecWriter::new(data).write_data(key_index, &pair_ref_data);
    }

    Ok(())
}

pub trait ExpressionValueConsumer {
    fn insert_at_value_cursor(&mut self, data: u8) -> Result<()>;
    fn insert_all_at_value_cursor(&mut self, data: &[u8]) -> Result<()>;
    fn insert_at_ref_cursor(&mut self, r: usize);
    fn get_value_cursor(&self) -> usize;
    fn get_data_mut(&mut self) -> &mut Vec<u8>;
    fn get_symbol_table_mut(&mut self) -> &mut HashMap<String, usize>;
    fn get_expression_table_mut(&mut self) -> &mut Vec<usize>;
    fn get_expression_map_mut(&mut self) -> &mut HashMap<String, usize>;
}

pub trait CopyValue {
    fn copy_value(&mut self, value: &ExpressionValue) -> Result<()>;
}

impl<T> CopyValue for T
where
    T: ExpressionValueConsumer,
{
    fn copy_value(&mut self, value: &ExpressionValue) -> Result<()> {
        let mut ref_pos = self.get_value_cursor();

        let data = value.get_data();
        let symbol_table = value.get_symbol_table();

        let start = self.get_value_cursor();

        let mut i = 0;
        while i < data.len() {
            let d = *data.get(i).unwrap();

            match DataType::try_from(d)? {
                DataType::Unit => {
                    self.insert_at_value_cursor(d)?;

                    i += 1;
                }
                DataType::Integer => {
                    if i + 1 >= data.len() {
                        return Err("Expecting number value, found end of data.".into());
                    }

                    let data_slice = &data[i..(i + 1 + INTEGER_LENGTH)];
                    self.insert_all_at_value_cursor(data_slice)?;

                    i += 1 + INTEGER_LENGTH;
                }
                DataType::Float => {
                    if i + 1 >= data.len() {
                        return Err("Expecting number value, found end of data.".into());
                    }

                    let data_slice = &data[i..(i + 1 + FLOAT_LENGTH)];
                    self.insert_all_at_value_cursor(data_slice)?;

                    i += 1 + FLOAT_LENGTH;
                }
                DataType::Character => {
                    let reader = DataVecReader::new(&data);
                    let length = reader.read_byte_size(skip_type(i));
                    let data_slice = &data[i..(skip_byte_sizes(i, 2 + length))];

                    self.insert_all_at_value_cursor(data_slice)?;

                    i += 2 + length * BYTE_SIZE_LENGTH;
                }
                DataType::CharacterList => {
                    // update ref cursor to point at character list
                    ref_pos = self.get_value_cursor();

                    let reader = DataVecReader::new(&data);

                    let length = reader.read_size(skip_type(i));

                    let mut value_refs: Vec<u8> = vec![];
                    let mut writer = DataVecWriter::new(&mut value_refs);

                    writer = writer
                        .push_data_type(DataType::CharacterList)
                        .push_size(length);

                    let sizes_start = skip_type_and_size(i);
                    for j in 0..length {
                        let j_start = skip_sizes(sizes_start, j);
                        let r = reader.read_size(j_start) + start;

                        writer = writer.push_size(r);
                    }

                    self.insert_all_at_value_cursor(&value_refs[..])?;

                    i += DATA_TYPE_LENGTH + SIZE_LENGTH * (1 + length);
                }
                DataType::Symbol | DataType::ExternalMethod => {
                    match data.get(i + 1) {
                        None => {
                            return Err("Expecting symbol value, found end of data".into());
                        }
                        Some(value) => {
                            // get existing string
                            match symbol_table.iter().find(|(_, v)| **v as u8 == *value) {
                                None => Err(format!("Unresolved symbol value {}", value))?,
                                Some((symbol_string, _)) => {
                                    // check for existing value
                                    let val = match self.get_symbol_table_mut().get(symbol_string) {
                                        // insert new
                                        None => {
                                            let val = self.get_symbol_table_mut().len();
                                            self.get_symbol_table_mut()
                                                .insert(symbol_string.clone(), val);
                                            val
                                        }
                                        // use existing value
                                        Some(val) => *val,
                                    };

                                    self.insert_at_value_cursor(d)?;

                                    let mut data: Vec<u8> = vec![];
                                    DataVecWriter::new(&mut data).push_size(val);
                                    self.insert_all_at_value_cursor(&data)?;
                                }
                            };
                        }
                    }

                    i += 1 + SIZE_LENGTH;
                }
                DataType::Expression => {
                    // insert type
                    self.insert_at_value_cursor(d)?;

                    match data.get(i + 1) {
                        None => {
                            return Err("Expecting symbol value, found end of data".into());
                        }
                        Some(value) => {
                            // get existing string
                            match symbol_table.iter().find(|(_, v)| **v as u8 == *value) {
                                None => {
                                    return Err(format!("Unresolved symbol value {}", value).into());
                                }
                                Some((symbol_string, _)) => {
                                    // check for existing value
                                    match self.get_symbol_table_mut().get(symbol_string) {
                                        // insert new
                                        None => {
                                            let val = self.get_symbol_table_mut().len();
                                            self.get_symbol_table_mut()
                                                .insert(symbol_string.clone(), val);
                                        }
                                        // symbol already exists, do nothing
                                        Some(_) => (),
                                    };

                                    let index =
                                        match self.get_expression_map_mut().get(symbol_string) {
                                            None => {
                                                // insert invalid expression into table and map
                                                let index =
                                                    self.get_expression_table_mut().len() + 1;
                                                self.get_expression_table_mut().push(0);
                                                self.get_expression_map_mut()
                                                    .insert(symbol_string.clone(), index);
                                                index
                                            }
                                            Some(i) => *i,
                                        };

                                    let mut index_data = vec![];
                                    DataVecWriter::new(&mut index_data).push_size(index);

                                    self.insert_all_at_value_cursor(&index_data[..])?;
                                }
                            }
                        }
                    }

                    i += 1 + SIZE_LENGTH;
                }
                DataType::Range => {
                    // update ref cursor to point at range
                    ref_pos = self.get_value_cursor();

                    self.insert_at_value_cursor(d)?;

                    let reader = DataVecReader::new(data);

                    let flags = reader.read_byte_size(skip_type(i)) as u8;
                    let sizes_to_copy = [has_start(flags), has_end(flags), has_step(flags)]
                        .iter()
                        .filter(|b| **b)
                        .count();

                    // insert flags
                    self.insert_at_value_cursor(data[i + 1])?;

                    let mut value_refs: Vec<u8> = vec![];
                    let mut writer = DataVecWriter::new(&mut value_refs);

                    let sizes_start = skip_type_and_byte_size(i);
                    for j in 0..sizes_to_copy {
                        let j_start = skip_sizes(sizes_start, j);
                        let r = reader.read_size(j_start) + start;

                        writer = writer.push_size(r);
                    }

                    self.insert_all_at_value_cursor(&value_refs[..])?;

                    i += 2 + SIZE_LENGTH * sizes_to_copy;
                }
                DataType::Pair => {
                    // update ref cursor to point at pair
                    ref_pos = self.get_value_cursor();

                    self.insert_at_value_cursor(d)?;

                    // add start to references of pair
                    // re-serialize and append to data
                    let reader = DataVecReader::new(data);
                    let key_ref = reader.read_size(i + 1) + start;
                    let value_ref = reader.read_size(i + 1 + SIZE_LENGTH) + start;
                    let mut value_refs: Vec<u8> = vec![];

                    DataVecWriter::new(&mut value_refs)
                        .push_size(key_ref)
                        .push_size(value_ref);

                    self.insert_all_at_value_cursor(&value_refs[..])?;

                    i += 1 + 2 * SIZE_LENGTH;
                }
                DataType::Partial => {
                    // update ref cursor to point at partial
                    ref_pos = self.get_value_cursor();

                    self.insert_at_value_cursor(d)?;

                    // add start to references of partial
                    // re-serialize and append to data
                    let reader = DataVecReader::new(data);
                    let key_ref = reader.read_size(i + 1) + start;
                    let value_ref = reader.read_size(i + 1 + SIZE_LENGTH) + start;
                    let mut value_refs: Vec<u8> = vec![];

                    DataVecWriter::new(&mut value_refs)
                        .push_size(key_ref)
                        .push_size(value_ref);

                    self.insert_all_at_value_cursor(&value_refs[..])?;

                    i += 1 + 2 * SIZE_LENGTH;
                }
                DataType::List => {
                    let value_start = i;
                    // data type, length and counts
                    let value_end = i + 1 + 2 * SIZE_LENGTH;

                    // update ref cursor to point at list
                    ref_pos = self.get_value_cursor();

                    let reader = DataVecReader::new(&data);

                    let length = reader.read_size(i + 1);
                    let key_count = reader.read_size(i + 1 + SIZE_LENGTH);
                    let list_start = value_end;

                    // write data type and counts
                    self.insert_all_at_value_cursor(&data[value_start..value_end])?;

                    // add total size of elements
                    let mut key_refs: Vec<usize> = vec![];
                    let mut value_refs: Vec<u8> = vec![];

                    for j in 0..length {
                        let element_start = list_start + j * SIZE_LENGTH;
                        let value_ref = DataVecReader::new(data).read_size(element_start) + start;
                        DataVecWriter::new(&mut value_refs).push_size(value_ref);

                        let abs_ref = value_ref;
                        if DataType::try_from(self.get_data_mut()[abs_ref])? == DataType::Pair {
                            let key_ref =
                                DataVecReader::new(self.get_data_mut()).read_size(abs_ref + 1);
                            let key_type = DataType::try_from(self.get_data_mut()[key_ref])?;
                            if key_type == DataType::Symbol || key_type == DataType::CharacterList {
                                key_refs.push(value_ref);
                            }
                        }
                    }

                    let key_start = list_start + length * SIZE_LENGTH + start;
                    let key_area = key_start..(key_start + key_count * SIZE_LENGTH);

                    insert_associative_list_keys(&mut self.get_data_mut(), key_area, &key_refs)?;

                    self.insert_all_at_value_cursor(&value_refs[..])?;

                    i += 1 + (2 + length + key_count) * SIZE_LENGTH;
                }
                _ => {
                    return Err(
                        format!("Cannot copy value of {} type.", DataType::try_from(d)?).into(),
                    );
                }
            }
        }

        self.insert_at_ref_cursor(ref_pos);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{hash_byte_slice, hash_of_character_list, DataType, DataVecWriter};

    #[test]
    fn hash_byte_slice_equals_hash_character_list() {
        let data = DataVecWriter::write_with(|w| {
            w.push_data_type(DataType::Character) // 0
                .push_byte_size(1)
                .push_character("b")
                .push_data_type(DataType::Character) // 3
                .push_byte_size(1)
                .push_character("e")
                .push_data_type(DataType::Character) // 6
                .push_byte_size(1)
                .push_character("a")
                .push_data_type(DataType::Character) // 9
                .push_byte_size(1)
                .push_character("r")
                .push_data_type(DataType::CharacterList) // 12
                .push_size(4) // 13
                .push_size(0) // 17
                .push_size(3) // 21
                .push_size(6) // 25
                .push_size(9)
        });

        let c = hash_of_character_list(12, &data);
        let b = hash_byte_slice(&("bear".as_bytes()));

        assert_eq!(c, b);
    }

    #[test]
    fn hash_byte_slice_equals_hash_character_list_grapheme_clusters() {
        let data = DataVecWriter::write_with(|w| {
            w.push_data_type(DataType::Character) // 0
                .push_byte_size(1) // 1
                .push_character("z") // 2
                .push_data_type(DataType::Character) // 3
                .push_byte_size(4) // 4
                .push_character("🐼") // 5
                .push_data_type(DataType::Character) // 9
                .push_byte_size(5) // 10
                .push_character("ö̲") // 11
                .push_data_type(DataType::CharacterList) // 16
                .push_size(3)
                .push_size(0)
                .push_size(3)
                .push_size(9)
        });

        let c = hash_of_character_list(16, &data);
        let b = hash_byte_slice(&("z🐼ö̲".as_bytes()));

        assert_eq!(c, b);
    }
}
