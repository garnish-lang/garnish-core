use crate::{
    characters_to_bytes, CharType, DataType, Instruction, Result, SizeType, CHAR_LENGTH,
    FLOAT_LENGTH, INTEGER_LENGTH, SIZE_LENGTH,
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::char;
use std::convert::{TryFrom, TryInto};
use std::io::Cursor;

pub struct DataVecWriter<'a> {
    data: &'a mut Vec<u8>,
}

impl<'a> DataVecWriter<'a> {
    pub fn new(data: &'a mut Vec<u8>) -> Self {
        return DataVecWriter { data };
    }

    // Used largely in test code for convenience
    #[allow(dead_code)]
    pub fn write_with<T>(f: T) -> Vec<u8>
    where
        T: FnOnce(DataVecWriter) -> DataVecWriter,
    {
        let mut data = vec![];
        f(DataVecWriter::new(&mut data));
        data
    }

    pub fn push_data_type(self, data_type: DataType) -> Self {
        self.data.push(data_type.try_into().unwrap());
        self
    }

    #[allow(dead_code)]
    pub fn write_data_type(self, i: usize, data_type: DataType) -> Self {
        self.data[i] = data_type.try_into().unwrap();
        self
    }

    pub fn push_instruction(self, instruction: Instruction) -> Self {
        self.data.push(instruction.try_into().unwrap());
        self
    }

    #[allow(dead_code)]
    pub fn write_instruction(self, i: usize, instruction: Instruction) -> Self {
        self.data[i] = instruction.try_into().unwrap();
        self
    }

    pub fn push_integer(self, num: i32) -> Self {
        let mut bs = [0u8; INTEGER_LENGTH];
        bs.as_mut()
            .write_i32::<LittleEndian>(num)
            .expect("Unable to write");

        for byte in bs.iter() {
            self.data.push(*byte);
        }

        self
    }

    pub fn push_float(self, num: f32) -> Self {
        let mut bs = [0u8; FLOAT_LENGTH];
        bs.as_mut()
            .write_f32::<LittleEndian>(num)
            .expect("Unable to write");

        for byte in bs.iter() {
            self.data.push(*byte);
        }

        self
    }

    #[allow(dead_code)]
    pub fn write_number(self, i: usize, num: i32) -> Self {
        let mut bs = [0u8; INTEGER_LENGTH];
        bs.as_mut()
            .write_i32::<LittleEndian>(num)
            .expect("Unable to write");

        self.write_data(i, &bs[..])
    }

    pub fn push_size(self, num: usize) -> Self {
        let mut bs = [0u8; SIZE_LENGTH];
        bs.as_mut()
            .write_u32::<LittleEndian>(num as SizeType)
            .expect("Unable to write");

        for byte in bs.iter() {
            self.data.push(*byte);
        }

        self
    }

    pub fn push_byte_size(self, num: u8) -> Self {
        self.data.push(num);
        self
    }

    #[allow(dead_code)]
    pub fn write_size(self, i: usize, num: usize) -> Self {
        let mut bs = [0u8; SIZE_LENGTH];
        bs.as_mut()
            .write_u32::<LittleEndian>(num as SizeType)
            .expect("Unable to write");

        self.write_data(i, &bs[..])
    }

    pub fn push_character(self, s: &str) -> Self {
        let data = characters_to_bytes(s);

        for d in data {
            self.data.push(d);
        }

        self
    }

    pub fn push_str(self, s: &str) -> Self {
        let mut bs = [0u8; CHAR_LENGTH];

        for c in s.chars() {
            bs.as_mut()
                .write_u32::<LittleEndian>(c as CharType)
                .expect("Unable to write");

            for byte in bs.iter() {
                self.data.push(*byte);
            }
        }

        self
    }

    pub fn write_data(self, i: usize, data: &[u8]) -> Self {
        self.data[i..(i + data.len())].clone_from_slice(data);
        self
    }
}

pub struct DataVecReader<'a> {
    data: &'a [u8],
}

impl<'a> DataVecReader<'a> {
    pub fn new(data: &'a Vec<u8>) -> Self {
        return DataVecReader { data };
    }

    pub fn new_from_slice(data: &'a [u8]) -> Self {
        return DataVecReader { data };
    }

    #[allow(dead_code)]
    pub fn read_data_type(&self, start: usize) -> Result<DataType> {
        DataType::try_from(self.data[start])
    }

    #[allow(dead_code)]
    pub fn read_instruction(&self, start: usize) -> Instruction {
        Instruction::try_from(self.data[start]).unwrap()
    }

    pub fn read_number(&self, start: usize) -> i32 {
        let slice = &self.data[start..(start + INTEGER_LENGTH)];
        let mut reader = Cursor::new(slice);
        let num = reader.read_i32::<LittleEndian>().unwrap();

        num
    }

