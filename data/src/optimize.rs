use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use crate::{DataError, SimpleDataList, SimpleGarnishData};

impl<T> SimpleGarnishData<T>
where
    T: Clone + PartialEq + Eq + PartialOrd + Debug + Hash,
{
    pub fn optimize(&mut self, retain_data: Vec<usize>) -> Result<(), DataError> {
        let old_data = self.data.clone();
        self.data = SimpleDataList::default();
        self.cache = HashMap::new();
        
        for data in retain_data {
            match old_data.get(data) {
                None => Err(DataError::from(format!("No data at {} while optimizing", data)))?,
                Some(value) => {
                    self.cache_add(value.clone())?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use garnish_lang_traits::GarnishData;
    use crate::{SimpleData, SimpleGarnishData, SimpleNumber};

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
}