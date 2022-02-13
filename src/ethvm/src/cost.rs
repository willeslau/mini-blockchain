//! Evm interface.

use crate::error::Error;
use common::{U128, U256, U512};
use std::{cmp, fmt, ops};

/// Cost calculation type. For low-gas usage we calculate costs using usize instead of U256
pub trait CostType:
    Sized
    + From<usize>
    + Copy
    + Send
    + ops::Mul<Output = Self>
    + ops::Div<Output = Self>
    + ops::Add<Output = Self>
    + ops::Sub<Output = Self>
    + ops::Shr<usize, Output = Self>
    + ops::Shl<usize, Output = Self>
    + cmp::Ord
    + fmt::Debug
{
    /// Converts this cost into `U256`
    fn as_u256(&self) -> U256;
    /// Tries to fit `U256` into this `Cost` type
    fn from_u256(val: U256) -> Result<Self, Error>;
    /// Convert to usize (may panic)
    fn as_usize(&self) -> usize;
    /// Add with overflow
    fn overflow_add(self, other: Self) -> (Self, bool);
    /// Multiple with overflow
    fn overflow_mul(self, other: Self) -> (Self, bool);
    /// Single-step full multiplication and shift: `(self*other) >> shr`
    /// Should not overflow on intermediate steps
    fn overflow_mul_shr(self, other: Self, shr: usize) -> (Self, bool);
}

impl CostType for usize {
    fn as_u256(&self) -> U256 {
        U256::from(*self)
    }

    fn from_u256(val: U256) -> Result<Self, Error> {
        let res = val.low_u64() as usize;

        // validate if value fits into usize
        if U256::from(res) != val {
            return Err(Error::OutOfGas);
        }

        Ok(res)
    }

    fn as_usize(&self) -> usize {
        *self
    }

    fn overflow_add(self, other: Self) -> (Self, bool) {
        self.overflowing_add(other)
    }

    fn overflow_mul(self, other: Self) -> (Self, bool) {
        self.overflowing_mul(other)
    }

    fn overflow_mul_shr(self, other: Self, shr: usize) -> (Self, bool) {
        let (c, o) = U128::from(self).overflowing_mul(U128::from(other));
        let U128(parts) = c;
        let overflow = o | (parts[1] > 0);
        let U128(parts) = c >> shr;
        let result = parts[0] as usize;
        let overflow = overflow | (parts[0] > result as u64);
        (result, overflow)
    }
}
