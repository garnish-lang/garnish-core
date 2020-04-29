use crate::value::range::{RANGE_HAS_STEP, RANGE_OPEN_END, RANGE_OPEN_START};
use crate::{
    get_value_with_hash, hash_byte_slice, skip_size, skip_sizes, skip_type, skip_type_and_2_sizes,
    skip_type_and_byte_size, skip_type_and_bytes_and_sizes, skip_type_and_size,
    skip_type_and_sizes, skip_type_byte_size, DataType, DataVecReader, Result, FLOAT_LENGTH,
    INTEGER_LENGTH, SIZE_LENGTH,
};
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::io::Cursor;

// TODO: look into using DataType::Unit instead of replicating its value here
const UNIT_ARRAY: [u8; 1] = [1];

#[derive(Debug)]
pub struct ExpressionValueRef<'a> {
    pub(crate) data: &'a [u8],
    pub(crate) value_start: usize,
    pub(crate) symbol_table: Option<&'a HashMap<String, usize>>,
}

impl<'a> ExpressionValueRef<'a> {
    pub fn new(data: &'a [u8], symbol_table: Option<&'a HashMap<String, usize>>) -> Result<Self> {
        ExpressionValueRef::new_with_start(data, symbol_table, 0)
    }

    pub fn new_with_start(
        data: &'a [u8],
        symbol_table: Option<&'a HashMap<String, usize>>,
        value_start: usize,
    ) -> Result<Self> {
        if data.len() < 1 {
            Err("Not enough data. Need at least one byte to represent a type.".into())
        } else if value_start >= data.len() {
            Err("Value start is set beyond data bounds.".into())
        } else {
            Ok(ExpressionValueRef {
                data,
                value_start,
                symbol_table,
            })
        }
    }

    pub fn unit() -> Self {
        ExpressionValueRef {
            data: &UNIT_ARRAY,
            symbol_table: None,
            value_start: 0,
        }
    }

    pub fn get_symbol_value(&self, s: &str) -> Option<usize> {
        self.symbol_table.and_then(|table| table.get(s).map(|v| *v))
    }

    pub fn get_type(&self) -> Result<DataType> {
        DataType::try_from(self.data[self.value_start])
    }

    pub fn is_unit(&self) -> bool {
        self.is_of_type(DataType::Unit)
    }

    fn is_of_type(&self, data_type: DataType) -> bool {
        DataType::try_from(self.data[self.value_start])
            .map(|d| d == data_type)
            .unwrap_or(false)
    }

    pub fn as_integer(&self) -> Result<i32> {
        if DataType::try_from(self.data[self.value_start])? == DataType::Integer {
            let value_start = self.value_start + 1;
            let value_slice = &self.data[value_start..(value_start + INTEGER_LENGTH)];

            let mut reader = Cursor::new(value_slice);
            let value = reader.read_i32::<LittleEndian>().unwrap();

            Ok(value)
        } else {
            Err("Not Integer type".into())
        }
    }

    pub fn as_float(&self) -> Result<f32> {
        if DataType::try_from(self.data[self.value_start])? == DataType::Float {
            let value_start = self.value_start + 1;
            let value_slice = &self.data[value_start..(value_start + FLOAT_LENGTH)];

            let mut reader = Cursor::new(value_slice);
            let value = reader.read_f32::<LittleEndian>().unwrap();

            Ok(value)
        } else {
            Err("Not Float type".into())
        }
    }

    pub fn as_char(&self) -> Result<char> {
        if DataType::try_from(self.data[self.value_start])? != DataType::Character {
            return Err("Not Character type".into());
        }

        let s = self.char_string()?;

        if s.len() > 4 {
            Err("Cannot convert characters greater than 4 bytes into char.".into())
        } else {
            match s.chars().next() {
                Some(c) => Ok(c),
                None => Err("Could not get next char in string".into()),
            }
        }
    }

    fn char_string(&self) -> Result<String> {
        let reader = DataVecReader::new_from_slice(self.data);
        let length = reader.read_byte_size(skip_type(self.value_start));
        Ok(reader.read_character(self.value_start + 2, length))
    }

    pub fn as_string(&self) -> Result<String> {
        let t = DataType::try_from(self.data[self.value_start])?;
        match t {
            DataType::CharacterList => {
                let reader = DataVecReader::new_from_slice(self.data);
                let length = reader.read_size(skip_type(self.value_start));
                let items_start = skip_type_and_size(self.value_start);

                let mut strs: Vec<String> = Vec::with_capacity(length);

                for i in 0..length {
                    let item_start = skip_sizes(items_start, i);
                    let item_ref = reader.read_size(item_start);

                    let char_length = reader.read_byte_size(skip_type(item_ref));
                    let char_start = skip_type_and_byte_size(item_ref);

                    strs.push(reader.read_character(char_start, char_length));
                }

                Ok(strs.join(""))
            }
            DataType::Character => self.char_string(),
            DataType::Symbol | DataType::Expression | DataType::ExternalMethod => {
                let reader = DataVecReader::new_from_slice(self.data);
                let symbol_value = reader.read_size(skip_type(self.value_start));

                match self.symbol_table.and_then(|table| {
                    table
                        .iter()
                        .find(|(_k, v)| **v == symbol_value)
                        .and_then(|(k, _v)| Some(k.clone()))
                }) {
                    Some(s) => Ok(s),
                    None => Err("Symbol like value not in symbol table".into()),
                }
            }
            _ => Err(format!("Type ({}) not convertible to string.", t).into()),
        }
    }

