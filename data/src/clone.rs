use crate::{DataError, SimpleGarnishData};
use garnish_lang_traits::helpers::clone_data;
use std::fmt::Debug;
use std::hash::Hash;

impl<T> SimpleGarnishData<T>
where
    T: Clone + PartialEq + Eq + PartialOrd + Debug + Hash,
{
    pub fn clone_with_retained_data(&mut self, retain_data: Vec<usize>) -> Result<SimpleGarnishData<T>, DataError> {
        let mut new_data = SimpleGarnishData::<T>::new_custom();
        new_data.instructions = self.instructions.clone();
        new_data.instruction_cursor = self.instruction_cursor.clone();
        new_data.expression_table = self.expression_table.clone();
        new_data.end_of_constant_data = self.end_of_constant_data;

        for i in new_data.data.len()..=new_data.end_of_constant_data {
            let data = self.get(i)?;
            new_data.data.push(data.clone());
        }

        for data in retain_data {
            clone_data(data, self, &mut new_data)?;
        }

        Ok(new_data)
    }

    pub fn clone_without_data(&mut self) -> Result<SimpleGarnishData<T>, DataError> {
        self.clone_with_retained_data(vec![])
    }
}

#[cfg(test)]
mod tests {
    use crate::{NoCustom, SimpleData, SimpleGarnishData, SimpleNumber};
    use garnish_lang_traits::GarnishData;

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

        let new_data = data.clone_with_retained_data(vec![]).unwrap();

        assert_eq!(new_data.get_data().len(), 3);
        assert_default_data(&new_data);
    }

    #[test]
    fn retain_constant_data() {
        let mut data = SimpleGarnishData::new();
        data.add_number(SimpleNumber::Integer(100)).unwrap();
        let num2 = data.add_number(SimpleNumber::Integer(200)).unwrap();
        data.add_number(SimpleNumber::Integer(300)).unwrap();
        data.set_end_of_constant(num2).unwrap();

        let new_data = data.clone_with_retained_data(vec![]).unwrap();

        assert_eq!(new_data.get_data().len(), 5);
        assert_eq!(new_data.get_data().get(3).unwrap(), &SimpleData::Number(100.into()));
        assert_eq!(new_data.get_data().get(4).unwrap(), &SimpleData::Number(200.into()));
        assert_eq!(new_data.get_data().get(5), None);
        assert_default_data(&new_data);
    }

    #[test]
    fn retain_one_number() {
        let mut data = SimpleGarnishData::new();
        data.add_number(SimpleNumber::Integer(100)).unwrap();
        data.add_number(SimpleNumber::Integer(200)).unwrap();
        data.add_number(SimpleNumber::Integer(300)).unwrap();

        let new_data = data.clone_with_retained_data(vec![5]).unwrap();

        assert_default_data(&new_data);
        assert_eq!(new_data.get_data().get(3).unwrap(), &SimpleData::Number(300.into()));
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

        let new_data = data.clone_with_retained_data(vec![pair]).unwrap();

        assert_default_data(&new_data);
        assert_eq!(new_data.get_data().get(3).unwrap(), &SimpleData::Symbol(sym_data));
        assert_eq!(new_data.get_data().get(4).unwrap(), &SimpleData::Number(300.into()));
        assert_eq!(new_data.get_data().get(5).unwrap(), &SimpleData::Pair(3, 4));
    }
}
