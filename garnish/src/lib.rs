//! This library does not contain any of its own functionality. Its available as a single place to find documentation for Garnish core libraries and as a convenience single dependency.
//!

pub use garnish_lang_traits::*;

pub mod simple {
    //! Re-exports for concrete implementations of garnish traits
    pub use garnish_lang_runtime::*;
    pub use garnish_lang_simple_data::*;
}

pub mod compiler {
    //! Re-exports for parsing and building garnish scripts.
    pub use garnish_lang_compiler::*;
}