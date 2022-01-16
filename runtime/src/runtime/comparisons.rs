use log::trace;
use std::fmt::Debug;

use crate::{next_two_raw_ref, push_boolean, state_error, ExpressionDataType, GarnishLangRuntimeData, RuntimeError, TypeConstants, get_range};
use crate::runtime::list::index_link;

pub(crate) fn equality_comparison<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    // hope that can get reduced to a constant
    let two = Data::Size::one() + Data::Size::one();
    if this.get_register_len() < two {
        state_error(format!("Not enough registers to perform comparison."))?;
    }

    let start = this.get_register_len() - two;

    while this.get_register_len() > start {
        let (right, left) = next_two_raw_ref(this)?;
        if !data_equal(this, left, right)? {
            // ending early need to remove any remaining values from registers
            while this.get_register_len() > start {
                this.pop_register();
            }

            push_boolean(this, false)?;

            return Ok(());
        }
    }

    push_boolean(this, true)
}

fn data_equal<Data: GarnishLangRuntimeData>(
    this: &mut Data,
    left_addr: Data::Size,
    right_addr: Data::Size,
) -> Result<bool, RuntimeError<Data::Error>> {
    let equal = match (this.get_data_type(left_addr)?, this.get_data_type(right_addr)?) {
        (ExpressionDataType::Unit, ExpressionDataType::Unit)
        | (ExpressionDataType::True, ExpressionDataType::True)
        | (ExpressionDataType::False, ExpressionDataType::False) => true,
        (ExpressionDataType::Expression, ExpressionDataType::Expression) => compare(this, left_addr, right_addr, Data::get_expression)?,
        (ExpressionDataType::External, ExpressionDataType::External) => compare(this, left_addr, right_addr, Data::get_external)?,
        (ExpressionDataType::Symbol, ExpressionDataType::Symbol) => compare(this, left_addr, right_addr, Data::get_symbol)?,
        (ExpressionDataType::Char, ExpressionDataType::Char) => compare(this, left_addr, right_addr, Data::get_char)?,
        (ExpressionDataType::Byte, ExpressionDataType::Byte) => compare(this, left_addr, right_addr, Data::get_byte)?,
        (ExpressionDataType::Float, ExpressionDataType::Float) => compare(this, left_addr, right_addr, Data::get_float)?,
        (ExpressionDataType::Integer, ExpressionDataType::Integer) => compare(this, left_addr, right_addr, Data::get_integer)?,
        (ExpressionDataType::Integer, ExpressionDataType::Float) => {
            let left = this.get_integer(left_addr)?;
            let right = this.get_float(right_addr)?;

            Data::integer_to_float(left) == right
        }
        (ExpressionDataType::Float, ExpressionDataType::Integer) => {
            let left = this.get_float(left_addr)?;
            let right = this.get_integer(right_addr)?;

            left == Data::integer_to_float(right)
        }
        (ExpressionDataType::Char, ExpressionDataType::CharList) => {
            if this.get_char_list_len(right_addr)? == Data::Size::one() {
                let c1 = this.get_char(left_addr)?;
                let c2 = this.get_char_list_item(right_addr, Data::Integer::zero())?;

                c1 == c2
            } else {
                false
            }
        }
        (ExpressionDataType::CharList, ExpressionDataType::Char) => {
            if this.get_char_list_len(left_addr)? == Data::Size::one() {
                let c1 = this.get_char_list_item(left_addr, Data::Integer::zero())?;
                let c2 = this.get_char(right_addr)?;

                c1 == c2
            } else {
                false
            }
        }
        (ExpressionDataType::Byte, ExpressionDataType::ByteList) => {
            if this.get_byte_list_len(right_addr)? == Data::Size::one() {
                let c1 = this.get_byte(left_addr)?;
                let c2 = this.get_byte_list_item(right_addr, Data::Integer::zero())?;

                c1 == c2
            } else {
                false
            }
        }
        (ExpressionDataType::ByteList, ExpressionDataType::Byte) => {
            if this.get_byte_list_len(left_addr)? == Data::Size::one() {
                let c1 = this.get_byte_list_item(left_addr, Data::Integer::zero())?;
                let c2 = this.get_byte(right_addr)?;

                c1 == c2
            } else {
                false
            }
        }
        (ExpressionDataType::CharList, ExpressionDataType::CharList) => {
            let len1 = this.get_char_list_len(left_addr)?;
            let len2 = this.get_char_list_len(right_addr)?;

            if len1 != len2 {
                false
            } else {
                let mut count = Data::Size::one();
                let mut equal = true;
                while count < len1 {
                    let i = Data::size_to_integer(count);
                    let c1 = this.get_char_list_item(left_addr, i)?;
                    let c2 = this.get_char_list_item(right_addr, i)?;

                    if c1 != c2 {
                        equal = false;
                    }

                    count += Data::Size::one();
                }

                equal
            }
        }
        (ExpressionDataType::ByteList, ExpressionDataType::ByteList) => {
            let len1 = this.get_byte_list_len(left_addr)?;
            let len2 = this.get_byte_list_len(right_addr)?;

            if len1 != len2 {
                false
            } else {
                let mut count = Data::Size::one();
                let mut equal = true;
                while count < len1 {
                    let i = Data::size_to_integer(count);
                    let c1 = this.get_byte_list_item(left_addr, i)?;
                    let c2 = this.get_byte_list_item(right_addr, i)?;

                    if c1 != c2 {
                        equal = false;
                    }

                    count += Data::Size::one();
                }

                equal
            }
        }
        (ExpressionDataType::Range, ExpressionDataType::Range) => {
            let (start1, end1) = this.get_range(left_addr)?;
            let (start2, end2) = this.get_range(right_addr)?;

            let start_equal = match (this.get_data_type(start1)?, this.get_data_type(start2)?) {
                (ExpressionDataType::Unit, ExpressionDataType::Unit) => true,
                (ExpressionDataType::Integer, ExpressionDataType::Integer) => this.get_integer(start1)? == this.get_integer(start2)?,
                _ => false,
            };

            let end_equal = match (this.get_data_type(end1)?, this.get_data_type(end2)?) {
                (ExpressionDataType::Unit, ExpressionDataType::Unit) => true,
                (ExpressionDataType::Integer, ExpressionDataType::Integer) => this.get_integer(end1)? == this.get_integer(end2)?,
                _ => false,
            };

            start_equal && end_equal
        }
        (ExpressionDataType::Pair, ExpressionDataType::Pair) => {
            let (left1, right1) = this.get_pair(left_addr)?;
            let (left2, right2) = this.get_pair(right_addr)?;

            this.push_register(left1)?;
            this.push_register(left2)?;

            this.push_register(right1)?;
            this.push_register(right2)?;

            true
        }
        (ExpressionDataType::Slice, ExpressionDataType::Slice) => {
            let (value1, range1) = this.get_slice(left_addr)?;
            let (value2, range2) = this.get_slice(right_addr)?;

            let (start1, end1, len1) = get_range(this, range1)?;
            let (start2, end2, len2) = get_range(this, range2)?;

            // slices need same len of range
            // if so, run through list and push to register

            if len1 != len2 {
                false
            } else {
                match (this.get_data_type(value1)?, this.get_data_type(value2)?) {
                    (ExpressionDataType::List, ExpressionDataType::List) => {
                        let mut index1 = start1;
                        let mut index2 = start2;
                        let mut count = Data::Integer::zero();

                        let list_len1 = Data::size_to_integer(this.get_list_len(value1)?);
                        let list_len2 = Data::size_to_integer(this.get_list_len(value2)?);

                        while count < len1 {
                            let item1 = if index1 < list_len1 {
                                this.get_list_item(value1, index1)?
                            } else {
                                this.add_unit()?
                            };

                            let item2 = if index2 < list_len2 {
                                this.get_list_item(value2, index2)?
                            } else {
                                this.add_unit()?
                            };

                            this.push_register(item1)?;
                            this.push_register(item2)?;

                            index1 += Data::Integer::one();
                            index2 += Data::Integer::one();
                            count += Data::Integer::one();
                        }

                        true
                    }
                    (ExpressionDataType::Link, ExpressionDataType::Link) => {
                        // worth revisiting, to find more efficiant solution

                        let mut index1 = start1;
                        let mut index2 = start2;
                        let mut count = Data::Integer::zero();

                        while count < len1 {
                            match (index_link(this, value1, index1)?, index_link(this, value2, index2)?) {
                                (Some(item1), Some(item2)) => {
                                    this.push_register(item1)?;
                                    this.push_register(item2)?;
                                }
                                (Some(item1), None) => {
                                    let a = this.add_unit()?;

                                    this.push_register(item1)?;
                                    this.push_register(a)?;
                                }
                                (None, Some(item2)) => {
                                    let a = this.add_unit()?;

                                    this.push_register(a)?;
                                    this.push_register(item2)?;
                                }
                                _ => {
                                    // neither have this item which means they're both Unit and equal
                                    // true is default, nothing to compare
                                }
                            }

                            index1 += Data::Integer::one();
                            index2 += Data::Integer::one();
                            count += Data::Integer::one();
                        }

                        true
                    }
                    _ => false,
                }
            }
        }
        (ExpressionDataType::List, ExpressionDataType::List) => {
            let association_len1 = this.get_list_associations_len(left_addr)?;
            let associations_len2 = this.get_list_associations_len(right_addr)?;
            let len1 = this.get_list_len(left_addr)?;
            let len2 = this.get_list_len(right_addr)?;

            // Equality is determined sequentially
            // associations and non associations must be in the same positions
            // Ex. (for list position x)
            //      left has association at x, right has association at x
            //          right is check for the same association at left.x
            //          if right has association, position not considered, values are pushed for equality check
            //      left has item at x, right has item at x
            //          both items are pushed for equality
            //      left has association at x, right has item at x (and vice versa)
            //          list is not equal, end comparison by returning false
            if association_len1 != associations_len2 || len1 != len2 {
                false
            } else {
                let mut count = Data::Size::zero();
                while count < len1 {
                    let i = Data::size_to_integer(count);
                    let left_item = this.get_list_item(left_addr, i)?;
                    let right_item = this.get_list_item(right_addr, i)?;

                    let (left_is_associative, pair_sym, pair_item) = match this.get_data_type(left_item)? {
                        ExpressionDataType::Pair => {
                            let (left, right) = this.get_pair(left_item)?;
                            match this.get_data_type(left)? {
                                ExpressionDataType::Symbol => (true, this.get_symbol(left)?, right),
                                _ => (false, Data::Symbol::zero(), Data::Size::zero()),
                            }
                        }
                        _ => (false, Data::Symbol::zero(), Data::Size::zero()),
                    };

                    if left_is_associative {
                        match this.get_list_item_with_symbol(right_addr, pair_sym)? {
                            Some(right_item) => {
                                // has same association, push both items for comparison
                                this.push_register(pair_item)?;
                                this.push_register(right_item)?;
                            }
                            None => {
                                // right does not have association
                                // lists are not equal, return false
                                return Ok(false);
                            }
                        }
                    } else {
                        // not an association, push both for comparision
                        this.push_register(left_item)?;
                        this.push_register(right_item)?;
                    }

                    count += Data::Size::one();
                }

                true
            }
        }
        _ => false,
    };

    Ok(equal)
}

