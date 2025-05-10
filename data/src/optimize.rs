use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use garnish_lang_traits::GarnishData;
use crate::{DataError, SimpleData, SimpleDataList, SimpleGarnishData};

impl<T> SimpleGarnishData<T>
where
    T: Clone + PartialEq + Eq + PartialOrd + Debug + Hash,
{
    fn optimize(&mut self, retain_data: Vec<usize>) -> Result<(), DataError> {
        let old_data = self.data.clone();
        self.data = SimpleDataList::<T>::default();
        self.cache = HashMap::new();
        
        for data in retain_data {
            self.optimize_retain_item(data, &old_data)?;
        }

        Ok(())
    }
    
    fn optimize_retain_item(&mut self, index: usize, reference_data: &SimpleDataList<T>) -> Result<usize, DataError> {
        match reference_data.get(index) {
            None => Err(DataError::from(format!("No data at {} while optimizing", index)))?,
            Some(value) => match value {
                SimpleData::Pair(left, right) => {
                    let new_left = self.optimize_retain_item(left.clone(), reference_data)?;
                    let new_right = self.optimize_retain_item(right.clone(), reference_data)?;
                    self.add_pair((new_left, new_right))
                }
                d => {
                    self.cache_add(d.clone())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use garnish_lang_traits::GarnishData;
    use crate::{NoCustom, SimpleData, SimpleGarnishData, SimpleNumber};

    fn assert_default_data(data: &SimpleGarnishData) {
        assert_eq!(data.data.get(0).unwrap(), &SimpleData::Unit);
        assert_eq!(data.data.get(1).unwrap(), &SimpleData::False);
        assert_eq!(data.data.get(2).unwrap(), &SimpleData::True);
    }

    #[test]
    fn optimize_with_no_retention() {
        let mut data = SimpleGarnishData::new();
        data.add_number(SimpleNumber::Integer(100)).unwrap();
        data.add_number(SimpleNumber::Integer(200)).unwrap();
        data.add_number(SimpleNumber::Integer(300)).unwrap();

        data.optimize(vec![]).unwrap();

        assert_eq!(data.get_data().len(), 3);
        assert_default_data(&data);
    }

    #[test]
    fn retain_one_number() {
        let mut data = SimpleGarnishData::new();
        data.add_number(SimpleNumber::Integer(100)).unwrap();
        data.add_number(SimpleNumber::Integer(200)).unwrap();
        data.add_number(SimpleNumber::Integer(300)).unwrap();

        data.optimize(vec![5]).unwrap();

        assert_default_data(&data);
        assert_eq!(data.get_data().get(3).unwrap(), &SimpleData::Number(300.into()));
    }

    #[test]
    fn retain_pair() {
        let mut data = SimpleGarnishData::new();
        data.add_number(SimpleNumber::Integer(100)).unwrap();
        data.add_number(SimpleNumber::Integer(200)).unwrap();
        let num = data.add_number(SimpleNumber::Integer(300)).unwrap();
        let sym_data = SimpleGarnishData::<NoCustom>::parse_symbol("number").unwrap();
        let sym = data.add_symbol(sym_data).unwrap();
        let pair = data.add_pair((sym, num)).unwrap();

        data.optimize(vec![pair]).unwrap();

        assert_default_data(&data);
        assert_eq!(data.get_data().get(3).unwrap(), &SimpleData::Symbol(sym_data));
        assert_eq!(data.get_data().get(4).unwrap(), &SimpleData::Number(300.into()));
        assert_eq!(data.get_data().get(5).unwrap(), &SimpleData::Pair(3, 4));
    }
}