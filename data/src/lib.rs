pub use error::DataError;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

mod clone;
mod data;
mod error;
mod instruction;
mod runtime;
mod simple;
mod basic;

pub use data::*;
pub use instruction::SimpleInstruction;
pub use simple::*;
pub use basic::*;

/// Utility to convert strings to [`u64`], the Symbol type for [`SimpleGarnishData`].
pub fn symbol_value(value: &str) -> u64 {
    let mut h = DefaultHasher::new();
    value.hash(&mut h);
    let hv = h.finish();

    hv
}
