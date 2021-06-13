mod mocked;
pub use mocked::{MockTransaction};
use primitives::StringSerializable;

pub trait Executable: StringSerializable {
    /// Checks if the executable is valid for execution
    fn is_valid() -> bool;
    /// Execute the executable
    fn execute() -> Result<(), ()>;
}
