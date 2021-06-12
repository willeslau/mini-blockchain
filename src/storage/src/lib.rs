use block::Block;
use serde::Serialize;
use num_traits::CheckedAdd;

/// The storage for the blockchain
pub trait Storage {
    /// The block type
    type Block: Block + Serialize;
    /// The block id
    type BlockId: Serialize + CheckedAdd;

    /// Insert one block into the storage
    fn insert(block: Self::Block);
    /// Get the latest block in the storage
    fn get_latest_block() -> Self::Block;
    /// Find the block by the block id
    fn find_block_by_id(block_id: BlockId) -> Option<Self::Block>;
}
