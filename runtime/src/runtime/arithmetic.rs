use garnish_lang_traits::Instruction;
use log::trace;

use crate::runtime::utilities::{next_ref, next_two_raw_ref, push_number, push_unit};
use garnish_lang_traits::{GarnishData, GarnishDataType, GarnishNumber, RuntimeError, TypeConstants};

pub fn add<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_op(this, Instruction::Add, Data::Number::plus)
}

pub fn subtract<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_op(this, Instruction::Subtract, Data::Number::subtract)
}

pub fn multiply<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_op(this, Instruction::Multiply, Data::Number::multiply)
}

pub fn power<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_op(this, Instruction::Power, Data::Number::power)
}

pub fn divide<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_op(this, Instruction::Divide, Data::Number::divide)
}

pub fn integer_divide<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_op(this, Instruction::IntegerDivide, Data::Number::integer_divide)
}

pub fn remainder<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_op(this, Instruction::Remainder, Data::Number::remainder)
}

pub fn absolute_value<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_unary_op(this, Instruction::AbsoluteValue, Data::Number::absolute_value)
}

pub fn opposite<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    perform_unary_op(this, Instruction::Opposite, Data::Number::opposite)
}

pub(crate) fn perform_unary_op<Data: GarnishData, Op>(this: &mut Data, op_name: Instruction, op: Op) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>
where
    Op: FnOnce(Data::Number) -> Option<Data::Number>,
{
    let addr = next_ref(this)?;

    let t = this.get_data_type(addr.clone())?;
    trace!("Attempting {:?} on {:?} at {:?}", op_name, t, addr,);

    match t {
        GarnishDataType::Number => {
            let value = this.get_number(addr)?;

            match op(value) {
                Some(result) => push_number(this, result)?,
                None => push_unit(this)?,
            }
        }
        l => {
            if !this.defer_op(op_name, (l, addr), (GarnishDataType::Unit, Data::Size::zero()))? {
                push_unit(this)?
            }
        }
    }

    Ok(None)
}

pub(crate) fn perform_op<Data: GarnishData, Op>(this: &mut Data, op_name: Instruction, op: Op) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>
where
    Op: FnOnce(Data::Number, Data::Number) -> Option<Data::Number>,
{
    let (right_addr, left_addr) = next_two_raw_ref(this)?;

    let types = (this.get_data_type(left_addr.clone())?, this.get_data_type(right_addr.clone())?);
    trace!("Attempting {:?} between {:?} at {:?} and {:?} at {:?}", op_name, types.0, left_addr, types.1, right_addr);

    match types {
        (GarnishDataType::Number, GarnishDataType::Number) => {
            let left = this.get_number(left_addr)?;
            let right = this.get_number(right_addr)?;

            match op(left, right) {
                Some(result) => push_number(this, result)?,
                None => push_unit(this)?,
            }
        }
        (l, r) => {
            if !this.defer_op(op_name, (l, left_addr), (r, right_addr))? {
                push_unit(this)?
            }
        }
    }

    Ok(None)
}

#[cfg(test)]
mod defer_op {
    use crate::ops::{absolute_value, add, divide, integer_divide, multiply, opposite, power, remainder, subtract};
    use crate::runtime::tests::MockGarnishData;
    use garnish_lang_traits::{GarnishData, GarnishDataType, Instruction};

