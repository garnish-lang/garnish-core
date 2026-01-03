use super::{BasicData, BasicDataCustom, BasicGarnishData};
use crate::{DataError, basic::companion::BasicDataCompanion};

pub fn merge_to_symbol_list<T, Companion>(data: &mut BasicGarnishData<T, Companion>, first: usize, second: usize) -> Result<usize, DataError>
where
    T: BasicDataCustom,
    Companion: BasicDataCompanion<T>,
{
    match (data.get_from_data_block_ensure_index(first)?, data.get_from_data_block_ensure_index(second)?) {
        (BasicData::Symbol(sym1), BasicData::Symbol(sym2)) => {
            let sym1 = BasicData::Symbol(*sym1);
            let sym2 = BasicData::Symbol(*sym2);
            let list_index = data.push_to_data_block(BasicData::SymbolList(2))?;
            data.push_to_data_block(sym1)?;
            data.push_to_data_block(sym2)?;
            Ok(list_index)
        }
        (BasicData::Number(num1), BasicData::Number(num2)) => {
            let num1 = BasicData::Number(*num1);
            let num2 = BasicData::Number(*num2);
            let list_index = data.push_to_data_block(BasicData::SymbolList(2))?;
            data.push_to_data_block(num1)?;
            data.push_to_data_block(num2)?;
            Ok(list_index)
        }
        (BasicData::Symbol(sym1), BasicData::Number(num2)) => {
            let sym1 = BasicData::Symbol(*sym1);
            let num2 = BasicData::Number(*num2);
            let list_index = data.push_to_data_block(BasicData::SymbolList(2))?;
            data.push_to_data_block(sym1)?;
            data.push_to_data_block(num2)?;
            Ok(list_index)
        }
        (BasicData::Number(num1), BasicData::Symbol(sym2)) => {
            let num1 = BasicData::Number(*num1);
            let sym2 = BasicData::Symbol(*sym2);
            let list_index = data.push_to_data_block(BasicData::SymbolList(2))?;
            data.push_to_data_block(num1)?;
            data.push_to_data_block(sym2)?;
            Ok(list_index)
        }
        (BasicData::SymbolList(size), BasicData::Symbol(sym)) => {
            let size = *size;
            let sym = *sym;
            let list_index = data.push_to_data_block(BasicData::SymbolList(size + 1))?;

            for i in first + 1..first + size + 1 {
                let item = data.get_from_data_block_ensure_index(i)?.clone();
                data.push_to_data_block(item)?;
            }

            data.push_to_data_block(BasicData::Symbol(sym))?;

            Ok(list_index)
        }
        (BasicData::Symbol(sym), BasicData::SymbolList(size)) => {
            let size = *size;
            let sym = *sym;
            let list_index = data.push_to_data_block(BasicData::SymbolList(size + 1))?;

            data.push_to_data_block(BasicData::Symbol(sym))?;

            for i in second + 1..second + size + 1 {
                let item = data.get_from_data_block_ensure_index(i)?.clone();
                data.push_to_data_block(item)?;
            }

            Ok(list_index)
        }
        (BasicData::SymbolList(size), BasicData::Number(num)) => {
            let size = *size;
            let num = *num;
            let list_index = data.push_to_data_block(BasicData::SymbolList(size + 1))?;

            for i in first + 1..first + size + 1 {
                let item = data.get_from_data_block_ensure_index(i)?.clone();
                data.push_to_data_block(item)?;
            }

            data.push_to_data_block(BasicData::Number(num))?;

            Ok(list_index)
        }
        (BasicData::Number(num), BasicData::SymbolList(size)) => {
            let size = *size;
            let num = *num;
            let list_index = data.push_to_data_block(BasicData::SymbolList(size + 1))?;

            data.push_to_data_block(BasicData::Number(num))?;

            for i in second + 1..second + size + 1 {
                let item = data.get_from_data_block_ensure_index(i)?.clone();
                data.push_to_data_block(item)?;
            }

            Ok(list_index)
        }
        (BasicData::SymbolList(size1), BasicData::SymbolList(size2)) => {
            let size1 = *size1;
            let size2 = *size2;
            let list_index = data.push_to_data_block(BasicData::SymbolList(size1 + size2))?;

            for i in first + 1..first + size1 + 1 {
                let item = data.get_from_data_block_ensure_index(i)?.clone();
                data.push_to_data_block(item)?;
            }

            for i in second + 1..second + size2 + 1 {
                let item = data.get_from_data_block_ensure_index(i)?.clone();
                data.push_to_data_block(item)?;
            }

            Ok(list_index)
        }
        _ => data.push_to_data_block(BasicData::Unit),
    }
}

#[cfg(test)]
mod tests {
    use crate::basic::utilities::test_data;

