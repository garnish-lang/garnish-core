use crate::Instruction;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// List of Garnish data types.
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum GarnishDataType {
    Invalid = 0,
    Unit = 1,
    Number,
    Type,
    Char,
    CharList,
    Byte,
    ByteList,
    Symbol,
    SymbolList,
    Pair,
    Range,
    Concatenation,
    Slice,
    Partial,
    List,
    Expression,
    External,
    True,
    False,
    Custom,
}

/// Trait to provide constant values that a runtime will need access to.
pub trait TypeConstants {
    fn zero() -> Self;
    fn one() -> Self;
    fn max_value() -> Self;
}

/// Arithmetic operations required so a runtime can resolve associated instruction.
pub trait GarnishNumber: Sized {
    fn plus(self, rhs: Self) -> Option<Self>;
    fn subtract(self, rhs: Self) -> Option<Self>;
    fn multiply(self, rhs: Self) -> Option<Self>;
    fn divide(self, rhs: Self) -> Option<Self>;
    fn integer_divide(self, rhs: Self) -> Option<Self>;
    fn power(self, rhs: Self) -> Option<Self>;
    fn remainder(self, rhs: Self) -> Option<Self>;
    fn absolute_value(self) -> Option<Self>;
    fn opposite(self) -> Option<Self>;
    fn increment(self) -> Option<Self>;
    fn decrement(self) -> Option<Self>;
    fn bitwise_not(self) -> Option<Self>;
    fn bitwise_and(self, rhs: Self) -> Option<Self>;
    fn bitwise_or(self, rhs: Self) -> Option<Self>;
    fn bitwise_xor(self, rhs: Self) -> Option<Self>;
    fn bitwise_shift_left(self, rhs: Self) -> Option<Self>;
    fn bitwise_shift_right(self, rhs: Self) -> Option<Self>;
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Debug)]
pub enum SymbolListPart<A, B> {
    Symbol(A),
    Number(B)
}

#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub struct Extents<T> where T: Clone + PartialOrd + PartialEq + Debug {
    start: T,
    end: T,
}

impl<T> Extents<T> where T: Clone + PartialOrd + PartialEq + Debug {
    pub fn new(start: T, end: T) -> Self {
        Self { start, end }
    }
}

impl<T> Extents<T> where T: Clone + PartialOrd + PartialEq + Debug {
    pub fn start(&self) -> &T {
        &self.start
    }

    pub fn end(&self) -> &T {
        &self.end
    }
}

pub trait GarnishDataFactory<Size, Number, Char, Byte, Symbol, Error, SizeIterator, NumberIterator> {
    fn size_to_number(from: Size) -> Number;
    fn number_to_size(from: Number) -> Option<Size>;
    fn number_to_char(from: Number) -> Option<Char>;
    fn number_to_byte(from: Number) -> Option<Byte>;
    fn char_to_number(from: Char) -> Option<Number>;
    fn char_to_byte(from: Char) -> Option<Byte>;
    fn byte_to_number(from: Byte) -> Option<Number>;
    fn byte_to_char(from: Byte) -> Option<Char>;

    fn parse_number(from: &str) -> Result<Number, Error>;
    fn parse_symbol(from: &str) -> Result<Symbol, Error>;
    fn parse_char(from: &str) -> Result<Char, Error>;
    fn parse_byte(from: &str) -> Result<Byte, Error>;
    fn parse_char_list(from: &str) -> Result<Vec<Char>, Error>;
    fn parse_byte_list(from: &str) -> Result<Vec<Byte>, Error>;

    fn make_size_iterator_range(min: Size, max: Size) -> SizeIterator;
    fn make_number_iterator_range(min: Number, max: Number) -> NumberIterator;
}

