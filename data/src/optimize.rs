use std::fmt::Debug;
use std::hash::Hash;
use crate::{DataError, SimpleDataList, SimpleGarnishData};

impl<T> SimpleGarnishData<T>
where
    T: Clone + PartialEq + Eq + PartialOrd + Debug + Hash,
{
    fn optimize(&mut self, retain_data: Vec<usize>) -> Result<(), DataError> {
        self.data = SimpleDataList::default();
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use garnish_lang_traits::GarnishData;
    use crate::{SimpleGarnishData, SimpleNumber};

    #[test]
    fn optimize_with_no_retention() {
        let mut data = SimpleGarnishData::new();
        data.add_number(SimpleNumber::Integer(100)).unwrap();
        data.add_number(SimpleNumber::Integer(200)).unwrap();
        data.add_number(SimpleNumber::Integer(300)).unwrap();
        
        data.optimize(vec![]).unwrap();
        
        assert_eq!(data.get_data().len(), 3);
    }
}