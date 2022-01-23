pub use crypto::keypair::*;
pub use crypto::ecdh::*;
pub use crypto::ecies::*;

pub use crate::error::*;
pub use crate::hash::*;
pub use crate::helper::*;
pub use crate::num::*;
#[cfg(any(feature = "std"))]
pub use crate::serialization::{from_vec, to_vec};

mod hash;
mod helper;

#[cfg(any(feature = "std"))]
mod serialization;
mod error;
mod num;
mod crypto;

use lazy_static::lazy_static;

lazy_static! {
	static ref SECP256K1: secp256k1::Secp256k1<secp256k1::All> = secp256k1::Secp256k1::new();
}