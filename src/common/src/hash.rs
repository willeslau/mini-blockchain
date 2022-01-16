use tiny_keccak::{Hasher as KeccakHasherTrait, Keccak};

pub const HASH_LENGTH: usize = 32;
pub type H256 = [u8; HASH_LENGTH];
pub type H512 = [u8; 64];

pub fn bytes_to_hash(v: &[u8]) -> H256 {
    let mut hash = H256::default();
    for i in 0..v.len().min(HASH_LENGTH) {
        hash[i] = v[i];
    }
    hash
}

/// Trait describing an object that can hash a slice of bytes. Used to abstract
/// other types over the hashing algorithm. Defines a single `hash` method and an
/// `Out` associated type with the necessary bounds.
pub trait Hasher: Sync + Send {
    /// The length in bytes of the `Hasher` output
    const LENGTH: usize;

    /// Compute the hash of the provided slice of bytes returning the `Out` type of the `Hasher`
    fn hash(x: &[u8]) -> H256;
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
        out
    }
}