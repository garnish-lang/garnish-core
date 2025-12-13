use super::{BasicData, BasicGarnishData, BasicDataCustom};
use crate::DataError;

pub fn merge_to_symbol_list<T>(
    data: &mut BasicGarnishData<T>,
    first: usize,
    second: usize,
) -> Result<usize, DataError>
where
    T: BasicDataCustom,
{
    match (data.get_basic_data(first), data.get_basic_data(second)) {
        (Some(BasicData::Symbol(sym1)), Some(BasicData::Symbol(sym2))) => {
            let sym1 = BasicData::Symbol(*sym1);
            let sym2 = BasicData::Symbol(*sym2);
            let list_index = data.push_basic_data(BasicData::SymbolList(2));
            data.push_basic_data(sym1);
            data.push_basic_data(sym2);
            Ok(list_index)
        }
        (Some(BasicData::Number(num1)), Some(BasicData::Number(num2))) => {
            let num1 = BasicData::Number(*num1);
            let num2 = BasicData::Number(*num2);
            let list_index = data.push_basic_data(BasicData::SymbolList(2));
            data.push_basic_data(num1);
            data.push_basic_data(num2);
            Ok(list_index)
        }
        (Some(BasicData::Symbol(sym1)), Some(BasicData::Number(num2))) => {
            let sym1 = BasicData::Symbol(*sym1);
            let num2 = BasicData::Number(*num2);
            let list_index = data.push_basic_data(BasicData::SymbolList(2));
            data.push_basic_data(sym1);
            data.push_basic_data(num2);
            Ok(list_index)
        }
        (Some(BasicData::Number(num1)), Some(BasicData::Symbol(sym2))) => {
            let num1 = BasicData::Number(*num1);
            let sym2 = BasicData::Symbol(*sym2);
            let list_index = data.push_basic_data(BasicData::SymbolList(2));
            data.push_basic_data(num1);
            data.push_basic_data(sym2);
            Ok(list_index)
        }
        (Some(BasicData::SymbolList(size)), Some(BasicData::Symbol(sym))) => {
            let size = *size;
            let sym = *sym;
            let list_index = data.push_basic_data(BasicData::SymbolList(size + 1));

            for i in first+1..first+size+1 {
                let item = data.get_data_ensure_index(i)?.clone();
                data.push_basic_data(item);
            }

            data.push_basic_data(BasicData::Symbol(sym));

            Ok(list_index)
        }
        (Some(BasicData::Symbol(sym)), Some(BasicData::SymbolList(size))) => {
            let size = *size;
            let sym = *sym;
            let list_index = data.push_basic_data(BasicData::SymbolList(size + 1));

            data.push_basic_data(BasicData::Symbol(sym));

            for i in second+1..second+size+1 {
                let item = data.get_data_ensure_index(i)?.clone();
                data.push_basic_data(item);
            }

            Ok(list_index)
        }
        (Some(BasicData::SymbolList(size)), Some(BasicData::Number(num))) => {
            let size = *size;
            let num = *num;
            let list_index = data.push_basic_data(BasicData::SymbolList(size + 1));

            for i in first+1..first+size+1 {
                let item = data.get_data_ensure_index(i)?.clone();
                data.push_basic_data(item);
            }

            data.push_basic_data(BasicData::Number(num));

            Ok(list_index)
        }
        (Some(BasicData::Number(num)), Some(BasicData::SymbolList(size))) => {
            let size = *size;
            let num = *num;
            let list_index = data.push_basic_data(BasicData::SymbolList(size + 1));

            data.push_basic_data(BasicData::Number(num));

            for i in second+1..second+size+1 {
                let item = data.get_data_ensure_index(i)?.clone();
                data.push_basic_data(item);
            }

            Ok(list_index)
        }
        (Some(BasicData::SymbolList(size1)), Some(BasicData::SymbolList(size2))) => {
            let size1 = *size1;
            let size2 = *size2;
            let list_index = data.push_basic_data(BasicData::SymbolList(size1 + size2));

            for i in first+1..first+size1+1 {
                let item = data.get_data_ensure_index(i)?.clone();
                data.push_basic_data(item);
            }

            for i in second+1..second+size2+1 {
                let item = data.get_data_ensure_index(i)?.clone();
                data.push_basic_data(item);
            }

            Ok(list_index)
        }
        _ => Ok(data.push_basic_data(BasicData::Unit)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::BasicGarnishDataUnit;
    use garnish_lang_traits::GarnishData;

    #[test]
    fn merge_to_symbol_list_symbol_symbol() {
        let mut data = BasicGarnishDataUnit::new();
        let v1 = data.push_basic_data(BasicData::Symbol(100));
        let v2 = data.push_basic_data(BasicData::Symbol(200));
        let v3 = data.merge_to_symbol_list(v1, v2).unwrap();

        assert_eq!(v3, 2);
        assert_eq!(
            data,
            BasicGarnishDataUnit::new_full(vec![
                BasicData::Symbol(100),
                BasicData::Symbol(200),
                BasicData::SymbolList(2),
                BasicData::Symbol(100),
                BasicData::Symbol(200),
            ])
        );
    }

    #[test]
    fn merge_to_symbol_list_number_number() {
        let mut data = BasicGarnishDataUnit::new();
        let v1 = data.push_basic_data(BasicData::Number(100.into()));
        let v2 = data.push_basic_data(BasicData::Number(200.into()));
        let v3 = data.merge_to_symbol_list(v1, v2).unwrap();

        assert_eq!(v3, 2);
        assert_eq!(
            data,
            BasicGarnishDataUnit::new_full(vec![
                BasicData::Number(100.into()),
                BasicData::Number(200.into()),
                BasicData::SymbolList(2),
                BasicData::Number(100.into()),
                BasicData::Number(200.into()),
            ])
        );
    }

    #[test]
    fn merge_to_symbol_list_symbol_number() {
        let mut data = BasicGarnishDataUnit::new();
        let v1 = data.push_basic_data(BasicData::Symbol(100));
        let v2 = data.push_basic_data(BasicData::Number(200.into()));
        let v3 = data.merge_to_symbol_list(v1, v2).unwrap();

        assert_eq!(v3, 2);
        assert_eq!(
            data,
            BasicGarnishDataUnit::new_full(vec![
                BasicData::Symbol(100),
                BasicData::Number(200.into()),
                BasicData::SymbolList(2),
                BasicData::Symbol(100),
                BasicData::Number(200.into()),
            ])
        );
    }

    #[test]
    fn merge_to_symbol_list_number_symbol() {
        let mut data = BasicGarnishDataUnit::new();
        let v1 = data.push_basic_data(BasicData::Number(100.into()));
        let v2 = data.push_basic_data(BasicData::Symbol(200));
        let v3 = data.merge_to_symbol_list(v1, v2).unwrap();

        assert_eq!(v3, 2);
        assert_eq!(
            data,
            BasicGarnishDataUnit::new_full(vec![
                BasicData::Number(100.into()),
                BasicData::Symbol(200),
                BasicData::SymbolList(2),
                BasicData::Number(100.into()),
                BasicData::Symbol(200),
            ])
        );
    }

    #[test]
    fn merge_to_symbol_list_symbol_list_symbol() {
        let mut data = BasicGarnishDataUnit::new();
        let v1 = data.push_basic_data(BasicData::SymbolList(2));
        data.push_basic_data(BasicData::Symbol(100));
        data.push_basic_data(BasicData::Symbol(200));
        let v2 = data.push_basic_data(BasicData::Symbol(300));

        let v3 = data.merge_to_symbol_list(v1, v2).unwrap();

        assert_eq!(v3, 4);
        assert_eq!(
            data,
            BasicGarnishDataUnit::new_full(vec![
                BasicData::SymbolList(2),
                BasicData::Symbol(100),
                BasicData::Symbol(200),
                BasicData::Symbol(300),
                BasicData::SymbolList(3),
                BasicData::Symbol(100),
                BasicData::Symbol(200),
                BasicData::Symbol(300),
            ])
        );
    }

    #[test]
    fn merge_to_symbol_list_symbol_symbol_list() {
        let mut data = BasicGarnishDataUnit::new();
        let v1 = data.push_basic_data(BasicData::SymbolList(2));
        data.push_basic_data(BasicData::Symbol(100));
        data.push_basic_data(BasicData::Symbol(200));
        let v2 = data.push_basic_data(BasicData::Symbol(300));

        let v3 = data.merge_to_symbol_list(v2, v1).unwrap();

        assert_eq!(v3, 4);
        assert_eq!(
            data,
            BasicGarnishDataUnit::new_full(vec![
                BasicData::SymbolList(2),
                BasicData::Symbol(100),
                BasicData::Symbol(200),
                BasicData::Symbol(300),
                BasicData::SymbolList(3),
                BasicData::Symbol(300),
                BasicData::Symbol(100),
                BasicData::Symbol(200),
            ])
        );
    }

    #[test]
    fn merge_to_symbol_list_symbol_list_number() {
        let mut data = BasicGarnishDataUnit::new();
        let v1 = data.push_basic_data(BasicData::SymbolList(2));
        data.push_basic_data(BasicData::Symbol(100));
        data.push_basic_data(BasicData::Symbol(200));
        let v2 = data.push_basic_data(BasicData::Number(300.into()));

        let v3 = data.merge_to_symbol_list(v1, v2).unwrap();

        assert_eq!(v3, 4);
        assert_eq!(
            data,
            BasicGarnishDataUnit::new_full(vec![
                BasicData::SymbolList(2),
                BasicData::Symbol(100),
                BasicData::Symbol(200),
                BasicData::Number(300.into()),
                BasicData::SymbolList(3),
                BasicData::Symbol(100),
                BasicData::Symbol(200),
                BasicData::Number(300.into()),
            ])
        );
    }

    #[test]
    fn merge_to_symbol_list_number_symbol_list() {
        let mut data = BasicGarnishDataUnit::new();
        let v1 = data.push_basic_data(BasicData::SymbolList(2));
        data.push_basic_data(BasicData::Symbol(100));
        data.push_basic_data(BasicData::Symbol(200));
        let v2 = data.push_basic_data(BasicData::Number(300.into()));

        let v3 = data.merge_to_symbol_list(v2, v1).unwrap();

        assert_eq!(v3, 4);
        assert_eq!(
            data,
            BasicGarnishDataUnit::new_full(vec![
                BasicData::SymbolList(2),
                BasicData::Symbol(100),
                BasicData::Symbol(200),
                BasicData::Number(300.into()),
                BasicData::SymbolList(3),
                BasicData::Number(300.into()),
                BasicData::Symbol(100),
                BasicData::Symbol(200),
            ])
        );
    }

    #[test]
    fn merge_to_symbol_list_symbol_list_symbol_list() {
        let mut data = BasicGarnishDataUnit::new();
        let v1 = data.push_basic_data(BasicData::SymbolList(2));
        data.push_basic_data(BasicData::Symbol(100));
        data.push_basic_data(BasicData::Symbol(200));
        let v2 = data.push_basic_data(BasicData::SymbolList(2));
        data.push_basic_data(BasicData::Symbol(300));
        data.push_basic_data(BasicData::Symbol(400));

        let v3 = data.merge_to_symbol_list(v1, v2).unwrap();

        assert_eq!(v3, 6);
        assert_eq!(
            data,
            BasicGarnishDataUnit::new_full(vec![
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
            ])
        );
    }

    #[test]
    fn merge_to_symbol_list_invalid_left() {
        let mut data = BasicGarnishDataUnit::new();
        let v1 = data.push_basic_data(BasicData::Number(100.into()));
        let v2 = data.push_basic_data(BasicData::Expression(2));

        let v3 = data.merge_to_symbol_list(v1, v2).unwrap();

        assert_eq!(v3, 2);
        assert_eq!(
            data,
            BasicGarnishDataUnit::new_full(vec![
                BasicData::Number(100.into()),
                BasicData::Expression(2),
                BasicData::Unit,
            ])
        );
    }

    #[test]
    fn merge_to_symbol_list_invalid_right() {
        let mut data = BasicGarnishDataUnit::new();
        let v1 = data.push_basic_data(BasicData::Number(100.into()));
        let v2 = data.push_basic_data(BasicData::Expression(2));

        let v3 = data.merge_to_symbol_list(v2, v1).unwrap();

        assert_eq!(v3, 2);
        assert_eq!(
            data,
            BasicGarnishDataUnit::new_full(vec![
                BasicData::Number(100.into()),
                BasicData::Expression(2),
                BasicData::Unit,
            ])
        );
    }

    #[test]
    fn merge_to_symbol_list_invalid_right_and_left() {
        let mut data = BasicGarnishDataUnit::new();
        let v1 = data.push_basic_data(BasicData::Expression(100));
        let v2 = data.push_basic_data(BasicData::Expression(2));

        let v3 = data.merge_to_symbol_list(v2, v1).unwrap();

        assert_eq!(v3, 2);
        assert_eq!(
            data,
            BasicGarnishDataUnit::new_full(vec![
                BasicData::Expression(100),
                BasicData::Expression(2),
                BasicData::Unit,
            ])
        );
    }
}
