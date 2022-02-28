use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use tiny_keccak::{Hasher as KeccakHasherTrait, Keccak};
use fixed_hash::construct_fixed_hash;
use fixed_hash::rustc_hex::FromHexError;
use crate::Error;
use crate::U256;

pub trait BigEndianHash {
	type Uint;
	fn from_uint(val: &Self::Uint) -> Self;
	fn into_uint(&self) -> Self::Uint;
}

pub const HASH_LENGTH: usize = 32;

macro_rules! impl_uint_conversions {
	($hash: ident, $uint: ident) => {
		impl BigEndianHash for $hash {
			type Uint = $uint;

			fn from_uint(value: &$uint) -> Self {
				let mut ret = $hash::zero();
				value.to_big_endian(ret.as_bytes_mut());
				ret
			}

			fn into_uint(&self) -> $uint {
				$uint::from(self.as_ref() as &[u8])
			}
		}
	};
}

construct_fixed_hash! {
    #[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
    pub struct H160(20);
}
construct_fixed_hash! {
    #[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
    pub struct H256(32);
}

construct_fixed_hash! { pub struct H520(65); }
construct_fixed_hash! { pub struct H512(64); }
construct_fixed_hash! { pub struct H128(16); }
construct_fixed_hash! { pub struct H64(8); }

/// Add RLP serialization support to a fixed-sized hash type created by `construct_fixed_hash!`.
#[macro_export]
macro_rules! impl_fixed_hash_rlp {
	($name: ident, $size: expr) => {
		impl rlp::Encodable for $name {
			fn encode(&self, stream: &mut rlp::RLPStream) {
				stream.write_iter(self.as_ref().iter().cloned());
			}
		}

		impl rlp::Decodable for $name {
			fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::Error> {
				rlp.decoder().decode_value(|bytes| match bytes.len().cmp(&$size) {
					core::cmp::Ordering::Less => Err(rlp::Error::RlpIsTooShort),
					core::cmp::Ordering::Greater => Err(rlp::Error::RlpIsTooBig),
					core::cmp::Ordering::Equal => {
						let mut t = [0u8; $size];
						t.copy_from_slice(bytes);
						Ok($name(t))
					}
				})
			}
		}
	}
}

impl_fixed_hash_rlp!(H256, 32);
impl_fixed_hash_rlp!(H512, 64);

impl_uint_conversions!(H256, U256);

/// Trait describing an object that can hash a slice of bytes. Used to abstract
/// other types over the hashing algorithm. Defines a single `hash` method and an
/// `Out` associated type with the necessary bounds.
pub trait Hasher: Sync + Send {
    /// The length in bytes of the `Hasher` output
    const LENGTH: usize;

    /// Compute the hash of the provided slice of bytes returning the `Out` type of the `Hasher`
    fn hash(x: &[u8]) -> H256;
}

pub fn sha256(data: &[u8]) -> H256 { H256::from_slice(Sha256::digest(data).as_slice()) }

pub fn hmac_sha256(key: &H256, input: &[u8], auth_data: &[u8]) -> H256 {
    let mut hmac = Hmac::<Sha256>::new_from_slice(key.as_bytes()).expect("invalid key to hmac");
    hmac.update(input);
    hmac.update(auth_data);
    H256::from_slice(&hmac.finalize().into_bytes())
}

pub fn keccak(x: &[u8]) -> H256 {
    KeccakHasher::hash(x)
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct KeccakHasher;
impl Hasher for KeccakHasher {
    const LENGTH: usize = HASH_LENGTH;

    fn hash(x: &[u8]) -> H256 {
        let mut keccak = Keccak::v256();
        keccak.update(x);
        let mut out = [0u8; 32];
        keccak.finalize(&mut out);
        H256::from(out)
    }
}

impl From<fixed_hash::rustc_hex::FromHexError> for Error{
    fn from(e: FromHexError) -> Self {
        Error::FromHexError(e)
    }
}