fn compare<Data: GarnishLangRuntimeData, F, V: PartialOrd + Debug>(
    this: &Data,
    left_addr: Data::Size,
    right_addr: Data::Size,
    get_func: F,
) -> Result<bool, Data::Error>
where
    F: Fn(&Data, Data::Size) -> Result<V, Data::Error>,
{
    let left = get_func(this, left_addr)?;
    let right = get_func(this, right_addr)?;

    trace!("Comparing {:?} == {:?}", left, right);

    Ok(left == right)
}

#[cfg(test)]
mod tests {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

    #[test]
    fn equality_no_references_is_err() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.add_integer(10).unwrap();
        runtime.add_integer(10).unwrap();

        runtime.push_instruction(Instruction::EqualityComparison, None).unwrap();

        let result = runtime.equality_comparison();

        assert!(result.is_err());
    }

    #[test]
    fn equality_of_unsupported_comparison_is_false() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_integer(10).unwrap();
        let exp1 = runtime.add_expression(10).unwrap();

        runtime.push_instruction(Instruction::EqualityComparison, None).unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(exp1).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }
}

#[cfg(test)]
mod simple_types {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn equality_units_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_unit().unwrap();
        let int2 = runtime.add_unit().unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_true_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_true().unwrap();
        let int2 = runtime.add_true().unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_false_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_false().unwrap();
        let int2 = runtime.add_false().unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }
}

