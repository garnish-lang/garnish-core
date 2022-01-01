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

            if association_len1 > Data::Size::zero() && association_len1 == associations_len2 {
                // comparing associations can be done similar to regular items since,
                // if they are the equal, they should be associated in the same order as well
                push_list_items(this, association_len1, left_addr, right_addr, lease, Data::get_list_association)?;
            } else {
                let len1 = this.get_list_len(left_addr)?;
                let len2 = this.get_list_len(right_addr)?;

                if len1 != len2 {
                    return Ok(false);
                }

                push_list_items(this, len1, left_addr, right_addr, lease, Data::get_list_item)?;
            }

            true
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

fn push_list_items<Data: GarnishLangRuntimeData, F>(
    this: &mut Data,
    len: Data::Size,
    left_addr: Data::Size,
    right_addr: Data::Size,
    lease: Data::DataLease,
    get_item_func: F,
) -> Result<(), Data::Error>
where
    F: Fn(&Data, Data::Size, Data::Integer) -> Result<Data::Size, Data::Error>,
{
    let mut count = Data::Size::zero();
    while count < len {
        let i = Data::size_to_integer(count);
        let item1 = get_item_func(this, left_addr, i)?;
        let item2 = get_item_func(this, right_addr, i)?;

        this.push_tmp_stack(lease, item1)?;
        this.push_tmp_stack(lease, item2)?;

        count += Data::Size::one();
    }

    Ok(())
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

        assert_eq!(runtime.get_register(), &vec![1]);
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

        assert_eq!(runtime.get_register(), &vec![2]);
    }

    #[test]
    fn equality_true_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_true().unwrap();
        let int2 = runtime.add_true().unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_register(), &vec![2]);
    }

    #[test]
    fn equality_false_equal() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_false().unwrap();
        let int2 = runtime.add_false().unwrap();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.equality_comparison().unwrap();

        assert_eq!(runtime.get_register(), &vec![2]);
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

        assert_eq!(runtime.get_register(), &vec![2]);
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

        assert_eq!(runtime.get_register(), &vec![1]);
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

        assert_eq!(runtime.get_register(), &vec![2]);
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

        assert_eq!(runtime.get_register(), &vec![1]);
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

        assert_eq!(runtime.get_register(), &vec![2]);
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

        assert_eq!(runtime.get_register(), &vec![1]);
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

        assert_eq!(runtime.get_register(), &vec![2]);
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

        assert_eq!(runtime.get_register(), &vec![1]);
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

        assert_eq!(runtime.get_register(), &vec![2]);
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

        assert_eq!(runtime.get_register(), &vec![1]);
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

        assert_eq!(runtime.get_register(), &vec![2]);
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

        assert_eq!(runtime.get_register(), &vec![1]);
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

        assert_eq!(runtime.get_register(), &vec![2]);
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

        assert_eq!(runtime.get_register(), &vec![1]);
    }
}