    pub fn as_symbol(&self) -> Result<usize> {
        let t = DataType::try_from(self.data[self.value_start])?;
        if t == DataType::Symbol || t == DataType::Expression || t == DataType::ExternalMethod {
            let value_start = self.value_start + 1;
            let value_slice = &self.data[value_start..(value_start + SIZE_LENGTH)];

            let mut reader = Cursor::new(value_slice);
            let value = reader.read_u32::<LittleEndian>().unwrap() as usize;

            Ok(value)
        } else {
            Err("Not Symbol like type".into())
        }
    }

    pub fn is_range(&self) -> bool {
        self.is_of_type(DataType::Range)
    }

    fn range_flags(&self) -> Result<u8> {
        Ok(
            DataVecReader::new_from_slice(self.data).read_byte_size(skip_type(self.value_start))
                as u8,
        )
    }

    pub fn get_range_flags(&self) -> Result<u8> {
        if !self.is_range() {
            Err("Not Range type".into())
        } else {
            self.range_flags()
        }
    }

    pub fn get_range_start(&self) -> Result<ExpressionValueRef> {
        if !self.is_range() {
            Ok(ExpressionValueRef::unit())
        } else {
            let flags = self.range_flags()?;
            if flags & RANGE_OPEN_START == 0 {
                let r = DataVecReader::new_from_slice(self.data)
                    .read_size(skip_type_and_byte_size(self.value_start));
                ExpressionValueRef::new_with_start(self.data, self.symbol_table, r)
            } else {
                Ok(ExpressionValueRef::unit())
            }
        }
    }

    pub fn get_range_end(&self) -> Result<ExpressionValueRef> {
        if !self.is_range() {
            Ok(ExpressionValueRef::unit())
        } else {
            let flags = self.range_flags()?;
            if flags & RANGE_OPEN_END == 0 {
                // we have an end value
                let offset = if flags & RANGE_OPEN_START == RANGE_OPEN_START {
                    // we have no start value
                    // don't skip extra size value
                    skip_type_and_byte_size(self.value_start)
                } else {
                    // have start value, skip extra size value
                    skip_type_byte_size(self.value_start)
                };

                let r = DataVecReader::new_from_slice(self.data).read_size(offset);
                ExpressionValueRef::new_with_start(self.data, self.symbol_table, r)
            } else {
                Ok(ExpressionValueRef::unit())
            }
        }
    }

    pub fn get_range_step(&self) -> Result<ExpressionValueRef> {
        if !self.is_range() {
            Ok(ExpressionValueRef::unit())
        } else {
            let flags = self.range_flags()?;
            if flags & RANGE_HAS_STEP == RANGE_HAS_STEP {
                // we have range value

                // range value can be in one of three positions
                // base on whether there are start and end values
                let have_start = flags & RANGE_OPEN_START == 0;
                let have_end = flags & RANGE_OPEN_END == 0;

                let sizes_to_skip = match (have_start, have_end) {
                    (true, true) => 2,
                    (true, false) | (false, true) => 1,
                    (false, false) => 0,
                };

                let r = DataVecReader::new_from_slice(self.data).read_size(
                    skip_type_and_bytes_and_sizes(self.value_start, 1, sizes_to_skip),
                );
                ExpressionValueRef::new_with_start(self.data, self.symbol_table, r)
            } else {
                Ok(ExpressionValueRef::unit())
            }
        }
    }

    pub fn is_pair(&self) -> bool {
        self.is_of_type(DataType::Pair)
    }

    pub fn get_pair_left(&self) -> Result<ExpressionValueRef> {
        if !self.is_pair() {
            Ok(ExpressionValueRef::unit())
        } else {
            let r = DataVecReader::new_from_slice(self.data).read_size(skip_type(self.value_start));
            ExpressionValueRef::new_with_start(self.data, self.symbol_table, r)
        }
    }

    pub fn get_pair_right(&self) -> Result<ExpressionValueRef> {
        if !self.is_pair() {
            Ok(ExpressionValueRef::unit())
        } else {
            let r = DataVecReader::new_from_slice(self.data)
                .read_size(skip_type_and_size(self.value_start));
            ExpressionValueRef::new_with_start(self.data, self.symbol_table, r)
        }
    }

    pub fn is_partial(&self) -> bool {
        self.is_of_type(DataType::Partial)
    }

    pub fn get_partial_base(&self) -> Result<ExpressionValueRef> {
        if !self.is_partial() {
            Ok(ExpressionValueRef::unit())
        } else {
            let r = DataVecReader::new_from_slice(self.data).read_size(skip_type(self.value_start));
            ExpressionValueRef::new_with_start(self.data, self.symbol_table, r)
        }
    }

    pub fn get_partial_value(&self) -> Result<ExpressionValueRef> {
        if !self.is_partial() {
            Ok(ExpressionValueRef::unit())
        } else {
            let r = DataVecReader::new_from_slice(self.data)
                .read_size(skip_type_and_size(self.value_start));
            ExpressionValueRef::new_with_start(self.data, self.symbol_table, r)
        }
    }

    pub fn is_slice(&self) -> bool {
        self.is_of_type(DataType::Slice)
    }