    #[test]
    fn add_defer_op() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Number, GarnishDataType::Symbol]);

        mock_data.stub_get_instruction_cursor = |_| 1;
        mock_data.stub_defer_op = |data, instruction, left, right| {
            assert_eq!(instruction, Instruction::Add);
            assert_eq!(left, (GarnishDataType::Number, 0));
            assert_eq!(right, (GarnishDataType::Symbol, 1));
            data.registers.push(200);
            Ok(true)
        };

        add(&mut mock_data).unwrap();

        assert_eq!(mock_data.pop_register().unwrap(), Some(200));
    }

    #[test]
    fn subtract_defer_op() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Number, GarnishDataType::Symbol]);

        mock_data.stub_get_instruction_cursor = |_| 1;
        mock_data.stub_defer_op = |data, instruction, left, right| {
            assert_eq!(instruction, Instruction::Subtract);
            assert_eq!(left, (GarnishDataType::Number, 0));
            assert_eq!(right, (GarnishDataType::Symbol, 1));
            data.registers.push(200);
            Ok(true)
        };

        subtract(&mut mock_data).unwrap();

        assert_eq!(mock_data.pop_register().unwrap(), Some(200));
    }

    #[test]
    fn multiply_defer_op() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Number, GarnishDataType::Symbol]);

        mock_data.stub_get_instruction_cursor = |_| 1;
        mock_data.stub_defer_op = |data, instruction, left, right| {
            assert_eq!(instruction, Instruction::Multiply);
            assert_eq!(left, (GarnishDataType::Number, 0));
            assert_eq!(right, (GarnishDataType::Symbol, 1));
            data.registers.push(200);
            Ok(true)
        };

        multiply(&mut mock_data).unwrap();

        assert_eq!(mock_data.pop_register().unwrap(), Some(200));
    }

    #[test]
    fn power_defer_op() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Number, GarnishDataType::Symbol]);

        mock_data.stub_get_instruction_cursor = |_| 1;
        mock_data.stub_defer_op = |data, instruction, left, right| {
            assert_eq!(instruction, Instruction::Power);
            assert_eq!(left, (GarnishDataType::Number, 0));
            assert_eq!(right, (GarnishDataType::Symbol, 1));
            data.registers.push(200);
            Ok(true)
        };

        power(&mut mock_data).unwrap();

        assert_eq!(mock_data.pop_register().unwrap(), Some(200));
    }

    #[test]
    fn divide_defer_op() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Number, GarnishDataType::Symbol]);

        mock_data.stub_get_instruction_cursor = |_| 1;
        mock_data.stub_defer_op = |data, instruction, left, right| {
            assert_eq!(instruction, Instruction::Divide);
            assert_eq!(left, (GarnishDataType::Number, 0));
            assert_eq!(right, (GarnishDataType::Symbol, 1));
            data.registers.push(200);
            Ok(true)
        };

        divide(&mut mock_data).unwrap();

        assert_eq!(mock_data.pop_register().unwrap(), Some(200));
    }

    #[test]
    fn integer_divide_defer_op() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Number, GarnishDataType::Symbol]);

        mock_data.stub_get_instruction_cursor = |_| 1;
        mock_data.stub_defer_op = |data, instruction, left, right| {
            assert_eq!(instruction, Instruction::IntegerDivide);
            assert_eq!(left, (GarnishDataType::Number, 0));
            assert_eq!(right, (GarnishDataType::Symbol, 1));
            data.registers.push(200);
            Ok(true)
        };

        integer_divide(&mut mock_data).unwrap();

        assert_eq!(mock_data.pop_register().unwrap(), Some(200));
    }

    #[test]
    fn remainder_defer_op() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Number, GarnishDataType::Symbol]);

        mock_data.stub_get_instruction_cursor = |_| 1;
        mock_data.stub_defer_op = |data, instruction, left, right| {
            assert_eq!(instruction, Instruction::Remainder);
            assert_eq!(left, (GarnishDataType::Number, 0));
            assert_eq!(right, (GarnishDataType::Symbol, 1));
            data.registers.push(200);
            Ok(true)
        };

        remainder(&mut mock_data).unwrap();

        assert_eq!(mock_data.pop_register().unwrap(), Some(200));
    }

    #[test]
    fn absolute_value_defer_op() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Number, GarnishDataType::Symbol]);

        mock_data.stub_get_instruction_cursor = |_| 1;
        mock_data.stub_defer_op = |data, instruction, left, right| {
            assert_eq!(instruction, Instruction::AbsoluteValue);
            assert_eq!(left, (GarnishDataType::Symbol, 1));
            assert_eq!(right, (GarnishDataType::Unit, 0));
            data.registers.push(200);
            Ok(true)
        };

        absolute_value(&mut mock_data).unwrap();

        assert_eq!(mock_data.pop_register().unwrap(), Some(200));
    }

    #[test]
    fn opposite_defer_op() {
        let mut mock_data = MockGarnishData::new_basic_data(vec![GarnishDataType::Number, GarnishDataType::Symbol]);

        mock_data.stub_get_instruction_cursor = |_| 1;
        mock_data.stub_defer_op = |data, instruction, left, right| {
            assert_eq!(instruction, Instruction::Opposite);
            assert_eq!(left, (GarnishDataType::Symbol, 1));
            assert_eq!(right, (GarnishDataType::Unit, 0));
            data.registers.push(200);
            Ok(true)
        };

        opposite(&mut mock_data).unwrap();

        assert_eq!(mock_data.pop_register().unwrap(), Some(200));
    }
}