    pub fn read_float(&self, start: usize) -> f32 {
        let slice = &self.data[start..(start + FLOAT_LENGTH)];
        let mut reader = Cursor::new(slice);
        let num = reader.read_f32::<LittleEndian>().unwrap();

        num
    }

    pub fn read_size(&self, start: usize) -> usize {
        let slice = &self.data[start..(start + SIZE_LENGTH)];
        let mut reader = Cursor::new(slice);
        let num = reader.read_u32::<LittleEndian>().unwrap();

        num as usize
    }

    pub fn read_byte_size(&self, start: usize) -> usize {
        self.data[start] as usize
    }

    #[allow(dead_code)]
    pub fn read_str(&self, start: usize, length: usize) -> String {
        let mut s = String::new();

        for i in 0..length {
            let start = start + i * CHAR_LENGTH;
            let slice = &self.data[start..(start + CHAR_LENGTH)];
            let mut reader = Cursor::new(slice);
            let c = char::from_u32(reader.read_u32::<LittleEndian>().unwrap()).unwrap();

            s.push(c);
        }

        s
    }

    pub fn read_character(&self, start: usize, length: usize) -> String {
        let length = length as usize;
        String::from_utf8_lossy(&self.data[start..(start + length)]).into()
    }
}

#[cfg(test)]
mod tests {
    use crate::byte_vec::{DataVecReader, DataVecWriter};
    use crate::{DataType, Instruction};
    use byteorder::{LittleEndian, WriteBytesExt};
    use std::convert::TryInto;
    use std::mem;

    #[test]
    fn new_byte_vec_has_no_data() {
        let mut data: Vec<u8> = vec![];
        DataVecWriter::new(&mut data);
        assert_eq!(data.len(), 0);
    }

    #[test]
    fn push_data_type() {
        let mut expected: Vec<u8> = vec![];
        expected.push(DataType::Unit.try_into().unwrap());

        let mut data: Vec<u8> = vec![];
        let writer = DataVecWriter::new(&mut data);
        writer.push_data_type(DataType::Unit);

        assert_eq!(data, expected);
    }

    #[test]
    fn push_instruction() {
        let mut expected: Vec<u8> = vec![];
        expected.push(Instruction::Put.try_into().unwrap());

        let mut data: Vec<u8> = vec![];
        let writer = DataVecWriter::new(&mut data);
        writer.push_instruction(Instruction::Put);

        assert_eq!(data, expected);
    }

    #[test]
    fn push_integer() {
        let mut expected: Vec<u8> = vec![];
        let mut bs = [0u8; mem::size_of::<i32>()];
        bs.as_mut()
            .write_i32::<LittleEndian>(100)
            .expect("Unable to write");

        for byte in bs.iter() {
            expected.push(*byte);
        }

        let mut data: Vec<u8> = vec![];
        let writer = DataVecWriter::new(&mut data);
        writer.push_integer(100);

        assert_eq!(data, expected);
    }

    #[test]
    fn push_float() {
        let mut expected: Vec<u8> = vec![];
        let mut bs = [0u8; mem::size_of::<f32>()];
        bs.as_mut()
            .write_f32::<LittleEndian>(3.14)
            .expect("Unable to write");

        for byte in bs.iter() {
            expected.push(*byte);
        }

        let mut data: Vec<u8> = vec![];
        let writer = DataVecWriter::new(&mut data);
        writer.push_float(3.14);

        assert_eq!(data, expected);
    }

    #[test]
    fn push_multiple_number() {
        let mut expected: Vec<u8> = vec![];
        let mut bs = [0u8; mem::size_of::<i32>()];
        bs.as_mut()
            .write_i32::<LittleEndian>(100)
            .expect("Unable to write");

        for byte in bs.iter() {
            expected.push(*byte);
        }

        bs.as_mut()
            .write_i32::<LittleEndian>(200)
            .expect("Unable to write");

        for byte in bs.iter() {
            expected.push(*byte);
        }

        let mut data: Vec<u8> = vec![];
        let writer = DataVecWriter::new(&mut data);
        writer.push_integer(100).push_integer(200);

        assert_eq!(data, expected);
    }

    #[test]
    fn push_size() {
        let mut expected: Vec<u8> = vec![];
        let mut bs = [0u8; mem::size_of::<u32>()];
        bs.as_mut()
            .write_u32::<LittleEndian>(100)
            .expect("Unable to write");

        for byte in bs.iter() {
            expected.push(*byte);
        }

        let mut data: Vec<u8> = vec![];
        let writer = DataVecWriter::new(&mut data);
        writer.push_size(100);

        assert_eq!(data, expected);
    }

    #[test]
    fn push_byte_size() {
        let expected = vec![3u8];

        let mut data: Vec<u8> = vec![];
        let writer = DataVecWriter::new(&mut data);
        writer.push_byte_size(3);

        assert_eq!(data, expected);
    }