    use super::*;
    use garnish_lang_traits::GarnishData;

    #[test]
    fn merge_to_symbol_list_symbol_symbol() {
        let mut data = test_data();
        let v1 = data.push_to_data_block(BasicData::Symbol(100)).unwrap();
        let v2 = data.push_to_data_block(BasicData::Symbol(200)).unwrap();
        let v3 = data.merge_to_symbol_list(v1, v2).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut().splice(
            0..5,
            [
                BasicData::Symbol(100),
                BasicData::Symbol(200),
                BasicData::SymbolList(2),
                BasicData::Symbol(100),
                BasicData::Symbol(200),
            ],
        );
        expected_data.data_block_mut().cursor = 5;

        assert_eq!(v3, 2);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn merge_to_symbol_list_number_number() {
        let mut data = test_data();
        let v1 = data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let v2 = data.push_to_data_block(BasicData::Number(200.into())).unwrap();
        let v3 = data.merge_to_symbol_list(v1, v2).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut().splice(
            0..5,
            [
                BasicData::Number(100.into()),
                BasicData::Number(200.into()),
                BasicData::SymbolList(2),
                BasicData::Number(100.into()),
                BasicData::Number(200.into()),
            ],
        );
        expected_data.data_block_mut().cursor = 5;

        assert_eq!(v3, 2);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn merge_to_symbol_list_symbol_number() {
        let mut data = test_data();
        let v1 = data.push_to_data_block(BasicData::Symbol(100)).unwrap();
        let v2 = data.push_to_data_block(BasicData::Number(200.into())).unwrap();
        let v3 = data.merge_to_symbol_list(v1, v2).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut().splice(
            0..5,
            [
                BasicData::Symbol(100),
                BasicData::Number(200.into()),
                BasicData::SymbolList(2),
                BasicData::Symbol(100),
                BasicData::Number(200.into()),
            ],
        );
        expected_data.data_block_mut().cursor = 5;

        assert_eq!(v3, 2);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn merge_to_symbol_list_number_symbol() {
        let mut data = test_data();
        let v1 = data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let v2 = data.push_to_data_block(BasicData::Symbol(200)).unwrap();
        let v3 = data.merge_to_symbol_list(v1, v2).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut().splice(
            0..5,
            [
                BasicData::Number(100.into()),
                BasicData::Symbol(200),
                BasicData::SymbolList(2),
                BasicData::Number(100.into()),
                BasicData::Symbol(200),
            ],
        );
        expected_data.data_block_mut().cursor = 5;

        assert_eq!(v3, 2);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn merge_to_symbol_list_symbol_list_symbol() {
        let mut data = test_data();
        let v1 = data.push_to_data_block(BasicData::SymbolList(2)).unwrap();
        data.push_to_data_block(BasicData::Symbol(100)).unwrap();
        data.push_to_data_block(BasicData::Symbol(200)).unwrap();
        let v2 = data.push_to_data_block(BasicData::Symbol(300)).unwrap();

        let v3 = data.merge_to_symbol_list(v1, v2).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut().splice(
            0..8,
            [
                BasicData::SymbolList(2),
                BasicData::Symbol(100),
                BasicData::Symbol(200),
                BasicData::Symbol(300),
                BasicData::SymbolList(3),
                BasicData::Symbol(100),
                BasicData::Symbol(200),
                BasicData::Symbol(300),
            ],
        );
        expected_data.data_block_mut().cursor = 8;

        assert_eq!(v3, 4);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn merge_to_symbol_list_symbol_symbol_list() {
        let mut data = test_data();
        let v1 = data.push_to_data_block(BasicData::SymbolList(2)).unwrap();
        data.push_to_data_block(BasicData::Symbol(100)).unwrap();
        data.push_to_data_block(BasicData::Symbol(200)).unwrap();
        let v2 = data.push_to_data_block(BasicData::Symbol(300)).unwrap();

        let v3 = data.merge_to_symbol_list(v2, v1).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut().splice(
            0..8,
            [
                BasicData::SymbolList(2),
                BasicData::Symbol(100),
                BasicData::Symbol(200),
                BasicData::Symbol(300),
                BasicData::SymbolList(3),
                BasicData::Symbol(300),
                BasicData::Symbol(100),
                BasicData::Symbol(200),
            ],
        );
        expected_data.data_block_mut().cursor = 8;

        assert_eq!(v3, 4);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn merge_to_symbol_list_symbol_list_number() {
        let mut data = test_data();
        let v1 = data.push_to_data_block(BasicData::SymbolList(2)).unwrap();
        data.push_to_data_block(BasicData::Symbol(100)).unwrap();
        data.push_to_data_block(BasicData::Symbol(200)).unwrap();
        let v2 = data.push_to_data_block(BasicData::Number(300.into())).unwrap();

        let v3 = data.merge_to_symbol_list(v1, v2).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut().splice(
            0..8,
            [
                BasicData::SymbolList(2),
                BasicData::Symbol(100),
                BasicData::Symbol(200),
                BasicData::Number(300.into()),
                BasicData::SymbolList(3),
                BasicData::Symbol(100),
                BasicData::Symbol(200),
                BasicData::Number(300.into()),
            ],
        );
        expected_data.data_block_mut().cursor = 8;

        assert_eq!(v3, 4);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn merge_to_symbol_list_number_symbol_list() {
        let mut data = test_data();
        let v1 = data.push_to_data_block(BasicData::SymbolList(2)).unwrap();
        data.push_to_data_block(BasicData::Symbol(100)).unwrap();
        data.push_to_data_block(BasicData::Symbol(200)).unwrap();
        let v2 = data.push_to_data_block(BasicData::Number(300.into())).unwrap();

        let v3 = data.merge_to_symbol_list(v2, v1).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut().splice(
            0..8,
            [
                BasicData::SymbolList(2),
                BasicData::Symbol(100),
                BasicData::Symbol(200),
                BasicData::Number(300.into()),
                BasicData::SymbolList(3),
                BasicData::Number(300.into()),
                BasicData::Symbol(100),
                BasicData::Symbol(200),
            ],
        );
        expected_data.data_block_mut().cursor = 8;

        assert_eq!(v3, 4);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn merge_to_symbol_list_symbol_list_symbol_list() {
        let mut data = test_data();
        let v1 = data.push_to_data_block(BasicData::SymbolList(2)).unwrap();
        data.push_to_data_block(BasicData::Symbol(100)).unwrap();
        data.push_to_data_block(BasicData::Symbol(200)).unwrap();
        let v2 = data.push_to_data_block(BasicData::SymbolList(2)).unwrap();
        data.push_to_data_block(BasicData::Symbol(300)).unwrap();
        data.push_to_data_block(BasicData::Symbol(400)).unwrap();

        let v3 = data.merge_to_symbol_list(v1, v2).unwrap();

        let mut expected_data = test_data();
        let data_len = data.data().len();
        expected_data.data_mut().resize(data_len, BasicData::Empty);
        expected_data.data_mut().splice(
            0..11,
            [
                BasicData::SymbolList(2),
                BasicData::Symbol(100),
                BasicData::Symbol(200),
                BasicData::SymbolList(2),
                BasicData::Symbol(300),
                BasicData::Symbol(400),
                BasicData::SymbolList(4),
                BasicData::Symbol(100),
                BasicData::Symbol(200),
                BasicData::Symbol(300),
                BasicData::Symbol(400),
            ],
        );
        expected_data.data_block_mut().cursor = 11;
        expected_data.data_block_mut().size = data.data_block().size;
        expected_data.custom_data_block_mut().start = 20;

        assert_eq!(v3, 6);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn merge_to_symbol_list_invalid_left() {
        let mut data = test_data();
        let v1 = data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let v2 = data.push_to_data_block(BasicData::Expression(2)).unwrap();

        let v3 = data.merge_to_symbol_list(v1, v2).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut().splice(
            0..3,
            [
                BasicData::Number(100.into()),
                BasicData::Expression(2),
                BasicData::Unit,
            ],
        );
        expected_data.data_block_mut().cursor = 3;

        assert_eq!(v3, 2);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn merge_to_symbol_list_invalid_right() {
        let mut data = test_data();
        let v1 = data.push_to_data_block(BasicData::Number(100.into())).unwrap();
        let v2 = data.push_to_data_block(BasicData::Expression(2)).unwrap();

        let v3 = data.merge_to_symbol_list(v2, v1).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut().splice(
            0..3,
            [
                BasicData::Number(100.into()),
                BasicData::Expression(2),
                BasicData::Unit,
            ],
        );
        expected_data.data_block_mut().cursor = 3;

        assert_eq!(v3, 2);
        assert_eq!(data, expected_data);
    }

    #[test]
    fn merge_to_symbol_list_invalid_right_and_left() {
        let mut data = test_data();
        let v1 = data.push_to_data_block(BasicData::Expression(100)).unwrap();
        let v2 = data.push_to_data_block(BasicData::Expression(2)).unwrap();

        let v3 = data.merge_to_symbol_list(v2, v1).unwrap();

        let mut expected_data = test_data();
        expected_data.data_mut().splice(
            0..3,
            [
                BasicData::Expression(100),
                BasicData::Expression(2),
                BasicData::Unit,
            ],
        );
        expected_data.data_block_mut().cursor = 3;

        assert_eq!(v3, 2);
        assert_eq!(data, expected_data);
    }
}
