use std::convert::TryInto;
use std::io::Cursor;
use std::mem;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::{DataType, Result};

pub(crate) type SizeType = u32;
pub(crate) type NumberType = i32;
pub(crate) type FloatType = f32;
pub(crate) type CharType = u32;

pub(crate) const DATA_TYPE_LENGTH: usize = mem::size_of::<DataType>();
pub(crate) const SIZE_LENGTH: usize = mem::size_of::<SizeType>();
pub(crate) const BYTE_SIZE_LENGTH: usize = mem::size_of::<u8>();
pub(crate) const INTEGER_LENGTH: usize = mem::size_of::<NumberType>();
pub(crate) const FLOAT_LENGTH: usize = mem::size_of::<FloatType>();
pub(crate) const CHAR_LENGTH: usize = mem::size_of::<CharType>();

pub fn skip_type(i: usize) -> usize {
    i + DATA_TYPE_LENGTH
}

pub fn skip_byte_size(i: usize) -> usize {
    skip_byte_sizes(i, 1)
}

pub fn skip_byte_sizes(i: usize, count: usize) -> usize {
    i + BYTE_SIZE_LENGTH * count
}

pub fn skip_type_and_byte_size(i: usize) -> usize {
    skip_byte_size(skip_type(i))
}

pub fn skip_sizes(i: usize, s: usize) -> usize {
    i + s * SIZE_LENGTH
}

pub fn skip_size(i: usize) -> usize {
    skip_sizes(i, 1)
}

pub fn skip_type_and_sizes(i: usize, s: usize) -> usize {
    skip_sizes(skip_type(i), s)
}

pub fn skip_type_and_size(i: usize) -> usize {
    skip_type_and_sizes(i, 1)
}

pub fn skip_type_and_2_sizes(i: usize) -> usize {
    skip_type_and_sizes(i, 2)
}

pub fn skip_type_and_bytes_and_sizes(i: usize, b: usize, s: usize) -> usize {
    skip_type(skip_byte_sizes(skip_sizes(i, s), b))
}

pub fn skip_type_byte_size(i: usize) -> usize {
    skip_type_and_bytes_and_sizes(i, 1, 1)
}

pub fn size_to_bytes(s: usize) -> [u8; SIZE_LENGTH] {
    let mut data = [0; SIZE_LENGTH];
    data.as_mut()
        .write_u32::<LittleEndian>(s as SizeType)
        // Appropriately sized array is provided, should never fail
        .expect("Could not write data to byte array.");

    data
}

pub fn read_size(data: &[u8]) -> Result<usize> {
    if data.len() < SIZE_LENGTH {
        return Err("Not enough data to read size from slice.".into());
    }

    let slice = &data[0..SIZE_LENGTH];
    let mut reader = Cursor::new(slice);
    match reader.read_u32::<LittleEndian>() {
        Err(e) => Err(e.to_string().into()),
        Ok(v) => Ok(v as usize),
    }
}

pub fn read_byte_size(data: &[u8]) -> Result<u8> {
    if data.len() < 1 {
        return Err("Not enough data to read byte size from slice.".into());
    }

    Ok(data[0])
}

pub fn read_integer(data: &[u8]) -> Result<i32> {
    if data.len() < SIZE_LENGTH {
        return Err("Not enough data to read size from slice.".into());
    }

    let slice = &data[0..INTEGER_LENGTH];
    let mut reader = Cursor::new(slice);
    match reader.read_i32::<LittleEndian>() {
        Err(e) => Err(e.to_string().into()),
        Ok(v) => Ok(v),
    }
}

pub fn read_float(data: &[u8]) -> Result<f32> {
    if data.len() < SIZE_LENGTH {
        return Err("Not enough data to read size from slice.".into());
    }

    let slice = &data[0..FLOAT_LENGTH];
    let mut reader = Cursor::new(slice);
    match reader.read_f32::<LittleEndian>() {
        Err(e) => Err(e.to_string().into()),
        Ok(v) => Ok(v),
    }
}

pub fn characters_to_bytes(s: &str) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::with_capacity(s.len() * mem::size_of::<u8>());

    let bytes = s.as_bytes();

    for d in bytes.iter() {
        result.push(*d);
    }

    result
}

pub fn character_type_to_bytes(s: &str) -> Vec<u8> {
    let mut result = characters_to_bytes(s);
    result.insert(0, DataType::Character.try_into().unwrap());
    result.insert(1, s.len() as u8);

    result
}

