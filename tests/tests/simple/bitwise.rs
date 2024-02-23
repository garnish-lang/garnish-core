#[cfg(test)]
mod deferring {
    use crate::simple::testing_utilities::{deferred_op, deferred_unary_op};
    use garnish_lang_traits::GarnishRuntime;

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
    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang_traits::{GarnishLangRuntimeData, GarnishRuntime, NO_CONTEXT};

    #[test]
    fn bitwise_not() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let new_data_start = runtime.get_data_mut().get_data_len();

        runtime.get_data_mut().push_register(int1).unwrap();

        runtime.bitwise_not(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_mut().get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_data_mut().get_number(new_data_start).unwrap(), (!10).into());
    }

    #[test]
    fn bitwise_and() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let int2 = runtime.get_data_mut().add_number(20.into()).unwrap();
        let new_data_start = runtime.get_data_mut().get_data_len();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.bitwise_and(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_mut().get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_data_mut().get_number(new_data_start).unwrap(), (10 & 20).into());
    }

    #[test]
    fn bitwise_or() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let int2 = runtime.get_data_mut().add_number(20.into()).unwrap();
        let new_data_start = runtime.get_data_mut().get_data_len();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.bitwise_or(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_mut().get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_data_mut().get_number(new_data_start).unwrap(), (10 | 20).into());
    }

    #[test]
    fn bitwise_xor() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let int2 = runtime.get_data_mut().add_number(20.into()).unwrap();
        let new_data_start = runtime.get_data_mut().get_data_len();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.bitwise_xor(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_mut().get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_data_mut().get_number(new_data_start).unwrap(), (10 ^ 20).into());
    }

    #[test]
    fn bitwise_shift_left() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let int2 = runtime.get_data_mut().add_number(3.into()).unwrap();
        let new_data_start = runtime.get_data_mut().get_data_len();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.bitwise_left_shift(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_mut().get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_data_mut().get_number(new_data_start).unwrap(), (10 << 3).into());
    }

    #[test]
    fn bitwise_shift_right() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();
        let int2 = runtime.get_data_mut().add_number(3.into()).unwrap();
        let new_data_start = runtime.get_data_mut().get_data_len();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.bitwise_right_shift(NO_CONTEXT).unwrap();

        assert_eq!(runtime.get_data_mut().get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_data_mut().get_number(new_data_start).unwrap(), (10 >> 3).into());
    }
}
