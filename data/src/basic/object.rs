use garnish_lang_traits::{GarnishDataType, SymbolListPart};

use crate::{BasicData, BasicDataCustom, BasicGarnishData, BasicNumber, DataError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BasicObject<T>
where
    T: BasicDataCustom,
{
    Unit,
    True,
    False,
    Type(GarnishDataType),
    Number(BasicNumber),
    Char(char),
    Byte(u8),
    Symbol(u64),
    SymbolList(Vec<SymbolListPart<u64, BasicNumber>>),
    Expression(usize),
    External(usize),
    CharList(String),
    ByteList(Vec<u8>),
    Pair(Box<BasicObject<T>>, Box<BasicObject<T>>),
    Range(Box<BasicObject<T>>, Box<BasicObject<T>>),
    Slice(Box<BasicObject<T>>, Box<BasicObject<T>>),
    Partial(Box<BasicObject<T>>, Box<BasicObject<T>>),
    List(Vec<Box<BasicObject<T>>>),
    Concatenation(Box<BasicObject<T>>, Box<BasicObject<T>>),
    Custom(Box<T>),
}

impl<T> BasicGarnishData<T>
where
    T: BasicDataCustom,
{
    pub fn push_object_to_data_block(&mut self, obj: BasicObject<T>) -> Result<usize, DataError> {
        match obj {
            BasicObject::Unit => self.push_to_data_block(BasicData::Unit),
            BasicObject::True => self.push_to_data_block(BasicData::True),
            BasicObject::False => self.push_to_data_block(BasicData::False),
            BasicObject::Type(ty) => self.push_to_data_block(BasicData::Type(ty)),
            BasicObject::Number(num) => self.push_to_data_block(BasicData::Number(num)),
            BasicObject::Char(ch) => self.push_to_data_block(BasicData::Char(ch)),
            BasicObject::Byte(b) => self.push_to_data_block(BasicData::Byte(b)),
            BasicObject::Symbol(sym) => self.push_to_data_block(BasicData::Symbol(sym)),
            BasicObject::SymbolList(syms) => {
                let list_index = self.push_to_data_block(BasicData::SymbolList(syms.len()))?;
                for part in syms {
                    match part {
                        SymbolListPart::Symbol(sym) => self.push_to_data_block(BasicData::Symbol(sym))?,
                        SymbolListPart::Number(num) => self.push_to_data_block(BasicData::Number(num))?,
                    };
                }
                Ok(list_index)
            }
            BasicObject::Expression(expr) => self.push_to_data_block(BasicData::Expression(expr)),
            BasicObject::External(ext) => self.push_to_data_block(BasicData::External(ext)),
            BasicObject::CharList(str) => {
                let list_index = self.push_to_data_block(BasicData::CharList(str.len()))?;
                for c in str.chars() {
                    self.push_to_data_block(BasicData::Char(c))?;
                }
                Ok(list_index)
            }
            BasicObject::ByteList(bytelist) => {
                let list_index = self.push_to_data_block(BasicData::ByteList(bytelist.len()))?;
                for b in bytelist {
                    self.push_to_data_block(BasicData::Byte(b))?;
                }
                Ok(list_index)
            }
            BasicObject::Pair(left, right) => {
                let left_index = self.push_object_to_data_block(*left)?;
                let right_index = self.push_object_to_data_block(*right)?;
                self.push_to_data_block(BasicData::Pair(left_index, right_index))
            }
            BasicObject::Range(start, end) => {
                let start_index = self.push_object_to_data_block(*start)?;
                let end_index = self.push_object_to_data_block(*end)?;
                self.push_to_data_block(BasicData::Range(start_index, end_index))
            }
            BasicObject::Slice(list, range) => {
                let list_index = self.push_object_to_data_block(*list)?;
                let range_index = self.push_object_to_data_block(*range)?;
                self.push_to_data_block(BasicData::Slice(list_index, range_index))
            }
            BasicObject::Partial(reciever, input) => {
                let reciever_index = self.push_object_to_data_block(*reciever)?;
                let input_index = self.push_object_to_data_block(*input)?;
                self.push_to_data_block(BasicData::Partial(reciever_index, input_index))
            }
            BasicObject::List(list) => {
                let indicies = list
                    .into_iter()
                    .map(|obj| self.push_object_to_data_block(*obj))
                    .collect::<Result<Vec<usize>, DataError>>()?;
                let list_index = self.push_to_data_block(BasicData::List(indicies.len(), 0))?;
                for index in indicies {
                    self.push_to_data_block(BasicData::ListItem(index))?;
                }
                Ok(list_index)
            }
            BasicObject::Concatenation(left, right) => {
                let left_index = self.push_object_to_data_block(*left)?;
                let right_index = self.push_object_to_data_block(*right)?;
                self.push_to_data_block(BasicData::Concatenation(left_index, right_index))
            }
            BasicObject::Custom(custom) => {
                let custom_index = self.push_to_data_block(BasicData::Custom(*custom.clone()))?;
                Ok(custom_index)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use garnish_lang_traits::{GarnishDataType, SymbolListPart};

    use crate::{
        BasicData,
        basic::{object::BasicObject, utilities::test_data},
    };

    #[test]
    fn push_unit_object() {
        let mut data = test_data();
        let v1 = data.push_object_to_data_block(BasicObject::Unit);

        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::Unit;
        expected_data.data_block.cursor = 1;

        assert_eq!(v1, Ok(0));
        assert_eq!(data, expected_data);
    }

    #[test]
    fn push_true_object() {
        let mut data = test_data();
        let v1 = data.push_object_to_data_block(BasicObject::True);
        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::True;
        expected_data.data_block.cursor = 1;

        assert_eq!(v1, Ok(0));
        assert_eq!(data, expected_data);
    }

    #[test]
    fn push_false_object() {
        let mut data = test_data();
        let v1 = data.push_object_to_data_block(BasicObject::False);
        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::False;
        expected_data.data_block.cursor = 1;

        assert_eq!(v1, Ok(0));
        assert_eq!(data, expected_data);
    }
    #[test]
    fn push_type_object() {
        let mut data = test_data();
        let v1 = data.push_object_to_data_block(BasicObject::Type(GarnishDataType::Number));
        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::Type(GarnishDataType::Number);
        expected_data.data_block.cursor = 1;

        assert_eq!(v1, Ok(0));
        assert_eq!(data, expected_data);
    }

    #[test]
    fn push_number_object() {
        let mut data = test_data();
        let v1 = data.push_object_to_data_block(BasicObject::Number(100.into()));
        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::Number(100.into());
        expected_data.data_block.cursor = 1;
        assert_eq!(v1, Ok(0));
        assert_eq!(data, expected_data);
    }

    #[test]
    fn push_symbol_object() {
        let mut data = test_data();
        let v1 = data.push_object_to_data_block(BasicObject::Symbol(100));
        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::Symbol(100);
        expected_data.data_block.cursor = 1;
        assert_eq!(v1, Ok(0));
        assert_eq!(data, expected_data);
    }

    #[test]
    fn push_char_object() {
        let mut data = test_data();
        let v1 = data.push_object_to_data_block(BasicObject::Char('a'));
        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::Char('a');
        expected_data.data_block.cursor = 1;
        assert_eq!(v1, Ok(0));
        assert_eq!(data, expected_data);
    }

    #[test]
    fn push_char_list_object() {
        let mut data = test_data();
        let v1 = data.push_object_to_data_block(BasicObject::CharList("abc".to_string()));
        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::CharList(3);
        expected_data.data[1] = BasicData::Char('a');
        expected_data.data[2] = BasicData::Char('b');
        expected_data.data[3] = BasicData::Char('c');
        expected_data.data_block.cursor = 4;
        assert_eq!(v1, Ok(0));
        assert_eq!(data, expected_data);
    }

    #[test]
    fn push_byte_list_object() {
        let mut data = test_data();
        let v1 = data.push_object_to_data_block(BasicObject::ByteList(vec![100, 200, 150]));
        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::ByteList(3);
        expected_data.data[1] = BasicData::Byte(100);
        expected_data.data[2] = BasicData::Byte(200);
        expected_data.data[3] = BasicData::Byte(150);
        expected_data.data_block.cursor = 4;
        assert_eq!(v1, Ok(0));
        assert_eq!(data, expected_data);
    }

    #[test]
    fn push_symbol_list_object() {
        let mut data = test_data();
        let v1 = data.push_object_to_data_block(BasicObject::SymbolList(vec![SymbolListPart::Symbol(100), SymbolListPart::Symbol(200)]));
        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::SymbolList(2);
        expected_data.data[1] = BasicData::Symbol(100);
        expected_data.data[2] = BasicData::Symbol(200);
        expected_data.data_block.cursor = 3;
        assert_eq!(v1, Ok(0));
        assert_eq!(data, expected_data);
    }

    #[test]
    fn push_pair_object() {
        let mut data = test_data();
        let v1 = data.push_object_to_data_block(BasicObject::Pair(
            Box::new(BasicObject::Number(100.into())),
            Box::new(BasicObject::Number(200.into())),
        ));

        assert_eq!(v1, Ok(2));
        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::Number(100.into());
        expected_data.data[1] = BasicData::Number(200.into());
        expected_data.data[2] = BasicData::Pair(0, 1);
        expected_data.data_block.cursor = 3;

        assert_eq!(data, expected_data);
    }

    #[test]
    fn push_range_object() {
        let mut data = test_data();
        let v1 = data.push_object_to_data_block(BasicObject::Range(
            Box::new(BasicObject::Number(100.into())),
            Box::new(BasicObject::Number(200.into())),
        ));
        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::Number(100.into());
        expected_data.data[1] = BasicData::Number(200.into());
        expected_data.data[2] = BasicData::Range(0, 1);
        expected_data.data_block.cursor = 3;
        assert_eq!(v1, Ok(2));
        assert_eq!(data, expected_data);
    }

    #[test]
    fn push_slice_object() {
        let mut data = test_data();
        let v1 = data.push_object_to_data_block(BasicObject::Slice(
            Box::new(BasicObject::Number(100.into())),
            Box::new(BasicObject::Number(200.into())),
        ));
        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::Number(100.into());
        expected_data.data[1] = BasicData::Number(200.into());
        expected_data.data[2] = BasicData::Slice(0, 1);
        expected_data.data_block.cursor = 3;
        assert_eq!(v1, Ok(2));
        assert_eq!(data, expected_data);
    }

    #[test]
    fn push_partial_object() {
        let mut data = test_data();
        let v1 = data.push_object_to_data_block(BasicObject::Partial(
            Box::new(BasicObject::Number(100.into())),
            Box::new(BasicObject::Number(200.into())),
        ));
        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::Number(100.into());
        expected_data.data[1] = BasicData::Number(200.into());
        expected_data.data[2] = BasicData::Partial(0, 1);
        expected_data.data_block.cursor = 3;
        assert_eq!(v1, Ok(2));
        assert_eq!(data, expected_data);
    }

    #[test]
    fn push_list_object() {
        let mut data = test_data();
        let v1 = data.push_object_to_data_block(BasicObject::List(vec![
            Box::new(BasicObject::Number(100.into())),
            Box::new(BasicObject::Number(200.into())),
        ]));
        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::Number(100.into());
        expected_data.data[1] = BasicData::Number(200.into());
        expected_data.data[2] = BasicData::List(2, 0);
        expected_data.data[3] = BasicData::ListItem(0);
        expected_data.data[4] = BasicData::ListItem(1);
        expected_data.data_block.cursor = 5;
        assert_eq!(v1, Ok(2));
        assert_eq!(data, expected_data);
    }

    #[test]
    fn push_concatenation_object() {
        let mut data = test_data();
        let v1 = data.push_object_to_data_block(BasicObject::Concatenation(
            Box::new(BasicObject::Number(100.into())),
            Box::new(BasicObject::Number(200.into())),
        ));
        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::Number(100.into());
        expected_data.data[1] = BasicData::Number(200.into());
        expected_data.data[2] = BasicData::Concatenation(0, 1);
        expected_data.data_block.cursor = 3;
        assert_eq!(v1, Ok(2));
        assert_eq!(data, expected_data);
    }

    #[test]
    fn push_custom_object() {
        let mut data = test_data();
        let v1 = data.push_object_to_data_block(BasicObject::Custom(Box::new(())));
        let mut expected_data = test_data();
        expected_data.data[0] = BasicData::Custom(());
        expected_data.data_block.cursor = 1;
        assert_eq!(v1, Ok(0));
        assert_eq!(data, expected_data);
    }
}
