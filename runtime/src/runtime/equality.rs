use std::fmt::Debug;

use log::trace;

use crate::runtime::error::state_error;
use crate::runtime::utilities::{get_range, next_two_raw_ref, push_boolean};
use garnish_lang_traits::{Extents, GarnishData, GarnishDataType, RuntimeError, TypeConstants};

pub fn equal<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
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
            this.get_char_list_iter(left_addr.clone(), Extents::new(Data::Number::zero(), Data::Number::max_value()))?,
            this.get_char_list_iter(right_addr.clone(), Extents::new(Data::Number::zero(), Data::Number::max_value()))?,
        )?,
        (GarnishDataType::ByteList, GarnishDataType::ByteList) => compare_index_iterator_values(
            this,
            this.get_byte_list_iter(left_addr.clone(), Extents::new(Data::Number::zero(), Data::Number::max_value()))?,
            this.get_byte_list_iter(right_addr.clone(), Extents::new(Data::Number::zero(), Data::Number::max_value()))?,
        )?,
        (GarnishDataType::SymbolList, GarnishDataType::SymbolList) => compare_index_iterator_values(
            this,
            this.get_symbol_list_iter(left_addr.clone(), Extents::new(Data::Number::zero(), Data::Number::max_value()))?,
            this.get_symbol_list_iter(right_addr.clone(), Extents::new(Data::Number::zero(), Data::Number::max_value()))?,
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
            compare_item_iterators_2(this, left_addr, right_addr, Data::get_list_item_iter, Data::get_concatenation_iter, Extents::new(Data::Number::zero(), Data::Number::max_value()), Extents::new(Data::Number::zero(), Data::Number::max_value()))?
        }
        (GarnishDataType::Concatenation, GarnishDataType::List) => {
            compare_item_iterators_2(this, left_addr, right_addr, Data::get_concatenation_iter, Data::get_list_item_iter, Extents::new(Data::Number::zero(), Data::Number::max_value()), Extents::new(Data::Number::zero(), Data::Number::max_value()))?
        }
        (GarnishDataType::Slice, GarnishDataType::CharList) => compare_slice_index_iterator_values(
            this,
            right_addr.clone(),
            left_addr.clone(),
            GarnishDataType::CharList,
            Data::get_char_list_iter,
        )?,
        (GarnishDataType::CharList, GarnishDataType::Slice) => compare_slice_index_iterator_values(
            this,
            left_addr.clone(),
            right_addr.clone(),
            GarnishDataType::CharList,
            Data::get_char_list_iter,
        )?,
        (GarnishDataType::Slice, GarnishDataType::ByteList) => compare_slice_index_iterator_values(
            this,
            right_addr.clone(),
            left_addr.clone(),
            GarnishDataType::ByteList,
            Data::get_byte_list_iter,
        )?,
        (GarnishDataType::ByteList, GarnishDataType::Slice) => compare_slice_index_iterator_values(
            this,
            left_addr.clone(),
            right_addr.clone(),
            GarnishDataType::ByteList,
            Data::get_byte_list_iter,
        )?,
        (GarnishDataType::Slice, GarnishDataType::SymbolList) => compare_slice_index_iterator_values(
            this,
            right_addr.clone(),
            left_addr.clone(),
            GarnishDataType::SymbolList,
            Data::get_symbol_list_iter,
        )?,
        (GarnishDataType::SymbolList, GarnishDataType::Slice) => compare_slice_index_iterator_values(
            this,
            left_addr.clone(),
            right_addr.clone(),
            GarnishDataType::SymbolList,
            Data::get_symbol_list_iter,
        )?,
        (GarnishDataType::List, GarnishDataType::Slice) => {
            compare_slice_item_iterators_2(this, right_addr.clone(), left_addr.clone(), Data::get_list_item_iter)?
        }
        (GarnishDataType::Slice, GarnishDataType::List) => {
            compare_slice_item_iterators_2(this, left_addr.clone(), right_addr.clone(), Data::get_list_item_iter)?
        }
        (GarnishDataType::Concatenation, GarnishDataType::Slice) => {
            compare_slice_item_iterators_2(this, right_addr.clone(), left_addr.clone(), Data::get_concatenation_iter)?
        }
        (GarnishDataType::Slice, GarnishDataType::Concatenation) => {
            compare_slice_item_iterators_2(this, left_addr.clone(), right_addr.clone(), Data::get_concatenation_iter)?
        }
        (GarnishDataType::Slice, GarnishDataType::Slice) => {
            let (value1, range1) = this.get_slice(left_addr.clone())?;
            let (value2, range2) = this.get_slice(right_addr.clone())?;

            let (start1, end1, _) = get_range(this, range1)?;
            let (start2, end2, _) = get_range(this, range2)?;

            let extents1 = Extents::new(start1, end1);
            let extents2 = Extents::new(start2, end2);

            match (this.get_data_type(value1.clone())?, this.get_data_type(value2.clone())?) {
                (GarnishDataType::CharList, GarnishDataType::CharList) => compare_index_iterator_values(
                    this,
                    this.get_char_list_iter(value1.clone(), extents1)?,
                    this.get_char_list_iter(value2.clone(), extents2)?,
                )?,
                (GarnishDataType::ByteList, GarnishDataType::ByteList) => compare_index_iterator_values(
                    this,
                    this.get_byte_list_iter(value1.clone(), extents1)?,
                    this.get_byte_list_iter(value2.clone(), extents2)?,
                )?,
                (GarnishDataType::SymbolList, GarnishDataType::SymbolList) => compare_index_iterator_values(
                    this,
                    this.get_symbol_list_iter(value1.clone(), extents1)?,
                    this.get_symbol_list_iter(value2.clone(), extents2)?,
                )?,
                (GarnishDataType::List, GarnishDataType::List) => compare_item_iterators_2(
                    this,
                    value1,
                    value2,
                    Data::get_list_item_iter,
                    Data::get_list_item_iter,
                    extents1,
                    extents2,
                )?,
                (GarnishDataType::List, GarnishDataType::Concatenation) => compare_item_iterators_2(
                    this,
                    value1,
                    value2,
                    Data::get_list_item_iter,
                    Data::get_concatenation_iter,
                    extents1,
                    extents2,
                )?,
                (GarnishDataType::Concatenation, GarnishDataType::List) => compare_item_iterators_2(
                    this,
                    value1,
                    value2,
                    Data::get_concatenation_iter,
                    Data::get_list_item_iter,
                    extents1,
                    extents2,
                )?,
                (GarnishDataType::Concatenation, GarnishDataType::Concatenation) => compare_item_iterators_2(
                    this,
                    value1,
                    value2,
                    Data::get_concatenation_iter,
                    Data::get_concatenation_iter,
                    extents1,
                    extents2,
                )?,
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
    ItemFn: Fn(&Data, Data::Size, Data::Number) -> Result<Option<T>, Data::Error>,
    GetFn: Fn(&Data, Data::Size) -> Result<T, Data::Error>,
{
    Ok(if len_fn(this, list_addr.clone())? == Data::Size::one() {
        let c1 = match item_fn(this, list_addr, Data::Number::zero())? {
            Some(c) => c,
            None => return Ok(false),
        };
        let c2 = get_fn(this, primitive_addr)?;

        c1 == c2
    } else {
        false
    })
}

fn compare_slice_index_iterator_values<Data, Iter, GetValueIterFn, Value>(
    this: &Data,
    value_addr: Data::Size,
    slice_addr: Data::Size,
    expected_data_type: GarnishDataType,
    get_value_iter: GetValueIterFn,
) -> Result<bool, RuntimeError<Data::Error>>
where
    Data: GarnishData,
    Iter: Iterator<Item = Value>,
    GetValueIterFn: Fn(&Data, Data::Size, Extents<Data::Number>) -> Result<Iter, Data::Error>,
    Value: PartialEq + Clone,
{
    let (slice_value, slice_range) = this.get_slice(slice_addr.clone())?;
    let (start, end, _) = get_range(this, slice_range)?;
    if this.get_data_type(slice_value.clone())? == expected_data_type {
        let mut iter1 = get_value_iter(this, value_addr.clone(), Extents::new(Data::Number::zero(), Data::Number::max_value()))?;
        let mut iter2 = get_value_iter(this, slice_addr.clone(), Extents::new(start, end))?;
        
        let mut index1 = iter1.next();
        let mut index2 = iter2.next();

        loop {
            match (index1.clone(), index2.clone()) {
                (Some(value1), Some(value2)) => {
                    if value1 != value2 {
                        return Ok(false);
                    }
                }
                _ => break,
            }
            index1 = iter1.next();
            index2 = iter2.next();
        }

        match_last_iter_values::<Data, Value, Value>(index1, index2)
    } else {
        Ok(false)
    }
}

fn compare_index_iterator_values<Data: GarnishData, Value, Iter>(
    _this: &Data,
    mut iter1: Iter,
    mut iter2: Iter,
) -> Result<bool, RuntimeError<Data::Error>>
where
    Value: PartialEq + Clone,
    Iter: Iterator<Item = Value>,
{
    let mut index1: Option<Value> = iter1.next();
    let mut index2: Option<Value> = iter2.next();
    loop {
        match (index1.clone(), index2.clone()) {
            (Some(value1), Some(value2)) => {
                if value1 != value2 {
                    return Ok(false);
                }
            }
            _ => break,
        }
        index1 = iter1.next();
        index2 = iter2.next();
    }

    match_last_iter_values::<Data, Value, Value>(index1, index2)
}

fn compare_item_iterators<Data: GarnishData, Iter, GetFn>(
    this: &mut Data,
    left_addr: Data::Size,
    right_addr: Data::Size,
    get_iter: GetFn,
) -> Result<bool, RuntimeError<Data::Error>>
where
    Iter: Iterator<Item = Data::Size>,
    GetFn: Fn(&Data, Data::Size, Extents<Data::Number>) -> Result<Iter, Data::Error>,
{
    let (iter1, iter2) = (get_iter(this, left_addr.clone(), Extents::new(Data::Number::zero(), Data::Number::max_value()))?, get_iter(this, right_addr.clone(), Extents::new(Data::Number::zero(), Data::Number::max_value()))?);
    push_iterator_values(this, iter1, iter2)
}

fn compare_slice_item_iterators_2<Data, Iter, GetValueIterFn>(
    this: &mut Data,
    slice_addr: Data::Size,
    list_addr: Data::Size,
    get_value_iter: GetValueIterFn,
) -> Result<bool, RuntimeError<Data::Error>>
where
    Data: GarnishData,
    Iter: Iterator<Item = Data::Size>,
    GetValueIterFn: Fn(&Data, Data::Size, Extents<Data::Number>) -> Result<Iter, Data::Error>,
{
    let (value2, range2) = this.get_slice(slice_addr.clone())?;
    let (start2, end2, _) = get_range(this, range2)?;
    let extents2 = Extents::new(start2, end2);

    match this.get_data_type(value2)? {
        GarnishDataType::List => compare_item_iterators_2(this, list_addr, slice_addr, get_value_iter, Data::get_list_item_iter, Extents::new(Data::Number::zero(), Data::Number::max_value()), extents2),
        GarnishDataType::Concatenation => compare_item_iterators_2(this, list_addr, slice_addr, get_value_iter, Data::get_concatenation_iter, Extents::new(Data::Number::zero(), Data::Number::max_value()), extents2),
        _ => Ok(false),
    }
}

fn compare_item_iterators_2<Data: GarnishData, Iter1, Iter2, GetFn1, GetFn2>(
    this: &mut Data,
    left_addr: Data::Size,
    right_addr: Data::Size,
    get_iter_left: GetFn1,
    get_iter_right: GetFn2,
    extents_left: Extents<Data::Number>,
    extents_right: Extents<Data::Number>,
) -> Result<bool, RuntimeError<Data::Error>>
where
    Iter1: Iterator<Item = Data::Size>,
    Iter2: Iterator<Item = Data::Size>,
    GetFn1: Fn(&Data, Data::Size, Extents<Data::Number>) -> Result<Iter1, Data::Error>,
    GetFn2: Fn(&Data, Data::Size, Extents<Data::Number>) -> Result<Iter2, Data::Error>,
{
    let (iter1, iter2) = (get_iter_left(this, left_addr.clone(), extents_left)?, get_iter_right(this, right_addr.clone(), extents_right)?);
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

    match_last_iter_values::<Data, Data::Size, Data::Size>(index1, index2)
}

fn match_last_iter_values<Data: GarnishData, T1, T2>(value_1: Option<T1>, value_2: Option<T2>) -> Result<bool, RuntimeError<Data::Error>> {
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
    use crate::runtime::tests::{MockByteIterator, MockCharIterator, MockError, MockGarnishData, MockIterator, MockSymbolListPartIterator};
    use garnish_lang_traits::{GarnishDataType, SymbolListPart};

    struct ListCompData {
        pub types: Vec<GarnishDataType>,
        pub registers: Vec<i32>,
        pub lens: Vec<i32>,
        pub items: Vec<Vec<u32>>,
    }

    impl ListCompData {
        fn get_symbol_list_item(&self, list: i32, index: i32) -> Result<Option<SymbolListPart<u32, i32>>, MockError> {
            let item = self.items.get(list as usize).unwrap().get(index as usize).unwrap().clone();
            Ok(Some(SymbolListPart::Symbol(item)))
        }

        fn get_character_list_item(&self, list: i32, index: i32) -> Result<Option<char>, MockError> {
            let item = self.items.get(list as usize).unwrap().get(index as usize).unwrap().clone();
            Ok(Some(char::from_u32(item).unwrap()))
        }

        fn get_byte_list_item(&self, list: i32, index: i32) -> Result<Option<u8>, MockError> {
            let item = self.items.get(list as usize).unwrap().get(index as usize).unwrap().clone();
            Ok(Some(item as u8))
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
        data.stub_get_byte_list_iter = |_data, _i| MockByteIterator::new(vec![10, 20]);
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
        data.stub_get_byte_list_iter = |_data, _i| MockByteIterator::new(vec![10, 20]);
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
        data.stub_get_char_list_iter = |_data, _i| MockCharIterator::new(String::from("ab"));
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
        data.stub_get_symbol_list_iter = |_data, _i| MockSymbolListPartIterator::new(vec![SymbolListPart::Symbol(10), SymbolListPart::Symbol(20)]);
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
        data.stub_get_symbol_list_iter = |_data, _i| MockSymbolListPartIterator::new(vec![SymbolListPart::Symbol(10), SymbolListPart::Symbol(20)]);
        data.stub_get_symbol_list_len = |data, i| Ok(data.lens.get(i as usize).unwrap().clone());
        data.stub_start_list = |_, _| Ok(0);
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
        data.stub_get_symbol_list_iter = |_data, _i| MockSymbolListPartIterator::new(vec![SymbolListPart::Symbol(10), SymbolListPart::Symbol(20)]);
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
        data.stub_get_slice_iter = |_data, _i| MockIterator::new(2);
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
    fn char_list_equals_slice_of_char_list() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![GarnishDataType::CharList, GarnishDataType::Slice, GarnishDataType::Range],
            registers: vec![0, 1],
            lens: vec![2, 2],
            items: vec![vec![10, 20], vec![10, 20]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_slice = |_, addr| {
            if addr == 1 {
                Ok((0, 1))
            } else {
                assert!(false);
                Err(MockError {})
            }
        };
        data.stub_get_slice_iter = |_, _| MockIterator::new(2);
        data.stub_get_char_list_iter = |_, _| MockCharIterator::new(String::from("ab"));
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
    fn slice_of_char_list_equals_char_list() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![GarnishDataType::CharList, GarnishDataType::Slice, GarnishDataType::Range],
            registers: vec![1, 0],
            lens: vec![2, 2],
            items: vec![vec![10, 20], vec![10, 20]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_slice = |_, addr| {
            if addr == 1 {
                Ok((0, 1))
            } else {
                assert!(false);
                Err(MockError {})
            }
        };
        data.stub_get_slice_iter = |_, _| MockIterator::new(2);
        data.stub_get_char_list_iter = |_, _| MockCharIterator::new(String::from("ab"));
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
    fn byte_list_equals_slice_of_byte_list() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![GarnishDataType::ByteList, GarnishDataType::Slice, GarnishDataType::Range],
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
        data.stub_get_byte_list_iter = |_, _| MockByteIterator::new(vec![10, 20]);
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
    fn slice_of_byte_list_equals_byte_list() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![GarnishDataType::ByteList, GarnishDataType::Slice, GarnishDataType::Range],
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
        data.stub_get_byte_list_iter = |_, _| MockByteIterator::new(vec![10, 20]);
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
    fn list_equals_slice_of_list() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![
                GarnishDataType::List,
                GarnishDataType::Range,
                GarnishDataType::Slice,
                GarnishDataType::Number,
                GarnishDataType::Number,
            ],
            registers: vec![0, 2],
            lens: vec![2, 2],
            items: vec![vec![6, 6], vec![6, 6]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_slice = |_, addr| {
            if addr == 2 {
                Ok((0, 1))
            } else {
                assert!(false);
                Err(MockError {})
            }
        };
        data.stub_get_list_slice_item_iter = |_, _| MockIterator::new_range(3, 5);
        data.stub_get_list_item_iter = |_, _| MockIterator::new_range(3, 5);
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
    fn slice_of_list_equals_list() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![
                GarnishDataType::List,
                GarnishDataType::Range,
                GarnishDataType::Slice,
                GarnishDataType::Number,
                GarnishDataType::Number,
            ],
            registers: vec![2, 0],
            lens: vec![2, 2],
            items: vec![vec![6, 6], vec![6, 6]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_slice = |_, addr| {
            if addr == 2 {
                Ok((0, 1))
            } else {
                assert!(false);
                Err(MockError {})
            }
        };
        data.stub_get_list_slice_item_iter = |_, _| MockIterator::new_range(3, 5);
        data.stub_get_list_item_iter = |_, _| MockIterator::new_range(3, 5);
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
    fn concatenation_equals_slice_of_concatenation() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![
                GarnishDataType::Concatenation,
                GarnishDataType::Range,
                GarnishDataType::Slice,
                GarnishDataType::Number,
                GarnishDataType::Number,
            ],
            registers: vec![0, 2],
            lens: vec![2, 2],
            items: vec![vec![6, 6], vec![6, 6]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_slice = |_, addr| {
            if addr == 2 {
                Ok((0, 1))
            } else {
                assert!(false);
                Err(MockError {})
            }
        };
        data.stub_get_concatenation_slice_iter = |_, _| MockIterator::new_range(3, 5);
        data.stub_get_concatenation_iter = |_, _| MockIterator::new_range(3, 5);
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
    fn slice_of_concatenation_equals_concatenation() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![
                GarnishDataType::Concatenation,
                GarnishDataType::Range,
                GarnishDataType::Slice,
                GarnishDataType::Number,
                GarnishDataType::Number,
            ],
            registers: vec![2, 0],
            lens: vec![2, 2],
            items: vec![vec![6, 6], vec![6, 6]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_slice = |_, addr| {
            if addr == 2 {
                Ok((0, 1))
            } else {
                assert!(false);
                Err(MockError {})
            }
        };
        data.stub_get_concatenation_slice_iter = |_, _| MockIterator::new_range(3, 5);
        data.stub_get_concatenation_iter = |_, _| MockIterator::new_range(3, 5);
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
    fn slice_of_concatenation_equals_list() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![
                GarnishDataType::Concatenation,
                GarnishDataType::Range,
                GarnishDataType::Slice,
                GarnishDataType::List,
                GarnishDataType::Number,
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
            } else {
                assert!(false);
                Err(MockError {})
            }
        };
        data.stub_get_list_item_iter = |_, _| MockIterator::new_range(4, 6);
        data.stub_get_concatenation_slice_iter = |_, _| MockIterator::new_range(4, 6);
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
    fn concatenation_equals_slice_of_list() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![
                GarnishDataType::Concatenation,
                GarnishDataType::Range,
                GarnishDataType::Slice,
                GarnishDataType::List,
                GarnishDataType::Number,
                GarnishDataType::Number,
            ],
            registers: vec![0, 2],
            lens: vec![2, 2],
            items: vec![vec![6, 6], vec![6, 6]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_slice = |_, addr| {
            if addr == 2 {
                Ok((3, 1))
            } else {
                assert!(false);
                Err(MockError {})
            }
        };
        data.stub_get_list_slice_item_iter = |_, _| MockIterator::new_range(4, 6);
        data.stub_get_concatenation_iter = |_, _| MockIterator::new_range(4, 6);
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
    fn slice_of_list_equals_concatenation() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![
                GarnishDataType::Concatenation,
                GarnishDataType::Range,
                GarnishDataType::Slice,
                GarnishDataType::List,
                GarnishDataType::Number,
                GarnishDataType::Number,
            ],
            registers: vec![2, 0],
            lens: vec![2, 2],
            items: vec![vec![6, 6], vec![6, 6]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_slice = |_, addr| {
            if addr == 2 {
                Ok((3, 1))
            } else {
                assert!(false);
                Err(MockError {})
            }
        };
        data.stub_get_list_slice_item_iter = |_, _| MockIterator::new_range(4, 6);
        data.stub_get_concatenation_iter = |_, _| MockIterator::new_range(4, 6);
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
    fn list_equals_slice_of_concatenation() {
        let mut data = MockGarnishData::default_with_data(ListCompData {
            types: vec![
                GarnishDataType::Concatenation,
                GarnishDataType::Range,
                GarnishDataType::Slice,
                GarnishDataType::List,
                GarnishDataType::Number,
                GarnishDataType::Number,
            ],
            registers: vec![3, 2],
            lens: vec![2, 2],
            items: vec![vec![6, 6], vec![6, 6]],
        });

        data.stub_get_data_type = |data, i| Ok(data.types.get(i as usize).unwrap().clone());
        data.stub_pop_register = |data| Ok(data.registers.pop());
        data.stub_get_register_len = |data| data.registers.len() as i32;
        data.stub_get_slice = |_, addr| {
            if addr == 2 {
                Ok((0, 1))
            } else {
                assert!(false);
                Err(MockError {})
            }
        };
        data.stub_get_list_item_iter = |_, _| MockIterator::new_range(4, 6);
        data.stub_get_concatenation_slice_iter = |_, _| MockIterator::new_range(4, 6);
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
        data.stub_get_symbol_list_iter = |_, _| MockSymbolListPartIterator::new(vec![SymbolListPart::Symbol(10), SymbolListPart::Symbol(20)]);
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
        data.stub_get_symbol_list_iter = |_, _| MockSymbolListPartIterator::new(vec![SymbolListPart::Symbol(10), SymbolListPart::Symbol(20)]);
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
