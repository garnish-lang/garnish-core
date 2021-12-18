// use log::trace;

use crate::{error, GarnishLangRuntimeData, GarnishLangRuntimeResult, NestInto};

pub(crate) fn next_ref<Data: GarnishLangRuntimeData>(this: &mut Data) -> GarnishLangRuntimeResult<Data::Error, usize> {
    match this.pop_register() {
        None => Err(error(format!("No references in register.")))?,
        Some(i) => Ok(i),
    }
}

pub(crate) fn next_two_raw_ref<Data: GarnishLangRuntimeData>(this: &mut Data) -> GarnishLangRuntimeResult<Data::Error, (usize, usize)> {
    let first_ref = next_ref(this)?;
    let second_ref = next_ref(this)?;

    Ok((first_ref, second_ref))
}

// push utilities

pub fn push_unit<Data: GarnishLangRuntimeData>(this: &mut Data) -> GarnishLangRuntimeResult<Data::Error> {
    this.add_unit().and_then(|v| this.push_register(v)).nest_into()
}

pub fn push_integer<Data: GarnishLangRuntimeData>(this: &mut Data, value: i64) -> GarnishLangRuntimeResult<Data::Error> {
    this.add_integer(value).and_then(|v| this.push_register(v)).nest_into()
}

pub fn push_boolean<Data: GarnishLangRuntimeData>(this: &mut Data, value: bool) -> GarnishLangRuntimeResult<Data::Error> {
    match value {
        true => this.add_true(),
        false => this.add_false(),
    }
    .and_then(|v| this.push_register(v))
    .nest_into()
}

pub fn push_list<Data: GarnishLangRuntimeData>(this: &mut Data, list: Vec<usize>, associations: Vec<usize>) -> GarnishLangRuntimeResult<Data::Error> {
    this.add_list(list, associations).and_then(|v| this.push_register(v)).nest_into()
}

pub fn push_pair<Data: GarnishLangRuntimeData>(this: &mut Data, left: usize, right: usize) -> GarnishLangRuntimeResult<Data::Error> {
    this.add_pair((left, right)).and_then(|v| this.push_register(v)).nest_into()
}

#[cfg(test)]
mod tests {
    use crate::{runtime::data::GarnishLangRuntimeData, ExpressionData, SimpleRuntimeData};

    #[test]
    fn add_data() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(100)).unwrap();

        assert_eq!(runtime.get_data_len(), 2);
    }

    #[test]
    fn get_data() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(100)).unwrap();
        runtime.add_data(ExpressionData::integer(200)).unwrap();

        assert_eq!(runtime.get_integer(2).unwrap(), 200);
    }

    #[test]
    fn end_constant_data() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(100)).unwrap();
        runtime.add_data(ExpressionData::integer(200)).unwrap();
        runtime.set_end_of_constant(runtime.get_data_len()).unwrap();

        assert_eq!(runtime.get_end_of_constant_data(), 3);
    }

    #[test]
    fn add_data_returns_addr() {
        let mut runtime = SimpleRuntimeData::new();

        assert_eq!(runtime.add_data(ExpressionData::integer(100)).unwrap(), 1);
        assert_eq!(runtime.add_data(ExpressionData::integer(100)).unwrap(), 2);
        assert_eq!(runtime.add_data(ExpressionData::integer(100)).unwrap(), 3);
        assert_eq!(runtime.add_data(ExpressionData::integer(100)).unwrap(), 4);
    }

    #[test]
    fn remove_data() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::symbol(&"false".to_string(), 0)).unwrap();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.set_end_of_constant(runtime.get_data_len()).unwrap();

        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        // runtime.add_instruction(Instruction::PerformAddition, None).unwrap();

        runtime.remove_non_constant_data().unwrap();

        assert_eq!(runtime.get_data_len(), 5);
    }
}

#[cfg(test)]
mod internal {
    use crate::{
        runtime::utilities::{next_ref, next_two_raw_ref},
        ExpressionData, GarnishLangRuntimeData, SimpleRuntimeData,
    };

    #[test]
    fn next_ref_test() {
        let mut runtime = SimpleRuntimeData::new();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.push_register(2).unwrap();

        let result = next_ref(&mut runtime).unwrap();

        assert_eq!(result, 2);
    }

    #[test]
    fn next_ref_data_no_ref_is_error() {
        let mut runtime = SimpleRuntimeData::new();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        let result = next_ref(&mut runtime);

        assert!(result.is_err());
    }

    #[test]
    fn next_two_ref_data() {
        let mut runtime = SimpleRuntimeData::new();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.push_register(1).unwrap();
        runtime.push_register(2).unwrap();

        let (first, second) = next_two_raw_ref(&mut runtime).unwrap();

        assert_eq!(first, 2);
        assert_eq!(second, 1);
    }

    #[test]
    fn next_two_ref_data_one_ref_is_error() {
        let mut runtime = SimpleRuntimeData::new();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.push_register(1).unwrap();

        let result = next_two_raw_ref(&mut runtime);

        assert!(result.is_err());
    }

    #[test]
    fn next_two_ref_data_zero_refs_is_error() {
        let mut runtime = SimpleRuntimeData::new();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        let result = next_two_raw_ref(&mut runtime);

        assert!(result.is_err());
    }
}
