use crate::{next_two_raw_ref, GarnishLangRuntimeData, RuntimeError};

pub(crate) fn concat<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    let (right_addr, left_addr) = next_two_raw_ref(this)?;

    this.add_concatenation(left_addr, right_addr).and_then(|v| this.push_register(v))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn make_pair() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_number(10.into()).unwrap();
        let i2 = runtime.add_symbol(20).unwrap();
        let start = runtime.get_data_len();

        runtime.push_register(i1).unwrap();
        runtime.push_register(i2).unwrap();

        runtime.concat().unwrap();

        assert_eq!(runtime.get_data_type(start).unwrap(), ExpressionDataType::Concatenation);
        assert_eq!(runtime.get_concatenation(start).unwrap(), (i1, i2));

        assert_eq!(runtime.get_register(0).unwrap(), start);
    }

    #[test]
    fn make_pair_no_refs_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_number(10.into()).unwrap();
        runtime.add_symbol(20).unwrap();

        let result = runtime.concat();

        assert!(result.is_err());
    }
}
