use transaction::{MockTransaction};
use std::time::SystemTime;
use crate::{Header, Block};

#[derive(Copy, Clone)]
pub struct SimpleHeader {
    version: u8,
    previous_hash: [u8; 32],
    hash: [u8; 32],
    merkle_root: [u8; 32],
    nonce: u32,
    timestamp: u128,
}

impl Header for SimpleHeader {
    fn new() -> Self {
        SimpleHeader{
            version: 0,
            previous_hash: [0; 32],
            hash: [0; 32],
            merkle_root: [0; 32],
            nonce: 0,
            timestamp:  SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis(),
        }
    }
}

pub type SimpleBlockId = u64;

#[derive(Clone)]
pub struct SimpleBlock {
    header: SimpleHeader,
    executables: Vec<MockTransaction>
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
}