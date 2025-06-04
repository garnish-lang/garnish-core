use crate::{DataError, SimpleDataType, SimpleGarnishData};
use garnish_lang_traits::helpers::clone_data;
use std::fmt::Debug;
use std::hash::Hash;

fn clone_with_retained_data_internal<T, A>(
    from: &SimpleGarnishData<T, A>,
    retain_data: Vec<usize>,
) -> Result<SimpleGarnishData<T, A>, DataError>
where
    T: SimpleDataType,
    A: Default,
{
    let mut new_data = SimpleGarnishData::<T, A>::new_custom();
    new_data.instructions = from.instructions.clone();
    new_data.instruction_cursor = from.instruction_cursor.clone();
    new_data.expression_table = from.expression_table.clone();
    new_data.end_of_constant_data = from.end_of_constant_data;
    new_data.resolver = from.resolver.clone();
    new_data.op_handler = from.op_handler.clone();

    new_data.data.expression_to_symbol = from.data.expression_to_symbol.clone();
    new_data.data.symbol_to_name = from.data.symbol_to_name.clone();
    new_data.data.external_to_symbol = from.data.external_to_symbol.clone();

    for i in new_data.data.len()..=new_data.end_of_constant_data {
        let data = from.get(i)?;
        new_data.cache_add(data.clone())?;
    }

    for data in retain_data {
        clone_data(data, from, &mut new_data)?;
    }

    Ok(new_data)
}

impl<T, A> SimpleGarnishData<T, A>
where
    T: SimpleDataType,
    A: Default,
{
    pub fn clone_with_retained_data(&mut self, retain_data: Vec<usize>) -> Result<SimpleGarnishData<T, A>, DataError> {
        clone_with_retained_data_internal(self, retain_data)
    }

    pub fn clone_without_data(&mut self) -> Result<SimpleGarnishData<T, A>, DataError> {
        self.clone_with_retained_data(vec![])
    }
}

impl<T, A> SimpleGarnishData<T, A>
where
    T: SimpleDataType,
    A: Default + Clone,
{
    pub fn clone_with_aux_and_retained_data(&mut self, retain_data: Vec<usize>) -> Result<SimpleGarnishData<T, A>, DataError> {
        let mut data = clone_with_retained_data_internal(self, retain_data)?;
        data.auxiliary_data = self.auxiliary_data().clone();
        Ok(data)
    }

    pub fn clone_with_aux_without_data(&mut self) -> Result<SimpleGarnishData<T, A>, DataError> {
        self.clone_with_aux_and_retained_data(vec![])
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
        data.end_of_constant_data = data.data.len() - 1;
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
        data.end_of_constant_data = data.data.len() - 1;
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
