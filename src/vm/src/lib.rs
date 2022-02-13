mod access_list;
mod env_info;
mod error;
mod ext;
mod return_data;
mod schedule;
mod tests;

pub use access_list::*;
pub use env_info::*;
pub use error::*;
pub use ext::*;
pub use return_data::*;
pub use schedule::*;

pub use tests::*;

/// Virtual Machine interface
pub trait Exec {
    /// This function should be used to execute transaction.
    /// It returns either an error, a known amount of gas left, or parameters to be used
    /// to compute the final gas left.
    fn exec(self: Box<Self>, ext: &mut dyn Ext) -> Result<GasLeft, Error>;
}