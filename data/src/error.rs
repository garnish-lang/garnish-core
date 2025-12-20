use std::fmt::{Debug, Display, Formatter};

use garnish_lang_traits::GarnishDataType;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum DataErrorType {
    Unknown,
    InvalidDataIndex(usize),
    InvalidListItemIndex(usize, usize),
    InvalidCharListItemIndex(usize, usize),
    ExceededInitialListLength(usize),
    NotFullyInitializedList(usize, usize),
    NotASymbolListPart(GarnishDataType),
    NotType(GarnishDataType, GarnishDataType),
    NotAssociativeItem,
    NotBasicType,
    NotAListItem(GarnishDataType),
    InstructionBlockExceededMaxItems(usize, usize),
    JumpTableBlockExceededMaxItems(usize, usize),
    DataBlockExceededMaxItems(usize, usize),
    CouldNotParse(String, GarnishDataType),
    NumberToLargeForByteValue(String),
    FailedToParseFloat(String),
}

/// Error implemenation for [`crate::SimpleGarnishData`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct DataError {
    message: String,
    error_type: DataErrorType,
}

impl DataError {
    pub fn new(message: &str, error_type: DataErrorType) -> Self {
        DataError { message: message.to_string(), error_type }
    }

    pub fn not_type_error(expected: GarnishDataType, got: GarnishDataType) -> Self {
        DataError::new("Not of type", DataErrorType::NotType(expected, got))
    }

    pub fn not_basic_type_error() -> Self {
        DataError::new("Not a basic type", DataErrorType::NotBasicType)
    }
}

impl Display for DataError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message.as_str())
    }
}

impl std::error::Error for DataError {}

impl From<&str> for DataError {
    fn from(s: &str) -> Self {
        DataError { message: s.to_string(), error_type: DataErrorType::Unknown }
    }
}

impl From<String> for DataError {
    fn from(s: String) -> Self {
        DataError { message: s, error_type: DataErrorType::Unknown }
    }
}

impl From<DataError> for String {
    fn from(err: DataError) -> Self {
        format!("{}", err)
    }
}