#[cfg(test)]
mod numbers {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn equality_integers_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_integer(10).unwrap();
        let int2 = runtime.add_integer(10).unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_integers_not_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_integer(10).unwrap();
        let int2 = runtime.add_integer(20).unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn equality_floats_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_float(10.0).unwrap();
        let int2 = runtime.add_float(10.0).unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_floats_not_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_float(10.0).unwrap();
        let int2 = runtime.add_float(20.0).unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn equality_integer_float_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_integer(10).unwrap();
        let int2 = runtime.add_float(10.0).unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_float_integer_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_float(10.0).unwrap();
        let int2 = runtime.add_integer(10).unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_integer_float_note_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_integer(10).unwrap();
        let int2 = runtime.add_float(20.0).unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn equality_float_integer_note_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_float(10.0).unwrap();
        let int2 = runtime.add_integer(20).unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }
}

#[cfg(test)]
mod chars {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn equality_chars_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_char('a').unwrap();
        let int2 = runtime.add_char('a').unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_chars_not_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_char('a').unwrap();
        let int2 = runtime.add_char('b').unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn equality_char_lists_equal() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.start_char_list().unwrap();
        runtime.add_to_char_list('a').unwrap();
        runtime.add_to_char_list('b').unwrap();
        runtime.add_to_char_list('c').unwrap();
        let int1 = runtime.end_char_list().unwrap();

