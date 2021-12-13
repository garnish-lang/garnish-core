use log::trace;

use crate::{error, ExpressionData, ExpressionDataType, GarnishLangRuntime, GarnishLangRuntimeResult};

use super::context::GarnishLangRuntimeContext;

impl GarnishLangRuntime {
    pub fn apply<T: GarnishLangRuntimeContext>(&mut self, context: Option<&mut T>) -> GarnishLangRuntimeResult {
        trace!("Instruction - Apply");
        self.apply_internal(context)
    }

    pub fn reapply(&mut self, index: usize) -> GarnishLangRuntimeResult {
        trace!("Instruction - Reapply | Data - {:?}", index);
        match self.reference_stack.len() {
            0 => Err(error(format!("Not enough references to perform reapply."))),
            _ => {
                let right_ref = self.reference_stack.pop().unwrap();

                let right_addr = self.addr_of_raw_data(right_ref)?;

                let point = self.get_jump_point(index)?;

                self.set_instruction_cursor(point - 1)?;
                self.inputs.pop();
                self.inputs.push(right_addr);

                Ok(())
            }
        }
    }

    pub fn empty_apply<T: GarnishLangRuntimeContext>(&mut self, context: Option<&mut T>) -> GarnishLangRuntimeResult {
        trace!("Instruction - Apply");
        let addr = self.add_data(ExpressionData::unit())?;
        self.reference_stack.push(addr);

        self.apply_internal(context)
    }

    fn apply_internal<T: GarnishLangRuntimeContext>(&mut self, context: Option<&mut T>) -> GarnishLangRuntimeResult {
        match self.reference_stack.len() {
            0 | 1 => Err(error(format!("Not enough references to perform apply."))),
            _ => {
                let right_ref = self.reference_stack.pop().unwrap();
                let left_ref = self.reference_stack.pop().unwrap();

                let right_addr = self.addr_of_raw_data(right_ref)?;
                let left_addr = self.addr_of_raw_data(left_ref)?;

                let left_data = self.get_data_internal(left_addr)?;
                match left_data.get_type() {
                    ExpressionDataType::Expression => {
                        let expression_index = match left_data.as_expression() {
                            Err(err) => Err(error(err))?,
                            Ok(v) => v,
                        };

                        let next_instruction = self.get_jump_point(expression_index)?;

                        // Expression stores index of expression table, look up actual instruction index

                        self.jump_path.push(self.instruction_cursor);
                        self.set_instruction_cursor(next_instruction - 1)?;
                        self.inputs.push(right_addr);

                        Ok(())
                    }
                    ExpressionDataType::External => {
                        let external_value = match left_data.as_external() {
                            Err(err) => Err(error(err))?,
                            Ok(v) => v,
                        };

                        match context {
                            None => {
                                self.reference_stack.push(self.data.len());
                                self.add_data(ExpressionData::unit())?;
                                Ok(())
                            }
                            Some(c) => match c.apply(external_value, right_addr, self)? {
                                true => Ok(()),
                                false => {
                                    self.reference_stack.push(self.data.len());
                                    self.add_data(ExpressionData::unit())?;
                                    Ok(())
                                }
                            },
                        }
                    }
                    _ => Err(error(format!(
                        "Data type {:?} not supported on left side of apply operation.",
                        left_data.get_type()
                    ))),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        error,
        runtime::context::{EmptyContext, GarnishLangRuntimeContext},
        ExpressionData, ExpressionDataType, GarnishLangRuntime, GarnishLangRuntimeResult, Instruction,
    };

    #[test]
    fn apply() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::expression(0)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();

        // 1
        runtime.add_instruction(Instruction::Put, Some(0)).unwrap();
        runtime.add_instruction(Instruction::PutInput, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::EndExpression, None).unwrap();

        // 5
        runtime.add_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.add_instruction(Instruction::Put, Some(2)).unwrap();
        runtime.add_instruction(Instruction::Apply, None).unwrap();

        runtime.expression_table.push(1);

        runtime.reference_stack.push(1);
        runtime.reference_stack.push(2);

        runtime.set_instruction_cursor(7).unwrap();

        runtime.apply::<EmptyContext>(None).unwrap();

        assert_eq!(*runtime.inputs.get(0).unwrap(), 2);
        assert_eq!(runtime.instruction_cursor, 0);
        assert_eq!(*runtime.jump_path.get(0).unwrap(), 7);
    }

    #[test]
    fn empty_apply() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::expression(0)).unwrap();

        // 1
        runtime.add_instruction(Instruction::Put, Some(0)).unwrap();
        runtime.add_instruction(Instruction::PutInput, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::EndExpression, None).unwrap();

        // 5
        runtime.add_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.add_instruction(Instruction::EmptyApply, None).unwrap();

        runtime.expression_table.push(1);

        runtime.reference_stack.push(1);

        runtime.set_instruction_cursor(6).unwrap();

        runtime.empty_apply::<EmptyContext>(None).unwrap();

        assert_eq!(*runtime.inputs.get(0).unwrap(), 2);
        assert_eq!(runtime.instruction_cursor, 0);
        assert_eq!(*runtime.jump_path.get(0).unwrap(), 6);
    }

