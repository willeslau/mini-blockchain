#![feature(exclusive_range_pattern)]

mod traits;
mod rlp;
mod impls;
mod rlpin;
mod error;

pub use crate::error::Error;
pub use crate::rlp::RLPStream;
pub use crate::rlpin::Rlp;
pub use crate::traits::{Encodable, Decodable};

