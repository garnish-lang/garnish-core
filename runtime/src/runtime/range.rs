use crate::{next_two_raw_ref, push_unit, ExpressionDataType, GarnishLangRuntimeData, RuntimeError, TypeConstants, state_error};

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

pub(crate) fn range_len<Data: GarnishLangRuntimeData>(start: Data::Integer, end: Data::Integer) -> Data::Integer {
    (end - start) + Data::Integer::one()
}

pub(crate) fn get_range_len<Data: GarnishLangRuntimeData>(this: &Data, addr: Data::Size) -> Result<Data::Integer, RuntimeError<Data::Error>> {
    let (start, end) = this.get_range(addr)?;
    let count = match (this.get_data_type(start)?, this.get_data_type(end)?) {
        (ExpressionDataType::Integer, ExpressionDataType::Integer) => {
            let (start, end) = (this.get_integer(start)?, this.get_integer(end)?);
            range_len::<Data>(start, end)
        }
        (s, e) => state_error(format!("Invalid range types {:?} {:?}", s, e))?
    };

    Ok(count)
}

fn make_range_internal<Data: GarnishLangRuntimeData>(this: &mut Data, start_exclusive: bool, end_exclusive: bool) -> Result<(), RuntimeError<Data::Error>> {
    let (right_addr, left_addr) = next_two_raw_ref(this)?;
    let types = (this.get_data_type(left_addr)?, this.get_data_type(right_addr)?);

    match types {
        (ExpressionDataType::Integer, ExpressionDataType::Integer) => {
            let left_addr = if start_exclusive {
                this.add_integer(this.get_integer(left_addr)? + Data::Integer::one())?
            } else {
                left_addr
            };

            let right_addr = if end_exclusive {
                this.add_integer(this.get_integer(right_addr)? - Data::Integer::one())?
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
    use crate::{runtime::GarnishRuntime, GarnishLangRuntimeData, SimpleRuntimeData, ExpressionDataType};

    #[test]
    fn range() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_integer(10).unwrap();
        let d2 = runtime.add_integer(20).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.make_range().unwrap();

        let (start, end) = runtime.get_range(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(runtime.get_integer(start).unwrap(), 10);
        assert_eq!(runtime.get_integer(end).unwrap(), 20);
    }

    #[test]
    fn range_with_incompatible_type() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_float(5.0).unwrap();
        let d2 = runtime.add_float(10.0).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.make_range().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn start_exclusive() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_integer(10).unwrap();
        let d2 = runtime.add_integer(20).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.make_start_exclusive_range().unwrap();

        let (start, end) = runtime.get_range(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(runtime.get_integer(start).unwrap(), 11);
        assert_eq!(runtime.get_integer(end).unwrap(), 20);
    }

    #[test]
    fn end_exclusive() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_integer(10).unwrap();
        let d2 = runtime.add_integer(20).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.make_end_exclusive_range().unwrap();

        let (start, end) = runtime.get_range(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(runtime.get_integer(start).unwrap(), 10);
        assert_eq!(runtime.get_integer(end).unwrap(), 19);
    }

    #[test]
    fn exclusive() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_integer(10).unwrap();
        let d2 = runtime.add_integer(20).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.make_exclusive_range().unwrap();

        let (start, end) = runtime.get_range(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(runtime.get_integer(start).unwrap(), 11);
        assert_eq!(runtime.get_integer(end).unwrap(), 19);
    }
}