        runtime.start_char_list().unwrap();
        runtime.add_to_char_list('a').unwrap();
        runtime.add_to_char_list('b').unwrap();
        runtime.add_to_char_list('c').unwrap();
        let int2 = runtime.end_char_list().unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_char_lists_not_equal() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.start_char_list().unwrap();
        runtime.add_to_char_list('a').unwrap();
        runtime.add_to_char_list('b').unwrap();
        runtime.add_to_char_list('c').unwrap();
        let int1 = runtime.end_char_list().unwrap();

        runtime.start_char_list().unwrap();
        runtime.add_to_char_list('a').unwrap();
        runtime.add_to_char_list('b').unwrap();
        runtime.add_to_char_list('d').unwrap();
        let int2 = runtime.end_char_list().unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn equality_char_char_list_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_char('a').unwrap();

        runtime.start_char_list().unwrap();
        runtime.add_to_char_list('a').unwrap();
        let int2 = runtime.end_char_list().unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_char_char_list_not_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_char('a').unwrap();

        runtime.start_char_list().unwrap();
        runtime.add_to_char_list('a').unwrap();
        runtime.add_to_char_list('b').unwrap();
        let int2 = runtime.end_char_list().unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn equality_char_list_char_equal() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.start_char_list().unwrap();
        runtime.add_to_char_list('a').unwrap();
        let int1 = runtime.end_char_list().unwrap();

        let int2 = runtime.add_char('a').unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_char_list_char_not_equal() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.start_char_list().unwrap();
        runtime.add_to_char_list('a').unwrap();
        runtime.add_to_char_list('b').unwrap();
        let int1 = runtime.end_char_list().unwrap();

        let int2 = runtime.add_char('a').unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }
}

#[cfg(test)]
mod bytes {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn equality_bytes_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_byte(10).unwrap();
        let int2 = runtime.add_byte(10).unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_bytes_not_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_byte(10).unwrap();
        let int2 = runtime.add_byte(20).unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn equality_byte_lists_equal() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.start_byte_list().unwrap();
        runtime.add_to_byte_list(10).unwrap();
        runtime.add_to_byte_list(20).unwrap();
        runtime.add_to_byte_list(30).unwrap();
        let int1 = runtime.end_byte_list().unwrap();

        runtime.start_byte_list().unwrap();
        runtime.add_to_byte_list(10).unwrap();
        runtime.add_to_byte_list(20).unwrap();
        runtime.add_to_byte_list(30).unwrap();
        let int2 = runtime.end_byte_list().unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_byte_lists_not_equal() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.start_byte_list().unwrap();
        runtime.add_to_byte_list(10).unwrap();
        runtime.add_to_byte_list(20).unwrap();
        runtime.add_to_byte_list(30).unwrap();
        let int1 = runtime.end_byte_list().unwrap();

        runtime.start_byte_list().unwrap();
        runtime.add_to_byte_list(10).unwrap();
        runtime.add_to_byte_list(20).unwrap();
        runtime.add_to_byte_list(40).unwrap();
        let int2 = runtime.end_byte_list().unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn equality_byte_byte_list_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_byte(10).unwrap();

        runtime.start_byte_list().unwrap();
        runtime.add_to_byte_list(10).unwrap();
        let int2 = runtime.end_byte_list().unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_byte_byte_list_not_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_byte(10).unwrap();

        runtime.start_byte_list().unwrap();
        runtime.add_to_byte_list(10).unwrap();
        runtime.add_to_byte_list(20).unwrap();
        let int2 = runtime.end_byte_list().unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn equality_byte_list_byte_equal() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.start_byte_list().unwrap();
        runtime.add_to_byte_list(10).unwrap();
        let int1 = runtime.end_byte_list().unwrap();

        let int2 = runtime.add_byte(10).unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_byte_list_byte_not_equal() {
        let mut runtime = SimpleRuntimeData::new();

        runtime.start_byte_list().unwrap();
        runtime.add_to_byte_list(10).unwrap();
        runtime.add_to_byte_list(20).unwrap();
        let int1 = runtime.end_byte_list().unwrap();

        let int2 = runtime.add_byte(10).unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }
}

#[cfg(test)]
mod symbols {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

    #[test]
    fn equality_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_symbol("sym").unwrap();
        let int2 = runtime.add_symbol("sym").unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_not_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_symbol("sym").unwrap();
        let int2 = runtime.add_symbol("val").unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.push_instruction(Instruction::EqualityComparison, None).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }
}

