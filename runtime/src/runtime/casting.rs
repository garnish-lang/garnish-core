use crate::runtime::internals::link_len_size;
use crate::{next_two_raw_ref, push_unit, ExpressionDataType, GarnishLangRuntimeData, RuntimeError};
use crate::runtime::list::iterate_link_internal;

pub(crate) fn type_cast<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    let (right, left) = next_two_raw_ref(this)?;

    match (this.get_data_type(left)?, this.get_data_type(right)?) {
        // NoOp re-push left to register
        (l, r) if l == r => this.push_register(left)?,
        // Numbers
        (ExpressionDataType::Integer, ExpressionDataType::Float) => {
            let i = this.get_integer(left)?;
            match Data::integer_to_float(i) {
                Some(i) => {
                    this.add_float(i).and_then(|r| this.push_register(r))?;
                }
                None => push_unit(this)?,
            }
        }
        (ExpressionDataType::Integer, ExpressionDataType::Char) => {
            let i = this.get_integer(left)?;
            match Data::integer_to_char(i) {
                Some(i) => {
                    this.add_char(i).and_then(|r| this.push_register(r))?;
                }
                None => push_unit(this)?,
            }
        }
        (ExpressionDataType::Integer, ExpressionDataType::Byte) => {
            let i = this.get_integer(left)?;
            match Data::integer_to_byte(i) {
                Some(i) => {
                    this.add_byte(i).and_then(|r| this.push_register(r))?;
                }
                None => push_unit(this)?,
            }
        }
        (ExpressionDataType::Float, ExpressionDataType::Integer) => {
            let f = this.get_float(left)?;
            match Data::float_to_integer(f) {
                Some(f) => {
                    this.add_integer(f).and_then(|r| this.push_register(r))?;
                }
                None => push_unit(this)?,
            }
        }
        (ExpressionDataType::Char, ExpressionDataType::Integer) => {
            let c = this.get_char(left)?;
            match Data::char_to_integer(c) {
                Some(i) => {
                    this.add_integer(i).and_then(|r| this.push_register(r))?;
                }
                None => push_unit(this)?,
            }
        }
        (ExpressionDataType::Char, ExpressionDataType::Byte) => {
            let c = this.get_char(left)?;
            match Data::char_to_byte(c) {
                Some(i) => {
                    this.add_byte(i).and_then(|r| this.push_register(r))?;
                }
                None => push_unit(this)?,
            }
        }
        (ExpressionDataType::Byte, ExpressionDataType::Integer) => {
            let c = this.get_byte(left)?;
            match Data::byte_to_integer(c) {
                Some(i) => {
                    this.add_integer(i).and_then(|r| this.push_register(r))?;
                }
                None => push_unit(this)?,
            }
        }
        (ExpressionDataType::Byte, ExpressionDataType::Char) => {
            let i = this.get_byte(left)?;
            match Data::byte_to_char(i) {
                Some(i) => {
                    this.add_char(i).and_then(|r| this.push_register(r))?;
                }
                None => push_unit(this)?,
            }
        }
        (ExpressionDataType::Link, ExpressionDataType::List) => {
            let len = link_len_size(this, left)?;
            this.start_list(len)?;

            iterate_link_internal(this, left, |this, addr, _current_index| {
                let is_associative = match this.get_data_type(addr)? {
                    ExpressionDataType::Pair => {
                        let (left, _) = this.get_pair(addr)?;
                        match this.get_data_type(left)? {
                            ExpressionDataType::Symbol => true,
                            _ => false
                        }
                    }
                    _ => false
                };

                this.add_to_list(addr, is_associative)?;
                Ok(false)
            })?;

            this.end_list().and_then(|r| this.push_register(r))?
        }
        // Unit and Boolean
        (ExpressionDataType::Unit, ExpressionDataType::True) | (ExpressionDataType::False, ExpressionDataType::True) => {
            this.add_false().and_then(|r| this.push_register(r))?;
        }
        (ExpressionDataType::Unit, ExpressionDataType::False) => this.add_true().and_then(|r| this.push_register(r))?,

        // Final Catches
        (ExpressionDataType::Unit, _) => push_unit(this)?,
        (_, ExpressionDataType::False) => this.add_false().and_then(|r| this.push_register(r))?,
        (_, ExpressionDataType::True) => this.add_true().and_then(|r| this.push_register(r))?,
        _ => push_unit(this)?,
    }

    Ok(())
}

#[cfg(test)]
mod simple {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn no_op_cast_expression() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_expression(10).unwrap();
        let d2 = runtime.add_expression(10).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast().unwrap();

