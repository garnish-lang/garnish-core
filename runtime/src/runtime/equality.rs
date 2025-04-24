use std::fmt::Debug;

use log::trace;

use crate::runtime::error::OrNumberError;
use crate::runtime::error::state_error;
use crate::runtime::internals::concatenation_len;
use crate::runtime::list::index_concatenation_for;
use crate::runtime::utilities::{next_two_raw_ref, push_boolean};
use garnish_lang_traits::{GarnishData, GarnishDataType, GarnishNumber, RuntimeError, TypeConstants};

pub(crate) fn equal<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let equal = perform_equality_check(this)?;
    push_boolean(this, equal)?;

    Ok(None)
}

pub fn not_equal<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let equal = perform_equality_check(this)?;
    push_boolean(this, !equal)?;

    Ok(None)
}

pub fn type_equal<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let (right, left) = next_two_raw_ref(this)?;
    let left_type = this.get_data_type(left)?;
    let right_type = this.get_data_type(right.clone())?;

    // if right is a Garnish Type value, then comparison is against that value
    let right_type = if right_type == GarnishDataType::Type {
        this.get_type(right)?
    } else {
        right_type
    };

    let equal = left_type == right_type;
    push_boolean(this, equal)?;

    Ok(None)
}

fn perform_equality_check<Data: GarnishData>(this: &mut Data) -> Result<bool, RuntimeError<Data::Error>> {
    // hope that can get reduced to a constant
    let two = Data::Size::one() + Data::Size::one();
    if this.get_register_len() < two {
        state_error("Not enough registers to perform comparison.".to_string())?;
    }

    let start = this.get_register_len() - two;

    while this.get_register_len() > start {
        let (right, left) = next_two_raw_ref(this)?;
        if !data_equal(this, left, right)? {
            // ending early need to remove any remaining values from registers
            while this.get_register_len() > start {
                this.pop_register()?;
            }

            return Ok(false);
        }
    }

    Ok(true)
}