#[cfg(test)]
mod expression {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

    #[test]
    fn equality_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_expression(10).unwrap();
        let int2 = runtime.add_expression(10).unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_not_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_expression(10).unwrap();
        let int2 = runtime.add_expression(20).unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.push_instruction(Instruction::EqualityComparison, None).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }
}

#[cfg(test)]
mod external {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

    #[test]
    fn equality_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_external(10).unwrap();
        let int2 = runtime.add_external(10).unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_not_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_external(10).unwrap();
        let int2 = runtime.add_external(20).unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.push_instruction(Instruction::EqualityComparison, None).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }
}

#[cfg(test)]
mod pairs {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

    #[test]
    fn equality_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();

        let i4 = runtime.add_integer(10).unwrap();
        let i5 = runtime.add_integer(10).unwrap();
        let i6 = runtime.add_pair((i4, i5)).unwrap();

        runtime.push_register(i3).unwrap();
        runtime.push_register(i6).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_not_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();

        let i4 = runtime.add_integer(10).unwrap();
        let i5 = runtime.add_integer(10).unwrap();
        let i6 = runtime.add_pair((i4, i5)).unwrap();

        runtime.push_register(i3).unwrap();
        runtime.push_register(i6).unwrap();

        runtime.push_instruction(Instruction::EqualityComparison, None).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }
}

#[cfg(test)]
mod ranges {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn equality_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_range(i1, i2).unwrap();

        let i4 = runtime.add_integer(10).unwrap();
        let i5 = runtime.add_integer(10).unwrap();
        let i6 = runtime.add_range(i4, i5).unwrap();

        runtime.push_register(i3).unwrap();
        runtime.push_register(i6).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_open_start_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_unit().unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_range(i1, i2).unwrap();

        let i4 = runtime.add_unit().unwrap();
        let i5 = runtime.add_integer(10).unwrap();
        let i6 = runtime.add_range(i4, i5).unwrap();

        runtime.push_register(i3).unwrap();
        runtime.push_register(i6).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_open_start_integer_not_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_unit().unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_range(i1, i2).unwrap();

        let i4 = runtime.add_integer(5).unwrap();
        let i5 = runtime.add_integer(10).unwrap();
        let i6 = runtime.add_range(i4, i5).unwrap();

        runtime.push_register(i3).unwrap();
        runtime.push_register(i6).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn equality_integer_open_start_not_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(5).unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_range(i1, i2).unwrap();

        let i4 = runtime.add_unit().unwrap();
        let i5 = runtime.add_integer(10).unwrap();
        let i6 = runtime.add_range(i4, i5).unwrap();

        runtime.push_register(i3).unwrap();
        runtime.push_register(i6).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn equality_open_end_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_unit().unwrap();
        let i3 = runtime.add_range(i1, i2).unwrap();

        let i4 = runtime.add_integer(10).unwrap();
        let i5 = runtime.add_unit().unwrap();
        let i6 = runtime.add_range(i4, i5).unwrap();

        runtime.push_register(i3).unwrap();
        runtime.push_register(i6).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_open_end_integer_not_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_unit().unwrap();
        let i3 = runtime.add_range(i1, i2).unwrap();

        let i4 = runtime.add_integer(10).unwrap();
        let i5 = runtime.add_integer(20).unwrap();
        let i6 = runtime.add_range(i4, i5).unwrap();

        runtime.push_register(i3).unwrap();
        runtime.push_register(i6).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn equality_integer_open_end_not_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(20).unwrap();
        let i3 = runtime.add_range(i1, i2).unwrap();

        let i4 = runtime.add_integer(10).unwrap();
        let i5 = runtime.add_unit().unwrap();
        let i6 = runtime.add_range(i4, i5).unwrap();

        runtime.push_register(i3).unwrap();
        runtime.push_register(i6).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::False);
    }

    #[test]
    fn equality_start_not_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_range(i1, i2).unwrap();

        let i4 = runtime.add_integer(5).unwrap();
        let i5 = runtime.add_integer(10).unwrap();
        let i6 = runtime.add_range(i4, i5).unwrap();

        runtime.push_register(i3).unwrap();
        runtime.push_register(i6).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn equality_end_not_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_range(i1, i2).unwrap();

        let i4 = runtime.add_integer(10).unwrap();
        let i5 = runtime.add_integer(20).unwrap();
        let i6 = runtime.add_range(i4, i5).unwrap();

        runtime.push_register(i3).unwrap();
        runtime.push_register(i6).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }
}

#[cfg(test)]
mod lists {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

