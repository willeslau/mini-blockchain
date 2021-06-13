use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use primitives::StringSerializable;
use transaction::MockTransaction;

use crate::{Block, Header};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct SimpleHeader {
    block_number: u64,
    version: u8,
    previous_hash: [u8; 32],
    hash: [u8; 32],
    merkle_root: [u8; 32],
    nonce: u32,
    timestamp: u128,
}

impl Header for SimpleHeader {
    type BlockNumber = u64;

    fn new() -> Self {
        SimpleHeader{
            block_number: 0,
            version: 0,
            previous_hash: [0; 32],
            hash: [0; 32],
            merkle_root: [0; 32],
            nonce: 0,
            timestamp:  SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis(),
        }
    }

    fn block_number(&self) -> Self::BlockNumber {
        self.block_number
    }
}

pub type SimpleBlockId = u64;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SimpleBlock {
    header: SimpleHeader,
    executables: Vec<MockTransaction>
}

/// Make sure the block can be converted to str
impl StringSerializable for SimpleBlock {
    fn serialize(&self) -> Box<str> {
        Box::from(serde_json::to_string(self).unwrap())
    }

    fn deserialize(data: &str) -> Self {
        serde_json::from_str(data).unwrap()
    }
}

impl Block for SimpleBlock {
    type Header = SimpleHeader;
    type Hash = [u8; 32];
    type Executable = MockTransaction;

    fn new(header: SimpleHeader, executables: Vec<MockTransaction>) -> Self {
        SimpleBlock{ header, executables }
    }

    fn set_previous_hash(&mut self, _hash: Self::Hash) {
    }

    fn get_previous_hash(&self,) -> Self::Hash {
        Self::Hash::default()
    }

    fn executables(&self) -> Vec<Self::Executable> {
        self.executables.clone()
    }

    fn header(&self) -> Self::Header {
        self.header.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::{SimpleBlock, Block, Header};
    use crate::simple::SimpleHeader;
    use transaction::MockTransaction;
    use primitives::StringSerializable;

    #[test]
    fn block_serialization_works() {
        let simple_block = SimpleBlock::new(
            SimpleHeader::new(),
            vec![MockTransaction::new("this is a test".parse().unwrap())]
        );

        let s = simple_block.serialize();
        SimpleBlock::deserialize(&s);
    }
}