pub fn number_type_to_bytes(num: i32) -> [u8; DATA_TYPE_LENGTH + INTEGER_LENGTH] {
    let mut data = [0; DATA_TYPE_LENGTH + INTEGER_LENGTH];

    data[0] = DataType::Integer.try_into().unwrap();
    data[DATA_TYPE_LENGTH..(DATA_TYPE_LENGTH + INTEGER_LENGTH)]
        .as_mut()
        .write_i32::<LittleEndian>(num)
        // Appropriately sized array is provided, should never fail
        .expect("Could not write data to byte array.");

    data
}

pub fn float_type_to_bytes(num: f32) -> [u8; DATA_TYPE_LENGTH + FLOAT_LENGTH] {
    let mut data = [0; DATA_TYPE_LENGTH + INTEGER_LENGTH];

    data[0] = DataType::Float.try_into().unwrap();
    data[DATA_TYPE_LENGTH..(DATA_TYPE_LENGTH + INTEGER_LENGTH)]
        .as_mut()
        .write_f32::<LittleEndian>(num)
        // Appropriately sized array is provided, should never fail
        .expect("Could not write data to byte array.");

    data
}

pub fn size_type_to_bytes(data_type: DataType, s: usize) -> [u8; DATA_TYPE_LENGTH + SIZE_LENGTH] {
    let mut data = [0; DATA_TYPE_LENGTH + SIZE_LENGTH];

    data[0] = data_type.try_into().unwrap();
    data[1..]
        .as_mut()
        .write_u32::<LittleEndian>(s as SizeType)
        // Appropriately sized array is provided, should never fail
        .expect("Could not write data to byte array.");

    data
}

// temporary until range serialization is done
pub fn range_type_to_bytes(
    flags: u8,
    start: Option<usize>,
    end: Option<usize>,
    step: Option<usize>,
) -> [u8; DATA_TYPE_LENGTH + BYTE_SIZE_LENGTH + 3 * SIZE_LENGTH] {
    let mut data = [0; DATA_TYPE_LENGTH + BYTE_SIZE_LENGTH + 3 * SIZE_LENGTH];
    data[0] = DataType::Range.try_into().unwrap();
    data[1] = flags;

    let mut data_cursor = 2;

    if start.is_some() {
        data[data_cursor..(data_cursor + SIZE_LENGTH)]
            .clone_from_slice(&size_to_bytes(start.unwrap()));
        data_cursor += SIZE_LENGTH;
    }

    if end.is_some() {
        data[data_cursor..(data_cursor + SIZE_LENGTH)]
            .clone_from_slice(&size_to_bytes(end.unwrap()));
        data_cursor += SIZE_LENGTH;
    }

    if step.is_some() {
        data[data_cursor..(data_cursor + SIZE_LENGTH)]
            .clone_from_slice(&size_to_bytes(step.unwrap()));
    }

    data
}

pub fn two_size_type_to_bytes(
    data_type: DataType,
    s1: usize,
    s2: usize,
) -> [u8; DATA_TYPE_LENGTH + 2 * SIZE_LENGTH] {
    let mut data = [0; DATA_TYPE_LENGTH + 2 * SIZE_LENGTH];
    data[0] = data_type.try_into().unwrap();

    let sizes = two_sizes_to_bytes(s1, s2);
    data[1..].clone_from_slice(&sizes);

    data
}

pub fn three_size_type_to_bytes(
    data_type: DataType,
    s1: usize,
    s2: usize,
    s3: usize,
) -> [u8; DATA_TYPE_LENGTH + 3 * SIZE_LENGTH] {
    let mut data = [0; DATA_TYPE_LENGTH + 3 * SIZE_LENGTH];
    data[0] = data_type.try_into().unwrap();

    let sizes = three_sizes_to_bytes(s1, s2, s3);
    data[1..].clone_from_slice(&sizes);

    data
}

pub fn two_sizes_to_bytes(s1: usize, s2: usize) -> [u8; 2 * SIZE_LENGTH] {
    let mut data = [0; 2 * SIZE_LENGTH];

    data[0..SIZE_LENGTH]
        .as_mut()
        .write_u32::<LittleEndian>(s1 as SizeType)
        // Appropriately sized array is provided, should never fail
        .expect("Could not write data to byte array.");

    data[SIZE_LENGTH..(2 * SIZE_LENGTH)]
        .as_mut()
        .write_u32::<LittleEndian>(s2 as SizeType)
        // Appropriately sized array is provided, should never fail
        .expect("Could not write data to byte array.");

    data
}

