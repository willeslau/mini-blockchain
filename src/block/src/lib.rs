use primitives::StringSerializable;
pub use simple::{SimpleBlock, SimpleBlockId, SimpleHeader};

mod simple;

pub trait Block: StringSerializable {
    type Header: Header;
    type Hash;
    type Executable: StringSerializable;

    fn new(header: Self::Header, executables: Vec<Self::Executable>) -> Self;
    fn set_previous_hash(&mut self, hash: Self::Hash);
    fn get_previous_hash(&self) -> Self::Hash;
    /// Get the list of executables as Vec
    fn executables(&self) -> Vec<Self::Executable>;
    /// Get the header of the block
    fn header(&self) -> Self::Header;
}

pub trait Header {
    type BlockNumber;

    fn new() -> Self;

    /// Get the block number
    fn block_number(&self) -> Self::BlockNumber;
}
