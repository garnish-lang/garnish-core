use crate::{next_ref, next_two_raw_ref, push_boolean, ExpressionDataType, GarnishLangRuntimeData, RuntimeError};

pub(crate) fn and<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    let (left, right) = next_two_raw_ref(this)?;

    let result = match (is_true_value(this, left)?, is_true_value(this, right)?) {
        (true, true) => true,
        _ => false,
    };

    push_boolean(this, result)
}

pub fn or<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    let (left, right) = next_two_raw_ref(this)?;

    let result = match (is_true_value(this, left)?, is_true_value(this, right)?) {
        (false, false) => false,
        _ => true,
    };

    push_boolean(this, result)
}

pub fn xor<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    let (left, right) = next_two_raw_ref(this)?;

    let result = match (is_true_value(this, left)?, is_true_value(this, right)?) {
        (false, false) | (true, true) => false,
        _ => true,
    };

    push_boolean(this, result)
}

pub(crate) fn not<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    let addr = next_ref(this)?;
    let result = is_true_value(this, addr)?;
    push_boolean(this, !result)
}

fn is_true_value<Data: GarnishLangRuntimeData>(this: &mut Data, addr: Data::Size) -> Result<bool, RuntimeError<Data::Error>> {
    Ok(match this.get_data_type(addr)? {
        ExpressionDataType::False | ExpressionDataType::Unit => false,
        _ => true,
    })
}

#[cfg(test)]
mod and {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData};
    use crate::testing_utilites::create_simple_runtime;

    #[test]
    fn and_true_booleans() {
        let mut runtime = create_simple_runtime();


        let int1 = runtime.get_data_mut().add_true().unwrap();
        let int2 = runtime.get_data_mut().add_true().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.and().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn and_false_on_left() {
        let mut runtime = create_simple_runtime();


        let int1 = runtime.get_data_mut().add_false().unwrap();
        let int2 = runtime.get_data_mut().add_true().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.and().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(
            runtime.get_data_mut().get_data_type(i).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn and_false_on_right() {
        let mut runtime = create_simple_runtime();


        let int1 = runtime.get_data_mut().add_true().unwrap();
        let int2 = runtime.get_data_mut().add_false().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.and().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(
            runtime.get_data_mut().get_data_type(i).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn and_false_booleans() {
        let mut runtime = create_simple_runtime();


        let int1 = runtime.get_data_mut().add_false().unwrap();
        let int2 = runtime.get_data_mut().add_false().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.and().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(
            runtime.get_data_mut().get_data_type(i).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn and_false_unit() {
        let mut runtime = create_simple_runtime();


        let int1 = runtime.get_data_mut().add_unit().unwrap();
        let int2 = runtime.get_data_mut().add_unit().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.and().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(
            runtime.get_data_mut().get_data_type(i).unwrap(),
            ExpressionDataType::False
        );
    }
}

#[cfg(test)]
mod or {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData};
    use crate::testing_utilites::create_simple_runtime;

    #[test]
    fn or_true_booleans() {
        let mut runtime = create_simple_runtime();


        let int1 = runtime.get_data_mut().add_true().unwrap();
        let int2 = runtime.get_data_mut().add_true().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.or().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn or_false_on_left() {
        let mut runtime = create_simple_runtime();


        let int1 = runtime.get_data_mut().add_false().unwrap();
        let int2 = runtime.get_data_mut().add_true().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.or().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn or_false_on_right() {
        let mut runtime = create_simple_runtime();


        let int1 = runtime.get_data_mut().add_true().unwrap();
        let int2 = runtime.get_data_mut().add_false().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.or().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn or_false_booleans() {
        let mut runtime = create_simple_runtime();


        let int1 = runtime.get_data_mut().add_false().unwrap();
        let int2 = runtime.get_data_mut().add_false().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.or().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(
            runtime.get_data_mut().get_data_type(i).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn or_false_unit() {
        let mut runtime = create_simple_runtime();


        let int1 = runtime.get_data_mut().add_unit().unwrap();
        let int2 = runtime.get_data_mut().add_unit().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.or().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(
            runtime.get_data_mut().get_data_type(i).unwrap(),
            ExpressionDataType::False
        );
    }
}

#[cfg(test)]
mod xor {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData};
    use crate::testing_utilites::create_simple_runtime;

    #[test]
    fn xor_true_booleans() {
        let mut runtime = create_simple_runtime();


        let int1 = runtime.get_data_mut().add_true().unwrap();
        let int2 = runtime.get_data_mut().add_true().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.xor().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(
            runtime.get_data_mut().get_data_type(i).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn xor_false_on_left() {
        let mut runtime = create_simple_runtime();


        let int1 = runtime.get_data_mut().add_false().unwrap();
        let int2 = runtime.get_data_mut().add_true().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.xor().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn xor_false_on_right() {
        let mut runtime = create_simple_runtime();


        let int1 = runtime.get_data_mut().add_true().unwrap();
        let int2 = runtime.get_data_mut().add_false().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.xor().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn xor_false_booleans() {
        let mut runtime = create_simple_runtime();


        let int1 = runtime.get_data_mut().add_false().unwrap();
        let int2 = runtime.get_data_mut().add_false().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.xor().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(
            runtime.get_data_mut().get_data_type(i).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn xor_false_unit() {
        let mut runtime = create_simple_runtime();


        let int1 = runtime.get_data_mut().add_unit().unwrap();
        let int2 = runtime.get_data_mut().add_unit().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.xor().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(
            runtime.get_data_mut().get_data_type(i).unwrap(),
            ExpressionDataType::False
        );
    }
}

#[cfg(test)]
mod not {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData};
    use crate::testing_utilites::create_simple_runtime;

    #[test]
    fn not_true_is_false() {
        let mut runtime = create_simple_runtime();


        let int1 = runtime.get_data_mut().add_true().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();

        runtime.not().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(
            runtime.get_data_mut().get_data_type(i).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn not_any_is_false() {
        let mut runtime = create_simple_runtime();


        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();

        runtime.not().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(
            runtime.get_data_mut().get_data_type(i).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn not_false_is_true() {
        let mut runtime = create_simple_runtime();


        let int1 = runtime.get_data_mut().add_false().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();

        runtime.not().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn not_unit_is_true() {
        let mut runtime = create_simple_runtime();


        let int1 = runtime.get_data_mut().add_unit().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();

        runtime.not().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), ExpressionDataType::True);
    }
}
