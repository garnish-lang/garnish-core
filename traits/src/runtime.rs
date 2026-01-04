use crate::{GarnishData, RuntimeError};


/// Trait containing instruction operations Garnish needs to execute.
/// All instruction methods (e.g. all except [`GarnishRuntime::get_data`], [`GarnishRuntime::get_data_mut`] should a Result.
/// With the Ok value being the next instruction address to be executed if not sequential, otherwise return None
pub trait GarnishRuntime<Data: GarnishData> {
    fn get_data(&self) -> &Data;
    fn get_data_mut(&mut self) -> &mut Data;

    fn apply(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn reapply(&mut self, index: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn empty_apply(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;

    fn add(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn subtract(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn multiply(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn power(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn divide(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn integer_divide(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn remainder(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn absolute_value(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn opposite(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;

    fn bitwise_not(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn bitwise_and(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn bitwise_or(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn bitwise_xor(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn bitwise_left_shift(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn bitwise_right_shift(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;

    fn and(&mut self, data: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn or(&mut self, data: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn xor(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn not(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn tis(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;

    fn type_of(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn type_equal(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn type_cast(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;

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
    fn access(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn access_left_internal(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn access_right_internal(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn access_length_internal(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;

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

    fn resolve(&mut self, data: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
}
