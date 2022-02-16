mod env_info;
mod ext;
mod return_data;
mod schedule;

#[cfg(test)]
mod tests;

mod access_list;

#[cfg(test)]
pub use tests::*;

use crate::error::Error;
pub use ext::*;
pub use return_data::*;
pub use schedule::*;

pub type Bytes = Vec<u8>;

/// Virtual Machine interface
pub trait Exec {
    /// This function should be used to execute transaction.
    /// It returns either an error, a known amount of gas left, or parameters to be used
    /// to compute the final gas left.
    fn exec(&mut self, ext: &mut dyn Ext) -> Result<GasLeft, Error>;
}
