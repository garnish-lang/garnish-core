
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
pub struct SimpleStackFrame {
    return_addr: usize,
    previous_frame: usize,
}

impl SimpleStackFrame {
    pub fn new(return_addr: usize, previous_frame: usize) -> Self {
        SimpleStackFrame { return_addr, previous_frame }
    }

    pub fn return_addr(&self) -> usize {
        self.return_addr
    }

    pub fn previous_frame(&self) -> usize {
        self.previous_frame
    }
}