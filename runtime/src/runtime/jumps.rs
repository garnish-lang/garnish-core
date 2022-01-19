use log::trace;

use crate::{next_ref, state_error, ExpressionDataType, GarnishLangRuntimeData, RuntimeError};

pub(crate) fn jump<Data: GarnishLangRuntimeData>(this: &mut Data, index: Data::Size) -> Result<(), RuntimeError<Data::Error>> {
    match this.get_jump_point(index) {
        None => state_error(format!("No jump point at index {:?}", index))?,
        Some(point) => {
            this.set_instruction_cursor(point)?;
        }
    }

    Ok(())
}

pub(crate) fn jump_if_true<Data: GarnishLangRuntimeData>(this: &mut Data, index: Data::Size) -> Result<(), RuntimeError<Data::Error>> {
    let point = match this.get_jump_point(index) {
        None => state_error(format!("No jump point at index {:?}", index))?,
        Some(point) => point,
    };

    let d = next_ref(this)?;

    match this.get_data_type(d)? {
        ExpressionDataType::False | ExpressionDataType::Unit => {
            trace!("Not jumping from value of type {:?} with addr {:?}", this.get_data_type(d)?, d);
        }
        // all other values are considered true
        t => {
            trace!("Jumping from value of type {:?} with addr {:?}", t, d);
            this.set_instruction_cursor(point)?
        }
    };

    Ok(())
}

pub(crate) fn jump_if_false<Data: GarnishLangRuntimeData>(this: &mut Data, index: Data::Size) -> Result<(), RuntimeError<Data::Error>> {
    let point = match this.get_jump_point(index) {
        None => state_error(format!("No jump point at index {:?}", index))?,
        Some(point) => point,
    };

    let d = next_ref(this)?;

    match this.get_data_type(d)? {
        ExpressionDataType::False | ExpressionDataType::Unit => {
            trace!("Jumping from value of type {:?} with addr {:?}", this.get_data_type(d)?, d);
            this.set_instruction_cursor(point)?
        }
        t => {
            trace!("Not jumping from value of type {:?} with addr {:?}", t, d);
        }
    };

    Ok(())
}