        assert_eq!(runtime.get_expression(runtime.get_register(0).unwrap()).unwrap(), 10);
    }

    #[test]
    fn cast_to_unit() {
        let mut runtime = SimpleRuntimeData::new();

        let int = runtime.add_integer(10).unwrap();
        let unit = runtime.add_unit().unwrap();

        runtime.push_register(int).unwrap();
        runtime.push_register(unit).unwrap();

        runtime.type_cast().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::Unit);
    }

    #[test]
    fn cast_to_true() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_integer(10).unwrap();
        let d2 = runtime.add_true().unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn cast_unit_to_true() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_unit().unwrap();
        let d2 = runtime.add_true().unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn cast_false_to_true() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_false().unwrap();
        let d2 = runtime.add_true().unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn cast_to_false() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_integer(10).unwrap();
        let d2 = runtime.add_false().unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn cast_unit_to_false() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_unit().unwrap();
        let d2 = runtime.add_false().unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn cast_true_to_false() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_true().unwrap();
        let d2 = runtime.add_false().unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }
}

#[cfg(test)]
mod primitive {
    use crate::{runtime::GarnishRuntime, GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn integer_to_float() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_integer(10).unwrap();
        let d2 = runtime.add_float(0.0).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast().unwrap();

        let expected = SimpleRuntimeData::integer_to_float(10).unwrap();

        assert_eq!(runtime.get_float(runtime.get_register(0).unwrap()).unwrap(), expected);
    }

    #[test]
    fn integer_to_char() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_integer('a' as i32).unwrap();
        let d2 = runtime.add_char('\0').unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast().unwrap();

        let expected = SimpleRuntimeData::integer_to_char('a' as i32).unwrap();

        assert_eq!(runtime.get_char(runtime.get_register(0).unwrap()).unwrap(), expected);
    }

    #[test]
    fn integer_to_byte() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_integer(10).unwrap();
        let d2 = runtime.add_byte(0).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast().unwrap();

        let expected = SimpleRuntimeData::integer_to_byte(10).unwrap();

        assert_eq!(runtime.get_byte(runtime.get_register(0).unwrap()).unwrap(), expected);
    }

    #[test]
    fn char_to_integer() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_char('a').unwrap();
        let d2 = runtime.add_integer(0).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast().unwrap();

        let expected = SimpleRuntimeData::char_to_integer('a').unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), expected);
    }

    #[test]
    fn char_to_byte() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_char('a').unwrap();
        let d2 = runtime.add_byte(0).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast().unwrap();

        let expected = SimpleRuntimeData::char_to_byte('a').unwrap();

        assert_eq!(runtime.get_byte(runtime.get_register(0).unwrap()).unwrap(), expected);
    }

    #[test]
    fn byte_to_integer() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_byte('a' as u8).unwrap();
        let d2 = runtime.add_integer(0).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast().unwrap();

        let expected = SimpleRuntimeData::byte_to_integer('a' as u8).unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), expected);
    }

    #[test]
    fn byte_to_char() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_byte('a' as u8).unwrap();
        let d2 = runtime.add_char('a').unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast().unwrap();

        let expected = SimpleRuntimeData::byte_to_char('a' as u8).unwrap();

        assert_eq!(runtime.get_char(runtime.get_register(0).unwrap()).unwrap(), expected);
    }

    #[test]
    fn float_to_integer() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_float(3.14).unwrap();
        let d2 = runtime.add_integer(0).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast().unwrap();

        let expected = SimpleRuntimeData::float_to_integer(3.14).unwrap();

        assert_eq!(runtime.get_integer(runtime.get_register(0).unwrap()).unwrap(), expected);
    }
}

#[cfg(test)]
mod lists {
    use crate::testing_utilites::{add_links_with_start, add_list_with_start};
    use crate::{runtime::GarnishRuntime, GarnishLangRuntimeData, SimpleRuntimeData, symbol_value};

    #[test]
    fn link_to_list() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_links_with_start(&mut runtime, 10, true, 20);
        let d2 = add_list_with_start(&mut runtime, 1, 0);

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.type_cast().unwrap();

        let addr = runtime.get_register(0).unwrap();
        let len = runtime.get_list_len(addr).unwrap();
        assert_eq!(len, 10);

        for i in 0..10 {
            let item_addr = runtime.get_list_item(addr, i).unwrap();
            let (left, right) = runtime.get_pair(item_addr).unwrap();
            let s = symbol_value(format!("val{}", 20 + i).as_ref());
            assert_eq!(runtime.get_symbol(left).unwrap(), s);
            assert_eq!(runtime.get_integer(right).unwrap(), 20 + i);

            let association = runtime.get_list_item_with_symbol(addr, s).unwrap().unwrap();
            assert_eq!(runtime.get_integer(association).unwrap(), 20 + i)
        }
    }
}
