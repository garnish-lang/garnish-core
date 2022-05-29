use crate::Instruction;
pub use garnish_traits::{ErrorType, RuntimeError};
use std::fmt::Debug;

pub trait OrNumberError<T, Source: 'static + std::error::Error> {
    fn or_num_err(self) -> Result<T, RuntimeError<Source>>;
}

impl<T, Source: 'static + std::error::Error> OrNumberError<T, Source> for Option<T> {
    fn or_num_err(self) -> Result<T, RuntimeError<Source>> {
        self.ok_or(RuntimeError::new("Number error"))
    }
}

// Creation utilites

pub(crate) fn instruction_error<T, E: std::error::Error + 'static, I: Debug>(instruction: Instruction, index: I) -> Result<T, RuntimeError<E>> {
    Err(RuntimeError::new_message(format!(
        "Expected instruction {:?} at index {:?} to have data. Found None.",
        instruction, index
    )))
}

pub(crate) fn state_error<T, E: std::error::Error>(message: String) -> Result<T, RuntimeError<E>> {
    Err(RuntimeError::new(message.as_str()))
}