pub(crate) fn end_expression<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    match this.pop_jump_path() {
        None => {
            // no more jumps, this should be the end of the entire execution
            let r = next_ref(this)?;
            trace!("No remaining return points. Pushing {:?} to values. Setting cursor to instruction length {:?}.", r, this.get_instruction_len());
            this.set_instruction_cursor(this.get_instruction_len())?;
            this.push_value_stack(r)?;
        }
        Some(jump_point) => {
            trace!("Setting cursor to {:?}", jump_point);
            this.set_instruction_cursor(jump_point)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{runtime::GarnishRuntime, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

    #[test]
    fn end_expression() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10).unwrap();
        runtime.push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.push_instruction(Instruction::Put, Some(1)).unwrap();
        let i1 = runtime.push_instruction(Instruction::EndExpression, None).unwrap();
        runtime.push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.push_instruction(Instruction::Put, Some(1)).unwrap();

        runtime.set_instruction_cursor(i1).unwrap();
        runtime.push_register(int1).unwrap();

        runtime.end_expression().unwrap();

        assert_eq!(runtime.get_instruction_cursor(), runtime.get_instruction_len());
        assert_eq!(runtime.get_number(runtime.get_current_value().unwrap()).unwrap(), 10);
    }

    #[test]
    fn end_expression_with_path() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_number(10).unwrap();
        runtime.push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.push_instruction(Instruction::EndExpression, Some(0)).unwrap();
        runtime.push_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.push_instruction(Instruction::EmptyApply, None).unwrap();

        runtime.push_register(1).unwrap();
        runtime.push_jump_path(4).unwrap();
        runtime.set_instruction_cursor(2).unwrap();

        runtime.end_expression().unwrap();

        assert_eq!(runtime.get_instruction_cursor(), 4);
    }

    #[test]
    fn jump() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_symbol("false").unwrap();
        runtime.push_instruction(Instruction::JumpIfFalse, Some(3)).unwrap();
        runtime.push_instruction(Instruction::JumpTo, Some(0)).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();
        let i1 = runtime.push_instruction(Instruction::Add, None).unwrap();

        runtime.push_jump_point(i1).unwrap();

        runtime.jump(0).unwrap();

        assert!(runtime.get_jump_path_vec().is_empty());
        assert_eq!(runtime.get_instruction_cursor(), i1);
    }

    #[test]
    fn jump_if_true_no_ref_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_true().unwrap();
        runtime.push_instruction(Instruction::JumpIfTrue, Some(3)).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();

        runtime.push_jump_point(3).unwrap();

        let result = runtime.jump_if_true(0);

        assert!(result.is_err());
    }

    #[test]
    fn jump_if_false_no_ref_is_error() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_true().unwrap();
        runtime.push_instruction(Instruction::JumpIfFalse, Some(3)).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();

        runtime.push_jump_point(3).unwrap();

        let result = runtime.jump_if_false(0);

        assert!(result.is_err());
    }

    #[test]
    fn jump_if_true_when_true() {
        let mut runtime = SimpleRuntimeData::new();

        let ta = runtime.add_true().unwrap();
        let i1 = runtime.push_instruction(Instruction::JumpIfTrue, Some(0)).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();
        let i2 = runtime.push_instruction(Instruction::Add, None).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();

        runtime.push_jump_point(i2).unwrap();

        runtime.set_instruction_cursor(i1).unwrap();

        runtime.push_register(ta).unwrap();

        runtime.jump_if_true(0).unwrap();

        assert_eq!(runtime.get_register_len(), 0);
        assert_eq!(runtime.get_data_len(), 3);
        assert_eq!(runtime.get_instruction_cursor(), i2);
    }

    #[test]
    fn jump_if_true_when_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let ua = runtime.add_unit().unwrap();
        runtime.push_instruction(Instruction::JumpIfTrue, Some(0)).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();

        runtime.push_jump_point(3).unwrap();
        runtime.set_instruction_cursor(1).unwrap();
        runtime.push_register(ua).unwrap();

        runtime.jump_if_true(0).unwrap();

        assert_eq!(runtime.get_register_len(), 0);
        assert_eq!(runtime.get_data_len(), 3);
        assert_eq!(runtime.get_instruction_cursor(), 1);
    }

    #[test]
    fn jump_if_true_when_false() {
        let mut runtime = SimpleRuntimeData::new();

        let fa = runtime.add_false().unwrap();
        runtime.push_instruction(Instruction::JumpIfTrue, Some(0)).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();

        runtime.push_jump_point(3).unwrap();
        runtime.set_instruction_cursor(1).unwrap();
        runtime.push_register(fa).unwrap();

        runtime.jump_if_true(0).unwrap();

        assert_eq!(runtime.get_register_len(), 0);
        assert_eq!(runtime.get_data_len(), 3);
        assert_eq!(runtime.get_instruction_cursor(), 1);
    }

    #[test]
    fn jump_if_false_when_true() {
        let mut runtime = SimpleRuntimeData::new();

        let ta = runtime.add_true().unwrap();
        runtime.push_instruction(Instruction::JumpIfFalse, Some(0)).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();

        runtime.push_jump_point(3).unwrap();
        runtime.set_instruction_cursor(1).unwrap();
        runtime.push_register(ta).unwrap();

        runtime.jump_if_false(0).unwrap();

        assert_eq!(runtime.get_register_len(), 0);
        assert_eq!(runtime.get_data_len(), 3);
        assert_eq!(runtime.get_instruction_cursor(), 1);
    }

    #[test]
    fn jump_if_false_when_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let ua = runtime.add_unit().unwrap();
        let i1 = runtime.push_instruction(Instruction::JumpIfFalse, Some(0)).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();
        let i2 = runtime.push_instruction(Instruction::Add, None).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();

        runtime.push_jump_point(i2).unwrap();
        runtime.set_instruction_cursor(i1).unwrap();
        runtime.push_register(ua).unwrap();

        runtime.jump_if_false(0).unwrap();

        assert_eq!(runtime.get_register_len(), 0);
        assert_eq!(runtime.get_data_len(), 3);
        assert_eq!(runtime.get_instruction_cursor(), i2);
    }

    #[test]
    fn jump_if_false_when_false() {
        let mut runtime = SimpleRuntimeData::new();

        let fa = runtime.add_false().unwrap();
        let i1 = runtime.push_instruction(Instruction::JumpIfFalse, Some(0)).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();
        let i2 = runtime.push_instruction(Instruction::Add, None).unwrap();
        runtime.push_instruction(Instruction::Add, None).unwrap();

        runtime.push_jump_point(i2).unwrap();
        runtime.set_instruction_cursor(i1).unwrap();
        runtime.push_register(fa).unwrap();

        runtime.jump_if_false(0).unwrap();

        assert_eq!(runtime.get_register_len(), 0);
        assert_eq!(runtime.get_data_len(), 3);
        assert_eq!(runtime.get_instruction_cursor(), i2);
    }
}
