use expr_lang_common::{ExpressionValue, ExpressionValueRef};

pub trait ExpressionContext {
    fn resolve(&self, name: String) -> ExpressionValue;
    fn execute(&self, name: String, input: ExpressionValueRef) -> ExpressionValue;
}

pub struct DefaultContext {}

pub fn default_expression_context() -> DefaultContext {
    DefaultContext {}
}

impl ExpressionContext for DefaultContext {
    fn resolve(&self, _name: String) -> ExpressionValue {
        ExpressionValue::unit().into()
    }

    fn execute(&self, _name: String, _input: ExpressionValueRef) -> ExpressionValue {
        ExpressionValue::unit().into()
    }
}
