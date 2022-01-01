use log::trace;
use std::fmt::Debug;

use crate::{next_two_raw_ref, push_boolean, ExpressionDataType, GarnishLangRuntimeData, RuntimeError, TypeConstants};

pub(crate) fn equality_comparison<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    trace!("Instruction - Equality Comparison");

    let (right_addr, left_addr) = next_two_raw_ref(this)?;

    let lease = this.lease_tmp_stack()?;
    this.push_tmp_stack(lease, left_addr)?;
    this.push_tmp_stack(lease, right_addr)?;

    while let (Some(right), Some(left)) = (this.pop_tmp_stack(lease)?, this.pop_tmp_stack(lease)?) {
        if !data_equal(this, left, right, lease)? {
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
    lease: Data::DataLease,
) -> Result<bool, RuntimeError<Data::Error>> {
    let equal = match (this.get_data_type(left_addr)?, this.get_data_type(right_addr)?) {
        (ExpressionDataType::Unit, ExpressionDataType::Unit)
        | (ExpressionDataType::True, ExpressionDataType::True)
        | (ExpressionDataType::False, ExpressionDataType::False) => true,
        (ExpressionDataType::Expression, ExpressionDataType::Expression) => compare(this, left_addr, right_addr, Data::get_expression)?,
        (ExpressionDataType::External, ExpressionDataType::External) => compare(this, left_addr, right_addr, Data::get_external)?,
        (ExpressionDataType::Symbol, ExpressionDataType::Symbol) => compare(this, left_addr, right_addr, Data::get_symbol)?,
        (ExpressionDataType::Integer, ExpressionDataType::Integer) => compare(this, left_addr, right_addr, Data::get_integer)?,
        (ExpressionDataType::Pair, ExpressionDataType::Pair) => {
            let (left1, right1) = this.get_pair(left_addr)?;
            let (left2, right2) = this.get_pair(right_addr)?;

            this.push_tmp_stack(lease, left1)?;
            this.push_tmp_stack(lease, left2)?;

            this.push_tmp_stack(lease, right1)?;
            this.push_tmp_stack(lease, right2)?;

            true
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
                                ExpressionDataType::Symbol => {
                                    (true, this.get_symbol(left)?, right)
                                },
                                _ => (false, Data::Symbol::zero(), Data::Size::zero())
                            }
                        }
                        _ => (false, Data::Symbol::zero(), Data::Size::zero())
                    };

                    if left_is_associative {
                        match this.get_list_item_with_symbol(right_addr, pair_sym)? {
                            Some(right_item) => {
                                // has same association, push both items for comparison
                                this.push_tmp_stack(lease, pair_item)?;
                                this.push_tmp_stack(lease, right_item)?;
                            }
                            None => {
                                // right does not have association
                                // lists are not equal, return false
                                return Ok(false)
                            }
                        }
                    } else {
                        // not an association, push both for comparision

                        this.push_tmp_stack(lease, left_item)?;
                        this.push_tmp_stack(lease, right_item)?;
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
    use crate::{runtime::GarnishRuntime, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

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

        assert_eq!(runtime.get_registers(), &vec![1]);
    }
}

#[cfg(test)]
mod simple_types {
    use crate::{runtime::GarnishRuntime, GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn equality_units_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_unit().unwrap();
        let int2 = runtime.add_unit().unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_registers(), &vec![2]);
    }

    #[test]
    fn equality_true_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_true().unwrap();
        let int2 = runtime.add_true().unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_registers(), &vec![2]);
    }

    #[test]
    fn equality_false_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_false().unwrap();
        let int2 = runtime.add_false().unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_registers(), &vec![2]);
    }
}

#[cfg(test)]
mod numbers {
    use crate::{runtime::GarnishRuntime, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

    #[test]
    fn equality_integers_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_integer(10).unwrap();
        let int2 = runtime.add_integer(10).unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_registers(), &vec![2]);
    }

    #[test]
    fn equality_integers_not_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_integer(10).unwrap();
        let int2 = runtime.add_integer(20).unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.push_instruction(Instruction::EqualityComparison, None).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_registers(), &vec![1]);
    }
}

#[cfg(test)]
mod symbols {
    use crate::{runtime::GarnishRuntime, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

    #[test]
    fn equality_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_symbol("sym").unwrap();
        let int2 = runtime.add_symbol("sym").unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_registers(), &vec![2]);
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

        assert_eq!(runtime.get_registers(), &vec![1]);
    }
}

#[cfg(test)]
mod expression {
    use crate::{runtime::GarnishRuntime, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

    #[test]
    fn equality_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_expression(10).unwrap();
        let int2 = runtime.add_expression(10).unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_registers(), &vec![2]);
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

        assert_eq!(runtime.get_registers(), &vec![1]);
    }
}

#[cfg(test)]
mod external {
    use crate::{runtime::GarnishRuntime, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

    #[test]
    fn equality_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_external(10).unwrap();
        let int2 = runtime.add_external(10).unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_registers(), &vec![2]);
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

        assert_eq!(runtime.get_registers(), &vec![1]);
    }
}

#[cfg(test)]
mod pairs {
    use crate::{runtime::GarnishRuntime, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

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

        assert_eq!(runtime.get_registers(), &vec![2]);
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

        assert_eq!(runtime.get_registers(), &vec![1]);
    }
}

#[cfg(test)]
mod lists {
    use crate::{runtime::GarnishRuntime, GarnishLangRuntimeData, Instruction, SimpleRuntimeData};

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

        assert_eq!(runtime.get_registers(), &vec![2]);
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

        assert_eq!(runtime.get_registers(), &vec![1]);
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

        assert_eq!(runtime.get_registers(), &vec![2]);
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

        assert_eq!(runtime.get_registers(), &vec![1]);
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

        assert_eq!(runtime.get_registers(), &vec![2]);
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

        assert_eq!(runtime.get_registers(), &vec![1]);
    }
}
