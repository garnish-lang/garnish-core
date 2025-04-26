use std::fmt::Debug;

use log::trace;

use crate::runtime::error::state_error;
use crate::runtime::utilities::{next_two_raw_ref, push_boolean};
use garnish_lang_traits::{GarnishData, GarnishDataType, RuntimeError, TypeConstants};

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
        (GarnishDataType::Char, GarnishDataType::CharList) => compare_list_to_primitive(
            this,
            right_addr,
            left_addr,
            Data::get_char_list_len,
            Data::get_char_list_item,
            Data::get_char,
        )?,
        (GarnishDataType::CharList, GarnishDataType::Char) => compare_list_to_primitive(
            this,
            left_addr,
            right_addr,
            Data::get_char_list_len,
            Data::get_char_list_item,
            Data::get_char,
        )?,
        (GarnishDataType::Byte, GarnishDataType::ByteList) => compare_list_to_primitive(
            this,
            right_addr,
            left_addr,
            Data::get_byte_list_len,
            Data::get_byte_list_item,
            Data::get_byte,
        )?,
        (GarnishDataType::ByteList, GarnishDataType::Byte) => compare_list_to_primitive(
            this,
            left_addr,
            right_addr,
            Data::get_byte_list_len,
            Data::get_byte_list_item,
            Data::get_byte,
        )?,
        (GarnishDataType::CharList, GarnishDataType::CharList) => compare_index_iterator_values(
            this,
            this.get_char_list_iter(left_addr.clone()),
            this.get_char_list_iter(right_addr.clone()),
            left_addr,
            right_addr,
            Data::get_char_list_item,
        )?,
        (GarnishDataType::ByteList, GarnishDataType::ByteList) => compare_index_iterator_values(
            this,
            this.get_byte_list_iter(left_addr.clone()),
            this.get_byte_list_iter(right_addr.clone()),
            left_addr,
            right_addr,
            Data::get_byte_list_item,
        )?,
        (GarnishDataType::SymbolList, GarnishDataType::SymbolList) => compare_index_iterator_values(
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
        (GarnishDataType::List, GarnishDataType::List) => compare_item_iterators(this, left_addr, right_addr, Data::get_list_item_iter)?,
        (GarnishDataType::List, GarnishDataType::Concatenation) => {
            compare_item_iterators_2(this, left_addr, right_addr, Data::get_list_item_iter, Data::get_concatenation_iter)?
        }
        (GarnishDataType::Concatenation, GarnishDataType::List) => {
            compare_item_iterators_2(this, left_addr, right_addr, Data::get_concatenation_iter, Data::get_list_item_iter)?
        }
        (GarnishDataType::Slice, GarnishDataType::SymbolList) => {
            compare_slice_index_iterator_values(
                this,
                right_addr.clone(),
                left_addr.clone(),
                GarnishDataType::SymbolList,
                Data::get_symbol_list_iter,
                Data::get_symbol_list_item
            )?
        }
        (GarnishDataType::SymbolList, GarnishDataType::Slice) => {
            compare_slice_index_iterator_values(
                this,
                left_addr.clone(),
                right_addr.clone(),
                GarnishDataType::SymbolList,
                Data::get_symbol_list_iter,
                Data::get_symbol_list_item
            )?
        }
        (GarnishDataType::Slice, GarnishDataType::Slice) => {
            let (value1, _) = this.get_slice(left_addr.clone())?;
            let (value2, _) = this.get_slice(right_addr.clone())?;

            match (this.get_data_type(value1.clone())?, this.get_data_type(value2.clone())?) {
                (GarnishDataType::CharList, GarnishDataType::CharList) => compare_index_iterator_values(
                    this,
                    this.get_slice_iter(left_addr.clone()),
                    this.get_slice_iter(right_addr.clone()),
                    value1,
                    value2,
                    Data::get_char_list_item,
                )?,
                (GarnishDataType::ByteList, GarnishDataType::ByteList) => compare_index_iterator_values(
                    this,
                    this.get_slice_iter(left_addr.clone()),
                    this.get_slice_iter(right_addr.clone()),
                    value1,
                    value2,
                    Data::get_byte_list_item,
                )?,
                (GarnishDataType::SymbolList, GarnishDataType::SymbolList) => compare_index_iterator_values(
                    this,
                    this.get_slice_iter(left_addr.clone()),
                    this.get_slice_iter(right_addr.clone()),
                    value1,
                    value2,
                    Data::get_symbol_list_item,
                )?,
                (GarnishDataType::List, GarnishDataType::List) => {
                    compare_item_iterators(this, left_addr, right_addr, Data::get_list_slice_item_iter)?
                }
                (GarnishDataType::List, GarnishDataType::Concatenation) => compare_item_iterators_2(
                    this,
                    left_addr,
                    right_addr,
                    Data::get_list_slice_item_iter,
                    Data::get_concatenation_slice_iter,
                )?,
                (GarnishDataType::Concatenation, GarnishDataType::List) => compare_item_iterators_2(
                    this,
                    left_addr,
                    right_addr,
                    Data::get_concatenation_slice_iter,
                    Data::get_list_slice_item_iter,
                )?,
                (GarnishDataType::Concatenation, GarnishDataType::Concatenation) => {
                    compare_item_iterators(this, left_addr, right_addr, Data::get_concatenation_slice_iter)?
                }
                _ => false,
            }
        }
        _ => false,
    };

    Ok(equal)
}