pub fn three_sizes_to_bytes(s1: usize, s2: usize, s3: usize) -> [u8; 3 * SIZE_LENGTH] {
    let mut data = [0; 3 * SIZE_LENGTH];

    data[0..SIZE_LENGTH]
        .as_mut()
        .write_u32::<LittleEndian>(s1 as SizeType)
        // Appropriately sized array is provided, should never fail
        .expect("Could not write data to byte array.");

    data[SIZE_LENGTH..(2 * SIZE_LENGTH)]
        .as_mut()
        .write_u32::<LittleEndian>(s2 as SizeType)
        // Appropriately sized array is provided, should never fail
        .expect("Could not write data to byte array.");

    data[(2 * SIZE_LENGTH)..(3 * SIZE_LENGTH)]
        .as_mut()
        .write_u32::<LittleEndian>(s3 as SizeType)
        // Appropriately sized array is provided, should never fail
        .expect("Could not write data to byte array.");

    data
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use crate::{
        characters_to_bytes, float_type_to_bytes, number_type_to_bytes, read_float, read_integer,
        read_size, size_to_bytes, size_type_to_bytes, skip_size, skip_sizes, skip_type,
        skip_type_and_2_sizes, skip_type_and_size, skip_type_and_sizes, two_size_type_to_bytes,
        DataType, DATA_TYPE_LENGTH, SIZE_LENGTH,
    };

    #[test]
    fn skip_data_type() {
        assert_eq!(skip_type(1), 1 + DATA_TYPE_LENGTH);
    }

    #[test]
    fn skip_sizes_2() {
        assert_eq!(skip_sizes(2, 2), 2 + 2 * SIZE_LENGTH);
    }

    #[test]
    fn skip_sizes_3() {
        assert_eq!(skip_sizes(2, 3), 2 + 3 * SIZE_LENGTH);
    }

    #[test]
    fn test_skip_size() {
        assert_eq!(skip_size(2), 2 + SIZE_LENGTH);
    }

    #[test]
    fn test_skip_type_and_sizes() {
        assert_eq!(
            skip_type_and_sizes(2, 2),
            2 + DATA_TYPE_LENGTH + 2 * SIZE_LENGTH
        );
    }

    #[test]
    fn test_skip_type_and_size() {
        assert_eq!(skip_type_and_size(2), 2 + DATA_TYPE_LENGTH + SIZE_LENGTH);
    }

    #[test]
    fn test_skip_type_and_2_sizes() {
        assert_eq!(
            skip_type_and_2_sizes(2),
            2 + DATA_TYPE_LENGTH + 2 * SIZE_LENGTH
        );
    }

    #[test]
    fn test_size_to_bytes() {
        let result = size_to_bytes(500);
        let expected: [u8; SIZE_LENGTH] = [244, 1, 0, 0];
        assert_eq!(result, expected);
    }

    #[test]
    fn create_size_type_array() {
        let result = size_type_to_bytes(DataType::Reference, 300);

        let expected: [u8; 1 + SIZE_LENGTH] =
            [DataType::Reference.try_into().unwrap(), 44, 1, 0, 0];
        assert_eq!(result, expected);
    }

    #[test]
    fn read_size_array() {
        let data = size_to_bytes(12345);
        assert_eq!(read_size(&data).unwrap(), 12345)
    }

    #[test]
    fn read_size_array_to_short_results_in_error() {
        let data = [12, 39, 34];
        assert!(read_size(&data).is_err());
    }

    #[test]
    fn read_integer_array() {
        let data = number_type_to_bytes(12345);
        assert_eq!(read_integer(&data[1..]).unwrap(), 12345)
    }

    #[test]
    fn read_integer_array_to_short_results_in_error() {
        let data = [12, 39, 34];
        assert!(read_integer(&data).is_err());
    }

    #[test]
    fn read_float_array() {
        let data = float_type_to_bytes(12345.5);
        assert_eq!(read_float(&data[1..]).unwrap(), 12345.5)
    }

    #[test]
    fn read_float_array_to_short_results_in_error() {
        let data = [12, 39, 34];
        assert!(read_float(&data).is_err());
    }

    #[test]
    fn create_two_size_type_array() {
        let result = two_size_type_to_bytes(DataType::List, 300, 400);

        let expected: [u8; 1 + 2 * SIZE_LENGTH] = [
            DataType::List.try_into().unwrap(),
            44,
            1,
            0,
            0,
            144,
            1,
            0,
            0,
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn character_to_bytes_ascii() {
        let result = characters_to_bytes("z");
        let expected: Vec<u8> = vec![122];

        assert_eq!(result, expected);
    }

    #[test]
    fn character_to_bytes_unicode() {
        let result = characters_to_bytes("🐼");
        let expected: Vec<u8> = vec![240, 159, 144, 188];

        assert_eq!(result, expected);
    }

    #[test]
    fn character_to_bytes_grapheme_cluster() {
        let s = "ö̲";
        let result = characters_to_bytes(s);
        let expected: Vec<u8> = vec![111, 204, 136, 204, 178];

        assert_eq!(result, expected);
    }
}
