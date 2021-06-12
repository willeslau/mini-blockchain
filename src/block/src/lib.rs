mod simple;
mod block_builder;
pub use simple::{SimpleBlock, SimpleBlockId };

pub trait Block {
    type Header;
    type Hash;
    type Executable;

    fn new(header: Self::Header, executables: Vec<Self::Executable>) -> Self;

    fn set_previous_hash(&mut self, hash: Self::Hash);
    fn get_previous_hash(&self) -> Self::Hash;
}

pub trait Header {
    fn new() -> Self;
}
