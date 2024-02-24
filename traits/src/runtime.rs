use crate::{GarnishLangRuntimeContext, GarnishLangRuntimeData, RuntimeError};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum GarnishLangRuntimeState {
    Running,
    End,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct GarnishLangRuntimeInfo {
    state: GarnishLangRuntimeState,
}

/// Information about the current execution state of a runtime.
impl GarnishLangRuntimeInfo {
    pub fn new(state: GarnishLangRuntimeState) -> Self {
        return GarnishLangRuntimeInfo { state };
    }

    pub fn get_state(&self) -> GarnishLangRuntimeState {
        self.state
    }
}

/// Trait containing instruction operations Garnish needs to execute.
/// All instruction methods (e.g. all except [`GarnishRuntime::get_data`], [`GarnishRuntime::get_data_mut`], [`GarnishRuntime::execute_current_instruction`] should a Result.
/// With the Ok value being the next instruction address to be executed if not sequential, otherwise return None
pub trait GarnishRuntime<Data: GarnishLangRuntimeData> {
    fn get_data(&self) -> &Data;
    fn get_data_mut(&mut self) -> &mut Data;

    fn execute_current_instruction<T: GarnishLangRuntimeContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> Result<GarnishLangRuntimeInfo, RuntimeError<Data::Error>>;

    fn apply<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn reapply(&mut self, index: Data::Size) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn empty_apply<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;

    fn add<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn subtract<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn multiply<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn power<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn divide<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn integer_divide<T: GarnishLangRuntimeContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn remainder<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn absolute_value<T: GarnishLangRuntimeContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn opposite<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;

    fn bitwise_not<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn bitwise_and<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn bitwise_or<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn bitwise_xor<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn bitwise_left_shift<T: GarnishLangRuntimeContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn bitwise_right_shift<T: GarnishLangRuntimeContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;

    fn and(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn or(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn xor(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn not(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn tis(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;

    fn type_of(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn type_equal(&mut self) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn type_cast<T: GarnishLangRuntimeContext<Data>>(&mut self, context: Option<&mut T>) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;

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
    fn access_left_internal<T: GarnishLangRuntimeContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn access_right_internal<T: GarnishLangRuntimeContext<Data>>(
        &mut self,
        context: Option<&mut T>,
    ) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
    fn access_length_internal<T: GarnishLangRuntimeContext<Data>>(
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

    fn resolve<T: GarnishLangRuntimeContext<Data>>(
        &mut self,
        data: Data::Size,
        context: Option<&mut T>,
    ) -> Result<Option<Data::Size>, RuntimeError<Data::Error>>;
}
