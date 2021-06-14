mod pow;

use block::Block;

/// Abstraction for different consensus algorithm
pub trait Consensus {
    type Block: Block;

    /// Seal the block to reach a correct state.
    /// The return result indicates whether the seal is successful.
    fn seal(&mut self) -> bool;
    /// Validate the correct data
    fn validate(&self) -> bool;
    /// Get the block
    fn block(&self) -> Self::Block;
}
