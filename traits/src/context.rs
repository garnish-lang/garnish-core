use crate::{GarnishDataType, GarnishData, Instruction, RuntimeError};

/// Functionality required by Garnish contexts.
#[deprecated(since = "0.0.25-alpha", note = "Implement matching methods on GarnishData trait.")]
pub trait GarnishContext<Data>
    where
        Data: GarnishData,
{
    /// Called during a [`Instruction::Resolve`], to convert a [`GarnishDataType::Symbol`] to a value.
    ///
    /// Return Ok(true) to tell the runtime that the symbol was resolved
    ///
    /// Return Ok(false) to let the runtime fill a default value, probably [`GarnishDataType::Unit`].
    ///
    fn resolve(&mut self, _symbol: Data::Symbol, _data: &mut Data) -> Result<bool, RuntimeError<Data::Error>> {
        Ok(false)
    }

    /// Called when an [`GarnishDataType::External`] is on the left side of an [`Instruction::Apply`] operation.
    ///
    /// Return Ok(true) to tell the runtime this apply operation was handled
    ///
    /// Return Ok(false) to tell the runtime this apply operation was not handled
    ///
    fn apply(&mut self, _external_value: Data::Size, _input_addr: Data::Size, _data: &mut Data) -> Result<bool, RuntimeError<Data::Error>> {
        Ok(false)
    }

    /// Called during any operation where the types given don't have defined functionality.
    /// Such as a [`GarnishDataType::List`] and a [`GarnishDataType::Number`] in an [`Instruction::Add`] operation
    ///
    /// Return Ok(true) to tell the runtime this operation was handled
    ///
    /// Return Ok(false) to tell the runtime this operation was not handled
    ///
    fn defer_op(
        &mut self,
        _data: &mut Data,
        _operation: Instruction,
        _left: (GarnishDataType, Data::Size),
        _right: (GarnishDataType, Data::Size),
    ) -> Result<bool, RuntimeError<Data::Error>> {
        Ok(false)
    }
}

/// Concrete object for when a runtime requires no context functionality. Can use constants [`EMPTY_CONTEXT`] and [`NO_CONTEXT`] if needed as type parameters.
#[deprecated(since = "0.0.25-alpha", note = "See GarnishContext deprecation.")]
pub struct EmptyContext {}

/// Constant instantiation of [`EmptyContext`].
#[deprecated(since = "0.0.25-alpha", note = "See GarnishContext deprecation.")]
pub const EMPTY_CONTEXT: EmptyContext = EmptyContext {};
/// An [`Option`] set to None. Typed as a mutable reference to an [`EmptyContext`] for use in [`crate::GarnishRuntime`] instruction methods that require a context be passed.
#[deprecated(since = "0.0.25-alpha", note = "See GarnishContext deprecation.")]
pub const NO_CONTEXT: Option<&mut EmptyContext> = None;

impl<Data> GarnishContext<Data> for EmptyContext where Data: GarnishData {}
