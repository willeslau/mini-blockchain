mod mocked;
pub use mocked::{MockTransaction};

pub trait Executable {
    /// Checks if the executable is valid for execution
    fn is_valid() -> bool;
    /// Execute the executable
    fn execute() -> Result<(), ()>;
}
