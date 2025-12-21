#[cfg(test)]
mod and {
    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang::{GarnishData, GarnishDataType, GarnishRuntime, Instruction};

    #[test]
    fn with_true() {
        let mut runtime = create_simple_runtime();

        let ta = runtime.get_data_mut().add_true().unwrap();

        let i1 = runtime.get_data_mut().push_instruction(Instruction::And, Some(0)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        let i2 = runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();

        runtime.get_data_mut().push_to_jump_table(i2).unwrap();

        runtime.get_data_mut().set_instruction_cursor(i1).unwrap();

        runtime.get_data_mut().push_register(ta).unwrap();

        let next = runtime.and(0).unwrap();

        assert_eq!(runtime.get_data_mut().get_register_len(), 0);
        assert_eq!(next.unwrap(), i2);
    }

    #[test]
    fn with_false() {
        let mut runtime = create_simple_runtime();

        let ta = runtime.get_data_mut().add_false().unwrap();

        let i1 = runtime.get_data_mut().push_instruction(Instruction::And, Some(0)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        let i2 = runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();

        runtime.get_data_mut().push_to_jump_table(i2).unwrap();

        runtime.get_data_mut().set_instruction_cursor(i1).unwrap();

        runtime.get_data_mut().push_register(ta).unwrap();

        let next = runtime.and(0).unwrap();

        assert_eq!(runtime.get_data_mut().get_register_len(), 1);
        let i = runtime.get_data().get_register(0).unwrap();
        assert_eq!(runtime.get_data().get_data_type(i).unwrap(), GarnishDataType::False);
        assert_eq!(next, None);
    }

    #[test]
    fn with_invalid_data() {
        let mut runtime = create_simple_runtime();

        let ta = runtime.get_data_mut().add_true().unwrap();

        let i1 = runtime.get_data_mut().push_instruction(Instruction::And, Some(0)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        let i2 = runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();

        runtime.get_data_mut().push_to_jump_table(i2).unwrap();

        runtime.get_data_mut().set_instruction_cursor(i1).unwrap();

        runtime.get_data_mut().push_register(ta).unwrap();

        let next = runtime.and(3);

        assert!(next.is_err())
    }
}

#[cfg(test)]
mod or {
    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang::{GarnishData, GarnishDataType, GarnishRuntime, Instruction};

    #[test]
    fn with_true() {
        let mut runtime = create_simple_runtime();

        let ta = runtime.get_data_mut().add_true().unwrap();

        let i1 = runtime.get_data_mut().push_instruction(Instruction::Or, Some(0)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        let i2 = runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();

        runtime.get_data_mut().push_to_jump_table(i2).unwrap();

        runtime.get_data_mut().set_instruction_cursor(i1).unwrap();

        runtime.get_data_mut().push_register(ta).unwrap();

        let next = runtime.or(0).unwrap();

        assert_eq!(runtime.get_data_mut().get_register_len(), 1);
        let i = runtime.get_data().get_register(0).unwrap();
        assert_eq!(runtime.get_data().get_data_type(i).unwrap(), GarnishDataType::True);
        assert_eq!(next, None);
    }

    #[test]
    fn with_false() {
        let mut runtime = create_simple_runtime();

        let ta = runtime.get_data_mut().add_false().unwrap();

        let i1 = runtime.get_data_mut().push_instruction(Instruction::Or, Some(0)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        let i2 = runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();

        runtime.get_data_mut().push_to_jump_table(i2).unwrap();

        runtime.get_data_mut().set_instruction_cursor(i1).unwrap();

        runtime.get_data_mut().push_register(ta).unwrap();

        let next = runtime.or(0).unwrap();

        assert_eq!(runtime.get_data_mut().get_register_len(), 0);
        assert_eq!(next.unwrap(), i2);
    }

    #[test]
    fn with_invalid_data() {
        let mut runtime = create_simple_runtime();

        let ta = runtime.get_data_mut().add_false().unwrap();

        let i1 = runtime.get_data_mut().push_instruction(Instruction::Or, Some(0)).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        let i2 = runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();
        runtime.get_data_mut().push_instruction(Instruction::Add, None).unwrap();

        runtime.get_data_mut().push_to_jump_table(i2).unwrap();

        runtime.get_data_mut().set_instruction_cursor(i1).unwrap();

        runtime.get_data_mut().push_register(ta).unwrap();

        let next = runtime.or(3);

        assert!(next.is_err())
    }
}

#[cfg(test)]
mod xor {
    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang::{GarnishData, GarnishDataType, GarnishRuntime};

    #[test]
    fn xor_true_booleans() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_true().unwrap();
        let int2 = runtime.get_data_mut().add_true().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();
        runtime.get_data_mut().push_register(int2).unwrap();

        runtime.xor().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::False);
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
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::True);
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
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::True);
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
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::False);
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
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::False);
    }
}

#[cfg(test)]
mod not {
    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang::{GarnishData, GarnishDataType, GarnishRuntime};

    #[test]
    fn not_true_is_false() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_true().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();

        runtime.not().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::False);
    }

    #[test]
    fn not_any_is_false() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();

        runtime.not().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::False);
    }

    #[test]
    fn not_false_is_true() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_false().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();

        runtime.not().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::True);
    }

    #[test]
    fn not_unit_is_true() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_unit().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();

        runtime.not().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::True);
    }
}

#[cfg(test)]
mod tis {
    use crate::simple::testing_utilities::create_simple_runtime;
    use garnish_lang::{GarnishData, GarnishDataType, GarnishRuntime};

    #[test]
    fn tis_false_is_false() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_false().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();

        runtime.tis().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::False);
    }

    #[test]
    fn tis_any_is_true() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_number(10.into()).unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();

        runtime.tis().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::True);
    }

    #[test]
    fn tis_true_is_true() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_true().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();

        runtime.tis().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::True);
    }

    #[test]
    fn tis_unit_is_false() {
        let mut runtime = create_simple_runtime();

        let int1 = runtime.get_data_mut().add_unit().unwrap();

        runtime.get_data_mut().push_register(int1).unwrap();

        runtime.tis().unwrap();

        let i = runtime.get_data_mut().get_register(0).unwrap();
        assert_eq!(runtime.get_data_mut().get_data_type(i).unwrap(), GarnishDataType::False);
    }
}
