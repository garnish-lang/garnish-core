// use log::trace;

use crate::runtime::list::iterate_link_internal;
use crate::runtime::range::range_len;
use crate::{state_error, ExpressionDataType, GarnishLangRuntimeData, GarnishNumber, OrNumberError, RuntimeError, TypeConstants};

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

pub(crate) fn get_range<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    addr: Data::Size,
) -> Result<(Data::Number, Data::Number, Data::Number), RuntimeError<Data::Error>> {
    let (start, end) = this.get_range(addr)?;
    let (start, end) = match (this.get_data_type(start)?, this.get_data_type(end)?) {
        (ExpressionDataType::Number, ExpressionDataType::Number) => (this.get_number(start)?, this.get_number(end)?),
        (s, e) => state_error(format!("Invalid range values {:?} {:?}", s, e))?,
    };

    Ok((start, end, range_len::<Data>(start, end)?))
}

// push utilities

pub(crate) fn push_unit<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    this.add_unit().and_then(|v| this.push_register(v))?;
    Ok(())
}

pub(crate) fn push_number<Data: GarnishLangRuntimeData>(this: &mut Data, value: Data::Number) -> Result<(), RuntimeError<Data::Error>> {
    this.add_number(value).and_then(|v| this.push_register(v))?;
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

// public utilities

// modify so 'this' doesn't have to be mutable
pub fn iterate_link<Data: GarnishLangRuntimeData, Callback>(
    this: &mut Data,
    link: Data::Size,
    func: Callback,
) -> Result<(), RuntimeError<Data::Error>>
where
    Callback: FnMut(&mut Data, Data::Size, Data::Number) -> Result<bool, RuntimeError<Data::Error>>,
{
    iterate_link_internal(this, link, func)
}

pub fn link_count<Data: GarnishLangRuntimeData>(this: &mut Data, link: Data::Size) -> Result<Data::Number, RuntimeError<Data::Error>> {
    let mut count = Data::Number::zero();

    iterate_link_internal(this, link, |_, _, _| {
        count = count.increment().or_num_err()?;
        Ok(false)
    })?;

    Ok(count)
}

#[cfg(test)]
mod tests {
    use crate::{GarnishLangRuntimeData};
    use crate::simple::SimpleRuntimeData;

    #[test]
    fn add_data() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_number(100.into()).unwrap();

        assert_eq!(runtime.get_data_len(), 4);
    }

    #[test]
    fn get_data() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_number(100.into()).unwrap();
        let i2 = runtime.add_number(200.into()).unwrap();

        assert_eq!(runtime.get_number(i2).unwrap(), 200.into());
    }

    #[test]
    fn add_data_returns_addr() {
        let mut runtime = SimpleRuntimeData::new();
        let start = runtime.get_data_len();

        assert_eq!(runtime.add_number(100.into()).unwrap(), start);
        assert_eq!(runtime.add_number(200.into()).unwrap(), start + 1);
        assert_eq!(runtime.add_number(300.into()).unwrap(), start + 2);
        assert_eq!(runtime.add_number(400.into()).unwrap(), start + 3);
    }
}

#[cfg(test)]
mod internal {
    use crate::{
        runtime::utilities::{next_ref, next_two_raw_ref},
        GarnishLangRuntimeData,
    };
    use crate::simple::SimpleRuntimeData;

    #[test]
    fn next_ref_test() {
        let mut runtime = SimpleRuntimeData::new();
        runtime.add_number(10.into()).unwrap();
        runtime.add_number(20.into()).unwrap();

        runtime.push_register(2).unwrap();

        let result = next_ref(&mut runtime).unwrap();

        assert_eq!(result, 2);
    }

    #[test]
    fn next_ref_data_no_ref_is_error() {
        let mut runtime = SimpleRuntimeData::new();
        runtime.add_number(10.into()).unwrap();
        runtime.add_number(20.into()).unwrap();

        let result = next_ref(&mut runtime);

        assert!(result.is_err());
    }

    #[test]
    fn next_two_ref_data() {
        let mut runtime = SimpleRuntimeData::new();
        runtime.add_number(10.into()).unwrap();
        runtime.add_number(20.into()).unwrap();

        runtime.push_register(1).unwrap();
        runtime.push_register(2).unwrap();

        let (first, second) = next_two_raw_ref(&mut runtime).unwrap();

        assert_eq!(first, 2);
        assert_eq!(second, 1);
    }

    #[test]
    fn next_two_ref_data_one_ref_is_error() {
        let mut runtime = SimpleRuntimeData::new();
        runtime.add_number(10.into()).unwrap();
        runtime.add_number(20.into()).unwrap();

        runtime.push_register(1).unwrap();

        let result = next_two_raw_ref(&mut runtime);

        assert!(result.is_err());
    }

    #[test]
    fn next_two_ref_data_zero_refs_is_error() {
        let mut runtime = SimpleRuntimeData::new();
        runtime.add_number(10.into()).unwrap();
        runtime.add_number(20.into()).unwrap();

        let result = next_two_raw_ref(&mut runtime);

        assert!(result.is_err());
    }
}
