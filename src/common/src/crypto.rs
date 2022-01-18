//! Secret key implementation.

use hex::{FromHex, FromHexError, ToHex};
use secp256k1::constants::SECRET_KEY_SIZE as SECP256K1_SECRET_KEY_SIZE;
use secp256k1::rand::rngs::OsRng;
use secp256k1::{Message, PublicKey, SecretKey};
// Why do we need this? http://www.daemonology.net/blog/2014-09-04-how-to-zero-a-buffer.html
use zeroize::Zeroize;
use crate::error::Error;
use crate::{H256, H512, HASH_LENGTH, xor};

use lazy_static::lazy_static;

// pub type Public = H512;
#[derive(Debug, PartialEq, Clone)]
pub struct Public {
    inner: H512,
}

impl Public {
    pub fn copy_from_slice(&mut self, data: &[u8]) {
        self.inner.copy_from_slice(data);
    }

    pub fn from_str(s: &str) -> Result<Self, Error> {
        let inner = <H512>::from_hex(s)?;
        Ok(Self { inner })
    }
}

impl From<FromHexError> for Error{
    fn from(_: FromHexError) -> Self {
        Error::CannotParseHexString
    }
}

impl Default for Public {
    fn default() -> Self {
        Self { inner: [0u8; 64] }
    }
}

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

        let mut public = Public::default();
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

    /// Creates a `Secret` from the given `str` representation,
    /// returning an error for hex big endian representation of
    /// the secret.
    /// Caller is responsible to zeroize input slice.
    pub fn copy_from_str(s: &str) -> Result<Self, Error> {
        let h = <H256>::from_hex(s)?;
        Ok(Secret { inner: Box::new(h) })
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

    pub fn as_bytes(&self) -> &[u8] {
        self.inner.as_ref()
    }

    pub fn to_hex(&self) -> String {
        self.inner.encode_hex::<String>()
    }

    pub fn xor(&self, other: &H256) -> H256 {
        xor(self.inner.as_ref(), other)
    }
}

impl AsRef<[u8]> for Secret {
    fn as_ref(&self) -> &[u8] {
        self.inner.as_ref()
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

/// Signs message with the given secret key.
/// Returns the corresponding signature.
pub fn sign(secret: &Secret, message: &H256) -> Result<[u8;65], Error> {
    let context = &SECP256K1;
    let sec = SecretKey::from_slice(secret.as_ref())?;
    let s = context.sign_ecdsa_recoverable(&Message::from_slice(&message[..])?, &sec);
    let (rec_id, data) = s.serialize_compact();
    let mut data_arr = [0; 65];

    // no need to check if s is low, it always is
    data_arr[0..64].copy_from_slice(&data[0..64]);
    data_arr[64] = rec_id.to_i32() as u8;
    Ok(data_arr)
}

pub mod ecdh {
    use secp256k1::{PublicKey, SecretKey};
    use secp256k1::ecdh::SharedSecret;
    use crate::{Public, Secret};
    use crate::error::Error;

    /// Create a shared secret for message exchange.
    /// See https://en.wikipedia.org/wiki/Diffie%E2%80%93Hellman_key_exchange#cite_note-imperfectfs-4
    pub fn agree(secret: &Secret, public: &Public) -> Result<Secret, Error> {
        let pdata = {
            let mut temp = [4u8; 65];
            (&mut temp[1..65]).copy_from_slice(&public.inner[0..64]);
            temp
        };

        let publ = PublicKey::from_slice(&pdata)?;
        let sec = SecretKey::from_slice(secret.as_bytes())?;
        let shared = SharedSecret::new_with_hash(&publ, &sec, |x, _| x.into());

        Secret::import_key(&shared[0..32]).map_err(|_| Error::Secp256k1(secp256k1::Error::InvalidSecretKey))
    }
}


#[cfg(test)]
mod tests {
    use crate::{ecdh::agree, Public, Secret};

    #[test]
    fn test_agree() {
        // Just some random values for secret/public to check we agree with previous implementation.
        let secret =
            Secret::copy_from_str(&"01a400760945613ff6a46383b250bf27493bfe679f05274916182776f09b28f1").unwrap();
        let public= Public::from_str("e37f3cbb0d0601dc930b8d8aa56910dd5629f2a0979cc742418960573efc5c0ff96bc87f104337d8c6ab37e597d4f9ffbd57302bc98a825519f691b378ce13f5").unwrap();
        let shared = agree(&secret, &public);

        assert!(shared.is_ok());
        assert_eq!(shared.unwrap().to_hex(), "28ab6fad6afd854ff27162e0006c3f6bd2daafc0816c85b5dfb05dbb865fa6ac",);
    }
}