fn compare_list_to_primitive<Data: GarnishData, LenFn, ItemFn, GetFn, T>(
    this: &Data,
    list_addr: Data::Size,
    primitive_addr: Data::Size,
    len_fn: LenFn,
    item_fn: ItemFn,
    get_fn: GetFn,
) -> Result<bool, RuntimeError<Data::Error>>
where
    Data: GarnishData,
    T: PartialEq,
    LenFn: Fn(&Data, Data::Size) -> Result<Data::Size, Data::Error>,
    ItemFn: Fn(&Data, Data::Size, Data::Number) -> Result<T, Data::Error>,
    GetFn: Fn(&Data, Data::Size) -> Result<T, Data::Error>,
{
    Ok(if len_fn(this, list_addr.clone())? == Data::Size::one() {
        let c1 = item_fn(this, list_addr, Data::Number::zero())?;
        let c2 = get_fn(this, primitive_addr)?;

        c1 == c2
    } else {
        false
    })
}

fn compare_slice_index_iterator_values<Data, GetValueIterFn, GetValueItemFn, Value>(
    this: &Data,
    value_addr: Data::Size,
    slice_addr: Data::Size,
    expected_data_type: GarnishDataType,
    get_value_iter: GetValueIterFn,
    get_value_item: GetValueItemFn,
) -> Result<bool, RuntimeError<Data::Error>>
where
    Data: GarnishData,
    GetValueIterFn: Fn(&Data, Data::Size) -> Data::ListIndexIterator,
    GetValueItemFn: Fn(&Data, Data::Size, Data::Number) -> Result<Value, Data::Error>,
    Value: PartialEq
{
    let (value1, _) = this.get_slice(slice_addr.clone())?;
    if this.get_data_type(value1.clone())? == expected_data_type {
        compare_index_iterator_values(
            this,
            get_value_iter(this, value1.clone()),
            this.get_slice_iter(slice_addr.clone()),
            value_addr,
            value1,
            get_value_item,
        )
    } else {
        Ok(false)
    }
}

fn compare_index_iterator_values<Data: GarnishData, Value, GetFn>(
    this: &Data,
    mut iter1: Data::ListIndexIterator,
    mut iter2: Data::ListIndexIterator,
    list_index_1: Data::Size,
    list_index_2: Data::Size,
    get_fn: GetFn,
) -> Result<bool, RuntimeError<Data::Error>>
where
    Value: PartialEq,
    GetFn: Fn(&Data, Data::Size, Data::Number) -> Result<Value, Data::Error>,
{
    let mut index1: Option<Data::Number> = iter1.next();
    let mut index2: Option<Data::Number> = iter2.next();
    loop {
        match (index1.clone(), index2.clone()) {
            (Some(index1), Some(index2)) => {
                let (item1, item2) = (get_fn(this, list_index_1.clone(), index1)?, get_fn(this, list_index_2.clone(), index2)?);
                if item1 != item2 {
                    return Ok(false);
                }
            }
            _ => break,
        }
        index1 = iter1.next();
        index2 = iter2.next();
    }

    match_last_iter_values::<Data, Data::Number>(index1, index2)
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
    let (iter1, iter2) = (get_iter(this, left_addr.clone()), get_iter(this, right_addr.clone()));
    push_iterator_values(this, iter1, iter2)
}

fn compare_item_iterators_2<Data: GarnishData, Iter1, Iter2, GetFn1, GetFn2>(
    this: &mut Data,
    left_addr: Data::Size,
    right_addr: Data::Size,
    get_iter_left: GetFn1,
    get_iter_right: GetFn2,
) -> Result<bool, RuntimeError<Data::Error>>
where
    Iter1: Iterator<Item = Data::Size>,
    Iter2: Iterator<Item = Data::Size>,
    GetFn1: Fn(&Data, Data::Size) -> Iter1,
    GetFn2: Fn(&Data, Data::Size) -> Iter2,
{
    let (iter1, iter2) = (get_iter_left(this, left_addr.clone()), get_iter_right(this, right_addr.clone()));
    push_iterator_values(this, iter1, iter2)
}

