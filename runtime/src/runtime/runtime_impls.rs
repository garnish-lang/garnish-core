use crate::runtime::access::access;
use crate::runtime::apply::*;
use crate::runtime::arithmetic::{absolute_value, add, divide, integer_divide, multiply, opposite, power, remainder, subtract};
use crate::runtime::bitwise::{bitwise_and, bitwise_left_shift, bitwise_not, bitwise_or, bitwise_right_shift, bitwise_xor};
use crate::runtime::casting::{type_cast, type_of};
use crate::runtime::comparison::{greater_than, greater_than_or_equal, less_than, less_than_or_equal};
use crate::runtime::concat::concat;
use crate::runtime::equality::equal;
use crate::runtime::equality::{not_equal, type_equal};
use crate::runtime::internals::{access_left_internal, access_length_internal, access_right_internal};
use crate::runtime::jumps::{end_expression, jump, jump_if_false, jump_if_true};
use crate::runtime::list::*;
use crate::runtime::logical::{and, not, or, tis, xor};
use crate::runtime::pair::make_pair;
use crate::runtime::put::{push_value, put, put_value, update_value};
use crate::runtime::range::{make_end_exclusive_range, make_exclusive_range, make_range, make_start_exclusive_range};
use crate::runtime::resolve::resolve;
use crate::runtime::sideeffect::*;

