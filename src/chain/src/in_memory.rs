use crate::BlockChain;
use block::{Block, SimpleBlock, SimpleBlockId};
use std::collections::BTreeMap;
use num_traits::CheckedAdd;

pub struct InMemoryBlockChain {
    blocks: Vec<SimpleBlock>,
}

impl InMemoryBlockChain {
    pub fn new(genesis: SimpleBlock) -> Self {
        let mut blocks = Vec::new();
        blocks.push(genesis);
        InMemoryBlockChain {blocks}
    }
}

impl BlockChain for InMemoryBlockChain {
    type Block = SimpleBlock;
    type BlockId = SimpleBlockId;

    fn genesis_block(&self) -> Self::Block {
        let block = self.blocks.get(0).unwrap();
        (*block).clone()
    }

    fn insert(&mut self, mut block: Self::Block) {
        let s = self.blocks.len();
        let last_block = self.blocks.get(s-1).unwrap();
        block.set_previous_hash(last_block.get_previous_hash());
        self.blocks.push(block);
    }

    fn find_block_by_id(&self, block_id: Self::BlockId) -> Option<Self::Block> {
        if let Some(v) = self.blocks.get(block_id) {
            return Some((*v).clone())
        }
        None
    }
}