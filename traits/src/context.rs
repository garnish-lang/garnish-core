use crate::{ExpressionDataType, GarnishLangRuntimeData, Instruction, RuntimeError};

/// Functionality required by Garnish contexts.
pub trait GarnishLangRuntimeContext<Data>
    where
        Data: GarnishLangRuntimeData,
{
    /// Called during a [`Instruction::Resolve`], to convert a [`ExpressionDataType::Symbol`] to a value.
    ///
    /// Return Ok(true) to tell the runtime that the symbol was resolved
    ///
    /// Return Ok(false) to let the runtime fill a default value, probably [`ExpressionDataType::Unit`].
    ///
    fn resolve(&mut self, _symbol: Data::Symbol, _runtime: &mut Data) -> Result<bool, RuntimeError<Data::Error>> {
        Ok(false)
    }

    /// Called when an [`ExpressionDataType::External`] is on the left side of an [`Instruction::Apply`] operation.
    ///
    /// Return Ok(true) to tell the runtime this apply operation was handled
    ///
    /// Return Ok(false) to tell the runtime this apply operation was not handled
    ///
    fn apply(&mut self, _external_value: Data::Size, _input_addr: Data::Size, _runtime: &mut Data) -> Result<bool, RuntimeError<Data::Error>> {
        Ok(false)
    }

    /// Called during any operation where the types given don't have defined functionality.
    /// Such as a [`ExpressionDataType::List`] and a [`ExpressionDataType::Number`] in an [`Instruction::Add`] operation
    ///
    /// Return Ok(true) to tell the runtime this operation was handled
    ///
    /// Return Ok(false) to tell the runtime this operation was not handled
    ///
    fn defer_op(
        &mut self,
        _runtime: &mut Data,
        _operation: Instruction,
        _left: (ExpressionDataType, Data::Size),
        _right: (ExpressionDataType, Data::Size),
    ) -> Result<bool, RuntimeError<Data::Error>> {
        Ok(false)
    }
}

/// Concrete object for when a runtime requires no context functionality. Can use constants [`EMPTY_CONTEXT`] and [`NO_CONTEXT`] if needed as type parameters.
pub struct EmptyContext {}

/// Constant instantiation of [`EmptyContext`].
pub const EMPTY_CONTEXT: EmptyContext = EmptyContext {};
/// An [`Option`] set to None. Typed as a mutable reference to an [`EmptyContext`] for use in [`crate::GarnishRuntime`] instruction methods that require a context be passed.
pub const NO_CONTEXT: Option<&mut EmptyContext> = None;

impl<Data> GarnishLangRuntimeContext<Data> for EmptyContext where Data: GarnishLangRuntimeData {}
