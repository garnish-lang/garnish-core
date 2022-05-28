use crate::{next_two_raw_ref, push_pair, GarnishLangRuntimeData, RuntimeError};

pub(crate) fn make_pair<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    let (right_addr, left_addr) = next_two_raw_ref(this)?;

    push_pair(this, left_addr, right_addr)
}

#[cfg(test)]
mod tests {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, Instruction};
    use crate::testing_utilites::create_simple_runtime;

    #[test]
    fn make_pair() {
        let mut runtime = create_simple_runtime();


        let i1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let i2 = runtime.get_data_mut().add_symbol(20).unwrap();
        let start = runtime.get_data_mut().get_data_len();

        runtime.get_data_mut().push_register(i1).unwrap();
        runtime.get_data_mut().push_register(i2).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::MakePair, None).unwrap();

        runtime.make_pair().unwrap();

        assert_eq!(runtime.get_data_mut().get_data_type(start).unwrap(), ExpressionDataType::Pair);
        assert_eq!(runtime.get_data_mut().get_pair(start).unwrap(), (i1, i2));

        assert_eq!(runtime.get_data_mut().get_register(0).unwrap(), start);
    }

    #[test]
    fn make_pair_no_refs_is_err() {
        let mut runtime = create_simple_runtime();


        runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().add_symbol(20).unwrap();

        runtime.get_data_mut().push_instruction(Instruction::MakePair, None).unwrap();

        let result = runtime.make_pair();

        assert!(result.is_err());
    }
}
