use std::fmt::Debug;

use log::trace;

use crate::runtime::error::state_error;
use crate::runtime::error::OrNumberError;
use crate::runtime::internals::concatenation_len;
use crate::runtime::list::index_concatenation_for;
use crate::runtime::utilities::{get_range, next_two_raw_ref, push_boolean};
use garnish_lang_traits::{GarnishDataType, GarnishData, GarnishNumber, RuntimeError, TypeConstants};

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
    let right_type = this.get_data_type(right)?;

    // check if right type needs to be corrected if Type type
    // but only if both aren't Type type
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

            return Ok(false);
        }
    }

    Ok(true)
}

fn data_equal<Data: GarnishData>(
    this: &mut Data,
    left_addr: Data::Size,
    right_addr: Data::Size,
) -> Result<bool, RuntimeError<Data::Error>> {
    let (left_type, right_type) = (this.get_data_type(left_addr)?, this.get_data_type(right_addr)?);

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
            if this.get_char_list_len(right_addr)? == Data::Size::one() {
                let c1 = this.get_char(left_addr)?;
                let c2 = this.get_char_list_item(right_addr, Data::Number::zero())?;

                c1 == c2
            } else {
                false
            }
        }
        (GarnishDataType::CharList, GarnishDataType::Char) => {
            if this.get_char_list_len(left_addr)? == Data::Size::one() {
                let c1 = this.get_char_list_item(left_addr, Data::Number::zero())?;
                let c2 = this.get_char(right_addr)?;

                c1 == c2
            } else {
                false
            }
        }
        (GarnishDataType::Byte, GarnishDataType::ByteList) => {
            if this.get_byte_list_len(right_addr)? == Data::Size::one() {
                let c1 = this.get_byte(left_addr)?;
                let c2 = this.get_byte_list_item(right_addr, Data::Number::zero())?;

                c1 == c2
            } else {
                false
            }
        }
        (GarnishDataType::ByteList, GarnishDataType::Byte) => {
            if this.get_byte_list_len(left_addr)? == Data::Size::one() {
                let c1 = this.get_byte_list_item(left_addr, Data::Number::zero())?;
                let c2 = this.get_byte(right_addr)?;

                c1 == c2
            } else {
                false
            }
        }
        (GarnishDataType::CharList, GarnishDataType::CharList) => {
            let len1 = this.get_char_list_len(left_addr)?;
            let len2 = this.get_char_list_len(right_addr)?;

            if len1 != len2 {
                false
            } else {
                let mut count = Data::Size::one();
                let mut equal = true;
                while count < len1 {
                    let i = Data::size_to_number(count);
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
        (GarnishDataType::ByteList, GarnishDataType::ByteList) => {
            let len1 = this.get_byte_list_len(left_addr)?;
            let len2 = this.get_byte_list_len(right_addr)?;

            if len1 != len2 {
                false
            } else {
                let mut count = Data::Size::one();
                let mut equal = true;
                while count < len1 {
                    let i = Data::size_to_number(count);
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
        (GarnishDataType::Range, GarnishDataType::Range) => {
            let (start1, end1) = this.get_range(left_addr)?;
            let (start2, end2) = this.get_range(right_addr)?;

            let start_equal = match (this.get_data_type(start1)?, this.get_data_type(start2)?) {
                (GarnishDataType::Unit, GarnishDataType::Unit) => true,
                (GarnishDataType::Number, GarnishDataType::Number) => this.get_number(start1)? == this.get_number(start2)?,
                _ => false,
            };

            let end_equal = match (this.get_data_type(end1)?, this.get_data_type(end2)?) {
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
            let len1 = concatenation_len(this, left_addr)?;
            let len2 = concatenation_len(this, right_addr)?;

            if len1 != len2 {
                false
            } else {
                let mut count = Data::Number::zero();
                let len = Data::size_to_number(len1);

                while count < len {
                    // need to find faster way
                    // curent way is to index each concatenation one item at a time
                    match (
                        index_concatenation_for(this, left_addr, count)?,
                        index_concatenation_for(this, right_addr, count)?,
                    ) {
                        (Some(left), Some(right)) => {
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
        (GarnishDataType::List, GarnishDataType::Concatenation) => {
            let len1 = this.get_list_len(left_addr)?;
            let len2 = concatenation_len(this, right_addr)?;

            if len1 != len2 {
                false
            } else {
                let mut count = Data::Number::zero();
                let len = Data::size_to_number(len1);

                while count < len {
                    // need to find faster way
                    // curent way is to index each concatenation one item at a time
                    match (this.get_list_item(left_addr, count)?, index_concatenation_for(this, right_addr, count)?) {
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
            let len1 = concatenation_len(this, left_addr)?;
            let len2 = this.get_list_len(right_addr)?;

            if len1 != len2 {
                false
            } else {
                let mut count = Data::Number::zero();
                let len = Data::size_to_number(len1);

                while count < len {
                    // need to find faster way
                    // curent way is to index each concatenation one item at a time
                    match (index_concatenation_for(this, left_addr, count)?, this.get_list_item(right_addr, count)?) {
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
            let (value1, range1) = this.get_slice(left_addr)?;
            let (value2, range2) = this.get_slice(right_addr)?;

            let (start1, _, len1) = get_range(this, range1)?;
            let (start2, _, len2) = get_range(this, range2)?;

            // slices need same len of range
            // if so, run through list and push to register

            if len1 != len2 {
                false
            } else {
                match (this.get_data_type(value1)?, this.get_data_type(value2)?) {
                    (GarnishDataType::CharList, GarnishDataType::CharList) => {
                        let mut index1 = start1;
                        let mut index2 = start2;
                        let mut count = Data::Number::zero();

                        let list_len1 = Data::size_to_number(this.get_char_list_len(value1)?);
                        let list_len2 = Data::size_to_number(this.get_char_list_len(value2)?);

                        while count < len1 {
                            let item1 = if index1 < list_len1 {
                                this.get_char_list_item(value1, index1)?
                            } else {
                                return Ok(false);
                            };

                            let item2 = if index2 < list_len2 {
                                this.get_char_list_item(value2, index2)?
                            } else {
                                return Ok(false);
                            };

                            if item1 != item2 {
                                return Ok(false);
                            }

                            index1 = index1.increment().or_num_err()?;
                            index2 = index2.increment().or_num_err()?;
                            count = count.increment().or_num_err()?;
                        }

                        true
                    }
                    (GarnishDataType::ByteList, GarnishDataType::ByteList) => {
                        let mut index1 = start1;
                        let mut index2 = start2;
                        let mut count = Data::Number::zero();

                        let list_len1 = Data::size_to_number(this.get_byte_list_len(value1)?);
                        let list_len2 = Data::size_to_number(this.get_byte_list_len(value2)?);

                        while count < len1 {
                            let item1 = if index1 < list_len1 {
                                this.get_byte_list_item(value1, index1)?
                            } else {
                                return Ok(false);
                            };

                            let item2 = if index2 < list_len2 {
                                this.get_byte_list_item(value2, index2)?
                            } else {
                                return Ok(false);
                            };

                            if item1 != item2 {
                                return Ok(false);
                            }

                            index1 = index1.increment().or_num_err()?;
                            index2 = index2.increment().or_num_err()?;
                            count = count.increment().or_num_err()?;
                        }

                        true
                    }
                    (GarnishDataType::List, GarnishDataType::List) => {
                        let mut index1 = start1;
                        let mut index2 = start2;
                        let mut count = Data::Number::zero();

                        let list_len1 = Data::size_to_number(this.get_list_len(value1)?);
                        let list_len2 = Data::size_to_number(this.get_list_len(value2)?);

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

                            index1 = index1.increment().or_num_err()?;
                            index2 = index2.increment().or_num_err()?;
                            count = count.increment().or_num_err()?;
                        }

                        true
                    }
                    _ => false,
                }
            }
        }
        (GarnishDataType::List, GarnishDataType::List) => {
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
                    let i = Data::size_to_number(count);
                    let left_item = this.get_list_item(left_addr, i)?;
                    let right_item = this.get_list_item(right_addr, i)?;

                    let (left_is_associative, pair_sym, pair_item) = match this.get_data_type(left_item)? {
                        GarnishDataType::Pair => {
                            let (left, right) = this.get_pair(left_item)?;
                            match this.get_data_type(left)? {
                                GarnishDataType::Symbol => (true, this.get_symbol(left)?, right),
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
