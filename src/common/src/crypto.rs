//! Secret key implementation.

use secp256k1::constants::SECRET_KEY_SIZE as SECP256K1_SECRET_KEY_SIZE;
use secp256k1::rand::rngs::OsRng;
use secp256k1::{PublicKey, SecretKey};
// Why do we need this? http://www.daemonology.net/blog/2014-09-04-how-to-zero-a-buffer.html
use zeroize::Zeroize;
use crate::error::Error;
use crate::{H256, H512, HASH_LENGTH};

use lazy_static::lazy_static;

pub type Public = H512;

lazy_static! {
	static ref SECP256K1: secp256k1::Secp256k1<secp256k1::All> = secp256k1::Secp256k1::new();
}

#[derive(Debug, PartialEq, Clone)]
pub struct KeyPair {
    secret: Secret,
    public: Public,
}

impl KeyPair {
    #[cfg(feature = "rand")]
    pub fn new() -> Self {
        let mut rng = OsRng::new().expect("cannot create random");
        let (secret_key, _) = &SECP256K1.generate_keypair(&mut rng);
        Self::from_secret_key(*secret_key)
    }

    pub fn from_secret_key(secret_key: SecretKey) -> Self {
        let public_key = PublicKey::from_secret_key(&SECP256K1, &secret_key);
        let serialized = public_key.serialize_uncompressed();

        let mut public = [0u8; 64];
        public.copy_from_slice(&serialized[1..65]);

        Self { secret: Secret::from(secret_key), public }
    }

    pub fn public(&self) -> &Public {
        &self.public
    }

    pub fn secret(&self) -> &Secret {
        &self.secret
    }
}

/// Represents secret key
#[derive(Debug, PartialEq, Clone)]
pub struct Secret {
    inner: Box<H256>,
}

impl Drop for Secret {
    fn drop(&mut self) {
        self.inner.zeroize()
    }
}

impl Secret {
    /// Creates a `Secret` from the given slice, returning `None` if the slice length != 32.
    /// Caller is responsible to zeroize input slice.
    pub fn copy_from_slice(key: &[u8]) -> Option<Self> {
        if key.len() != 32 {
            return None;
        }
        let mut h = [0u8; HASH_LENGTH];
        h.copy_from_slice(&key[0..32]);
        Some(Secret { inner: Box::new(h) })
    }

    /// Creates zero key, which is invalid for crypto operations, but valid for math operation.
    pub fn zero() -> Self {
        Secret { inner: Box::new([0u8; HASH_LENGTH]) }
    }

    /// Imports and validates the key.
    /// Caller is responsible to zeroize input slice.
    pub fn import_key(key: &[u8]) -> Result<Self, Error> {
        let secret = SecretKey::from_slice(key)?;
        Ok(secret.into())
    }

    /// Create a `secp256k1::key::SecretKey` based on this secret.
    /// Warning the resulting secret key need to be zeroized manually.
    pub fn to_secp256k1_secret(&self) -> Result<SecretKey, Error> {
        SecretKey::from_slice(&self.inner[..]).map_err(Into::into)
    }
}

impl From<[u8; 32]> for Secret {
    #[inline(always)]
    fn from(mut k: [u8; 32]) -> Self {
        let result = Secret { inner: Box::new(k) };
        k.zeroize();
        result
    }
}

impl From<SecretKey> for Secret {
    #[inline(always)]
    fn from(key: SecretKey) -> Self {
        let mut a = [0; SECP256K1_SECRET_KEY_SIZE];
        a.copy_from_slice(&key[0..SECP256K1_SECRET_KEY_SIZE]);
        ZeroizeSecretKey(key).zeroize();
        a.into()
    }
}

/// A wrapper type around `SecretKey` to prevent leaking secret key data. This
/// type will properly zeroize the secret key to `ONE_KEY` in a way that will
/// not get optimized away by the compiler nor be prone to leaks that take
/// advantage of access reordering.
#[derive(Clone, Copy)]
pub struct ZeroizeSecretKey(pub secp256k1::SecretKey);

impl Default for ZeroizeSecretKey {
    fn default() -> Self {
        ZeroizeSecretKey(secp256k1::ONE_KEY)
    }
}

impl std::ops::Deref for ZeroizeSecretKey {
    type Target = secp256k1::SecretKey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl zeroize::DefaultIsZeroes for ZeroizeSecretKey {}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Error::Secp256k1(e)
    }
}