    #[test]
    fn reapply() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::integer(10)).unwrap();
        runtime.add_data(ExpressionData::expression(0)).unwrap();
        runtime.add_data(ExpressionData::integer(20)).unwrap();
        runtime.add_data(ExpressionData::integer(30)).unwrap();
        runtime.add_data(ExpressionData::integer(40)).unwrap();

        // 1
        runtime.add_instruction(Instruction::Put, Some(1)).unwrap();
        runtime.add_instruction(Instruction::Put, Some(2)).unwrap();
        runtime.add_instruction(Instruction::Apply, None).unwrap();

        // 4
        runtime.add_instruction(Instruction::Put, Some(0)).unwrap();
        runtime.add_instruction(Instruction::PutInput, None).unwrap();
        runtime.add_instruction(Instruction::PerformAddition, None).unwrap();
        runtime.add_instruction(Instruction::PutResult, None).unwrap();
        runtime.add_instruction(Instruction::Reapply, Some(0)).unwrap();
        runtime.add_instruction(Instruction::EndExpression, None).unwrap();

        runtime.expression_table.push(4);

        runtime.reference_stack.push(4);

        runtime.inputs.push(2);
        runtime.current_result = Some(4);
        runtime.jump_path.push(9);

        runtime.set_instruction_cursor(8).unwrap();

        runtime.reapply(0).unwrap();

        assert_eq!(runtime.inputs.len(), 1);
        assert_eq!(*runtime.inputs.get(0).unwrap(), 4);
        assert_eq!(runtime.instruction_cursor, 3);
        assert_eq!(*runtime.jump_path.get(0).unwrap(), 9);
    }

    #[test]
    fn apply_from_context() {
        let mut runtime = GarnishLangRuntime::new();

        runtime.add_data(ExpressionData::external(3)).unwrap();
        runtime.add_data(ExpressionData::integer(100)).unwrap();

        runtime.add_instruction(Instruction::Resolve, None).unwrap();

        runtime.reference_stack.push(0);
        runtime.reference_stack.push(1);

        struct MyContext {}

        impl GarnishLangRuntimeContext for MyContext {
            fn resolve(&mut self, _: usize, _: &mut GarnishLangRuntime) -> GarnishLangRuntimeResult<bool> {
                Ok(false)
            }

            fn apply(&mut self, external_value: usize, input_addr: usize, runtime: &mut GarnishLangRuntime) -> GarnishLangRuntimeResult<bool> {
                assert_eq!(external_value, 3);

                let value = match runtime.get_data(input_addr) {
                    None => Err(error(format!("Input address given to external apply doesn't have data.")))?,
                    Some(data) => match data.get_type() {
                        ExpressionDataType::Integer => match data.as_integer() {
                            Err(e) => Err(error(e))?,
                            Ok(i) => i * 2,
                        },
                        _ => return Ok(false),
                    },
                };

                let addr = runtime.get_data_len();
                runtime.add_data(ExpressionData::integer(value))?;
                let raddr = runtime.get_data_len();
                runtime.add_reference_data(addr)?;
                runtime.reference_stack.push(raddr);
                Ok(true)
            }
        }

        let mut context = MyContext {};

        runtime.apply(Some(&mut context)).unwrap();

        assert_eq!(runtime.get_data(2).unwrap().as_integer().unwrap(), 200);
        assert_eq!(runtime.reference_stack.get(0).unwrap(), &3);
        assert_eq!(runtime.get_data(3).unwrap().as_reference().unwrap(), 2);
    }
}
