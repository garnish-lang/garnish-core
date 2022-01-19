use crate::runtime::logical::or;
use crate::{
    next_ref, next_two_raw_ref, push_boolean, push_number, push_unit, ExpressionDataType, GarnishLangRuntimeData, GarnishNumber, RuntimeError,
};
use std::cmp::Ordering;

pub fn less_than<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    perform_comparison(this, Ordering::is_lt)
}

pub fn less_than_or_equal<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    perform_comparison(this, Ordering::is_le)
}

pub fn greater_than<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    perform_comparison(this, Ordering::is_gt)
}

pub fn greater_than_or_equal<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    perform_comparison(this, Ordering::is_ge)
}

fn perform_comparison<Data: GarnishLangRuntimeData, NumberOp>(
    this: &mut Data,
    to_bool: NumberOp
) -> Result<(), RuntimeError<Data::Error>>
where NumberOp: Fn(Ordering) -> bool
{
    let (right, left) = next_two_raw_ref(this)?;

    let result = match (this.get_data_type(left)?, this.get_data_type(right)?) {
        (ExpressionDataType::Number, ExpressionDataType::Number) => this.get_number(left)?.cmp(&this.get_number(right)?),
        (ExpressionDataType::Char, ExpressionDataType::Char) =>  this.get_char(left)?.cmp(&this.get_char(right)?),
        (ExpressionDataType::Byte, ExpressionDataType::Byte) => this.get_byte(left)?.cmp(&this.get_byte(right)?),
        _ => {
            push_boolean(this, false)?;
            return Ok(())
        },
    };

    push_boolean(this, to_bool(result))
}

#[cfg(test)]
mod general {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

    #[test]
    fn less_than_no_references_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_number(10).unwrap();
        runtime.add_number(10).unwrap();
        let result = runtime.less_than();

        assert!(result.is_err());
    }

    #[test]
    fn less_than_of_unsupported_comparison_is_false() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10).unwrap();
        let exp1 = runtime.add_expression(10).unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(exp1).unwrap();

        runtime.less_than().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }
}

#[cfg(test)]
mod less_than {
    use crate::{runtime::GarnishRuntime, DataError, ExpressionDataType, GarnishLangRuntimeData, Instruction, RuntimeError, SimpleRuntimeData};

    fn perform_compare<Setup, Op>(expected: bool, op_name: &str, op: Op, mut setup: Setup)
    where
        Op: Fn(&mut SimpleRuntimeData) -> Result<(), RuntimeError<DataError>>,
        Setup: Copy + Fn(&mut SimpleRuntimeData) -> (usize, usize),
    {
        let mut runtime = SimpleRuntimeData::new();

        let registers = setup(&mut runtime);

        runtime.push_register(registers.0).unwrap();
        runtime.push_register(registers.1).unwrap();

        op(&mut runtime).unwrap();

        let result = runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap();
        let expected_type = match expected {
            true => ExpressionDataType::True,
            false => ExpressionDataType::False,
        };

        assert_eq!(result, expected_type, "For op {:?}", op_name);
    }

    fn perform_all_compare<Setup>(less_than: bool, less_than_equal: bool, greater_than: bool, greater_than_equal: bool, mut setup: Setup)
    where
        Setup: Copy + Fn(&mut SimpleRuntimeData) -> (usize, usize),
    {
        perform_compare(less_than, "less than", SimpleRuntimeData::less_than, setup);
        perform_compare(less_than_equal, "less than or equal", SimpleRuntimeData::less_than_or_equal, setup);
        perform_compare(greater_than, "greater than", SimpleRuntimeData::greater_than, setup);
        perform_compare(greater_than_equal, "greater than or equal", SimpleRuntimeData::greater_than_or_equal, setup);
    }

    #[test]
    fn units_are_false() {
        perform_all_compare(
            false,
            false,
            false,
            false,
            | runtime | (runtime.add_unit().unwrap(), runtime.add_unit().unwrap()),
        );
    }

    #[test]
    fn trues_are_false() {
        perform_all_compare(
            false,
            false,
            false,
            false,
            | runtime | (runtime.add_true().unwrap(), runtime.add_true().unwrap()),
        );
    }

    #[test]
    fn falses_are_false() {
        perform_all_compare(
            false,
            false,
            false,
            false,
            | runtime | (runtime.add_false().unwrap(), runtime.add_false().unwrap()),
        );
    }

    #[test]
    fn numbers_less_than() {
        perform_all_compare(
            true,
            true,
            false,
            false,
            | runtime | (runtime.add_number(10).unwrap(), runtime.add_number(20).unwrap()),
        );
    }

    #[test]
    fn numbers_equal() {
        perform_all_compare(
            false,
            true,
            false,
            true,
            | runtime | (runtime.add_number(20).unwrap(), runtime.add_number(20).unwrap()),
        );
    }

    #[test]
    fn numbers_greater_than() {
        perform_all_compare(
            false,
            false,
            true,
            true,
            | runtime | (runtime.add_number(20).unwrap(), runtime.add_number(10).unwrap()),
        );
    }

    #[test]
    fn chars_less_than() {
        perform_all_compare(
            true,
            true,
            false,
            false,
            | runtime | (runtime.add_char('d').unwrap(), runtime.add_char('f').unwrap()),
        );
    }

    #[test]
    fn chars_equal() {
        perform_all_compare(
            false,
            true,
            false,
            true,
            | runtime | (runtime.add_char('d').unwrap(), runtime.add_char('d').unwrap()),
        );
    }

    #[test]
    fn chars_greater_than() {
        perform_all_compare(
            false,
            false,
            true,
            true,
            | runtime | (runtime.add_char('d').unwrap(), runtime.add_char('b').unwrap()),
        );
    }

    #[test]
    fn bytes_less_than() {
        perform_all_compare(
            true,
            true,
            false,
            false,
            | runtime | (runtime.add_byte(10).unwrap(), runtime.add_byte(20).unwrap()),
        );
    }

    #[test]
    fn bytes_equal() {
        perform_all_compare(
            false,
            true,
            false,
            true,
            | runtime | (runtime.add_byte(20).unwrap(), runtime.add_byte(20).unwrap()),
        );
    }

    #[test]
    fn bytes_greater_than() {
        perform_all_compare(
            false,
            false,
            true,
            true,
            | runtime | (runtime.add_byte(20).unwrap(), runtime.add_byte(10).unwrap()),
        );
    }

}
