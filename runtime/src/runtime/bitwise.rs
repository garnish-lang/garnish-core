use crate::runtime::arithmetic::{perform_op, perform_unary_op};
use garnish_lang_traits::{GarnishContext, GarnishData, GarnishNumber, Instruction, RuntimeError};

pub fn bitwise_not<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_unary_op(this, Instruction::BitwiseNot, Data::Number::bitwise_not)
}

pub fn bitwise_and<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_op(this, Instruction::BitwiseAnd, Data::Number::bitwise_and)
}

pub fn bitwise_or<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_op(this, Instruction::BitwiseOr, Data::Number::bitwise_or)
}

pub fn bitwise_xor<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_op(this, Instruction::BitwiseXor, Data::Number::bitwise_xor)
}

pub fn bitwise_left_shift<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_op(this, Instruction::BitwiseShiftLeft, Data::Number::bitwise_shift_left)
}

pub fn bitwise_right_shift<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_op(this, Instruction::BitwiseShiftRight, Data::Number::bitwise_shift_right)
}

#[cfg(test)]
mod defer_op {
    use crate::ops::{add, bitwise_and, bitwise_left_shift, bitwise_not, bitwise_or, bitwise_right_shift, bitwise_xor};
    use crate::runtime::tests::MockGarnishData;
    use garnish_lang_traits::{GarnishData, GarnishDataType, Instruction};

    #[test]
    fn bitwise_not_defer_op() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Number, GarnishDataType::Symbol]);

        mock_data.stub_get_instruction_cursor = |_| 1;
        mock_data.stub_defer_op = |data, instruction, left, right| {
            assert_eq!(instruction, Instruction::BitwiseNot);
            assert_eq!(left, (GarnishDataType::Symbol, 1));
            assert_eq!(right, (GarnishDataType::Unit, 0));
            data.registers.push(200);
            Ok(true)
        };

        bitwise_not(&mut mock_data).unwrap();

        assert_eq!(mock_data.pop_register().unwrap(), Some(200));
    }

    #[test]
    fn bitwise_and_defer_op() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Number, GarnishDataType::Symbol]);

        mock_data.stub_get_instruction_cursor = |_| 1;
        mock_data.stub_defer_op = |data, instruction, left, right| {
            assert_eq!(instruction, Instruction::BitwiseAnd);
            assert_eq!(left, (GarnishDataType::Number, 0));
            assert_eq!(right, (GarnishDataType::Symbol, 1));
            data.registers.push(200);
            Ok(true)
        };

        bitwise_and(&mut mock_data).unwrap();

        assert_eq!(mock_data.pop_register().unwrap(), Some(200));
    }

    #[test]
    fn bitwise_or_defer_op() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Number, GarnishDataType::Symbol]);

        mock_data.stub_get_instruction_cursor = |_| 1;
        mock_data.stub_defer_op = |data, instruction, left, right| {
            assert_eq!(instruction, Instruction::BitwiseOr);
            assert_eq!(left, (GarnishDataType::Number, 0));
            assert_eq!(right, (GarnishDataType::Symbol, 1));
            data.registers.push(200);
            Ok(true)
        };

        bitwise_or(&mut mock_data).unwrap();

        assert_eq!(mock_data.pop_register().unwrap(), Some(200));
    }

    #[test]
    fn bitwise_xor_defer_op() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Number, GarnishDataType::Symbol]);

        mock_data.stub_get_instruction_cursor = |_| 1;
        mock_data.stub_defer_op = |data, instruction, left, right| {
            assert_eq!(instruction, Instruction::BitwiseXor);
            assert_eq!(left, (GarnishDataType::Number, 0));
            assert_eq!(right, (GarnishDataType::Symbol, 1));
            data.registers.push(200);
            Ok(true)
        };

        bitwise_xor(&mut mock_data).unwrap();

        assert_eq!(mock_data.pop_register().unwrap(), Some(200));
    }

    #[test]
    fn bitwise_left_shift_defer_op() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Number, GarnishDataType::Symbol]);

        mock_data.stub_get_instruction_cursor = |_| 1;
        mock_data.stub_defer_op = |data, instruction, left, right| {
            assert_eq!(instruction, Instruction::BitwiseShiftLeft);
            assert_eq!(left, (GarnishDataType::Number, 0));
            assert_eq!(right, (GarnishDataType::Symbol, 1));
            data.registers.push(200);
            Ok(true)
        };

        bitwise_left_shift(&mut mock_data).unwrap();

        assert_eq!(mock_data.pop_register().unwrap(), Some(200));
    }

    #[test]
    fn bitwise_right_shift_defer_op() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Number, GarnishDataType::Symbol]);

        mock_data.stub_get_instruction_cursor = |_| 1;
        mock_data.stub_defer_op = |data, instruction, left, right| {
            assert_eq!(instruction, Instruction::BitwiseShiftRight);
            assert_eq!(left, (GarnishDataType::Number, 0));
            assert_eq!(right, (GarnishDataType::Symbol, 1));
            data.registers.push(200);
            Ok(true)
        };

        bitwise_right_shift(&mut mock_data).unwrap();

        assert_eq!(mock_data.pop_register().unwrap(), Some(200));
    }
}