    #[test]
    fn equality_only_items_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_integer(10).unwrap();
        runtime.start_list(3).unwrap();
        runtime.add_to_list(i1, false).unwrap();
        runtime.add_to_list(i2, false).unwrap();
        runtime.add_to_list(i3, false).unwrap();
        let i4 = runtime.end_list().unwrap();

        let i5 = runtime.add_integer(10).unwrap();
        let i6 = runtime.add_integer(10).unwrap();
        let i7 = runtime.add_integer(10).unwrap();
        runtime.start_list(3).unwrap();
        runtime.add_to_list(i5, false).unwrap();
        runtime.add_to_list(i6, false).unwrap();
        runtime.add_to_list(i7, false).unwrap();
        let i8 = runtime.end_list().unwrap();

        runtime.push_register(i4).unwrap();
        runtime.push_register(i8).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_only_items_not_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_integer(10).unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_integer(10).unwrap();
        runtime.start_list(3).unwrap();
        runtime.add_to_list(i1, false).unwrap();
        runtime.add_to_list(i2, false).unwrap();
        runtime.add_to_list(i3, false).unwrap();
        let i4 = runtime.end_list().unwrap();

        let i5 = runtime.add_integer(10).unwrap();
        let i6 = runtime.add_integer(20).unwrap();
        let i7 = runtime.add_integer(30).unwrap();
        runtime.start_list(3).unwrap();
        runtime.add_to_list(i5, false).unwrap();
        runtime.add_to_list(i6, false).unwrap();
        runtime.add_to_list(i7, false).unwrap();
        let i8 = runtime.end_list().unwrap();

        runtime.push_register(i4).unwrap();
        runtime.push_register(i8).unwrap();

        runtime.push_instruction(Instruction::EqualityComparison, None).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn equality_associations_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("val1").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();

        let i4 = runtime.add_symbol("val2").unwrap();
        let i5 = runtime.add_integer(20).unwrap();
        let i6 = runtime.add_pair((i4, i5)).unwrap();

        let i7 = runtime.add_symbol("val3").unwrap();
        let i8 = runtime.add_integer(30).unwrap();
        let i9 = runtime.add_pair((i7, i8)).unwrap();

        runtime.start_list(3).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        runtime.add_to_list(i6, true).unwrap();
        runtime.add_to_list(i9, true).unwrap();
        let i10 = runtime.end_list().unwrap();

        let i11 = runtime.add_symbol("val3").unwrap();
        let i12 = runtime.add_integer(30).unwrap();
        let i13 = runtime.add_pair((i11, i12)).unwrap();

        let i14 = runtime.add_symbol("val1").unwrap();
        let i15 = runtime.add_integer(10).unwrap();
        let i16 = runtime.add_pair((i14, i15)).unwrap();

        let i17 = runtime.add_symbol("val2").unwrap();
        let i18 = runtime.add_integer(20).unwrap();
        let i19 = runtime.add_pair((i17, i18)).unwrap();

        runtime.start_list(3).unwrap();
        runtime.add_to_list(i13, true).unwrap();
        runtime.add_to_list(i16, true).unwrap();
        runtime.add_to_list(i19, true).unwrap();
        let i20 = runtime.end_list().unwrap();

        runtime.push_register(i10).unwrap();
        runtime.push_register(i20).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_associations_not_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("val1").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();

        let i4 = runtime.add_symbol("val2").unwrap();
        let i5 = runtime.add_integer(20).unwrap();
        let i6 = runtime.add_pair((i4, i5)).unwrap();

        let i7 = runtime.add_symbol("val3").unwrap();
        let i8 = runtime.add_integer(30).unwrap();
        let i9 = runtime.add_pair((i7, i8)).unwrap();

        runtime.start_list(3).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        runtime.add_to_list(i6, true).unwrap();
        runtime.add_to_list(i9, true).unwrap();
        let i10 = runtime.end_list().unwrap();

        let i11 = runtime.add_symbol("val3").unwrap();
        let i12 = runtime.add_integer(30).unwrap();
        let i13 = runtime.add_pair((i11, i12)).unwrap();

        let i14 = runtime.add_symbol("val1").unwrap();
        let i15 = runtime.add_integer(10).unwrap();
        let i16 = runtime.add_pair((i14, i15)).unwrap();

        let i17 = runtime.add_symbol("val2").unwrap();
        let i18 = runtime.add_integer(100).unwrap();
        let i19 = runtime.add_pair((i17, i18)).unwrap();

        runtime.start_list(3).unwrap();
        runtime.add_to_list(i13, true).unwrap();
        runtime.add_to_list(i16, true).unwrap();
        runtime.add_to_list(i19, true).unwrap();
        let i20 = runtime.end_list().unwrap();