    pub fn get_slice_source(&self) -> Result<ExpressionValueRef> {
        if !self.is_slice() {
            Ok(ExpressionValueRef::unit())
        } else {
            let r = DataVecReader::new_from_slice(self.data).read_size(skip_type(self.value_start));
            ExpressionValueRef::new_with_start(self.data, self.symbol_table, r)
        }
    }

    pub fn get_slice_range(&self) -> Result<ExpressionValueRef> {
        if !self.is_slice() {
            Ok(ExpressionValueRef::unit())
        } else {
            let r = DataVecReader::new_from_slice(self.data)
                .read_size(skip_type_and_size(self.value_start));
            ExpressionValueRef::new_with_start(self.data, self.symbol_table, r)
        }
    }

    pub fn is_link(&self) -> bool {
        self.is_of_type(DataType::Link)
    }

    pub fn get_link_head(&self) -> Result<ExpressionValueRef> {
        if !self.is_link() {
            Ok(ExpressionValueRef::unit())
        } else {
            let r = DataVecReader::new_from_slice(self.data).read_size(skip_type(self.value_start));
            ExpressionValueRef::new_with_start(self.data, self.symbol_table, r)
        }
    }

    pub fn get_link_value(&self) -> Result<ExpressionValueRef> {
        if !self.is_link() {
            Ok(ExpressionValueRef::unit())
        } else {
            let r = DataVecReader::new_from_slice(self.data)
                .read_size(skip_type_and_size(self.value_start));
            ExpressionValueRef::new_with_start(self.data, self.symbol_table, r)
        }
    }

    pub fn get_link_next(&self) -> Result<ExpressionValueRef> {
        if !self.is_link() {
            Ok(ExpressionValueRef::unit())
        } else {
            let r = DataVecReader::new_from_slice(self.data)
                .read_size(skip_type_and_sizes(self.value_start, 2));
            ExpressionValueRef::new_with_start(self.data, self.symbol_table, r)
        }
    }

    pub fn is_list(&self) -> bool {
        self.is_of_type(DataType::List)
    }

    pub fn list_len(&self) -> Result<usize> {
        Ok(DataVecReader::new_from_slice(self.data).read_size(skip_type(self.value_start)))
    }

    pub fn get_list_item(&self, index: usize) -> Result<ExpressionValueRef> {
        if !self.is_list() {
            return Ok(ExpressionValueRef::unit());
        }

        let length_start = skip_type(self.value_start);
        let length_slice = &self.data[length_start..skip_size(length_start)];

        let mut reader = Cursor::new(length_slice);
        let length = reader.read_u32::<LittleEndian>().unwrap() as usize;

        if index >= length {
            return Ok(ExpressionValueRef::unit());
        }

        let values_start = skip_type_and_2_sizes(self.value_start);
        let offset = index * SIZE_LENGTH;

        let value_start = values_start + offset;

        let value_slice = &self.data[value_start..skip_size(value_start)];

        let mut reader = Cursor::new(value_slice);
        let item_ref = reader.read_u32::<LittleEndian>().unwrap() as usize;

        ExpressionValueRef::new_with_start(self.data, self.symbol_table, item_ref as usize)
    }

    pub fn get_list_item_by_key(&self, symbol: &str) -> Result<ExpressionValueRef> {
        match self
            .symbol_table
            .and_then(|table| table.get(symbol).and_then(|v| Some(*v)))
            .or_else(|| Some(hash_byte_slice(symbol.as_bytes())))
            .map(|hash| self.get_list_item_by_hash(hash))
            .unwrap_or(Ok(Some(ExpressionValueRef::unit())))
        {
            Err(e) => Err(e),
            Ok(x) => match x {
                Some(r) => Ok(r),
                None => Ok(ExpressionValueRef::unit()),
            },
        }
    }

    pub fn get_list_item_by_symbol(&self, symbol: usize) -> Result<ExpressionValueRef> {
        let x = match self.get_list_item_by_hash(symbol)? {
            Some(value) => value,
            None => match self
                .symbol_table
                .and_then(|table| table.iter().find(|(_k, v)| **v == symbol))
            {
                Some((key, _value)) => self
                    .get_list_item_by_hash(hash_byte_slice(key.as_bytes()))?
                    .unwrap_or(ExpressionValueRef::unit()),
                None => ExpressionValueRef::unit(),
            },
        };

        Ok(x)
    }