    #[test]
    fn push_character() {
        let mut data: Vec<u8> = vec![];
        let writer = DataVecWriter::new(&mut data);
        writer.push_character("z");

        let expected: Vec<u8> = vec![122];

        assert_eq!(data, expected);
    }

    #[test]
    fn push_str() {
        let s = "The quick brown fox";

        let mut expected: Vec<u8> = vec![];
        let mut bs = [0u8; mem::size_of::<u32>()];

        for c in s.chars() {
            bs.as_mut()
                .write_u32::<LittleEndian>(c as u32)
                .expect("Unable to write");

            for byte in bs.iter() {
                expected.push(*byte);
            }
        }

        let mut data: Vec<u8> = vec![];
        let writer = DataVecWriter::new(&mut data);
        writer.push_str(s);

        assert_eq!(data, expected);
    }

    #[test]
    fn write_data_type() {
        let expected: Vec<u8> = vec![0, 0, 0, 0, 2, 0, 0, 0];

        let mut data: Vec<u8> = vec![0, 0, 0, 0, 0, 0, 0, 0];
        let writer = DataVecWriter::new(&mut data);
        writer.write_data_type(4, DataType::Integer);

        assert_eq!(data, expected);
    }

    #[test]
    fn write_instruction() {
        let expected: Vec<u8> = vec![0, 0, 0, 0, 1, 0, 0, 0];

        let mut data: Vec<u8> = vec![0, 0, 0, 0, 0, 0, 0, 0];
        let writer = DataVecWriter::new(&mut data);
        writer.write_instruction(4, Instruction::Put);

        assert_eq!(data, expected);
    }

    #[test]
    fn write_number() {
        let expected: Vec<u8> = vec![0, 0, 0, 0, 100, 0, 0, 0];

        let mut data: Vec<u8> = vec![0, 0, 0, 0, 0, 0, 0, 0];
        let writer = DataVecWriter::new(&mut data);
        writer.write_number(4, 100);

        assert_eq!(data, expected);
    }

    #[test]
    fn write_size() {
        let expected: Vec<u8> = vec![0, 0, 0, 0, 100, 0, 0, 0];

        let mut data: Vec<u8> = vec![0, 0, 0, 0, 0, 0, 0, 0];
        let writer = DataVecWriter::new(&mut data);
        writer.write_size(4, 100);

        assert_eq!(data, expected);
    }

    #[test]
    fn write_data_with_slice() {
        let expected: Vec<u8> = vec![0, 0, 0, 0, 10, 20, 30, 40];

        let mut result: Vec<u8> = vec![0, 0, 0, 0, 0, 0, 0, 0];
        let writer = DataVecWriter::new(&mut result);
        let data = vec![10, 20, 30, 40];
        writer.write_data(4, &data[..]);

        assert_eq!(result, expected);
    }

    #[test]
    fn create_to_data_vec_and_write() {
        let data = DataVecWriter::write_with(|writer| {
            writer.push_data_type(DataType::Integer).push_integer(10000)
        });

        let mut expected: Vec<u8> = vec![];
        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Integer)
            .push_integer(10000);

        assert_eq!(data, expected);
    }

    #[test]
    fn new_data_vec_reader_with_no_errors() {
        let data: Vec<u8> = vec![];
        DataVecReader::new(&data);
    }

    #[test]
    fn read_data_type() {
        let data = DataVecWriter::write_with(|w| w.push_data_type(DataType::Unit));
        let data_type = DataVecReader::new(&data).read_data_type(0).unwrap();

        assert_eq!(data_type, DataType::Unit);
    }

    #[test]
    fn read_instruction() {
        let data = DataVecWriter::write_with(|w| w.push_instruction(Instruction::Put));
        let instruction = DataVecReader::new(&data).read_instruction(0);

        assert_eq!(instruction, Instruction::Put);
    }

    #[test]
    fn read_number() {
        let data = DataVecWriter::write_with(|w| w.push_integer(1000));
        let num = DataVecReader::new(&data).read_number(0);

        assert_eq!(num, 1000);
    }

    #[test]
    fn read_size() {
        let data = DataVecWriter::write_with(|w| w.push_size(1000));
        let num = DataVecReader::new(&data).read_size(0);

        assert_eq!(num, 1000);
    }

    #[test]
    fn read_byte_size() {
        let data = DataVecWriter::write_with(|w| w.push_byte_size(3));
        let num = DataVecReader::new(&data).read_byte_size(0);

        assert_eq!(num, 3);
    }

    #[test]
    fn read_string() {
        let data = DataVecWriter::write_with(|w| w.push_str("bears"));
        let s = DataVecReader::new(&data).read_str(0, 5);

        assert_eq!(s, String::from("bears"));
    }

    #[test]
    fn read_character_ascii() {
        let data = DataVecWriter::write_with(|w| w.push_character("z"));
        let s = DataVecReader::new(&data).read_character(0, 1);

        assert_eq!(s, "z");
    }
}
