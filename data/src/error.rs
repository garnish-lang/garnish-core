use std::{backtrace::Backtrace, cmp::Ordering, fmt::{Debug, Display, Formatter}};

use garnish_lang_traits::GarnishDataType;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum DataErrorType {
    Unknown,
    InvalidDataIndex(usize),
    InvalidInstructionIndex(usize),
    InvalidJumpTableIndex(usize),
    InvalidSymbolTableIndex(usize),
    InvalidListItemIndex(usize, usize),
    InvalidCharListItemIndex(usize, usize),
    ExceededInitialListLength(usize),
    NotFullyInitializedList(usize, usize),
    NotASymbolListPart(GarnishDataType),
    NotType(GarnishDataType, GarnishDataType),
    NotAssociativeItem(GarnishDataType),
    NotBasicType,
    NotAListItem(GarnishDataType),
    InstructionBlockExceededMaxItems(usize, usize),
    JumpTableBlockExceededMaxItems(usize, usize),
    SymbolTableBlockExceededMaxItems(usize, usize),
    ExpressionSymbolBlockExceededMaxItems(usize, usize),
    DataBlockExceededMaxItems(usize, usize),
    CouldNotParse(String, GarnishDataType),
    NumberToLargeForByteValue(String),
    FailedToParseFloat(String),
    NotACloneNode,
    NoMappedIndexFoundDuringClone(usize),
    UninitializedListContainsNonListItem(GarnishDataType),
    CannotClone,
    CloneLimitReached,
}

/// Error implemenation for [`crate::SimpleGarnishData`].
pub struct DataError {
    message: String,
    error_type: DataErrorType,
    backtrace: Backtrace,
}

impl Debug for DataError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.error_type, self.message)?;
        write!(f, "\n{}", self.backtrace)?;
        Ok(())
    }
}

impl PartialEq for DataError {
    fn eq(&self, other: &Self) -> bool {
        self.message == other.message && self.error_type == other.error_type
    }
}

impl Eq for DataError {}

impl PartialOrd for DataError {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DataError {
    fn cmp(&self, other: &Self) -> Ordering {
        self.message.cmp(&other.message)
    }
}

impl DataError {
    pub fn new(message: &str, error_type: DataErrorType) -> Self {
        DataError { message: message.to_string(), error_type, backtrace: Backtrace::capture() }
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
        DataError { message: s.to_string(), error_type: DataErrorType::Unknown, backtrace: Backtrace::capture() }
    }
}

impl From<String> for DataError {
    fn from(s: String) -> Self {
        DataError { message: s, error_type: DataErrorType::Unknown, backtrace: Backtrace::capture() }
    }
}

impl From<DataError> for String {
    fn from(err: DataError) -> Self {
        format!("{}", err)
    }
}