/// Trait defining what a data access operations are required by a runtime.
pub trait GarnishData {
    type Error: std::error::Error + 'static;
    type Symbol: Default + Display + Debug + PartialOrd + TypeConstants + Clone;
    type Byte: Default + Display + Debug + PartialOrd + Clone;
    type Char: Default + Display + Debug + PartialOrd + Clone;
    type Number: Default + Display + Debug + PartialOrd + TypeConstants + Clone + GarnishNumber;
    type Size: Default
        + Display
        + Debug
        + Add<Output = Self::Size>
        + AddAssign
        + SubAssign
        + Sub<Output = Self::Size>
        + PartialOrd
        + TypeConstants
        + Clone;
    type SizeIterator: DoubleEndedIterator<Item = Self::Size>;
    type NumberIterator: DoubleEndedIterator<Item = Self::Number>;
    type InstructionIterator: DoubleEndedIterator<Item = Self::Size>;
    type DataIndexIterator: Iterator<Item = Self::Size>;
    type ValueIndexIterator: DoubleEndedIterator<Item = Self::Size>;
    type RegisterIndexIterator: DoubleEndedIterator<Item = Self::Size>;
    type JumpTableIndexIterator: DoubleEndedIterator<Item = Self::Size>;
    type JumpPathIndexIterator: DoubleEndedIterator<Item = Self::Size>;
    type ListIndexIterator: DoubleEndedIterator<Item = Self::Number>;
    type ListItemIterator: Iterator<Item = Self::Size>;
    type ConcatenationItemIterator: Iterator<Item = Self::Size>;
    type CharIterator: Iterator<Item = Self::Char>;
    type ByteIterator: Iterator<Item = Self::Byte>;
    type SymbolListPartIterator: Iterator<Item = SymbolListPart<Self::Symbol, Self::Number>>;
    type DataFactory: GarnishDataFactory<Self::Size, Self::Number, Self::Char, Self::Byte, Self::Symbol, Self::Error, Self::SizeIterator, Self::NumberIterator>;

    fn get_data_len(&self) -> Self::Size;
    fn get_data_iter(&self) -> Self::DataIndexIterator;

    fn get_value_stack_len(&self) -> Self::Size;
    fn push_value_stack(&mut self, addr: Self::Size) -> Result<(), Self::Error>;
    fn pop_value_stack(&mut self) -> Option<Self::Size>;
    fn get_value(&self, addr: Self::Size) -> Option<Self::Size>;
    fn get_value_mut(&mut self, addr: Self::Size) -> Option<&mut Self::Size>;
    fn get_current_value(&self) -> Option<Self::Size>;
    fn get_current_value_mut(&mut self) -> Option<&mut Self::Size>;
    fn get_value_iter(&self) -> Self::ValueIndexIterator;

    fn get_data_type(&self, addr: Self::Size) -> Result<GarnishDataType, Self::Error>;

