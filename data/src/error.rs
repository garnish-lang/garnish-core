use std::{backtrace::{Backtrace, BacktraceStatus}, cmp::Ordering, fmt::{Debug, Display, Formatter}};

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
        write!(f, "DataError {{ ")?;
        write!(f, "message: \"{}\", ", self.message)?;
        write!(f, "error_type: {}", format_error_type(&self.error_type))?;
        match self.backtrace.status() {
            BacktraceStatus::Captured => write!(f, "\n{}", self.backtrace)?,
            _ => {}
        }
        write!(f, " }}\n")?;
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

fn format_error_type(error_type: &DataErrorType) -> String {
    match error_type {
        DataErrorType::Unknown => "Unknown error".to_string(),
        DataErrorType::InvalidDataIndex(index) => format!("Invalid data index: {}", index),
        DataErrorType::InvalidInstructionIndex(index) => format!("Invalid instruction index: {}", index),
        DataErrorType::InvalidJumpTableIndex(index) => format!("Invalid jump table index: {}", index),
        DataErrorType::InvalidSymbolTableIndex(index) => format!("Invalid symbol table index: {}", index),
        DataErrorType::InvalidListItemIndex(list_index, item_index) => {
            format!("Invalid list item index: list={}, item={}", list_index, item_index)
        }
        DataErrorType::InvalidCharListItemIndex(list_index, item_index) => {
            format!("Invalid char list item index: list={}, item={}", list_index, item_index)
        }
        DataErrorType::ExceededInitialListLength(length) => {
            format!("Exceeded initial list length: {}", length)
        }
        DataErrorType::NotFullyInitializedList(expected, actual) => {
            format!("List not fully initialized: expected {} items, got {}", expected, actual)
        }
        DataErrorType::NotASymbolListPart(got_type) => {
            format!("Not a symbol list part, got type: {:?}", got_type)
        }
        DataErrorType::NotType(expected, got) => {
            format!("Type mismatch: expected {:?}, got {:?}", expected, got)
        }
        DataErrorType::NotAssociativeItem(got_type) => {
            format!("Not an associative item, got type: {:?}", got_type)
        }
        DataErrorType::NotBasicType => "Not a basic type".to_string(),
        DataErrorType::NotAListItem(got_type) => {
            format!("Not a list item, got type: {:?}", got_type)
        }
        DataErrorType::InstructionBlockExceededMaxItems(current, max) => {
            format!("Instruction block exceeded max items: {} > {}", current, max)
        }
        DataErrorType::JumpTableBlockExceededMaxItems(current, max) => {
            format!("Jump table block exceeded max items: {} > {}", current, max)
        }
        DataErrorType::SymbolTableBlockExceededMaxItems(current, max) => {
            format!("Symbol table block exceeded max items: {} > {}", current, max)
        }
        DataErrorType::ExpressionSymbolBlockExceededMaxItems(current, max) => {
            format!("Expression symbol block exceeded max items: {} > {}", current, max)
        }
        DataErrorType::DataBlockExceededMaxItems(current, max) => {
            format!("Data block exceeded max items: {} > {}", current, max)
        }
        DataErrorType::CouldNotParse(value, target_type) => {
            format!("Could not parse \"{}\" as {:?}", value, target_type)
        }
        DataErrorType::NumberToLargeForByteValue(value) => {
            format!("Number too large for byte value: {}", value)
        }
        DataErrorType::FailedToParseFloat(value) => {
            format!("Failed to parse float: \"{}\"", value)
        }
        DataErrorType::NotACloneNode => "Not a clone node".to_string(),
        DataErrorType::NoMappedIndexFoundDuringClone(index) => {
            format!("No mapped index found during clone: {}", index)
        }
        DataErrorType::UninitializedListContainsNonListItem(got_type) => {
            format!("Uninitialized list contains non-list item, got type: {:?}", got_type)
        }
        DataErrorType::CannotClone => "Cannot clone".to_string(),
        DataErrorType::CloneLimitReached => "Clone limit reached".to_string(),
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
        write!(f, "{}", self.message)?;
        let error_details = format_error_type(&self.error_type);
        if !error_details.is_empty() {
            write!(f, " ({})", error_details)?;
        }
        Ok(())
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
