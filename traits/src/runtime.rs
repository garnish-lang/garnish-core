use crate::{GarnishContext, GarnishData, RuntimeError};


/// Trait containing instruction operations Garnish needs to execute.
/// All instruction methods (e.g. all except [`GarnishRuntime::get_data`], [`GarnishRuntime::get_data_mut`] should a Result.
/// With the Ok value being the next instruction address to be executed if not sequential, otherwise return None
pub trait GarnishRuntime<Data: GarnishData> {
    fn get_data(&self) -> &Data;
    fn get_data_mut(&mut self) -> &mut Data;

    fn apply<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn reapply(&mut self, index: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn empty_apply<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;

    fn add<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn subtract<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn multiply<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn power<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn divide<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn integer_divide<T: GarnishContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn remainder<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn absolute_value<T: GarnishContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn opposite<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;

    fn bitwise_not<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn bitwise_and<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn bitwise_or<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn bitwise_xor<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn bitwise_left_shift<T: GarnishContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn bitwise_right_shift<T: GarnishContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;

    fn and(&mut self, data: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn or(&mut self, data: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn xor(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn not(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn tis(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;

    fn type_of(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn type_equal(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn type_cast<T: GarnishContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;

    fn equal(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn not_equal(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn less_than(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn less_than_or_equal(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn greater_than(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn greater_than_or_equal(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;

    fn jump(&mut self, index: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn jump_if_true(&mut self, index: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn jump_if_false(&mut self, index: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn end_expression(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;

    fn make_list(&mut self, len: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn access_left_internal<T: GarnishContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn access_right_internal<T: GarnishContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn access_length_internal<T: GarnishContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;

    fn make_range(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn make_start_exclusive_range(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn make_end_exclusive_range(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn make_exclusive_range(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;

    fn make_pair(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;

    fn concat(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;

    fn put(&mut self, i: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn put_value(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn push_value(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn update_value(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;

    fn start_side_effect(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn end_side_effect(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;

    fn resolve<T: GarnishContext<Data>>(
        &mut self,
        data: Data::Size,
        context: Option<&mut T>,
    ) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
}
