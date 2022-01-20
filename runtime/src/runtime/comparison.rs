use crate::{
    next_two_raw_ref, push_boolean, ExpressionDataType, GarnishLangRuntimeData, GarnishNumber, OrNumberError,
    RuntimeError, TypeConstants,
};
use std::cmp::Ordering;

pub fn less_than<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    perform_comparison(this, Ordering::Greater).and_then(|result| push_boolean(this, result.is_lt()))
}

pub fn less_than_or_equal<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    perform_comparison(this, Ordering::Greater).and_then(|result| push_boolean(this, result.is_le()))
}

pub fn greater_than<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    perform_comparison(this, Ordering::Less).and_then(|result| push_boolean(this, result.is_gt()))
}

pub fn greater_than_or_equal<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    perform_comparison(this, Ordering::Less).and_then(|result| push_boolean(this, result.is_ge()))
}

fn perform_comparison<Data: GarnishLangRuntimeData>(this: &mut Data, false_ord: Ordering) -> Result<Ordering, RuntimeError<Data::Error>> {
    let (right, left) = next_two_raw_ref(this)?;

    let result = match (this.get_data_type(left)?, this.get_data_type(right)?) {
        (ExpressionDataType::Number, ExpressionDataType::Number) => this.get_number(left)?.cmp(&this.get_number(right)?),
        (ExpressionDataType::Char, ExpressionDataType::Char) => this.get_char(left)?.cmp(&this.get_char(right)?),
        (ExpressionDataType::Byte, ExpressionDataType::Byte) => this.get_byte(left)?.cmp(&this.get_byte(right)?),
        (ExpressionDataType::CharList, ExpressionDataType::CharList) => {
            cmp_list(this, left, right, Data::get_char_list_item, Data::get_char_list_len)?
        }
        (ExpressionDataType::ByteList, ExpressionDataType::ByteList) => {
            cmp_list(this, left, right, Data::get_byte_list_item, Data::get_byte_list_len)?
        }
        _ => return Ok(false_ord),
    };

    Ok(result)
}

fn cmp_list<Data: GarnishLangRuntimeData, T: Ord, GetFunc, LenFunc>(
    this: &mut Data,
    left: Data::Size,
    right: Data::Size,
    get_func: GetFunc,
    len_func: LenFunc,
) -> Result<Ordering, RuntimeError<Data::Error>>
where
    GetFunc: Fn(&Data, Data::Size, Data::Number) -> Result<T, Data::Error>,
    LenFunc: Fn(&Data, Data::Size) -> Result<Data::Size, Data::Error>,
{
    let (len1, len2) = (
        Data::size_to_integer(len_func(this, left)?),
        Data::size_to_integer(len_func(this, right)?),
    );

    let min = if len1 < len2 { len1 } else { len2 };
    let mut count = Data::Number::zero();

    while count < min {
        match get_func(this, left, count)?.cmp(&get_func(this, right, count)?) {
            Ordering::Equal => (),
            non_eq => return Ok(non_eq),
        }
        count = count.increment().or_num_err()?;
    }

    Ok(len1.cmp(&len2))
}

