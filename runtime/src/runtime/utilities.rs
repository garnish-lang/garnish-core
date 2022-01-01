// use log::trace;

use crate::{GarnishLangRuntimeData, RuntimeError, state_error};

pub(crate) fn next_ref<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<Data::Size, RuntimeError<Data::Error>> {
    match this.pop_register() {
        None => state_error(format!("No references in register."))?,
        Some(i) => Ok(i),
    }
}

pub(crate) fn next_two_raw_ref<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(Data::Size, Data::Size), RuntimeError<Data::Error>> {
    let first_ref = next_ref(this)?;
    let second_ref = next_ref(this)?;

    Ok((first_ref, second_ref))
}

// push utilities

pub(crate) fn push_unit<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    this.add_unit().and_then(|v| this.push_register(v))?;
    Ok(())
}

pub(crate) fn push_integer<Data: GarnishLangRuntimeData>(this: &mut Data, value: Data::Integer) -> Result<(), RuntimeError<Data::Error>> {
    this.add_integer(value).and_then(|v| this.push_register(v))?;
    Ok(())
}

pub(crate) fn push_boolean<Data: GarnishLangRuntimeData>(this: &mut Data, value: bool) -> Result<(), RuntimeError<Data::Error>> {
    match value {
        true => this.add_true(),
        false => this.add_false(),
    }
    .and_then(|v| this.push_register(v))?;

    Ok(())
}

pub(crate) fn push_pair<Data: GarnishLangRuntimeData>(this: &mut Data, left: Data::Size, right: Data::Size) -> Result<(), RuntimeError<Data::Error>> {
    this.add_pair((left, right)).and_then(|v| this.push_register(v))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn add_data() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_integer(100).unwrap();

        assert_eq!(runtime.get_data_len(), 4);
    }

    #[test]
    fn get_data() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_integer(100).unwrap();
        let i2 = runtime.add_integer(200).unwrap();

        assert_eq!(runtime.get_integer(i2).unwrap(), 200);
    }

    #[test]
    fn add_data_returns_addr() {
        let mut runtime = SimpleRuntimeData::new();
        let start = runtime.get_data_len();

        assert_eq!(runtime.add_integer(100).unwrap(), start);
        assert_eq!(runtime.add_integer(200).unwrap(), start + 1);
        assert_eq!(runtime.add_integer(300).unwrap(), start + 2);
        assert_eq!(runtime.add_integer(400).unwrap(), start + 3);
    }
}

#[cfg(test)]
mod internal {
    use crate::{
        runtime::utilities::{next_ref, next_two_raw_ref}, GarnishLangRuntimeData, SimpleRuntimeData,
    };

    #[test]
    fn next_ref_test() {
        let mut runtime = SimpleRuntimeData::new();
        runtime.add_integer(10).unwrap();
        runtime.add_integer(20).unwrap();

        runtime.push_register(2).unwrap();

        let result = next_ref(&mut runtime).unwrap();

        assert_eq!(result, 2);
    }

    #[test]
    fn next_ref_data_no_ref_is_error() {
        let mut runtime = SimpleRuntimeData::new();
        runtime.add_integer(10).unwrap();
        runtime.add_integer(20).unwrap();

        let result = next_ref(&mut runtime);

        assert!(result.is_err());
    }

    #[test]
    fn next_two_ref_data() {
        let mut runtime = SimpleRuntimeData::new();
        runtime.add_integer(10).unwrap();
        runtime.add_integer(20).unwrap();

        runtime.push_register(1).unwrap();
        runtime.push_register(2).unwrap();

        let (first, second) = next_two_raw_ref(&mut runtime).unwrap();

        assert_eq!(first, 2);
        assert_eq!(second, 1);
    }

    #[test]
    fn next_two_ref_data_one_ref_is_error() {
        let mut runtime = SimpleRuntimeData::new();
        runtime.add_integer(10).unwrap();
        runtime.add_integer(20).unwrap();

        runtime.push_register(1).unwrap();

        let result = next_two_raw_ref(&mut runtime);

        assert!(result.is_err());
    }

    #[test]
    fn next_two_ref_data_zero_refs_is_error() {
        let mut runtime = SimpleRuntimeData::new();
        runtime.add_integer(10).unwrap();
        runtime.add_integer(20).unwrap();

        let result = next_two_raw_ref(&mut runtime);

        assert!(result.is_err());
    }
}
