#![feature(exclusive_range_pattern)]

mod traits;
mod rlp;
mod impls;

pub use crate::rlp::RLPStream;
pub use crate::traits::{Encodable, Decodable};

