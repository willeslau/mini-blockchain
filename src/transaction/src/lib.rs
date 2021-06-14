mod mocked;
pub use mocked::{MockedExecutable};
use primitives::StringSerializable;

pub trait Executable: StringSerializable + Clone + Send {
    /// Checks if the executable is valid for execution
    fn is_valid(&self) -> bool;
    /// Execute the executable
    fn execute(&self) -> Result<(), ()>;
}