fn data_equal<Data: GarnishData>(this: &mut Data, left_addr: Data::Size, right_addr: Data::Size) -> Result<bool, RuntimeError<Data::Error>> {
    let (left_type, right_type) = (this.get_data_type(left_addr.clone())?, this.get_data_type(right_addr.clone())?);

    let equal = match (left_type, right_type) {
        (GarnishDataType::Unit, GarnishDataType::Unit)
        | (GarnishDataType::True, GarnishDataType::True)
        | (GarnishDataType::False, GarnishDataType::False) => true,
        (GarnishDataType::Type, GarnishDataType::Type) => this.get_type(left_addr)? == this.get_type(right_addr)?,
        (GarnishDataType::Expression, GarnishDataType::Expression) => compare(this, left_addr, right_addr, Data::get_expression)?,
        (GarnishDataType::External, GarnishDataType::External) => compare(this, left_addr, right_addr, Data::get_external)?,
        (GarnishDataType::Symbol, GarnishDataType::Symbol) => compare(this, left_addr, right_addr, Data::get_symbol)?,
        (GarnishDataType::Char, GarnishDataType::Char) => compare(this, left_addr, right_addr, Data::get_char)?,
        (GarnishDataType::Byte, GarnishDataType::Byte) => compare(this, left_addr, right_addr, Data::get_byte)?,
        (GarnishDataType::Number, GarnishDataType::Number) => compare(this, left_addr, right_addr, Data::get_number)?,
        (GarnishDataType::Char, GarnishDataType::CharList) => {
            if this.get_char_list_len(right_addr.clone())? == Data::Size::one() {
                let c1 = this.get_char(left_addr)?;
                let c2 = this.get_char_list_item(right_addr, Data::Number::zero())?;

                c1 == c2
            } else {
                false
            }
        }
        (GarnishDataType::CharList, GarnishDataType::Char) => {
            if this.get_char_list_len(left_addr.clone())? == Data::Size::one() {
                let c1 = this.get_char_list_item(left_addr, Data::Number::zero())?;
                let c2 = this.get_char(right_addr)?;

                c1 == c2
            } else {
                false
            }
        }
        (GarnishDataType::Byte, GarnishDataType::ByteList) => {
            if this.get_byte_list_len(right_addr.clone())? == Data::Size::one() {
                let c1 = this.get_byte(left_addr)?;
                let c2 = this.get_byte_list_item(right_addr, Data::Number::zero())?;

                c1 == c2
            } else {
                false
            }
        }
        (GarnishDataType::ByteList, GarnishDataType::Byte) => {
            if this.get_byte_list_len(left_addr.clone())? == Data::Size::one() {
                let c1 = this.get_byte_list_item(left_addr, Data::Number::zero())?;
                let c2 = this.get_byte(right_addr)?;

                c1 == c2
            } else {
                false
            }
        }
        (GarnishDataType::CharList, GarnishDataType::CharList) => {
            let len1 = this.get_char_list_len(left_addr.clone())?;
            let len2 = this.get_char_list_len(right_addr.clone())?;

            if len1 != len2 {
                false
            } else {
                let mut count = Data::Size::one();
                let mut equal = true;
                while count < len1 {
                    let i = Data::size_to_number(count.clone());
                    let c1 = this.get_char_list_item(left_addr.clone(), i.clone())?;
                    let c2 = this.get_char_list_item(right_addr.clone(), i)?;

                    if c1 != c2 {
                        equal = false;
                    }

                    count += Data::Size::one();
                }

                equal
            }
        }
        (GarnishDataType::ByteList, GarnishDataType::ByteList) => {
            let len1 = this.get_byte_list_len(left_addr.clone())?;
            let len2 = this.get_byte_list_len(right_addr.clone())?;

            if len1 != len2 {
                false
            } else {
                let mut count = Data::Size::one();
                let mut equal = true;
                while count < len1 {
                    let i = Data::size_to_number(count.clone());
                    let c1 = this.get_byte_list_item(left_addr.clone(), i.clone())?;
                    let c2 = this.get_byte_list_item(right_addr.clone(), i)?;

                    if c1 != c2 {
                        equal = false;
                    }

                    count += Data::Size::one();
                }

                equal
            }
        }
        (GarnishDataType::SymbolList, GarnishDataType::SymbolList) => compare_iterator_values(
            this,
            this.get_symbol_list_iter(left_addr.clone()),
            this.get_symbol_list_iter(right_addr.clone()),
            left_addr,
            right_addr,
            Data::get_symbol_list_item,
        )?,
        (GarnishDataType::Range, GarnishDataType::Range) => {
            let (start1, end1) = this.get_range(left_addr)?;
            let (start2, end2) = this.get_range(right_addr)?;

            let start_equal = match (this.get_data_type(start1.clone())?, this.get_data_type(start2.clone())?) {
                (GarnishDataType::Unit, GarnishDataType::Unit) => true,
                (GarnishDataType::Number, GarnishDataType::Number) => this.get_number(start1)? == this.get_number(start2)?,
                _ => false,
            };

            let end_equal = match (this.get_data_type(end1.clone())?, this.get_data_type(end2.clone())?) {
                (GarnishDataType::Unit, GarnishDataType::Unit) => true,
                (GarnishDataType::Number, GarnishDataType::Number) => this.get_number(end1)? == this.get_number(end2)?,
                _ => false,
            };

            start_equal && end_equal
        }
        (GarnishDataType::Pair, GarnishDataType::Pair) => {
            let (left1, right1) = this.get_pair(left_addr)?;
            let (left2, right2) = this.get_pair(right_addr)?;

            this.push_register(left1)?;
            this.push_register(left2)?;

            this.push_register(right1)?;
            this.push_register(right2)?;

            true
        }
        (GarnishDataType::Concatenation, GarnishDataType::Concatenation) => {
            compare_item_iterators(this, left_addr, right_addr, Data::get_concatenation_iter)?
        }
        (GarnishDataType::List, GarnishDataType::Concatenation) => {
            let len1 = this.get_list_len(left_addr.clone())?;
            let len2 = concatenation_len(this, right_addr.clone())?;

            if len1 != len2 {
                false
            } else {
                let mut count = Data::Number::zero();
                let len = Data::size_to_number(len1);

                while count < len {
                    // need to find faster way
                    // current way is to index each concatenation one item at a time
                    match (
                        this.get_list_item(left_addr.clone(), count.clone())?,
                        index_concatenation_for(this, right_addr.clone(), count.clone())?,
                    ) {
                        (left, Some(right)) => {
                            this.push_register(left)?;
                            this.push_register(right)?;
                        }
                        _ => {
                            return Ok(false);
                        }
                    }
                    count = count.increment().or_num_err()?;
                }

                true
            }
        }
        (GarnishDataType::Concatenation, GarnishDataType::List) => {
            let len1 = concatenation_len(this, left_addr.clone())?;
            let len2 = this.get_list_len(right_addr.clone())?;

            if len1 != len2 {
                false
            } else {
                let mut count = Data::Number::zero();
                let len = Data::size_to_number(len1);

                while count < len {
                    // need to find faster way
                    // current way is to index each concatenation one item at a time
                    match (
                        index_concatenation_for(this, left_addr.clone(), count.clone())?,
                        this.get_list_item(right_addr.clone(), count.clone())?,
                    ) {
                        (Some(left), right) => {
                            this.push_register(left)?;
                            this.push_register(right)?;
                        }
                        _ => {
                            return Ok(false);
                        }
                    }
                    count = count.increment().or_num_err()?;
                }

                true
            }
        }
        (GarnishDataType::Slice, GarnishDataType::Slice) => {
            let (value1, _) = this.get_slice(left_addr.clone())?;
            let (value2, _) = this.get_slice(right_addr.clone())?;

            let (mut slice_iter1, mut slice_iter2) = (this.get_slice_iter(left_addr.clone()), this.get_slice_iter(right_addr.clone()));

            match (this.get_data_type(value1.clone())?, this.get_data_type(value2.clone())?) {
                (GarnishDataType::CharList, GarnishDataType::CharList) => {
                    compare_iterator_values(this, slice_iter1, slice_iter2, value1, value2, Data::get_char_list_item)?
                }
                (GarnishDataType::ByteList, GarnishDataType::ByteList) => {
                    compare_iterator_values(this, slice_iter1, slice_iter2, value1, value2, Data::get_byte_list_item)?
                }
                (GarnishDataType::SymbolList, GarnishDataType::SymbolList) => {
                    compare_iterator_values(this, slice_iter1, slice_iter2, value1, value2, Data::get_symbol_list_item)?
                }
                (GarnishDataType::List, GarnishDataType::List) => {
                    while let (Some(index1), Some(index2)) = (slice_iter1.next(), slice_iter2.next()) {
                        let (item1, item2) = (this.get_list_item(value1.clone(), index1)?, this.get_list_item(value2.clone(), index2)?);
                        this.push_register(item1)?;
                        this.push_register(item2)?;
                    }

                    check_last_iter_values::<Data, Data::ListIndexIterator>(slice_iter1, slice_iter2)?
                }
                _ => false,
            }
        }
        (GarnishDataType::List, GarnishDataType::List) => {
            compare_item_iterators(this, left_addr, right_addr, Data::get_list_item_iter)?
        }
        _ => false,
    };

    Ok(equal)
}

