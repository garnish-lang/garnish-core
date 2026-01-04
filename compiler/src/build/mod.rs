mod build;

pub use build::*;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialOrd, Eq, PartialEq, Clone)]
pub struct InstructionMetadata {
    parse_node_index: Option<usize>,
}

impl InstructionMetadata {
    fn new(parse_node_index: Option<usize>) -> Self {
        InstructionMetadata { parse_node_index }
    }

    pub fn get_parse_node_index(&self) -> Option<usize> {
        self.parse_node_index
    }
}
