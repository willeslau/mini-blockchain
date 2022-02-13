//! Return data structures

use common::U256;

/// Return data buffer. Holds memory from a previous call and a slice into that memory.
#[derive(Debug)]
pub struct ReturnData {
    mem: Vec<u8>,
    offset: usize,
    size: usize,
}

impl ::std::ops::Deref for ReturnData {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.mem[self.offset..self.offset + self.size]
    }
}

impl ReturnData {
    /// Create empty `ReturnData`.
    pub fn empty() -> Self {
        ReturnData {
            mem: Vec::new(),
            offset: 0,
            size: 0,
        }
    }
    /// Create `ReturnData` from give buffer and slice.
    pub fn new(mem: Vec<u8>, offset: usize, size: usize) -> Self {
        ReturnData { mem, offset, size }
    }
}

/// Gas Left: either it is a known value, or it needs to be computed by processing
/// a return instruction.
#[derive(Debug)]
pub enum GasLeft {
    /// Known gas left
    Known(U256),
    /// Return or Revert instruction must be processed.
    NeedsReturn {
        /// Amount of gas left.
        gas_left: U256,
        /// Return data buffer.
        data: ReturnData,
        /// Apply or revert state changes on revert.
        apply_state: bool,
    },
}
