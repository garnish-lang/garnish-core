use crate::{next_ref, push_unit, state_error, GarnishLangRuntimeData, RuntimeError};

pub(crate) fn put<Data: GarnishLangRuntimeData>(this: &mut Data, i: Data::Size) -> Result<(), RuntimeError<Data::Error>> {
    match i >= this.get_data_len() {
        true => state_error(format!(
            "Attempting to put reference to {:?} which is outside of data bounds {:?}.",
            i,
            this.get_data_len()
        ))?,
        false => this.push_register(i)?,
    }

    Ok(())
}

pub(crate) fn put_input<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    match this.get_current_value() {
        None => push_unit(this)?,
        Some(i) => this.push_register(i)?,
    }

    Ok(())
}

pub(crate) fn push_input<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    let r = next_ref(this)?;

    this.push_value_stack(r)?;

    Ok(())
}

pub(crate) fn push_result<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    let r = next_ref(this)?;
    match this.get_current_value_mut() {
        None => state_error(format!("No inputs availble to update for update value operation."))?,
        Some(v) => *v = r,
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, Instruction};
    use crate::testing_utilites::create_simple_runtime;

    #[test]
    fn put() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().add_number(10.into()).unwrap();
        let max = runtime.get_data_mut().get_data_len();
        runtime.get_data_mut().set_end_of_constant(max).unwrap();

        runtime.put(1).unwrap();

        assert_eq!(runtime.get_data_mut().get_register(0).unwrap(), 1);
    }

    #[test]
    fn put_outside_of_constant_data() {
        let mut runtime = create_simple_runtime();

        runtime.get_data_mut().add_number(10.into()).unwrap();

        let result = runtime.put(10);

        assert!(result.is_err());
    }

    #[test]
    fn put_input() {
        let mut runtime = create_simple_runtime();


        runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.get_data_mut().push_value_stack(2).unwrap();

        runtime.put_value().unwrap();

        assert_eq!(runtime.get_data_mut().get_register(0).unwrap(), 2);
    }

    #[test]
    fn put_input_is_unit_if_no_input() {
        let mut runtime = create_simple_runtime();


        runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.put_value().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn push_input() {
        let mut runtime = create_simple_runtime();


        runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().add_number(20.into()).unwrap();

        runtime.get_data_mut().push_register(2).unwrap();

        runtime.push_value().unwrap();

        assert_eq!(runtime.get_data_mut().get_value_stack_len(), 1);
        assert_eq!(runtime.get_data_mut().get_value(0).unwrap(), 2usize);
        assert_eq!(runtime.get_data_mut().get_current_value().unwrap(), 2usize);
    }

    #[test]
    fn push_input_no_register_is_err() {
        let mut runtime = create_simple_runtime();


        runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().add_number(20.into()).unwrap();

        assert!(runtime.push_value().is_err());
    }

    #[test]
    fn push_result() {
        let mut runtime = create_simple_runtime();


        let i1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::UpdateValue, None).unwrap();

        runtime.get_data_mut().push_register(i1).unwrap();

        runtime.get_data_mut().push_value_stack(i1).unwrap();

        runtime.update_value().unwrap();

        let i = runtime.get_data_mut().get_current_value().unwrap();
        assert_eq!(runtime.get_data_mut().get_value_stack_len(), 1);
        assert_eq!(runtime.get_data_mut().get_current_value().unwrap(), i1);
        assert_eq!(runtime.get_data_mut().get_number(i).unwrap(), 10.into());
    }

    #[test]
    fn push_result_no_register_is_err() {
        let mut runtime = create_simple_runtime();


        runtime.get_data_mut().add_number(10.into()).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::UpdateValue, None).unwrap();

        assert!(runtime.update_value().is_err());
    }
}