fn push_iterator_values<Data, Iter1, Iter2>(this: &mut Data, mut iter1: Iter1, mut iter2: Iter2) -> Result<bool, RuntimeError<Data::Error>>
where
    Data: GarnishData,
    Iter1: Iterator<Item = Data::Size>,
    Iter2: Iterator<Item = Data::Size>,
{
    let mut index1: Option<Data::Size> = iter1.next();
    let mut index2: Option<Data::Size> = iter2.next();
    loop {
        match (index1.clone(), index2.clone()) {
            (Some(index1), Some(index2)) => {
                this.push_register(index1)?;
                this.push_register(index2)?;
            }
            _ => break,
        }
        index1 = iter1.next();
        index2 = iter2.next();
    }

    match_last_iter_values::<Data, Data::Size>(index1, index2)
}

fn match_last_iter_values<Data: GarnishData, T>(value_1: Option<T>, value_2: Option<T>) -> Result<bool, RuntimeError<Data::Error>> {
    Ok(match (value_1, value_2) {
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
    fn byte_list_equal_to_byte() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![GarnishDataType::ByteList, GarnishDataType::Byte],
            registers: vec![0, 1],
            lens: vec![1],
            items: vec![vec![10]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_byte_list_len = |_, _| Ok(1);
        data.stub_get_byte_list_item = ListCompData::get_byte_list_item;
        data.stub_get_byte = |_, _| Ok(10);

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
    fn byte_equal_to_byte_list() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![GarnishDataType::Byte, GarnishDataType::ByteList],
            registers: vec![0, 1],
            lens: vec![0, 1],
            items: vec![vec![], vec![10]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_byte_list_len = |_, _| Ok(1);
        data.stub_get_byte_list_item = ListCompData::get_byte_list_item;
        data.stub_get_byte = |_, _| Ok(10);

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
    fn char_list_equal_to_char() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![GarnishDataType::CharList, GarnishDataType::Char],
            registers: vec![0, 1],
            lens: vec![1],
            items: vec![vec!['a' as u32]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_char_list_len = |_, _| Ok(1);
        data.stub_get_char_list_item = ListCompData::get_character_list_item;
        data.stub_get_char = |_, _| Ok('a');

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
    fn char_equal_to_char_list() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![GarnishDataType::Char, GarnishDataType::CharList],
            registers: vec![0, 1],
            lens: vec![0, 1],
            items: vec![vec![], vec!['a' as u32]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_char_list_len = |_, _| Ok(1);
        data.stub_get_char_list_item = ListCompData::get_character_list_item;
        data.stub_get_char = |_, _| Ok('a');

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
    fn byte_list_equal_to_byte_list() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![GarnishDataType::ByteList, GarnishDataType::ByteList],
            registers: vec![0, 1],
            lens: vec![2, 2],
            items: vec![vec![10, 20], vec![10, 20]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_byte_list_iter = |data, i| MockIterator::new(data.lens.get(i as usize).unwrap().clone());
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
    fn byte_list_not_equal_to_larger_byte_list() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![GarnishDataType::ByteList, GarnishDataType::ByteList],
            registers: vec![0, 1],
            lens: vec![2, 3],
            items: vec![vec![10, 20], vec![10, 20]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_byte_list_iter = |data, i| MockIterator::new(data.lens.get(i as usize).unwrap().clone());
        data.stub_get_byte_list_item = ListCompData::get_byte_list_item;

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
    fn char_list_equal_to_char_list() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![GarnishDataType::CharList, GarnishDataType::CharList],
            registers: vec![0, 1],
            lens: vec![2, 2],
            items: vec![vec![10, 20], vec![10, 20]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_char_list_iter = |data, i| MockIterator::new(data.lens.get(i as usize).unwrap().clone());
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
                GarnishDataType::Range,
                GarnishDataType::Slice,
                GarnishDataType::Slice,
                GarnishDataType::Number,
            ],
            registers: vec![2, 3],
            lens: vec![2, 2],
            items: vec![vec![6, 6], vec![6, 6]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_slice = |_, addr| {
            if addr == 2 {
                Ok((0, 1))
            } else if addr == 3 {
                Ok((0, 1))
            } else {
                assert!(false);
                Err(MockError {})
            }
        };
        data.stub_get_list_slice_item_iter = |_, _| MockIterator::new_range(4, 5);
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
    fn slice_of_concatenation_equals_slice_of_concatenation() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![
                GarnishDataType::Concatenation,
                GarnishDataType::Range,
                GarnishDataType::Slice,
                GarnishDataType::Slice,
                GarnishDataType::Number,
            ],
            registers: vec![2, 3],
            lens: vec![2, 2],
            items: vec![vec![6, 6], vec![6, 6]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_slice = |_, addr| {
            if addr == 2 {
                Ok((0, 1))
            } else if addr == 3 {
                Ok((0, 1))
            } else {
                assert!(false);
                Err(MockError {})
            }
        };
        data.stub_get_concatenation_slice_iter = |_, _| MockIterator::new_range(4, 5);
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
    fn slice_of_concatenation_equals_slice_of_list() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![
                GarnishDataType::Concatenation,
                GarnishDataType::Range,
                GarnishDataType::Slice,
                GarnishDataType::Slice,
                GarnishDataType::Number,
                GarnishDataType::List,
            ],
            registers: vec![2, 3],
            lens: vec![2, 2],
            items: vec![vec![6, 6], vec![6, 6]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_slice = |_, addr| {
            if addr == 2 {
                Ok((0, 1))
            } else if addr == 3 {
                Ok((5, 1))
            } else {
                assert!(false);
                Err(MockError {})
            }
        };
        data.stub_get_list_slice_item_iter = |_, _| MockIterator::new_range(4, 5);
        data.stub_get_concatenation_slice_iter = |_, _| MockIterator::new_range(4, 5);
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
    fn slice_of_list_equals_slice_of_concatenation() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![
                GarnishDataType::Concatenation,
                GarnishDataType::Range,
                GarnishDataType::Slice,
                GarnishDataType::Slice,
                GarnishDataType::Number,
                GarnishDataType::List,
            ],
            registers: vec![2, 3],
            lens: vec![2, 2],
            items: vec![vec![6, 6], vec![6, 6]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_slice = |_, addr| {
            if addr == 2 {
                Ok((5, 1))
            } else if addr == 3 {
                Ok((0, 1))
            } else {
                assert!(false);
                Err(MockError {})
            }
        };
        data.stub_get_list_slice_item_iter = |_, _| MockIterator::new_range(4, 5);
        data.stub_get_concatenation_slice_iter = |_, _| MockIterator::new_range(4, 5);
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
    fn symbol_list_equals_slice_of_symbol_list() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![GarnishDataType::SymbolList, GarnishDataType::Slice, GarnishDataType::Range],
            registers: vec![0, 1],
            lens: vec![2, 2],
            items: vec![vec![10, 20], vec![10, 20]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_slice = |_, addr| {
            if addr == 1 {
                Ok((0, 2))
            } else {
                assert!(false);
                Err(MockError {})
            }
        };
        data.stub_get_slice_iter = |_, _| MockIterator::new(2);
        data.stub_get_symbol_list_iter = |_, _| MockIterator::new(2);
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
    fn slice_of_symbol_list_equals_symbol_list() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![GarnishDataType::SymbolList, GarnishDataType::Slice, GarnishDataType::Range],
            registers: vec![1, 0],
            lens: vec![2, 2],
            items: vec![vec![10, 20], vec![10, 20]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_slice = |_, addr| {
            if addr == 1 {
                Ok((0, 2))
            } else {
                assert!(false);
                Err(MockError {})
            }
        };
        data.stub_get_slice_iter = |_, _| MockIterator::new(2);
        data.stub_get_symbol_list_iter = |_, _| MockIterator::new(2);
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

    #[test]
    fn larger_list_does_not_equal_list() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![
                GarnishDataType::List,
                GarnishDataType::List,
                GarnishDataType::Number,
                GarnishDataType::Number,
            ],
            registers: vec![0, 1],
            lens: vec![2, 3],
            items: vec![],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_list_item_iter = |data, i| MockIterator::new_range(2, data.lens[i as usize]);
        data.stub_get_number = |_, _| Ok(10);

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
    fn list_equals_concatenation() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![
                GarnishDataType::List,
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
        data.stub_get_list_item_iter = |_, _| MockIterator::new_range(2, 4);
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
    fn list_not_equal_to_larger_concatenation() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![
                GarnishDataType::List,
                GarnishDataType::Concatenation,
                GarnishDataType::Number,
                GarnishDataType::Number,
            ],
            registers: vec![0, 1],
            lens: vec![2, 3],
            items: vec![],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_list_item_iter = |data, i| MockIterator::new_range(2, data.lens[i as usize]);
        data.stub_get_concatenation_iter = |data, i| MockIterator::new_range(2, data.lens[i as usize]);
        data.stub_get_number = |_, _| Ok(10);

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
    fn concatenation_equals_list() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![
                GarnishDataType::Concatenation,
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
}