    fn get_list_item_by_hash(&self, hash: usize) -> Result<Option<ExpressionValueRef>> {
        let key_count_start = skip_type_and_size(self.value_start);
        if key_count_start >= self.data.len() {
            return Err("Value reaches out of data bounds.".into());
        }

        let key_count = DataVecReader::new_from_slice(self.data).read_size(key_count_start);

        let key_area_start = skip_size(key_count_start);
        let key_range = key_area_start..(skip_sizes(key_area_start, key_count));

        Ok(
            match get_value_with_hash(hash, self.data, key_count, &key_range)? {
                Some(value) => Some(ExpressionValueRef::new_with_start(
                    self.data,
                    self.symbol_table,
                    value,
                )?),
                None => None,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::byte_vec::DataVecWriter;
    use crate::data_type::DataType;
    use crate::value::range::RANGE_END_EXCLUSIVE;
    use crate::value::ExpressionValueRef;
    use crate::ExpressionValue;
    use std::convert::TryInto;

    #[test]
    fn new() {
        let v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let result = ExpressionValueRef::new(&v[3..8], None).unwrap();

        let actual = Vec::from(result.data);
        let expected = Vec::from(&v[3..8]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn new_with_zero_slice_results_in_error() {
        let v = vec![];
        assert!(ExpressionValueRef::new(&v[0..0], None).is_err());
    }

    #[test]
    fn new_with_value_beyond_slice_results_in_error() {
        let v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        assert!(ExpressionValueRef::new_with_start(&v[1..10], None, 20).is_err());
    }

    #[test]
    fn as_integer() {
        let v = ExpressionValue::from(ExpressionValue::integer(1000000000));
        let result = ExpressionValueRef::new(&v.get_data()[..], None).unwrap();

        assert_eq!(result.as_integer().unwrap(), 1000000000);
    }

    #[test]
    fn as_float() {
        let v = ExpressionValue::from(ExpressionValue::float(3.14));
        let result = ExpressionValueRef::new(&v.get_data()[..], None).unwrap();

        assert_eq!(result.as_float().unwrap(), 3.14);
    }

    #[test]
    fn make_unit() {
        let result = ExpressionValueRef::unit();

        assert!(result.is_unit());
    }

    #[test]
    fn is_unit_true() {
        let v = ExpressionValue::from(ExpressionValue::unit());
        let result = ExpressionValueRef::new(&v.get_data()[..], None).unwrap();

        assert!(result.is_unit());
    }

    #[test]
    fn is_unit_false() {
        let v = ExpressionValue::from(ExpressionValue::integer(1000000000));
        let result = ExpressionValueRef::new(&v.get_data()[..], None).unwrap();

        assert!(!result.is_unit());
    }

    #[test]
    fn as_string_char() {
        let v = ExpressionValue::from(ExpressionValue::character("ö̲".into()));

        let result = ExpressionValueRef::new(&v.get_data()[..], None).unwrap();

        assert_eq!(result.as_string().unwrap(), "ö̲");
    }

    #[test]
    fn as_string_character_list() {
        let v = ExpressionValue::from(ExpressionValue::character_list("z🐼ö̲".into()));

        let result = ExpressionValueRef::new_with_start(&v.get_data()[..], None, 16).unwrap();

        assert_eq!(result.as_string().unwrap(), "z🐼ö̲");
    }

    #[test]
    fn as_char_valid_ascii() {
        let v = ExpressionValue::from(ExpressionValue::character("z".into()));

        let result = ExpressionValueRef::new(&v.get_data()[..], None).unwrap();

        assert_eq!(result.as_char().unwrap(), 'z');
    }

    #[test]
    fn as_char_valid_unicode() {
        let v = ExpressionValue::from(ExpressionValue::character("🐼".into()));

        let result = ExpressionValueRef::new(&v.get_data()[..], None).unwrap();

        assert_eq!(result.as_char().unwrap(), '🐼');
    }

    #[test]
    fn as_char_invalid_grapheme() {
        let v = ExpressionValue::from(ExpressionValue::character("ö̲".into()));

        let result = ExpressionValueRef::new(&v.get_data()[..], None).unwrap();

        assert!(result.as_char().is_err());
    }

    #[test]
    fn as_char_not_character() {
        // need number that will put valid value for char length and also a value char value
        let v = ExpressionValue::from(ExpressionValue::integer(38403));

        let result = ExpressionValueRef::new(&v.get_data()[..], None).unwrap();

        assert!(result.as_char().is_err());
    }

    #[test]
    fn symbol_as_string() {
        let v = ExpressionValue::from(ExpressionValue::symbol("my_symbol"));

        let result =
            ExpressionValueRef::new(&v.get_data()[..], Some(&v.get_symbol_table())).unwrap();

        assert_eq!(result.as_string().unwrap(), String::from("my_symbol"));
    }

    #[test]
    fn expression_as_string() {
        let v = ExpressionValue::from(ExpressionValue::expression("expression"));

        let result =
            ExpressionValueRef::new(&v.get_data()[..], Some(&v.get_symbol_table())).unwrap();

        assert_eq!(result.as_string().unwrap(), String::from("expression"));
    }

    #[test]
    fn external_method_as_string() {
        let v = ExpressionValue::from(ExpressionValue::external_method("external_method"));

        let result =
            ExpressionValueRef::new(&v.get_data()[..], Some(&v.get_symbol_table())).unwrap();

        assert_eq!(result.as_string().unwrap(), String::from("external_method"));
    }

    #[test]
    fn as_symbol() {
        let mut v = vec![];
        DataVecWriter::new(&mut v)
            .push_data_type(DataType::Symbol)
            .push_size(1000);

        let result = ExpressionValueRef::new(&v[..], None).unwrap();

        assert_eq!(result.as_symbol().unwrap(), 1000);
    }

    #[test]
    fn expression_as_symbol() {
        let v = ExpressionValue::from(ExpressionValue::expression("bear"));

        let result = ExpressionValueRef::new(&v.get_data()[..], None).unwrap();

        assert_eq!(result.as_symbol().unwrap(), 1);
    }

    #[test]
    fn external_method_as_symbol() {
        let v = ExpressionValue::from(ExpressionValue::external_method("bear"));

        let result = ExpressionValueRef::new(&v.get_data()[..], None).unwrap();

        assert_eq!(result.as_symbol().unwrap(), 1);
    }

    #[test]
    fn is_range() {
        let v = ExpressionValue::from(ExpressionValue::integer_range(Some(10), Some(20)));
        let r = ExpressionValueRef::new_with_start(&v.get_data(), None, 10).unwrap();

        assert!(r.is_range());
    }

    #[test]
    fn get_range_flags() {
        let v =
            ExpressionValue::from(ExpressionValue::integer_range(Some(10), Some(20)).exclude_end());
        let r = ExpressionValueRef::new_with_start(&v.get_data(), None, 10).unwrap();

        assert_eq!(r.get_range_flags().unwrap(), RANGE_END_EXCLUSIVE);
    }

    #[test]
    fn get_range_min() {
        let v =
            ExpressionValue::from(ExpressionValue::integer_range(Some(10), Some(20)).exclude_end());
        let r = ExpressionValueRef::new_with_start(&v.get_data(), None, 10).unwrap();

        assert_eq!(r.get_range_start().unwrap().as_integer().unwrap(), 10);
    }

    #[test]
    fn get_range_min_no_min() {
        let v = ExpressionValue::from(ExpressionValue::integer_range(None, Some(20)).exclude_end());
        let r = ExpressionValueRef::new_with_start(&v.get_data(), None, 5).unwrap();

        assert!(r.get_range_start().unwrap().is_unit());
    }

    #[test]
    fn get_range_max() {
        let v =
            ExpressionValue::from(ExpressionValue::integer_range(Some(10), Some(20)).exclude_end());
        let r = ExpressionValueRef::new_with_start(&v.get_data(), None, 10).unwrap();

        assert_eq!(r.get_range_end().unwrap().as_integer().unwrap(), 20);
    }

    #[test]
    fn get_range_max_when_no_min() {
        let v = ExpressionValue::from(ExpressionValue::integer_range(None, Some(20)).exclude_end());
        let r = ExpressionValueRef::new_with_start(&v.get_data(), None, 5).unwrap();

        assert_eq!(r.get_range_end().unwrap().as_integer().unwrap(), 20);
    }

    #[test]
    fn get_range_max_no_max() {
        let v = ExpressionValue::from(ExpressionValue::integer_range(Some(10), None));
        let r = ExpressionValueRef::new_with_start(&v.get_data(), None, 5).unwrap();

        assert!(r.get_range_end().unwrap().is_unit());
    }

    #[test]
    fn get_range_step() {
        let v = ExpressionValue::from(
            ExpressionValue::integer_range(Some(10), Some(20))
                .exclude_end()
                .with_step(ExpressionValue::integer(3)),
        );
        let r = ExpressionValueRef::new_with_start(&v.get_data(), None, 15).unwrap();

        assert_eq!(r.get_range_step().unwrap().as_integer().unwrap(), 3);
    }

    #[test]
    fn get_range_step_when_no_min() {
        let v = ExpressionValue::from(
            ExpressionValue::integer_range(None, Some(20)).with_step(ExpressionValue::integer(3)),
        );
        let r = ExpressionValueRef::new_with_start(&v.get_data(), None, 10).unwrap();

        assert_eq!(r.get_range_step().unwrap().as_integer().unwrap(), 3);
    }

    #[test]
    fn get_range_step_when_no_max() {
        let v = ExpressionValue::from(
            ExpressionValue::integer_range(Some(10), None).with_step(ExpressionValue::integer(3)),
        );
        let r = ExpressionValueRef::new_with_start(&v.get_data(), None, 10).unwrap();

        assert_eq!(r.get_range_step().unwrap().as_integer().unwrap(), 3);
    }

    #[test]
    fn get_range_step_when_no_min_or_max() {
        let v = ExpressionValue::from(
            ExpressionValue::integer_range(None, None).with_step(ExpressionValue::integer(3)),
        );
        let r = ExpressionValueRef::new_with_start(&v.get_data(), None, 5).unwrap();

        assert_eq!(r.get_range_step().unwrap().as_integer().unwrap(), 3);
    }

    #[test]
    fn get_range_step_no_step() {
        let v =
            ExpressionValue::from(ExpressionValue::integer_range(Some(10), Some(20)).exclude_end());
        let r = ExpressionValueRef::new_with_start(&v.get_data(), None, 10).unwrap();

        assert!(r.get_range_step().unwrap().is_unit());
    }

    #[test]
    fn get_range_min_when_not_range() {
        let v = ExpressionValue::from(ExpressionValue::integer(10));
        let r = ExpressionValueRef::new_with_start(&v.get_data(), None, 0).unwrap();

        assert!(r.get_range_start().unwrap().is_unit());
    }

    #[test]
    fn get_range_max_when_not_range() {
        let v = ExpressionValue::from(ExpressionValue::integer(10));
        let r = ExpressionValueRef::new_with_start(&v.get_data(), None, 0).unwrap();

        assert!(r.get_range_start().unwrap().is_unit());
    }

    #[test]
    fn is_pair() {
        let v = ExpressionValue::from(ExpressionValue::pair(
            ExpressionValue::symbol("cat"),
            ExpressionValue::integer(20),
        ));

        let r = ExpressionValueRef::new_with_start(&v.get_data(), None, 10).unwrap();

        assert!(r.is_pair());
    }

    #[test]
    fn get_pair_left() {
        let v = ExpressionValue::from(ExpressionValue::pair(
            ExpressionValue::symbol("cat"),
            ExpressionValue::integer(20),
        ));

        let r = ExpressionValueRef::new_with_start(&v.get_data(), None, 10).unwrap();

        assert_eq!(r.get_pair_left().unwrap().as_symbol().unwrap(), 1);
    }

    #[test]
    fn get_pair_right() {
        let v = ExpressionValue::from(ExpressionValue::pair(
            ExpressionValue::symbol("cat"),
            ExpressionValue::integer(20),
        ));

        let r = ExpressionValueRef::new_with_start(&v.get_data(), None, 10).unwrap();

        assert_eq!(r.get_pair_right().unwrap().as_integer().unwrap(), 20);
    }

    #[test]
    fn get_pair_left_when_not_pair() {
        let v = ExpressionValue::from(ExpressionValue::integer(20));

        let r = ExpressionValueRef::new_with_start(&v.get_data(), None, 0).unwrap();

        assert!(r.get_pair_left().unwrap().is_unit());
    }

    #[test]
    fn get_pair_right_when_not_pair() {
        let v = ExpressionValue::from(ExpressionValue::integer(20));

        let r = ExpressionValueRef::new_with_start(&v.get_data(), None, 0).unwrap();

        assert!(r.get_pair_right().unwrap().is_unit());
    }

    #[test]
    fn is_partial() {
        let v = ExpressionValue::from(ExpressionValue::partial_expression(
            "my_expression",
            ExpressionValue::integer(20),
        ));

        let r = ExpressionValueRef::new_with_start(&v.get_data(), None, 10).unwrap();

        assert!(r.is_partial());
    }

    #[test]
    fn get_partial_left() {
        let v = ExpressionValue::from(ExpressionValue::partial_expression(
            "my_expression",
            ExpressionValue::integer(20),
        ));

        let r = ExpressionValueRef::new_with_start(&v.get_data(), None, 10).unwrap();

        assert_eq!(r.get_partial_base().unwrap().as_symbol().unwrap(), 1);
    }

    #[test]
    fn get_partial_right() {
        let v = ExpressionValue::from(ExpressionValue::partial_expression(
            "my_expression",
            ExpressionValue::integer(20),
        ));

        let r = ExpressionValueRef::new_with_start(&v.get_data(), None, 10).unwrap();

        assert_eq!(r.get_partial_value().unwrap().as_integer().unwrap(), 20);
    }

    #[test]
    fn get_partial_left_when_not_partial() {
        let v = ExpressionValue::from(ExpressionValue::integer(20));

        let r = ExpressionValueRef::new_with_start(&v.get_data(), None, 0).unwrap();

        assert!(r.get_partial_base().unwrap().is_unit());
    }

    #[test]
    fn get_partial_right_when_not_partial() {
        let v = ExpressionValue::from(ExpressionValue::integer(20));

        let r = ExpressionValueRef::new_with_start(&v.get_data(), None, 0).unwrap();

        assert!(r.get_partial_value().unwrap().is_unit());
    }

    #[test]
    fn is_list() {
        let v = ExpressionValue::from(
            ExpressionValue::list()
                .add(ExpressionValue::integer(1000))
                .add(ExpressionValue::integer(2000))
                .add(ExpressionValue::integer(3000)),
        );

        let result = ExpressionValueRef::new_with_start(&v.get_data()[..], None, 15).unwrap();

        assert!(result.is_list());
    }

    #[test]
    fn is_not_list() {
        let v = ExpressionValue::from(ExpressionValue::integer(1000000000));
        let result = ExpressionValueRef::new(&v.get_data()[..], None).unwrap();

        assert!(!result.is_list());
    }

    #[test]
    fn get_list_item() {
        let v = ExpressionValue::from(
            ExpressionValue::list()
                .add(ExpressionValue::integer(1000))
                .add(ExpressionValue::integer(2000))
                .add(ExpressionValue::integer(3000)),
        );

        let result = ExpressionValueRef::new_with_start(&v.get_data()[..], None, 15).unwrap();

        assert_eq!(result.get_list_item(0).unwrap().as_integer().unwrap(), 1000);
        assert_eq!(result.get_list_item(1).unwrap().as_integer().unwrap(), 2000);
        assert_eq!(result.get_list_item(2).unwrap().as_integer().unwrap(), 3000);
    }

    #[test]
    fn get_list_item_with_symbol_symbol_keys() {
        let v = ExpressionValue::from(
            ExpressionValue::list()
                .add(ExpressionValue::pair(
                    ExpressionValue::symbol("bear"),
                    ExpressionValue::integer(1000),
                ))
                .add(ExpressionValue::pair(
                    ExpressionValue::symbol("cat"),
                    ExpressionValue::integer(2000),
                ))
                .add(ExpressionValue::pair(
                    ExpressionValue::symbol("dog"),
                    ExpressionValue::integer(3000),
                )),
        );

        let result =
            ExpressionValueRef::new_with_start(&v.get_data()[..], Some(&v.get_symbol_table()), 57)
                .unwrap();

        let bear_symbol = result.get_symbol_value("bear").unwrap();
        let bear_item = result.get_list_item_by_symbol(bear_symbol).unwrap();

        assert_eq!(bear_item.get_pair_left().unwrap().as_symbol().unwrap(), 1);
        assert_eq!(
            bear_item.get_pair_right().unwrap().as_integer().unwrap(),
            1000
        );

        let cat_symbol = result.get_symbol_value("cat").unwrap();
        let cat_item = result.get_list_item_by_symbol(cat_symbol).unwrap();

        assert_eq!(cat_item.get_pair_left().unwrap().as_symbol().unwrap(), 2);
        assert_eq!(
            cat_item.get_pair_right().unwrap().as_integer().unwrap(),
            2000
        );

        let dog_symbol = result.get_symbol_value("dog").unwrap();
        let dog_item = result.get_list_item_by_symbol(dog_symbol).unwrap();

        assert_eq!(dog_item.get_pair_left().unwrap().as_symbol().unwrap(), 3);
        assert_eq!(
            dog_item.get_pair_right().unwrap().as_integer().unwrap(),
            3000
        );
    }

    #[test]
    fn get_list_item_with_symbol_character_list_keys() {
        let v = ExpressionValue::from(
            ExpressionValue::list()
                .add(ExpressionValue::pair(
                    ExpressionValue::character_list("bear".into()),
                    ExpressionValue::integer(1000),
                ))
                .add(ExpressionValue::pair(
                    ExpressionValue::character_list("cat".into()),
                    ExpressionValue::integer(2000),
                ))
                .add(ExpressionValue::pair(
                    ExpressionValue::character_list("dog".into()),
                    ExpressionValue::integer(3000),
                )),
        );

        let mut symbol_table = v.get_symbol_table().clone();
        symbol_table.insert("bear".into(), 1);
        symbol_table.insert("cat".into(), 2);
        symbol_table.insert("dog".into(), 3);

        let result =
            ExpressionValueRef::new_with_start(&v.get_data()[..], Some(&symbol_table), 127)
                .unwrap();

        let bear_symbol = result.get_symbol_value("bear").unwrap();
        let bear_item = result.get_list_item_by_symbol(bear_symbol).unwrap();

        assert_eq!(
            bear_item.get_pair_left().unwrap().as_string().unwrap(),
            "bear".to_string()
        );
        assert_eq!(
            bear_item.get_pair_right().unwrap().as_integer().unwrap(),
            1000
        );

        let cat_symbol = result.get_symbol_value("cat").unwrap();
        let cat_item = result.get_list_item_by_symbol(cat_symbol).unwrap();

        assert_eq!(
            cat_item.get_pair_left().unwrap().as_string().unwrap(),
            "cat".to_string()
        );
        assert_eq!(
            cat_item.get_pair_right().unwrap().as_integer().unwrap(),
            2000
        );

        let dog_symbol = result.get_symbol_value("dog").unwrap();
        let dog_item = result.get_list_item_by_symbol(dog_symbol).unwrap();

        assert_eq!(
            dog_item.get_pair_left().unwrap().as_string().unwrap(),
            "dog".to_string()
        );
        assert_eq!(
            dog_item.get_pair_right().unwrap().as_integer().unwrap(),
            3000
        );
    }

    #[test]
    fn get_list_item_with_symbol_that_does_exist_character_list_keys() {
        let v = ExpressionValue::from(
            ExpressionValue::list()
                .add(ExpressionValue::pair(
                    ExpressionValue::character_list("bear".into()),
                    ExpressionValue::integer(1000),
                ))
                .add(ExpressionValue::pair(
                    ExpressionValue::character_list("cat".into()),
                    ExpressionValue::integer(2000),
                ))
                .add(ExpressionValue::pair(
                    ExpressionValue::character_list("dog".into()),
                    ExpressionValue::integer(3000),
                )),
        );

        let result =
            ExpressionValueRef::new_with_start(&v.get_data()[..], Some(&v.get_symbol_table()), 127)
                .unwrap();

        let item = result.get_list_item_by_symbol(1).unwrap();

        assert!(item.is_unit());
    }

    #[test]
    fn get_list_item_with_symbol_that_does_not_exist() {
        let v = ExpressionValue::from(
            ExpressionValue::list()
                .add(ExpressionValue::pair(
                    ExpressionValue::symbol("bear"),
                    ExpressionValue::integer(1000),
                ))
                .add(ExpressionValue::pair(
                    ExpressionValue::symbol("cat"),
                    ExpressionValue::integer(2000),
                ))
                .add(ExpressionValue::pair(
                    ExpressionValue::symbol("dog"),
                    ExpressionValue::integer(3000),
                )),
        );

        let result =
            ExpressionValueRef::new_with_start(&v.get_data()[..], Some(&v.get_symbol_table()), 57)
                .unwrap();

        let item = result.get_list_item_by_symbol(10).unwrap();

        assert!(item.is_unit());
    }

    #[test]
    fn get_list_item_with_string_symbol_keys() {
        let v = ExpressionValue::from(
            ExpressionValue::list()
                .add(ExpressionValue::pair(
                    ExpressionValue::symbol("bear"),
                    ExpressionValue::integer(1000),
                ))
                .add(ExpressionValue::pair(
                    ExpressionValue::symbol("cat"),
                    ExpressionValue::integer(2000),
                ))
                .add(ExpressionValue::pair(
                    ExpressionValue::symbol("dog"),
                    ExpressionValue::integer(3000),
                )),
        );

        let result =
            ExpressionValueRef::new_with_start(&v.get_data()[..], Some(&v.get_symbol_table()), 57)
                .unwrap();

        let bear_item = result.get_list_item_by_key("bear").unwrap();

        assert_eq!(bear_item.get_pair_left().unwrap().as_symbol().unwrap(), 1);
        assert_eq!(
            bear_item.get_pair_right().unwrap().as_integer().unwrap(),
            1000
        );

        let cat_item = result.get_list_item_by_key("cat").unwrap();

        assert_eq!(cat_item.get_pair_left().unwrap().as_symbol().unwrap(), 2);
        assert_eq!(
            cat_item.get_pair_right().unwrap().as_integer().unwrap(),
            2000
        );

        let dog_item = result.get_list_item_by_key("dog").unwrap();

        assert_eq!(dog_item.get_pair_left().unwrap().as_symbol().unwrap(), 3);
        assert_eq!(
            dog_item.get_pair_right().unwrap().as_integer().unwrap(),
            3000
        );
    }

    #[test]
    fn get_list_item_with_string_that_does_not_exist_symbol_keys() {
        let v = ExpressionValue::from(
            ExpressionValue::list()
                .add(ExpressionValue::pair(
                    ExpressionValue::symbol("bear"),
                    ExpressionValue::integer(1000),
                ))
                .add(ExpressionValue::pair(
                    ExpressionValue::symbol("cat"),
                    ExpressionValue::integer(2000),
                ))
                .add(ExpressionValue::pair(
                    ExpressionValue::symbol("dog"),
                    ExpressionValue::integer(3000),
                )),
        );

        let result =
            ExpressionValueRef::new_with_start(&v.get_data()[..], Some(&v.get_symbol_table()), 57)
                .unwrap();

        let item = result.get_list_item_by_key("panda").unwrap();

        assert!(item.is_unit());
    }

    #[test]
    fn get_list_item_with_string_from_character_list_keys() {
        let v = ExpressionValue::from(
            ExpressionValue::list()
                .add(ExpressionValue::pair(
                    ExpressionValue::character_list("bear".into()),
                    ExpressionValue::integer(1000),
                ))
                .add(ExpressionValue::pair(
                    ExpressionValue::character_list("cat".into()),
                    ExpressionValue::integer(2000),
                ))
                .add(ExpressionValue::pair(
                    ExpressionValue::character_list("dog".into()),
                    ExpressionValue::integer(3000),
                )),
        );

        let result =
            ExpressionValueRef::new_with_start(&v.get_data()[..], Some(&v.get_symbol_table()), 127)
                .unwrap();

        let bear_item = result.get_list_item_by_key("bear").unwrap();

        assert_eq!(
            bear_item.get_pair_left().unwrap().as_string().unwrap(),
            "bear".to_string()
        );
        assert_eq!(
            bear_item.get_pair_right().unwrap().as_integer().unwrap(),
            1000
        );

        let cat_item = result.get_list_item_by_key("cat").unwrap();

        assert_eq!(
            cat_item.get_pair_left().unwrap().as_string().unwrap(),
            "cat".to_string()
        );
        assert_eq!(
            cat_item.get_pair_right().unwrap().as_integer().unwrap(),
            2000
        );

        let dog_item = result.get_list_item_by_key("dog").unwrap();

        assert_eq!(
            dog_item.get_pair_left().unwrap().as_string().unwrap(),
            "dog".to_string()
        );
        assert_eq!(
            dog_item.get_pair_right().unwrap().as_integer().unwrap(),
            3000
        );
    }

    #[test]
    fn get_list_item_with_string_that_does_not_exist_character_list_keys() {
        let v = ExpressionValue::from(
            ExpressionValue::list()
                .add(ExpressionValue::pair(
                    ExpressionValue::character_list("bear".into()),
                    ExpressionValue::integer(1000),
                ))
                .add(ExpressionValue::pair(
                    ExpressionValue::character_list("cat".into()),
                    ExpressionValue::integer(2000),
                ))
                .add(ExpressionValue::pair(
                    ExpressionValue::character_list("dog".into()),
                    ExpressionValue::integer(3000),
                )),
        );

        let result =
            ExpressionValueRef::new_with_start(&v.get_data()[..], Some(&v.get_symbol_table()), 127)
                .unwrap();

        let item = result.get_list_item_by_key("panda").unwrap();

        assert!(item.is_unit());
    }

    #[test]
    fn get_list_length() {
        let v = ExpressionValue::from(
            ExpressionValue::list()
                .add(ExpressionValue::integer(1000))
                .add(ExpressionValue::integer(2000))
                .add(ExpressionValue::integer(3000)),
        );

        let result = ExpressionValueRef::new_with_start(&v.get_data()[..], None, 15).unwrap();

        assert_eq!(result.list_len().unwrap(), 3);
    }

    #[test]
    fn get_list_item_out_of_bounds() {
        let v = ExpressionValue::from(
            ExpressionValue::list()
                .add(ExpressionValue::integer(1000))
                .add(ExpressionValue::integer(2000))
                .add(ExpressionValue::integer(3000)),
        );

        let result = ExpressionValueRef::new_with_start(&v.get_data()[..], None, 15).unwrap();

        assert!(result.get_list_item(4).unwrap().is_unit())
    }

    #[test]
    fn get_list_item_with_not_list() {
        let v = vec![DataType::Integer.try_into().unwrap(), 10];

        let result = ExpressionValueRef::new_with_start(&v[..], None, 0).unwrap();

        assert!(result.get_list_item(0).unwrap().is_unit())
    }
}
