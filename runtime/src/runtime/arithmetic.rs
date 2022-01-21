use log::trace;

use crate::{
    next_ref, next_two_raw_ref, push_number, push_unit, ExpressionDataType, GarnishLangRuntimeContext, GarnishLangRuntimeData, GarnishNumber,
    Instruction, RuntimeError, TypeConstants,
};

pub fn add<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, Instruction::Add, Data::Number::plus, context)
}

pub fn subtract<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, Instruction::Subtract, Data::Number::subtract, context)
}

pub fn multiply<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, Instruction::Multiply, Data::Number::multiply, context)
}

pub fn power<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, Instruction::Power, Data::Number::power, context)
}

pub fn divide<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, Instruction::Power, Data::Number::divide, context)
}

pub fn integer_divide<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, Instruction::IntegerDivide, Data::Number::integer_divide, context)
}

pub fn remainder<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, Instruction::Remainder, Data::Number::remainder, context)
}

pub fn absolute_value<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<(), RuntimeError<Data::Error>> {
    perform_unary_op(this, Instruction::AbsoluteValue, Data::Number::absolute_value, context)
}

pub fn opposite<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<(), RuntimeError<Data::Error>> {
    perform_unary_op(this, Instruction::Opposite, Data::Number::opposite, context)
}

pub(crate) fn perform_unary_op<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>, Op>(
    this: &mut Data,
    op_name: Instruction,
    op: Op,
    context: Option<&mut Context>,
) -> Result<(), RuntimeError<Data::Error>>
where
    Op: FnOnce(Data::Number) -> Option<Data::Number>,
{
    let addr = next_ref(this)?;

    let t = this.get_data_type(addr)?;
    trace!("Attempting {:?} on {:?} at {:?}", op_name, t, addr,);

    match t {
        ExpressionDataType::Number => {
            let value = this.get_number(addr)?;

            match op(value) {
                Some(result) => push_number(this, result)?,
                None => push_unit(this)?,
            }
        }
        l => match context {
            None => push_unit(this)?,
            Some(c) => {
                if !c.defer_op(this, op_name, (l, addr), (ExpressionDataType::Unit, Data::Size::zero()))? {
                    push_unit(this)?
                }
            }
        },
    }

    Ok(())
}

pub(crate) fn perform_op<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>, Op>(
    this: &mut Data,
    op_name: Instruction,
    op: Op,
    context: Option<&mut Context>,
) -> Result<(), RuntimeError<Data::Error>>
where
    Op: FnOnce(Data::Number, Data::Number) -> Option<Data::Number>,
{
    let (right_addr, left_addr) = next_two_raw_ref(this)?;

    let types = (this.get_data_type(left_addr)?, this.get_data_type(right_addr)?);
    trace!(
        "Attempting {:?} between {:?} at {:?} and {:?} at {:?}",
        op_name,
        types.0,
        left_addr,
        types.1,
        right_addr
    );

    match types {
        (ExpressionDataType::Number, ExpressionDataType::Number) => {
            let left = this.get_number(left_addr)?;
            let right = this.get_number(right_addr)?;

            match op(left, right) {
                Some(result) => push_number(this, result)?,
                None => push_unit(this)?,
            }
        }
        (l, r) => match context {
            None => push_unit(this)?,
            Some(c) => {
                if !c.defer_op(this, op_name, (l, left_addr), (r, right_addr))? {
                    push_unit(this)?
                }
            }
        },
    }

    Ok(())
}

#[cfg(test)]
mod deferring {
    use crate::runtime::GarnishRuntime;
    use crate::testing_utilites::{deferred_op, deferred_unary_op};

    #[test]
    fn add() {
        deferred_op(|runtime, context| {
            runtime.add(Some(context)).unwrap();
        })
    }

    #[test]
    fn subtract() {
        deferred_op(|runtime, context| {
            runtime.subtract(Some(context)).unwrap();
        })
    }

    #[test]
    fn multiply() {
        deferred_op(|runtime, context| {
            runtime.multiply(Some(context)).unwrap();
        });
    }

    #[test]
    fn divide() {
        deferred_op(|runtime, context| {
            runtime.divide(Some(context)).unwrap();
        });
    }

    #[test]
    fn integer_divide() {
        deferred_op(|runtime, context| {
            runtime.integer_divide(Some(context)).unwrap();
        });
    }

    #[test]
    fn remainder() {
        deferred_op(|runtime, context| {
            runtime.remainder(Some(context)).unwrap();
        });
    }

    #[test]
    fn power() {
        deferred_op(|runtime, context| {
            runtime.power(Some(context)).unwrap();
        });
    }

    #[test]
    fn absolute_value() {
        deferred_unary_op(|runtime, context| {
            runtime.absolute_value(Some(context)).unwrap();
        });
    }

    #[test]
    fn opposite() {
        deferred_unary_op(|runtime, context| {
            runtime.opposite(Some(context)).unwrap();
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::{runtime::GarnishRuntime, EmptyContext, ExpressionDataType, GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn add() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10).unwrap();
        let int2 = runtime.add_number(20).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.add::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), 30);
    }

    #[test]
    fn add_no_refs_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_number(10).unwrap();
        runtime.add_number(20).unwrap();

        let result = runtime.add::<EmptyContext>(None);

        assert!(result.is_err());
    }

    #[test]
    fn add_with_non_numbers() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_symbol(SimpleRuntimeData::parse_symbol("sym1").unwrap()).unwrap();
        runtime.add_symbol(SimpleRuntimeData::parse_symbol("sym2").unwrap()).unwrap();

        runtime.push_register(1).unwrap();
        runtime.push_register(2).unwrap();

        runtime.add::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn subtract() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10).unwrap();
        let int2 = runtime.add_number(20).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.subtract::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), -10);
    }

    #[test]
    fn multiply() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10).unwrap();
        let int2 = runtime.add_number(20).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.multiply::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), 200);
    }

    #[test]
    fn divide() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(20).unwrap();
        let int2 = runtime.add_number(10).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.divide::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), 2);
    }

    #[test]
    fn integer_divide() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(20).unwrap();
        let int2 = runtime.add_number(10).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.integer_divide::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), 2);
    }

    #[test]
    fn power() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10).unwrap();
        let int2 = runtime.add_number(3).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.power::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), 1000);
    }

    #[test]
    fn remainder() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(23).unwrap();
        let int2 = runtime.add_number(20).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.remainder::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), 3);
    }

    #[test]
    fn absolute_value() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(-10).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();

        runtime.absolute_value::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), 10);
    }

    #[test]
    fn opposite() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();

        runtime.opposite::<EmptyContext>(None).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), -10);
    }
}