        runtime.push_register(i10).unwrap();
        runtime.push_register(i20).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn equality_mixed_values_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("val1").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();

        let i4 = runtime.add_integer(40).unwrap();

        let i5 = runtime.add_symbol("val3").unwrap();
        let i6 = runtime.add_integer(30).unwrap();
        let i7 = runtime.add_pair((i5, i6)).unwrap();

        runtime.start_list(3).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        runtime.add_to_list(i4, false).unwrap();
        runtime.add_to_list(i7, true).unwrap();
        let i8 = runtime.end_list().unwrap();

        let i9 = runtime.add_symbol("val1").unwrap();
        let i10 = runtime.add_integer(10).unwrap();
        let i11 = runtime.add_pair((i9, i10)).unwrap();

        let i12 = runtime.add_integer(40).unwrap();

        let i13 = runtime.add_symbol("val3").unwrap();
        let i14 = runtime.add_integer(30).unwrap();
        let i15 = runtime.add_pair((i13, i14)).unwrap();

        runtime.start_list(3).unwrap();
        runtime.add_to_list(i11, true).unwrap();
        runtime.add_to_list(i12, false).unwrap();
        runtime.add_to_list(i15, true).unwrap();
        let i16 = runtime.end_list().unwrap();

        runtime.push_register(i8).unwrap();
        runtime.push_register(i16).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(), ExpressionDataType::True);
    }

    #[test]
    fn equality_mixed_values_not_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let i1 = runtime.add_symbol("val1").unwrap();
        let i2 = runtime.add_integer(10).unwrap();
        let i3 = runtime.add_pair((i1, i2)).unwrap();

        let i4 = runtime.add_integer(40).unwrap();

        let i5 = runtime.add_symbol("val3").unwrap();
        let i6 = runtime.add_integer(30).unwrap();
        let i7 = runtime.add_pair((i5, i6)).unwrap();

        runtime.start_list(3).unwrap();
        runtime.add_to_list(i3, true).unwrap();
        runtime.add_to_list(i4, false).unwrap();
        runtime.add_to_list(i7, true).unwrap();
        let i8 = runtime.end_list().unwrap();

        let i9 = runtime.add_symbol("val1").unwrap();
        let i10 = runtime.add_integer(10).unwrap();
        let i11 = runtime.add_pair((i9, i10)).unwrap();

        let i12 = runtime.add_integer(20).unwrap();

        let i13 = runtime.add_symbol("val3").unwrap();
        let i14 = runtime.add_integer(30).unwrap();
        let i15 = runtime.add_pair((i13, i14)).unwrap();

        runtime.start_list(3).unwrap();
        runtime.add_to_list(i11, true).unwrap();
        runtime.add_to_list(i12, false).unwrap();
        runtime.add_to_list(i15, true).unwrap();
        let i16 = runtime.end_list().unwrap();

        runtime.push_register(i8).unwrap();
        runtime.push_register(i16).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }
}

#[cfg(test)]
mod slices {
    use crate::{runtime::GarnishRuntime, ExpressionDataType, GarnishLangRuntimeData, SimpleRuntimeData};
    use crate::testing_utilites::{add_links_with_start, add_list_with_start, add_range};

    #[test]
    fn slice_of_list_slice_of_list() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_list_with_start(&mut runtime, 10, 15);
        let d2 = add_range(&mut runtime, 0, 4);
        let d3 = runtime.add_slice(d1, d2).unwrap();

        let d4 = add_list_with_start(&mut runtime, 10, 10);
        let d5 = add_range(&mut runtime, 5, 9);
        let d6 = runtime.add_slice(d4, d5).unwrap();

