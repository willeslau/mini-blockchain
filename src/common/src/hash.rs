use rand::Rng;
use sha2::{Digest, Sha256};
use tiny_keccak::{Hasher as KeccakHasherTrait, Keccak};

pub const HASH_LENGTH: usize = 32;
pub type H128 = [u8; 16];
pub type H256 = [u8; HASH_LENGTH];
pub type H512 = [u8; 64];

pub fn random_h256() -> H256 {
    H256::default().map(|_| rand::thread_rng().gen())
}

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

pub fn sha256(data: &[u8]) -> H256 {
    let v = Sha256::digest(data);
    H256::from(v)
}

pub fn hmac_sha256(key: &[u8], input: &[&[u8]], auth_data: &[u8]) -> H256 {
    let mut hmac = Hmac::<Sha256>::new_from_slice(key).unwrap();
    for input in input {
        hmac.update(input);
    }
    hmac.update(auth_data);
    H256::from_slice(&*hmac.finalize().into_bytes())
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

#[cfg(test)]
mod tests {
    use crate::{random_h256};

    #[test]
    fn random_works() {
        let r = random_h256();
        assert_eq!(r.len(), 32);
    }
}