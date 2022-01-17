use crate::{next_two_raw_ref, push_unit, ExpressionDataType, GarnishLangRuntimeData, RuntimeError};

pub(crate) fn type_cast<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    let (right, left) = next_two_raw_ref(this)?;

    match (this.get_data_type(left)?, this.get_data_type(right)?) {
        // Unit and Boolean
        (ExpressionDataType::Unit, ExpressionDataType::True) | (ExpressionDataType::False, ExpressionDataType::True) => {
            this.add_false().and_then(|r| this.push_register(r))?
        }
        (ExpressionDataType::Unit, ExpressionDataType::False) => this.add_true().and_then(|r| this.push_register(r))?,

        // Final Catches
        (ExpressionDataType::Unit, _) => push_unit(this)?,
        (_, ExpressionDataType::False) => this.add_false().and_then(|r| this.push_register(r))?,
        (_, ExpressionDataType::True) => this.add_true().and_then(|r| this.push_register(r))?,
        _ => push_unit(this)?,
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn cast_to_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let int = runtime.add_integer(10).unwrap();
        let unit = runtime.add_unit().unwrap();

        runtime.push_register(int).unwrap();
        runtime.push_register(unit).unwrap();

        runtime.type_cast().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn cast_to_true() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_integer(10).unwrap();
        let d2 = runtime.add_true().unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn cast_unit_to_true() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_unit().unwrap();
        let d2 = runtime.add_true().unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn cast_false_to_true() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_false().unwrap();
        let d2 = runtime.add_true().unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn cast_to_false() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_integer(10).unwrap();
        let d2 = runtime.add_false().unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn cast_unit_to_false() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_unit().unwrap();
        let d2 = runtime.add_false().unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn cast_true_to_false() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_true().unwrap();
        let d2 = runtime.add_false().unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }
}
