use crate::{next_two_raw_ref, push_unit, ExpressionDataType, GarnishLangRuntimeData, RuntimeError};

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

fn make_range_internal<Data: GarnishLangRuntimeData>(this: &mut Data, start_exclusive: bool, end_exclusive: bool) -> Result<(), RuntimeError<Data::Error>> {
    let (right_addr, left_addr) = next_two_raw_ref(this)?;
    let types = (this.get_data_type(left_addr)?, this.get_data_type(right_addr)?);

    match types {
        (ExpressionDataType::Integer, ExpressionDataType::Integer)
        | (ExpressionDataType::Integer, ExpressionDataType::Unit)
        | (ExpressionDataType::Unit, ExpressionDataType::Integer)
        | (ExpressionDataType::Unit, ExpressionDataType::Unit) => {
            let addr = this.add_range(left_addr, right_addr, start_exclusive, end_exclusive)?;
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

        assert_eq!(runtime.get_range(runtime.get_register(0).unwrap()).unwrap(), (d1, d2, false, false))
    }

    #[test]
    fn range_integer_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_integer(10).unwrap();
        let d2 = runtime.add_unit().unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.make_range().unwrap();

        assert_eq!(runtime.get_range(runtime.get_register(0).unwrap()).unwrap(), (d1, d2, false, false))
    }

    #[test]
    fn range_unit_integer() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_unit().unwrap();
        let d2 = runtime.add_integer(10).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.make_range().unwrap();

        assert_eq!(runtime.get_range(runtime.get_register(0).unwrap()).unwrap(), (d1, d2, false, false))
    }

    #[test]
    fn range_unit_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_unit().unwrap();
        let d2 = runtime.add_unit().unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.make_range().unwrap();

        assert_eq!(runtime.get_range(runtime.get_register(0).unwrap()).unwrap(), (d1, d2, false, false))
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

        assert_eq!(runtime.get_range(runtime.get_register(0).unwrap()).unwrap(), (d1, d2, true, false))
    }

    #[test]
    fn end_exclusive() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_integer(10).unwrap();
        let d2 = runtime.add_integer(20).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.make_end_exclusive_range().unwrap();

        assert_eq!(runtime.get_range(runtime.get_register(0).unwrap()).unwrap(), (d1, d2, false, true))
    }

    #[test]
    fn exclusive() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_integer(10).unwrap();
        let d2 = runtime.add_integer(20).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.make_exclusive_range().unwrap();

        assert_eq!(runtime.get_range(runtime.get_register(0).unwrap()).unwrap(), (d1, d2, true, true))
    }
}