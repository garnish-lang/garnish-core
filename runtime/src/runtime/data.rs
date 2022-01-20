use crate::{ExpressionDataType, Instruction};
use std::fmt::{Debug, Display};
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::str::FromStr;

pub trait TypeConstants {
    fn zero() -> Self;
    fn one() -> Self;
    fn max_value() -> Self;
}

pub trait GarnishNumber: Sized {
    fn add(self, rhs: Self) -> Option<Self>;
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

pub trait GarnishLangRuntimeData {
    type Error: std::error::Error + 'static;
    type DataLease: Copy;
    type Symbol: Display + Debug + PartialOrd + TypeConstants + Copy;
    type Byte: Display + Debug + PartialOrd + Ord + Copy;
    type Char: Display + Debug + PartialOrd + Ord + Copy;
    type Number: Display + Debug + PartialOrd + Ord + TypeConstants + Copy + GarnishNumber + FromStr;
    type Size: Display + Debug + Add<Output = Self::Size> + AddAssign + SubAssign + Sub<Output = Self::Size> + PartialOrd + TypeConstants + Copy;

    fn get_data_len(&self) -> Self::Size;

    fn get_value_stack_len(&self) -> Self::Size;
    fn push_value_stack(&mut self, addr: Self::Size) -> Result<(), Self::Error>;
    fn pop_value_stack(&mut self) -> Option<Self::Size>;
    fn get_value(&self, addr: Self::Size) -> Option<Self::Size>;
    fn get_value_mut(&mut self, addr: Self::Size) -> Option<&mut Self::Size>;
    fn get_current_value(&self) -> Option<Self::Size>;
    fn get_current_value_mut(&mut self) -> Option<&mut Self::Size>;

    fn get_data_type(&self, addr: Self::Size) -> Result<ExpressionDataType, Self::Error>;

