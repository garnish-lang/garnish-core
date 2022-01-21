use crate::runtime::arithmetic::{perform_op, perform_unary_op};
use crate::{GarnishLangRuntimeContext, GarnishLangRuntimeData, GarnishNumber, Instruction, RuntimeError};

pub fn bitwise_not<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<(), RuntimeError<Data::Error>> {
    perform_unary_op(this, Instruction::BitwiseNot, Data::Number::bitwise_not, context)
}

pub fn bitwise_and<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, Instruction::BitwiseAnd, Data::Number::bitwise_and, context)
}

pub fn bitwise_or<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, Instruction::BitwiseOr, Data::Number::bitwise_or, context)
}

pub fn bitwise_xor<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, Instruction::BitwiseXor, Data::Number::bitwise_xor, context)
}

pub fn bitwise_left_shift<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, Instruction::BitwiseShiftLeft, Data::Number::bitwise_shift_left, context)
}

pub fn bitwise_right_shift<Data: GarnishLangRuntimeData, Context: GarnishLangRuntimeContext<Data>>(
    this: &mut Data,
    context: Option<&mut Context>,
) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, Instruction::BitwiseShiftRight, Data::Number::bitwise_shift_right, context)
}

#[cfg(test)]
mod deferring {
    use crate::runtime::GarnishRuntime;
    use crate::testing_utilites::{deferred_op, deferred_unary_op};

    #[test]
    fn bitwise_not() {
        deferred_unary_op(|runtime, context| {
            runtime.bitwise_not(Some(context)).unwrap();
        })
    }

    #[test]
    fn bitwise_and() {
        deferred_op(|runtime, context| {
            runtime.bitwise_and(Some(context)).unwrap();
        })
    }

    #[test]
    fn bitwise_or() {
        deferred_op(|runtime, context| {
            runtime.bitwise_or(Some(context)).unwrap();
        })
    }

    #[test]
    fn bitwise_xor() {
        deferred_op(|runtime, context| {
            runtime.bitwise_xor(Some(context)).unwrap();
        })
    }

    #[test]
    fn bitwise_left_shift() {
        deferred_op(|runtime, context| {
            runtime.bitwise_left_shift(Some(context)).unwrap();
        })
    }

    #[test]
    fn bitwise_right_shift() {
        deferred_op(|runtime, context| {
            runtime.bitwise_right_shift(Some(context)).unwrap();
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::runtime::context::NO_CONTEXT;
    use crate::{runtime::GarnishRuntime, GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn bitwise_not() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();

        runtime.bitwise_not(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), !10);
    }

    #[test]
    fn bitwise_and() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10).unwrap();
        let int2 = runtime.add_number(20).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.bitwise_and(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), 10 & 20);
    }

    #[test]
    fn bitwise_or() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10).unwrap();
        let int2 = runtime.add_number(20).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.bitwise_or(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), 10 | 20);
    }

    #[test]
    fn bitwise_xor() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10).unwrap();
        let int2 = runtime.add_number(20).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.bitwise_xor(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), 10 ^ 20);
    }

    #[test]
    fn bitwise_shift_left() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10).unwrap();
        let int2 = runtime.add_number(3).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.bitwise_left_shift(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), 10 << 3);
    }

    #[test]
    fn bitwise_shift_right() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10).unwrap();
        let int2 = runtime.add_number(3).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.bitwise_right_shift(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), 10 >> 3);
    }
}