fn compare_iterator_values<Data: GarnishData, R, GetFn>(
    this: &mut Data,
    iter1: Data::ListIndexIterator,
    iter2: Data::ListIndexIterator,
    list_index_1: Data::Size,
    list_index_2: Data::Size,
    get_fn: GetFn,
) -> Result<bool, RuntimeError<Data::Error>>
where
    R: PartialEq,
    GetFn: Fn(&Data, Data::Size, Data::Number) -> Result<R, Data::Error>,
{
    let mut iter1 = iter1;
    let mut iter2 = iter2;

    while let (Some(index1), Some(index2)) = (iter1.next(), iter2.next()) {
        let (item1, item2) = (get_fn(this, list_index_1.clone(), index1)?, get_fn(this, list_index_2.clone(), index2)?);
        if item1 != item2 {
            return Ok(false);
        }
    }

    check_last_iter_values::<Data, Data::ListIndexIterator>(iter1, iter2)
}

fn compare_item_iterators<Data: GarnishData, Iter, GetFn>(
    this: &mut Data,
    left_addr: Data::Size,
    right_addr: Data::Size,
    get_iter: GetFn,
) -> Result<bool, RuntimeError<Data::Error>>
where
    Iter: Iterator<Item = Data::Size>,
    GetFn: Fn(&Data, Data::Size) -> Iter,
{
    let (mut iter1, mut iter2) = (get_iter(this, left_addr.clone()), get_iter(this, right_addr.clone()));

    while let (Some(index1), Some(index2)) = (iter1.next(), iter2.next()) {
        this.push_register(index1)?;
        this.push_register(index2)?;
    }

    check_last_iter_values::<Data, Iter>(iter1, iter2)
}

fn check_last_iter_values<Data: GarnishData, Iter: Iterator>(mut iter1: Iter, mut iter2: Iter) -> Result<bool, RuntimeError<Data::Error>> {
    Ok(match (iter1.next(), iter2.next()) {
        (Some(_), Some(_)) => state_error("Both slice operand's have remaining values in iterators after comparison".to_string())?,
        (None, Some(_)) | (Some(_), None) => false, // one operand as more items, automatically not equal
        (None, None) => true,
    })
}

