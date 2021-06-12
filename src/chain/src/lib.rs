mod in_memory;

use block::Block;
use num_traits::ops::checked::CheckedAdd;

pub trait BlockChain {
    // TODO: maybe we can use deref?
    type Block: Block + Clone;
    type BlockId: CheckedAdd;

    /// Get the genesis block
    fn genesis_block(&self) -> Self::Block;
    /// Insert new block
    fn insert(&mut self, block: Self::Block);
    /// Find the block by the block id
    fn find_block_by_id(&self, block_id: Self::BlockId) -> Option<Self::Block>;
}