#[cfg(test)]
mod general {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, SimpleRuntimeData};

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
    use crate::testing_utilites::{add_byte_list, add_char_list};
    use crate::{runtime::GarnishRuntime, DataError, ExpressionDataType, GarnishLangRuntimeData, RuntimeError, SimpleRuntimeData};

    fn perform_compare<Setup, Op>(expected: bool, op_name: &str, op: Op, setup: Setup)
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

    fn perform_all_compare<Setup>(less_than: bool, less_than_equal: bool, greater_than: bool, greater_than_equal: bool, setup: Setup)
    where
        Setup: Copy + Fn(&mut SimpleRuntimeData) -> (usize, usize),
    {
        perform_compare(less_than, "less than", SimpleRuntimeData::less_than, setup);
        perform_compare(less_than_equal, "less than or equal", SimpleRuntimeData::less_than_or_equal, setup);
        perform_compare(greater_than, "greater than", SimpleRuntimeData::greater_than, setup);
        perform_compare(
            greater_than_equal,
            "greater than or equal",
            SimpleRuntimeData::greater_than_or_equal,
            setup,
        );
    }

    #[test]
    fn units_are_false() {
        perform_all_compare(false, false, false, false, |runtime| {
            (runtime.add_unit().unwrap(), runtime.add_unit().unwrap())
        });
    }

    #[test]
    fn trues_are_false() {
        perform_all_compare(false, false, false, false, |runtime| {
            (runtime.add_true().unwrap(), runtime.add_true().unwrap())
        });
    }

    #[test]
    fn falses_are_false() {
        perform_all_compare(false, false, false, false, |runtime| {
            (runtime.add_false().unwrap(), runtime.add_false().unwrap())
        });
    }

    #[test]
    fn numbers_less_than() {
        perform_all_compare(true, true, false, false, |runtime| {
            (runtime.add_number(10).unwrap(), runtime.add_number(20).unwrap())
        });
    }

    #[test]
    fn numbers_equal() {
        perform_all_compare(false, true, false, true, |runtime| {
            (runtime.add_number(20).unwrap(), runtime.add_number(20).unwrap())
        });
    }

    #[test]
    fn numbers_greater_than() {
        perform_all_compare(false, false, true, true, |runtime| {
            (runtime.add_number(20).unwrap(), runtime.add_number(10).unwrap())
        });
    }

    #[test]
    fn chars_less_than() {
        perform_all_compare(true, true, false, false, |runtime| {
            (runtime.add_char('d').unwrap(), runtime.add_char('f').unwrap())
        });
    }

    #[test]
    fn chars_equal() {
        perform_all_compare(false, true, false, true, |runtime| {
            (runtime.add_char('d').unwrap(), runtime.add_char('d').unwrap())
        });
    }

    #[test]
    fn chars_greater_than() {
        perform_all_compare(false, false, true, true, |runtime| {
            (runtime.add_char('d').unwrap(), runtime.add_char('b').unwrap())
        });
    }

    #[test]
    fn bytes_less_than() {
        perform_all_compare(true, true, false, false, |runtime| {
            (runtime.add_byte(10).unwrap(), runtime.add_byte(20).unwrap())
        });
    }

    #[test]
    fn bytes_equal() {
        perform_all_compare(false, true, false, true, |runtime| {
            (runtime.add_byte(20).unwrap(), runtime.add_byte(20).unwrap())
        });
    }

    #[test]
    fn bytes_greater_than() {
        perform_all_compare(false, false, true, true, |runtime| {
            (runtime.add_byte(20).unwrap(), runtime.add_byte(10).unwrap())
        });
    }

    #[test]
    fn char_list_less_than() {
        perform_all_compare(true, true, false, false, |runtime| {
            (add_char_list(runtime, "aaa"), add_char_list(runtime, "bbb"))
        });
    }

    #[test]
    fn char_list_less_than_dif_len() {
        perform_all_compare(true, true, false, false, |runtime| {
            (add_char_list(runtime, "aaa"), add_char_list(runtime, "aaaaa"))
        });
    }

    #[test]
    fn char_list_equal() {
        perform_all_compare(false, true, false, true, |runtime| {
            (add_char_list(runtime, "aaa"), add_char_list(runtime, "aaa"))
        });
    }

    #[test]
    fn char_list_greater_than() {
        perform_all_compare(false, false, true, true, |runtime| {
            (add_char_list(runtime, "bbb"), add_char_list(runtime, "aaa"))
        });
    }

    #[test]
    fn char_list_greater_than_dif_len() {
        perform_all_compare(false, false, true, true, |runtime| {
            (add_char_list(runtime, "aaaaa"), add_char_list(runtime, "aaa"))
        });
    }

    #[test]
    fn byte_list_less_than() {
        perform_all_compare(true, true, false, false, |runtime| {
            (add_byte_list(runtime, "aaa"), add_byte_list(runtime, "bbb"))
        });
    }

    #[test]
    fn byte_list_less_than_dif_len() {
        perform_all_compare(true, true, false, false, |runtime| {
            (add_byte_list(runtime, "aaa"), add_byte_list(runtime, "aaaaa"))
        });
    }

    #[test]
    fn byte_list_equal() {
        perform_all_compare(false, true, false, true, |runtime| {
            (add_byte_list(runtime, "aaa"), add_byte_list(runtime, "aaa"))
        });
    }

    #[test]
    fn byte_list_greater_than() {
        perform_all_compare(false, false, true, true, |runtime| {
            (add_byte_list(runtime, "bbb"), add_byte_list(runtime, "aaa"))
        });
    }

    #[test]
    fn byte_list_greater_than_dif_len() {
        perform_all_compare(false, false, true, true, |runtime| {
            (add_byte_list(runtime, "aaaaa"), add_byte_list(runtime, "aaa"))
        });
    }
}