use crate::execute::{execute_current_instruction, SimpleRuntimeInfo};
use garnish_lang_traits::{
    GarnishContext, GarnishData, GarnishRuntime, RuntimeError
    ,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Implementation of a [`GarnishRuntime`]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
#[deprecated(since = "0.0.20-alpha", note = "Use execute::execute_current_instruction instead of SimpleGarnishData.")]
pub struct SimpleGarnishRuntime<Data: GarnishData> {
    data: Data,
}

#[deprecated(since = "0.0.20-alpha", note = "Use execute::execute_current_instruction instead of SimpleGarnishData.")]
impl<Data: GarnishData> SimpleGarnishRuntime<Data> {
    #[deprecated(since = "0.0.20-alpha", note = "Use execute::execute_current_instruction instead of SimpleGarnishData.")]
    pub fn new(data: Data) -> SimpleGarnishRuntime<Data> {
        SimpleGarnishRuntime { data }
    }

    #[deprecated(since = "0.0.20-alpha", note = "Use execute::execute_current_instruction instead of SimpleGarnishData.")]
    pub fn get_data_owned(self) -> Data {
        self.data
    }

    #[deprecated(since = "0.0.20-alpha", note = "Use execute::execute_current_instruction instead of SimpleGarnishData.")]
    pub fn execute_current_instruction<T: GarnishContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> Result<SimpleRuntimeInfo, RuntimeError<Data::Error>> {
        execute_current_instruction(self.get_data_mut(), context)
    }
}

impl<Data> GarnishRuntime<Data> for SimpleGarnishRuntime<Data>
where
    Data: GarnishData,
{
    fn get_data(&self) -> &Data {
        &self.data
    }

    fn get_data_mut(&mut self) -> &mut Data {
        &mut self.data
    }

    // Apply

    fn apply<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        apply(self.get_data_mut())
    }

    fn reapply(&mut self, index: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        reapply(self.get_data_mut(), index)
    }

    fn empty_apply<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        empty_apply(self.get_data_mut())
    }

    //
    // Arithmetic
    //

    fn add<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        add(self.get_data_mut())
    }

    fn subtract<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        subtract(self.get_data_mut())
    }

    fn multiply<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        multiply(self.get_data_mut())
    }

    fn power<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        power(self.get_data_mut())
    }

    fn divide<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        divide(self.get_data_mut())
    }

    fn integer_divide<T: GarnishContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        integer_divide(self.get_data_mut())
    }

    fn remainder<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        remainder(self.get_data_mut())
    }

    fn absolute_value<T: GarnishContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        absolute_value(self.get_data_mut())
    }

    fn opposite<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        opposite(self.get_data_mut())
    }

    //
    // Bitwise
    //

    fn bitwise_not<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        bitwise_not(self.get_data_mut())
    }

    fn bitwise_and<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        bitwise_and(self.get_data_mut())
    }

    fn bitwise_or<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        bitwise_or(self.get_data_mut())
    }

    fn bitwise_xor<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        bitwise_xor(self.get_data_mut())
    }

    fn bitwise_left_shift<T: GarnishContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        bitwise_left_shift(self.get_data_mut())
    }

    fn bitwise_right_shift<T: GarnishContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        bitwise_right_shift(self.get_data_mut())
    }

    //
    // Logical
    //

    fn and(&mut self, data: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        and(self.get_data_mut(), data)
    }

    fn or(&mut self, data: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        or(self.get_data_mut(), data)
    }

    fn xor(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        xor(self.get_data_mut())
    }

    fn not(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        not(self.get_data_mut())
    }

    fn tis(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        tis(self.get_data_mut())
    }

    //
    // Type Ops
    //

    fn type_of(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        type_of(self.get_data_mut())
    }

    fn type_equal(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        type_equal(self.get_data_mut())
    }

    fn type_cast<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        type_cast(self.get_data_mut(), context)
    }

    //
    // Comparison
    //

    fn equal(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        equal(self.get_data_mut())
    }

    fn not_equal(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        not_equal(self.get_data_mut())
    }

    fn less_than(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        less_than(self.get_data_mut())
    }

    fn less_than_or_equal(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        less_than_or_equal(self.get_data_mut())
    }

    fn greater_than(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        greater_than(self.get_data_mut())
    }

    fn greater_than_or_equal(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        greater_than_or_equal(self.get_data_mut())
    }

    //
    // Jumps
    //

    fn jump(&mut self, index: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        jump(self.get_data_mut(), index)
    }

    fn jump_if_true(&mut self, index: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        jump_if_true(self.get_data_mut(), index)
    }

    fn jump_if_false(&mut self, index: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        jump_if_false(self.get_data_mut(), index)
    }

    fn end_expression(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        end_expression(self.get_data_mut())
    }

    //
    // List
    //

    fn make_list(&mut self, len: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        make_list(self.get_data_mut(), len)
    }

    fn access<T: GarnishContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        access(self.get_data_mut(), context)
    }

    fn access_left_internal<T: GarnishContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        access_left_internal(self.get_data_mut(), context)
    }

    fn access_right_internal<T: GarnishContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        access_right_internal(self.get_data_mut(), context)
    }

    fn access_length_internal<T: GarnishContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        access_length_internal(self.get_data_mut(), context)
    }

    //
    // Range
    //

    fn make_range(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        make_range(self.get_data_mut())
    }

    fn make_start_exclusive_range(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        make_start_exclusive_range(self.get_data_mut())
    }

    fn make_end_exclusive_range(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        make_end_exclusive_range(self.get_data_mut())
    }

    fn make_exclusive_range(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        make_exclusive_range(self.get_data_mut())
    }

    //
    // Pair
    //

    fn make_pair(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        make_pair(self.get_data_mut())
    }

    //
    // Concatenation
    //

    fn concat(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        concat(self.get_data_mut())
    }

    //
    // Put
    //

    fn put(&mut self, i: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        put(self.get_data_mut(), i)
    }

    fn put_value(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        put_value(self.get_data_mut())
    }

    fn push_value(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        push_value(self.get_data_mut())
    }

    fn update_value(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        update_value(self.get_data_mut())
    }

    //
    // Side Effect
    //

    fn start_side_effect(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        start_side_effect(self.get_data_mut())
    }

    fn end_side_effect(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        end_side_effect(self.get_data_mut())
    }

    //
    // Resolve
    //

    fn resolve<T: GarnishContext<Data>>(
        &mut self,
        data: Data::Size,
        context: Option<&mut T>,
    ) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
        resolve(self.get_data_mut(), data)
    }
}