    fn get_number(&self, addr: Self::Size) -> Result<Self::Number, Self::Error>;
    fn get_type(&self, addr: Self::Size) -> Result<GarnishDataType, Self::Error>;
    fn get_char(&self, addr: Self::Size) -> Result<Self::Char, Self::Error>;
    fn get_byte(&self, addr: Self::Size) -> Result<Self::Byte, Self::Error>;
    fn get_symbol(&self, addr: Self::Size) -> Result<Self::Symbol, Self::Error>;
    fn get_expression(&self, addr: Self::Size) -> Result<Self::Size, Self::Error>;
    fn get_external(&self, addr: Self::Size) -> Result<Self::Size, Self::Error>;
    fn get_pair(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error>;
    fn get_concatenation(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error>;
    fn get_range(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error>;
    fn get_slice(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error>;
    fn get_partial(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error>;

    fn get_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error>;
    fn get_list_item(&self, list_addr: Self::Size, item_addr: Self::Number) -> Result<Option<Self::Size>, Self::Error>;
    fn get_list_associations_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error>;
    fn get_list_association(&self, list_addr: Self::Size, item_addr: Self::Number) -> Result<Option<Self::Size>, Self::Error>;
    fn get_list_item_with_symbol(&self, list_addr: Self::Size, sym: Self::Symbol) -> Result<Option<Self::Size>, Self::Error>;

    fn get_char_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error>;
    fn get_char_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Option<Self::Char>, Self::Error>;

    fn get_byte_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error>;
    fn get_byte_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Option<Self::Byte>, Self::Error>;

    fn get_symbol_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error>;
    fn get_symbol_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Option<SymbolListPart<Self::Symbol, Self::Number>>, Self::Error>;

    fn get_char_list_iter(&self, list_addr: Self::Size, extents: Extents<Self::Number>) -> Result<Self::CharIterator, Self::Error>;
    fn get_byte_list_iter(&self, list_addr: Self::Size, extents: Extents<Self::Number>) -> Result<Self::ByteIterator, Self::Error>;
    fn get_symbol_list_iter(&self, list_addr: Self::Size, extents: Extents<Self::Number>) -> Result<Self::SymbolListPartIterator, Self::Error>;
    fn get_list_items_iter(&self, list_addr: Self::Size, extents: Extents<Self::Number>) -> Result<Self::ListIndexIterator, Self::Error>;
    fn get_list_associations_iter(&self, list_addr: Self::Size, extents: Extents<Self::Number>) -> Result<Self::ListIndexIterator, Self::Error>;
    fn get_list_item_iter(&self, list_addr: Self::Size, extents: Extents<Self::Number>) -> Result<Self::ListItemIterator, Self::Error>;
    fn get_concatenation_iter(&self, addr: Self::Size, extents: Extents<Self::Number>) -> Result<Self::ConcatenationItemIterator, Self::Error>;

    fn add_unit(&mut self) -> Result<Self::Size, Self::Error>;
    fn add_true(&mut self) -> Result<Self::Size, Self::Error>;
    fn add_false(&mut self) -> Result<Self::Size, Self::Error>;

    fn add_number(&mut self, value: Self::Number) -> Result<Self::Size, Self::Error>;
    fn add_type(&mut self, value: GarnishDataType) -> Result<Self::Size, Self::Error>;
    fn add_char(&mut self, value: Self::Char) -> Result<Self::Size, Self::Error>;
    fn add_byte(&mut self, value: Self::Byte) -> Result<Self::Size, Self::Error>;
    fn add_symbol(&mut self, value: Self::Symbol) -> Result<Self::Size, Self::Error>;
    fn add_expression(&mut self, value: Self::Size) -> Result<Self::Size, Self::Error>;
    fn add_external(&mut self, value: Self::Size) -> Result<Self::Size, Self::Error>;
    fn add_pair(&mut self, value: (Self::Size, Self::Size)) -> Result<Self::Size, Self::Error>;
    fn add_concatenation(&mut self, left: Self::Size, right: Self::Size) -> Result<Self::Size, Self::Error>;
    fn add_range(&mut self, start: Self::Size, end: Self::Size) -> Result<Self::Size, Self::Error>;
    fn add_slice(&mut self, list: Self::Size, range: Self::Size) -> Result<Self::Size, Self::Error>;
    fn add_partial(&mut self, reciever: Self::Size, input: Self::Size) -> Result<Self::Size, Self::Error>;

    fn merge_to_symbol_list(&mut self, first: Self::Size, second: Self::Size) -> Result<Self::Size, Self::Error>;

    fn start_list(&mut self, len: Self::Size) -> Result<Self::Size, Self::Error>;
    fn add_to_list(&mut self, list_index: Self::Size, item_index: Self::Size) -> Result<Self::Size, Self::Error>;
    fn end_list(&mut self, list_index: Self::Size) -> Result<Self::Size, Self::Error>;

    fn start_char_list(&mut self) -> Result<(), Self::Error>;
    fn add_to_char_list(&mut self, c: Self::Char) -> Result<(), Self::Error>;
    fn end_char_list(&mut self) -> Result<Self::Size, Self::Error>;

    fn start_byte_list(&mut self) -> Result<(), Self::Error>;
    fn add_to_byte_list(&mut self, c: Self::Byte) -> Result<(), Self::Error>;
    fn end_byte_list(&mut self) -> Result<Self::Size, Self::Error>;

    fn get_register_len(&self) -> Self::Size;
    fn push_register(&mut self, addr: Self::Size) -> Result<(), Self::Error>;
    fn get_register(&self, addr: Self::Size) -> Option<Self::Size>;
    fn pop_register(&mut self) -> Result<Option<Self::Size>, Self::Error>;
    fn get_register_iter(&self) -> Self::RegisterIndexIterator;

    fn get_instruction_len(&self) -> Self::Size;
    fn push_instruction(&mut self, instruction: Instruction, data: Option<Self::Size>) -> Result<Self::Size, Self::Error>;
    fn get_instruction(&self, addr: Self::Size) -> Option<(Instruction, Option<Self::Size>)>;
    fn get_instruction_iter(&self) -> Self::InstructionIterator;

    fn get_instruction_cursor(&self) -> Self::Size;
    fn set_instruction_cursor(&mut self, addr: Self::Size) -> Result<(), Self::Error>;

    fn get_jump_table_len(&self) -> Self::Size;
    fn push_to_jump_table(&mut self, index: Self::Size) -> Result<(), Self::Error>;
    fn get_from_jump_table(&self, index: Self::Size) -> Option<Self::Size>;
    fn get_from_jump_table_mut(&mut self, index: Self::Size) -> Option<&mut Self::Size>;
    fn get_jump_table_iter(&self) -> Self::JumpTableIndexIterator;

    fn push_jump_path(&mut self, index: Self::Size) -> Result<(), Self::Error>;
    fn pop_jump_path(&mut self) -> Option<Self::Size>;
    fn get_jump_path_iter(&self) -> Self::JumpPathIndexIterator;

    // mut conversions
    fn add_char_list_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error>;
    fn add_byte_list_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error>;
    fn add_symbol_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error>;
    fn add_byte_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error>;
    fn add_number_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error>;

    // parsing, to be moved to separate object
    // will require moving simple data to its own crate

    // parse and add to data
    fn parse_add_number(&mut self, from: &str) -> Result<Self::Size, Self::Error> {
        self.add_number(Self::DataFactory::parse_number(from)?)
    }

    fn parse_add_symbol(&mut self, from: &str) -> Result<Self::Size, Self::Error> {
        self.add_symbol(Self::DataFactory::parse_symbol(from)?)
    }

    fn parse_add_char(&mut self, from: &str) -> Result<Self::Size, Self::Error> {
        self.add_char(Self::DataFactory::parse_char(from)?)
    }

    fn parse_add_byte(&mut self, from: &str) -> Result<Self::Size, Self::Error> {
        self.add_byte(Self::DataFactory::parse_byte(from)?)
    }

    fn parse_add_char_list(&mut self, from: &str) -> Result<Self::Size, Self::Error> {
        self.start_char_list()?;
        for c in Self::DataFactory::parse_char_list(from)? {
            self.add_to_char_list(c)?;
        }
        self.end_char_list()
    }

    fn parse_add_byte_list(&mut self, from: &str) -> Result<Self::Size, Self::Error> {
        self.start_byte_list()?;
        for b in Self::DataFactory::parse_byte_list(from)? {
            self.add_to_byte_list(b)?;
        }
        self.end_byte_list()
    }

    // Execution checks

    /// Called during a [`Instruction::Resolve`], to convert a [`GarnishDataType::Symbol`] to a value.
    ///
    /// Return Ok(true) to tell the runtime that the symbol was resolved
    ///
    /// Return Ok(false) to let the runtime fill a default value, probably [`GarnishDataType::Unit`].
    ///
    #[allow(unused)]
    fn resolve(&mut self, symbol: Self::Symbol) -> Result<bool, Self::Error> {
        Ok(false)
    }

    /// Called when an [`GarnishDataType::External`] is on the left side of an [`Instruction::Apply`] operation.
    ///
    /// Return Ok(true) to tell the runtime this apply operation was handled
    ///
    /// Return Ok(false) to tell the runtime this apply operation was not handled
    ///
    #[allow(unused)]
    fn apply(&mut self, external_value: Self::Size, input_addr: Self::Size) -> Result<bool, Self::Error> {
        Ok(false)
    }

    /// Called during any operation where the types given don't have defined functionality.
    /// Such as a [`GarnishDataType::List`] and a [`GarnishDataType::Number`] in an [`Instruction::Add`] operation
    ///
    /// Return Ok(true) to tell the runtime this operation was handled
    ///
    /// Return Ok(false) to tell the runtime this operation was not handled
    ///
    #[allow(unused)]
    fn defer_op(
        &mut self,
        operation: Instruction,
        left: (GarnishDataType, Self::Size),
        right: (GarnishDataType, Self::Size),
    ) -> Result<bool, Self::Error> {
        Ok(false)
    }
}

impl GarnishNumber for i32 {
    fn plus(self, rhs: Self) -> Option<Self> {
        Some(self + rhs)
    }

    fn subtract(self, rhs: Self) -> Option<Self> {
        Some(self - rhs)
    }

    fn multiply(self, rhs: Self) -> Option<Self> {
        Some(self * rhs)
    }

    fn divide(self, rhs: Self) -> Option<Self> {
        Some(self / rhs)
    }

    fn integer_divide(self, rhs: Self) -> Option<Self> {
        Some(self / rhs)
    }

    fn power(self, rhs: Self) -> Option<Self> {
        Some(self.pow(rhs as u32))
    }

    fn remainder(self, rhs: Self) -> Option<Self> {
        Some(self % rhs)
    }

    fn absolute_value(self) -> Option<Self> {
        Some(self.abs())
    }

    fn opposite(self) -> Option<Self> {
        Some(-self)
    }

    fn increment(self) -> Option<Self> {
        Some(self + 1)
    }

    fn decrement(self) -> Option<Self> {
        Some(self - 1)
    }

    fn bitwise_not(self) -> Option<Self> {
        Some(!self)
    }

    fn bitwise_and(self, rhs: Self) -> Option<Self> {
        Some(self & rhs)
    }

    fn bitwise_or(self, rhs: Self) -> Option<Self> {
        Some(self | rhs)
    }

    fn bitwise_xor(self, rhs: Self) -> Option<Self> {
        Some(self ^ rhs)
    }

    fn bitwise_shift_left(self, rhs: Self) -> Option<Self> {
        Some(self << rhs)
    }

    fn bitwise_shift_right(self, rhs: Self) -> Option<Self> {
        Some(self >> rhs)
    }
}

impl TypeConstants for i8 {
    fn zero() -> Self {
        0
    }

    fn one() -> Self {
        1
    }

    fn max_value() -> Self {
        i8::MAX
    }
}

impl TypeConstants for i16 {
    fn zero() -> Self {
        0
    }

    fn one() -> Self {
        1
    }

    fn max_value() -> Self {
        i16::MAX
    }
}

impl TypeConstants for i32 {
    fn zero() -> Self {
        0
    }

    fn one() -> Self {
        1
    }

    fn max_value() -> Self {
        i32::MAX
    }
}

impl TypeConstants for i64 {
    fn zero() -> Self {
        0
    }

    fn one() -> Self {
        1
    }

    fn max_value() -> Self {
        i64::MAX
    }
}

impl TypeConstants for i128 {
    fn zero() -> Self {
        0
    }

    fn one() -> Self {
        1
    }

    fn max_value() -> Self {
        i128::MAX
    }
}

impl TypeConstants for u8 {
    fn zero() -> Self {
        0
    }

    fn one() -> Self {
        1
    }

    fn max_value() -> Self {
        u8::MAX
    }
}

impl TypeConstants for u16 {
    fn zero() -> Self {
        0
    }

    fn one() -> Self {
        1
    }

    fn max_value() -> Self {
        u16::MAX
    }
}

impl TypeConstants for u32 {
    fn zero() -> Self {
        0
    }

    fn one() -> Self {
        1
    }

    fn max_value() -> Self {
        u32::MAX
    }
}

impl TypeConstants for u64 {
    fn zero() -> Self {
        0
    }

    fn one() -> Self {
        1
    }

    fn max_value() -> Self {
        u64::MAX
    }
}

impl TypeConstants for u128 {
    fn zero() -> Self {
        0
    }

    fn one() -> Self {
        1
    }

    fn max_value() -> Self {
        u128::MAX
    }
}

impl TypeConstants for isize {
    fn zero() -> Self {
        0
    }

    fn one() -> Self {
        1
    }

    fn max_value() -> Self {
        isize::MAX
    }
}

impl TypeConstants for usize {
    fn zero() -> Self {
        0
    }

    fn one() -> Self {
        1
    }

    fn max_value() -> Self {
        usize::MAX
    }
}

impl TypeConstants for f32 {
    fn zero() -> Self {
        0.0
    }

    fn one() -> Self {
        1.0
    }

    fn max_value() -> Self {
        f32::MAX
    }
}

impl TypeConstants for f64 {
    fn zero() -> Self {
        0.0
    }

    fn one() -> Self {
        1.0
    }

    fn max_value() -> Self {
        f64::MAX
    }
}

impl TypeConstants for char {
    fn zero() -> Self {
        0 as char
    }

    fn one() -> Self {
        1 as char
    }

    fn max_value() -> Self {
        char::MAX
    }
}
