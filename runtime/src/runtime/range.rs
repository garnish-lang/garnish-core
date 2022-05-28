use crate::{next_two_raw_ref, push_unit, ExpressionDataType, GarnishLangRuntimeData, GarnishNumber, OrNumberError, RuntimeError};

pub(crate) fn make_range<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    make_range_internal(this, false, false)
}

pub(crate) fn make_start_exclusive_range<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    make_range_internal(this, true, false)
}

pub(crate) fn make_end_exclusive_range<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    make_range_internal(this, false, true)
}

pub(crate) fn make_exclusive_range<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    make_range_internal(this, true, true)
}

pub(crate) fn range_len<Data: GarnishLangRuntimeData>(start: Data::Number, end: Data::Number) -> Result<Data::Number, RuntimeError<Data::Error>> {
    end.subtract(start).or_num_err()?.increment().or_num_err()
}

fn make_range_internal<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    start_exclusive: bool,
    end_exclusive: bool,
) -> Result<(), RuntimeError<Data::Error>> {
    let (right_addr, left_addr) = next_two_raw_ref(this)?;
    let types = (this.get_data_type(left_addr)?, this.get_data_type(right_addr)?);

    match types {
        (ExpressionDataType::Number, ExpressionDataType::Number) => {
            let left_addr = if start_exclusive {
                this.add_number(this.get_number(left_addr)?.increment().or_num_err()?)?
            } else {
                left_addr
            };

            let right_addr = if end_exclusive {
                this.add_number(this.get_number(right_addr)?.decrement().or_num_err()?)?
            } else {
                right_addr
            };

            let addr = this.add_range(left_addr, right_addr)?;
            this.push_register(addr)?;
        }
        _ => {
            push_unit(this)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{runtime::GarnishRuntime, GarnishLangRuntimeData};
    use crate::testing_utilites::create_simple_runtime;

    #[test]
    fn range() {
        let mut runtime = create_simple_runtime();


        let d1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let d2 = runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.make_range().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        let (start, end) = runtime.get_data_mut().get_range(i).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(start).unwrap(), 10.into());
        assert_eq!(runtime.get_data_mut().get_number(end).unwrap(), 20.into());
    }

    #[test]
    fn start_exclusive() {
        let mut runtime = create_simple_runtime();


        let d1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let d2 = runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.make_start_exclusive_range().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        let (start, end) = runtime.get_data_mut().get_range(i).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(start).unwrap(), 11.into());
        assert_eq!(runtime.get_data_mut().get_number(end).unwrap(), 20.into());
    }

    #[test]
    fn end_exclusive() {
        let mut runtime = create_simple_runtime();


        let d1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let d2 = runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.make_end_exclusive_range().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        let (start, end) = runtime.get_data_mut().get_range(i).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(start).unwrap(), 10.into());
        assert_eq!(runtime.get_data_mut().get_number(end).unwrap(), 19.into());
    }

    #[test]
    fn exclusive() {
        let mut runtime = create_simple_runtime();


        let d1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let d2 = runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.get_data_mut().push_register(d1).unwrap();
        runtime.get_data_mut().push_register(d2).unwrap();

        runtime.make_exclusive_range().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        let (start, end) = runtime.get_data_mut().get_range(i).unwrap();
        assert_eq!(runtime.get_data_mut().get_number(start).unwrap(), 11.into());
        assert_eq!(runtime.get_data_mut().get_number(end).unwrap(), 19.into());
    }
}