        runtime.push_register(d3).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::True
        );
    }

    #[test]
    fn slice_of_list_slice_of_incomplete_list_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let unit = runtime.add_unit().unwrap();
        let d1 = runtime.add_integer(10).unwrap();

        runtime.start_list(3).unwrap();
        runtime.add_to_list(d1, false).unwrap();
        runtime.add_to_list(unit, false).unwrap();
        runtime.add_to_list(unit, false).unwrap();
        let list1 = runtime.end_list().unwrap();

        runtime.start_list(1).unwrap();
        runtime.add_to_list(d1, false).unwrap();
        let list2 = runtime.end_list().unwrap();

        let range = add_range(&mut runtime, 0, 2);
        let slice1 = runtime.add_slice(list1, range).unwrap();
        let slice2 = runtime.add_slice(list2, range).unwrap();

        runtime.push_register(slice1).unwrap();
        runtime.push_register(slice2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::True
        );
    }

    #[test]
    fn slice_of_list_slice_of_incomplete_list_not_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let unit = runtime.add_unit().unwrap();
        let d1 = runtime.add_integer(10).unwrap();
        let d2 = runtime.add_integer(20).unwrap();

        runtime.start_list(3).unwrap();
        runtime.add_to_list(d1, false).unwrap();
        runtime.add_to_list(d2, false).unwrap();
        runtime.add_to_list(unit, false).unwrap();
        let list1 = runtime.end_list().unwrap();

        runtime.start_list(1).unwrap();
        runtime.add_to_list(d1, false).unwrap();
        let list2 = runtime.end_list().unwrap();

        let range = add_range(&mut runtime, 0, 2);
        let slice1 = runtime.add_slice(list1, range).unwrap();
        let slice2 = runtime.add_slice(list2, range).unwrap();

        runtime.push_register(slice1).unwrap();
        runtime.push_register(slice2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn slice_of_link_slice_of_link() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = add_links_with_start(&mut runtime, 10, true, 15);
        let d2 = add_range(&mut runtime, 0, 4);
        let d3 = runtime.add_slice(d1, d2).unwrap();

        let d4 = add_links_with_start(&mut runtime, 10, true, 10);
        let d5 = add_range(&mut runtime, 5, 9);
        let d6 = runtime.add_slice(d4, d5).unwrap();

        runtime.push_register(d3).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::True
        );
    }

    #[test]
    fn slice_of_link_slice_of_incomplete_link_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let unit = runtime.add_unit().unwrap();

        let d1 = runtime.add_integer(10).unwrap();

        let link1 = runtime.add_link(d1, unit, true).unwrap();
        let link2 = runtime.add_link(unit, link1, true).unwrap();
        let link3 = runtime.add_link(unit, link2, true).unwrap();

        let link4 = runtime.add_link(d1, unit, true).unwrap();

        let range1 = add_range(&mut runtime, 0, 2);
        let slice1 = runtime.add_slice(link3, range1).unwrap();
        let slice2 = runtime.add_slice(link4, range1).unwrap();

        runtime.push_register(slice1).unwrap();
        runtime.push_register(slice2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::True
        );
    }

    #[test]
    fn slice_of_link_slice_of_incomplete_link_not_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let unit = runtime.add_unit().unwrap();

        let d1 = runtime.add_integer(10).unwrap();
        let d2 = runtime.add_integer(20).unwrap();

        let link1 = runtime.add_link(d1, unit, true).unwrap();
        let link2 = runtime.add_link(d2, link1, true).unwrap();
        let link3 = runtime.add_link(unit, link2, true).unwrap();

        let link4 = runtime.add_link(d1, unit, true).unwrap();

        let range1 = add_range(&mut runtime, 0, 2);
        let slice1 = runtime.add_slice(link3, range1).unwrap();
        let slice2 = runtime.add_slice(link4, range1).unwrap();

        runtime.push_register(slice1).unwrap();
        runtime.push_register(slice2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }

    #[test]
    fn slice_of_incomplete_link_slice_of_link_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let unit = runtime.add_unit().unwrap();

        let d1 = runtime.add_integer(10).unwrap();

        let link1 = runtime.add_link(d1, unit, true).unwrap();
        let link2 = runtime.add_link(unit, link1, true).unwrap();
        let link3 = runtime.add_link(unit, link2, true).unwrap();

        let link4 = runtime.add_link(d1, unit, true).unwrap();

        let range1 = add_range(&mut runtime, 0, 2);
        let slice1 = runtime.add_slice(link3, range1).unwrap();
        let slice2 = runtime.add_slice(link4, range1).unwrap();

        runtime.push_register(slice2).unwrap();
        runtime.push_register(slice1).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::True
        );
    }

    #[test]
    fn slice_of_incomplete_link_slice_of_link_not_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let unit = runtime.add_unit().unwrap();

        let d1 = runtime.add_integer(10).unwrap();
        let d2 = runtime.add_integer(20).unwrap();

        let link1 = runtime.add_link(d1, unit, true).unwrap();
        let link2 = runtime.add_link(d2, link1, true).unwrap();
        let link3 = runtime.add_link(unit, link2, true).unwrap();

        let link4 = runtime.add_link(d1, unit, true).unwrap();

        let range1 = add_range(&mut runtime, 0, 2);
        let slice1 = runtime.add_slice(link3, range1).unwrap();
        let slice2 = runtime.add_slice(link4, range1).unwrap();

        runtime.push_register(slice2).unwrap();
        runtime.push_register(slice1).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(
            runtime.get_data_type(runtime.get_register(0).unwrap()).unwrap(),
            ExpressionDataType::False
        );
    }
}
