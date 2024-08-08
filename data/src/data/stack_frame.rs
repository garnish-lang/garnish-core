
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
pub struct SimpleStackFrame {
    return_addr: usize,
}

impl SimpleStackFrame {
    pub fn new(return_addr: usize) -> Self {
        SimpleStackFrame { return_addr }
    }

    pub fn return_addr(&self) -> usize {
        self.return_addr
    }

}