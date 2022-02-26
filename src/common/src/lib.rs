pub use crypto::keypair::*;
pub use crypto::ecdh::*;
pub use crypto::ecies::*;

pub use crate::error::*;
pub use crate::hash::*;
pub use crate::helper::*;
pub use crate::num::*;
pub use crate::uint::*;

pub type Address = H160;

#[cfg(any(feature = "std"))]
pub use crate::serialization::{from_vec, to_vec};

mod hash;
mod helper;

#[cfg(any(feature = "std"))]
mod serialization;
mod error;
mod num;
mod crypto;
mod uint;

use lazy_static::lazy_static;

lazy_static! {
	static ref SECP256K1: secp256k1::Secp256k1<secp256k1::All> = secp256k1::Secp256k1::new();
}

/// Get the KECCAK (i.e. Keccak) hash of the empty bytes string.
pub const KECCAK_EMPTY: H256 = H256([
	0xc5, 0xd2, 0x46, 0x01, 0x86, 0xf7, 0x23, 0x3c, 0x92, 0x7e, 0x7d, 0xb2, 0xdc, 0xc7, 0x03, 0xc0, 0xe5, 0x00, 0xb6,
	0x53, 0xca, 0x82, 0x27, 0x3b, 0x7b, 0xfa, 0xd8, 0x04, 0x5d, 0x85, 0xa4, 0x70,
]);