    fn get_number(&self, addr: Self::Size) -> Result<Self::Number, Self::Error>;
    fn get_char(&self, addr: Self::Size) -> Result<Self::Char, Self::Error>;
    fn get_byte(&self, addr: Self::Size) -> Result<Self::Byte, Self::Error>;
    fn get_symbol(&self, addr: Self::Size) -> Result<Self::Symbol, Self::Error>;
    fn get_expression(&self, addr: Self::Size) -> Result<Self::Size, Self::Error>;
    fn get_external(&self, addr: Self::Size) -> Result<Self::Size, Self::Error>;
    fn get_pair(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error>;
    fn get_range(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error>;
    fn get_slice(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size), Self::Error>;
    fn get_link(&self, addr: Self::Size) -> Result<(Self::Size, Self::Size, bool), Self::Error>;

    fn get_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error>;
    fn get_list_item(&self, list_addr: Self::Size, item_addr: Self::Number) -> Result<Self::Size, Self::Error>;
    fn get_list_associations_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error>;
    fn get_list_association(&self, list_addr: Self::Size, item_addr: Self::Number) -> Result<Self::Size, Self::Error>;
    fn get_list_item_with_symbol(&self, list_addr: Self::Size, sym: Self::Symbol) -> Result<Option<Self::Size>, Self::Error>;

    fn get_char_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error>;
    fn get_char_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Self::Char, Self::Error>;

    fn get_byte_list_len(&self, addr: Self::Size) -> Result<Self::Size, Self::Error>;
    fn get_byte_list_item(&self, addr: Self::Size, item_index: Self::Number) -> Result<Self::Byte, Self::Error>;

    fn add_unit(&mut self) -> Result<Self::Size, Self::Error>;
    fn add_true(&mut self) -> Result<Self::Size, Self::Error>;
    fn add_false(&mut self) -> Result<Self::Size, Self::Error>;

    fn add_number(&mut self, value: Self::Number) -> Result<Self::Size, Self::Error>;
    fn add_char(&mut self, value: Self::Char) -> Result<Self::Size, Self::Error>;
    fn add_byte(&mut self, value: Self::Byte) -> Result<Self::Size, Self::Error>;
    fn add_symbol(&mut self, value: &str) -> Result<Self::Size, Self::Error>;
    fn add_expression(&mut self, value: Self::Size) -> Result<Self::Size, Self::Error>;
    fn add_external(&mut self, value: Self::Size) -> Result<Self::Size, Self::Error>;
    fn add_pair(&mut self, value: (Self::Size, Self::Size)) -> Result<Self::Size, Self::Error>;
    fn add_range(&mut self, start: Self::Size, end: Self::Size) -> Result<Self::Size, Self::Error>;
    fn add_slice(&mut self, list: Self::Size, range: Self::Size) -> Result<Self::Size, Self::Error>;
    fn add_link(&mut self, value: Self::Size, linked: Self::Size, is_append: bool) -> Result<Self::Size, Self::Error>;

    fn start_list(&mut self, len: Self::Size) -> Result<(), Self::Error>;
    fn add_to_list(&mut self, addr: Self::Size, is_associative: bool) -> Result<(), Self::Error>;
    fn end_list(&mut self) -> Result<Self::Size, Self::Error>;

    fn start_char_list(&mut self) -> Result<(), Self::Error>;
    fn add_to_char_list(&mut self, c: Self::Char) -> Result<(), Self::Error>;
    fn end_char_list(&mut self) -> Result<Self::Size, Self::Error>;

    fn start_byte_list(&mut self) -> Result<(), Self::Error>;
    fn add_to_byte_list(&mut self, c: Self::Byte) -> Result<(), Self::Error>;
    fn end_byte_list(&mut self) -> Result<Self::Size, Self::Error>;

    fn get_register_len(&self) -> Self::Size;
    fn push_register(&mut self, addr: Self::Size) -> Result<(), Self::Error>;
    fn get_register(&self, addr: Self::Size) -> Option<Self::Size>;
    fn pop_register(&mut self) -> Option<Self::Size>;

    fn get_instruction_len(&self) -> Self::Size;
    fn push_instruction(&mut self, instruction: Instruction, data: Option<Self::Size>) -> Result<Self::Size, Self::Error>;
    fn get_instruction(&self, addr: Self::Size) -> Option<(Instruction, Option<Self::Size>)>;

    fn get_instruction_cursor(&self) -> Self::Size;
    fn set_instruction_cursor(&mut self, addr: Self::Size) -> Result<(), Self::Error>;

    fn get_jump_table_len(&self) -> Self::Size;
    fn push_jump_point(&mut self, index: Self::Size) -> Result<(), Self::Error>;
    fn get_jump_point(&self, index: Self::Size) -> Option<Self::Size>;
    fn get_jump_point_mut(&mut self, index: Self::Size) -> Option<&mut Self::Size>;

    fn push_jump_path(&mut self, index: Self::Size) -> Result<(), Self::Error>;
    fn pop_jump_path(&mut self) -> Option<Self::Size>;

    // deferred conversions
    fn size_to_integer(from: Self::Size) -> Self::Number;
    fn integer_to_char(from: Self::Number) -> Option<Self::Char>;
    fn integer_to_byte(from: Self::Number) -> Option<Self::Byte>;
    fn char_to_integer(from: Self::Char) -> Option<Self::Number>;
    fn char_to_byte(from: Self::Char) -> Option<Self::Byte>;
    fn byte_to_integer(from: Self::Byte) -> Option<Self::Number>;
    fn byte_to_char(from: Self::Byte) -> Option<Self::Char>;

    // mut conversions
    fn add_char_list_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error>;
    fn add_byte_list_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error>;
    fn add_symbol_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error>;
    fn add_byte_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error>;
    fn add_number_from(&mut self, from: Self::Size) -> Result<Self::Size, Self::Error>;

    // parsing, to be moved to separate object
    // will require moving simple data to its own crate
    fn parse_char_list(from: &str) -> Vec<Self::Char>;
    fn parse_byte_list(from: &str) -> Vec<Self::Byte>;

    // data lease methods
    fn lease_tmp_stack(&mut self) -> Result<Self::DataLease, Self::Error>;
    fn push_tmp_stack(&mut self, lease: Self::DataLease, item: Self::Size) -> Result<(), Self::Error>;
    fn pop_tmp_stack(&mut self, lease: Self::DataLease) -> Result<Option<Self::Size>, Self::Error>;
    fn release_tmp_stack(&mut self, lease: Self::DataLease) -> Result<(), Self::Error>;
}

impl GarnishNumber for i32 {
    fn add(self, rhs: Self) -> Option<Self> {
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