fn compare<Data: GarnishData, F, V: PartialOrd + Debug>(
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
    use crate::runtime::equality::equal;
    use crate::runtime::tests::{MockError, MockGarnishData, MockIterator};
    use garnish_lang_traits::GarnishDataType;

    struct ListCompData {
        pub types: Vec<GarnishDataType>,
        pub registers: Vec<i32>,
        pub lens: Vec<i32>,
        pub items: Vec<Vec<u32>>,
    }

    impl ListCompData {
        fn get_symbol_list_item(&self, list: i32, index: i32) -> Result<u32, MockError> {
            let item = self.items.get(list as usize).unwrap().get(index as usize).unwrap().clone();
            Ok(item)
        }

        fn get_character_list_item(&self, list: i32, index: i32) -> Result<char, MockError> {
            let item = self.items.get(list as usize).unwrap().get(index as usize).unwrap().clone();
            Ok(char::from_u32(item).unwrap())
        }

        fn get_byte_list_item(&self, list: i32, index: i32) -> Result<u8, MockError> {
            let item = self.items.get(list as usize).unwrap().get(index as usize).unwrap().clone();
            Ok(item as u8)
        }

        fn get_list_item(&self, list: i32, index: i32) -> Result<i32, MockError> {
            let item = self.items.get(list as usize).unwrap().get(index as usize).unwrap().clone();
            Ok(item as i32)
        }
    }

    impl Default for ListCompData {
        fn default() -> Self {
            Self {
                types: vec![],
                registers: vec![],
                lens: vec![],
                items: vec![],
            }
        }
    }

    #[test]
    fn symbol_list_equal_to_symbol_list() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![GarnishDataType::SymbolList, GarnishDataType::SymbolList],
            registers: vec![0, 1],
            lens: vec![2, 2],
            items: vec![vec![10, 20], vec![10, 20]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_symbol_list_iter = |data, i| MockIterator::new(data.lens.get(i as usize).unwrap().clone());
        data.stub_get_symbol_list_len = |data, i| Ok(data.lens.get(i as usize).unwrap().clone());
        data.stub_start_list = |_, _| Ok(());
        data.stub_get_symbol_list_item = ListCompData::get_symbol_list_item;

        data.stub_add_true = |_| Ok(999);
        data.stub_push_register = |data, i| {
            data.registers.push(i);
            Ok(())
        };

        let result = equal(&mut data);

        assert_eq!(data.data.registers, vec![999]);
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn symbol_list_items_not_equal_to_symbol_list() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![GarnishDataType::SymbolList, GarnishDataType::SymbolList],
            registers: vec![0, 1],
            lens: vec![2, 2],
            items: vec![vec![10, 30], vec![10, 20]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_symbol_list_iter = |data, i| MockIterator::new(data.lens.get(i as usize).unwrap().clone());
        data.stub_get_symbol_list_len = |data, i| Ok(data.lens.get(i as usize).unwrap().clone());
        data.stub_start_list = |_, _| Ok(());
        data.stub_get_symbol_list_item = ListCompData::get_symbol_list_item;

        data.stub_add_false = |_| Ok(999);
        data.stub_push_register = |data, i| {
            data.registers.push(i);
            Ok(())
        };

        let result = equal(&mut data);

        assert_eq!(data.data.registers, vec![999]);
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn symbol_list_not_equal_to_symbol_list() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![GarnishDataType::SymbolList, GarnishDataType::SymbolList],
            registers: vec![0, 1],
            lens: vec![2, 4],
            items: vec![vec![10, 20], vec![10, 20]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_symbol_list_iter = |data, i| MockIterator::new(data.lens.get(i as usize).unwrap().clone());
        data.stub_get_symbol_list_item = ListCompData::get_symbol_list_item;

        data.stub_add_false = |_| Ok(999);
        data.stub_push_register = |data, i| {
            data.registers.push(i);
            Ok(())
        };

        let result = equal(&mut data);

        assert_eq!(data.data.registers, vec![999]);
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn slice_of_char_list_equals_slice_of_char_list() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![
                GarnishDataType::CharList,
                GarnishDataType::CharList,
                GarnishDataType::Range,
                GarnishDataType::Range,
                GarnishDataType::Slice,
                GarnishDataType::Slice,
            ],
            registers: vec![4, 5],
            lens: vec![2, 2],
            items: vec![vec![10, 20], vec![10, 20]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_slice = |_, addr| {
            if addr == 4 {
                Ok((0, 2))
            } else if addr == 5 {
                Ok((1, 3))
            } else {
                assert!(false);
                Err(MockError {})
            }
        };
        data.stub_get_slice_iter = |_, _| MockIterator::new(2);
        data.stub_get_char_list_item = ListCompData::get_character_list_item;

        data.stub_add_true = |_| Ok(999);
        data.stub_push_register = |data, i| {
            data.registers.push(i);
            Ok(())
        };

        let result = equal(&mut data);

        assert_eq!(data.data.registers, vec![999]);
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn slice_of_byte_list_equals_slice_of_byte_list() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![
                GarnishDataType::ByteList,
                GarnishDataType::ByteList,
                GarnishDataType::Range,
                GarnishDataType::Range,
                GarnishDataType::Slice,
                GarnishDataType::Slice,
            ],
            registers: vec![4, 5],
            lens: vec![2, 2],
            items: vec![vec![10, 20], vec![10, 20]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_slice = |_, addr| {
            if addr == 4 {
                Ok((0, 2))
            } else if addr == 5 {
                Ok((1, 3))
            } else {
                assert!(false);
                Err(MockError {})
            }
        };
        data.stub_get_slice_iter = |_, _| MockIterator::new(2);
        data.stub_get_byte_list_item = ListCompData::get_byte_list_item;

        data.stub_add_true = |_| Ok(999);
        data.stub_push_register = |data, i| {
            data.registers.push(i);
            Ok(())
        };

        let result = equal(&mut data);

        assert_eq!(data.data.registers, vec![999]);
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn slice_of_list_equals_slice_of_list() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![
                GarnishDataType::List,
                GarnishDataType::List,
                GarnishDataType::Range,
                GarnishDataType::Range,
                GarnishDataType::Slice,
                GarnishDataType::Slice,
                GarnishDataType::Number,
            ],
            registers: vec![4, 5],
            lens: vec![2, 2],
            items: vec![vec![6, 6], vec![6, 6]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_slice = |_, addr| {
            if addr == 4 {
                Ok((0, 2))
            } else if addr == 5 {
                Ok((1, 3))
            } else {
                assert!(false);
                Err(MockError {})
            }
        };
        data.stub_get_slice_iter = |_, _| MockIterator::new(2);
        data.stub_get_list_item = ListCompData::get_list_item;
        data.stub_get_number = |_, _| Ok(10);

        data.stub_add_true = |_| Ok(999);
        data.stub_push_register = |data, i| {
            data.registers.push(i);
            Ok(())
        };

        let result = equal(&mut data);

        assert_eq!(data.data.registers, vec![999]);
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn slice_of_symbol_list_equals_slice_of_symbol_list() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![
                GarnishDataType::SymbolList,
                GarnishDataType::SymbolList,
                GarnishDataType::Range,
                GarnishDataType::Range,
                GarnishDataType::Slice,
                GarnishDataType::Slice,
            ],
            registers: vec![4, 5],
            lens: vec![2, 2],
            items: vec![vec![10, 20], vec![10, 20]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_slice = |_, addr| {
            if addr == 4 {
                Ok((0, 2))
            } else if addr == 5 {
                Ok((1, 3))
            } else {
                assert!(false);
                Err(MockError {})
            }
        };
        data.stub_get_slice_iter = |_, _| MockIterator::new(2);
        data.stub_get_symbol_list_item = ListCompData::get_symbol_list_item;

        data.stub_add_true = |_| Ok(999);
        data.stub_push_register = |data, i| {
            data.registers.push(i);
            Ok(())
        };

        let result = equal(&mut data);

        assert_eq!(data.data.registers, vec![999]);
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn concatenation_equals_concatenation() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![
                GarnishDataType::Concatenation,
                GarnishDataType::Concatenation,
                GarnishDataType::Number,
                GarnishDataType::Number,
            ],
            registers: vec![0, 1],
            lens: vec![],
            items: vec![],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_concatenation_iter = |_, _| MockIterator::new_range(2, 4);
        data.stub_get_number = |_, _| Ok(10);

        data.stub_add_true = |_| Ok(999);
        data.stub_push_register = |data, i| {
            data.registers.push(i);
            Ok(())
        };

        let result = equal(&mut data);

        assert_eq!(data.data.registers, vec![999]);
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn list_equals_list() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![
                GarnishDataType::List,
                GarnishDataType::List,
                GarnishDataType::Number,
                GarnishDataType::Number,
            ],
            registers: vec![0, 1],
            lens: vec![],
            items: vec![],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_list_item_iter = |_, _| MockIterator::new_range(2, 4);
        data.stub_get_number = |_, _| Ok(10);

        data.stub_add_true = |_| Ok(999);
        data.stub_push_register = |data, i| {
            data.registers.push(i);
            Ok(())
        };

        let result = equal(&mut data);

        assert_eq!(data.data.registers, vec![999]);
        assert_eq!(result.unwrap(), None);
    }
}
