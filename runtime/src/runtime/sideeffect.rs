use log::trace;

use crate::{state_error, GarnishLangRuntimeData, RuntimeError};

pub(crate) fn start_side_effect<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    trace!("Instruction - Start Side Effect");
    match this.get_current_value() {
        None => {
            let r = this.add_unit()?;
            this.push_value_stack(r)?;
        }
        Some(r) => this.push_value_stack(r)?
    }

    Ok(())
}

pub(crate) fn end_side_effect<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    trace!("Instruction - End Side Effect");
    match this.pop_value_stack() {
        Some(_) => Ok(()),
        None => state_error("Could not pop value at end of side effect.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::{runtime::GarnishRuntime, GarnishLangRuntimeData, SimpleRuntimeData, ExpressionDataType};

    #[test]
    fn start_side_effect() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_integer(10).unwrap();

        runtime.start_side_effect().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_current_value().unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn start_side_effect_with_value() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_integer(10).unwrap();

        runtime.push_value_stack(d1).unwrap();

        runtime.start_side_effect().unwrap();

        assert_eq!(runtime.get_current_value().unwrap(), d1);
    }

    #[test]
    fn end_side_effect() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_integer(10).unwrap();

        runtime.push_value_stack(d1).unwrap();

        runtime.end_side_effect().unwrap();

        assert_eq!(runtime.get_current_value(), None);
    }

    #[test]
    fn end_side_effect_no_value_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_integer(10).unwrap();

        let result = runtime.end_side_effect();

        assert!(result.is_err());
    }
}