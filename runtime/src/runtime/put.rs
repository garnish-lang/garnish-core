use crate::{next_ref, push_unit, GarnishLangRuntimeData, GarnishLangRuntimeResult, NestInto, error};
use log::trace;

pub(crate) fn put<Data: GarnishLangRuntimeData>(this: &mut Data, i: usize) -> GarnishLangRuntimeResult<Data::Error> {
    trace!("Instruction - Put | Data - {:?}", i);
    match i >= this.get_end_of_constant_data() {
        true => Err(error(format!(
            "Attempting to put reference to {:?} which is out of bounds of constant data that ends at {:?}.",
            i,
            this.get_end_of_constant_data()
        ))),
        false => this.push_register(i).nest_into(),
    }
}

pub(crate) fn put_input<Data: GarnishLangRuntimeData>(this: &mut Data) -> GarnishLangRuntimeResult<Data::Error> {
    trace!("Instruction - Put Input");

    match this.get_current_value() {
        None => push_unit(this),
        Some(i) => this.push_register(i).nest_into(),
    }
}

pub(crate) fn push_input<Data: GarnishLangRuntimeData>(this: &mut Data) -> GarnishLangRuntimeResult<Data::Error> {
    trace!("Instruction - Push Input");
    let r = next_ref(this)?;

    this.push_value_stack(r).nest_into()?;
    this.push_value_stack(r).nest_into()
}

pub(crate) fn push_result<Data: GarnishLangRuntimeData>(this: &mut Data) -> GarnishLangRuntimeResult<Data::Error> {
    trace!("Instruction - Output Result");

    let r = next_ref(this)?;
    match this.get_current_value_mut() {
        None => Err(error(format!("No inputs availble to update for update value operation.")))?,
        Some(v) => *v = r
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{runtime::GarnishRuntime, ExpressionData, ExpressionDataType, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

    #[test]
    fn put() {
        let mut runtime = SimpleRuntimeData::new();
        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.set_end_of_constant(runtime.get_data_len()).unwrap();

        runtime.put(1).unwrap();

        assert_eq!(*runtime.get_register().get(0).unwrap(), 1);
    }

    #[test]
    fn put_outside_of_constant_data() {
        let mut runtime = SimpleRuntimeData::new();
        runtime.add_data(ExpressionData::integer(10)).unwrap();

        let result = runtime.put(0);

        assert!(result.is_err());
    }

    #[test]
    fn put_input() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.push_value_stack(2).unwrap();

        runtime.put_value().unwrap();

        assert_eq!(*runtime.get_register().get(0).unwrap(), 2);
    }

    #[test]
    fn put_input_is_unit_if_no_input() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.put_value().unwrap();

        assert_eq!(runtime.get_data_type(3).unwrap(), ExpressionDataType::Unit);
        assert_eq!(*runtime.get_register().get(0).unwrap(), 3);
    }

    #[test]
    fn push_input() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        runtime.push_register(2).unwrap();

        runtime.push_value().unwrap();

        assert_eq!(runtime.get_value(0).unwrap(), 2usize);
        assert_eq!(runtime.get_current_value().unwrap(), 2usize);
    }

    #[test]
    fn push_input_no_register_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        assert!(runtime.push_value().is_err());
    }

    #[test]
    fn push_result() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.push_instruction(Instruction::UpdateValue, None).unwrap();

        runtime.push_register(1).unwrap();

        runtime.push_value_stack(1).unwrap();

        runtime.update_value().unwrap();

        assert_eq!(runtime.get_value_count(), 1);
        assert_eq!(runtime.get_current_value().unwrap(), 1usize);
        assert_eq!(runtime.get_integer(runtime.get_current_value().unwrap()).unwrap(), 10i64);
    }

    #[test]
    fn push_result_no_register_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.push_instruction(Instruction::UpdateValue, None).unwrap();

        assert!(runtime.update_value().is_err());
    }
}
