//! Secret key implementation.

use hex::{FromHex, FromHexError, ToHex};
use secp256k1::constants::SECRET_KEY_SIZE as SECP256K1_SECRET_KEY_SIZE;
use secp256k1::{Message, PublicKey, SecretKey};
// Why do we need this? http://www.daemonology.net/blog/2014-09-04-how-to-zero-a-buffer.html
use zeroize::Zeroize;
use crate::error::Error;
use crate::{H256, H512, HASH_LENGTH, xor};

use lazy_static::lazy_static;
use secp256k1::rand::rngs::OsRng;

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

    pub fn from_slice(s: &[u8]) -> Self {
        let mut inner = [0u8; 64];
        for i in 0..s.len() { inner[i] = s[i]; }
        Self { inner }
    }
}

impl AsRef<[u8]> for Public {
    fn as_ref(&self) -> &[u8] {
        self.inner.as_ref()
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
    pub fn random() -> Self {
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

#[cfg(test)]
mod tests {
    use crate::{Public, Secret, sign};

    #[test]
    fn test_sign() {
        // Just some random values for secret/public to check we agree with previous implementation.
        let secret =
            Secret::copy_from_str(&"b71c71a67e1177ad4e901695e1b4b9ee17ae16c6668d313eac2f96dbcda3f291").unwrap();
        let message = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
        let s = sign(&secret, &message).unwrap();
        assert_eq!(s, [182, 182, 244, 193, 65, 89, 128, 178, 40, 121, 127, 32, 179, 105, 30, 133, 208, 112, 255, 162, 45, 171, 138, 47, 71, 75, 182, 177, 36, 223, 7, 174, 101, 191, 217, 45, 254, 26, 10, 67, 76, 22, 29, 43, 57, 71, 4, 67, 127, 138, 165, 169, 203, 93, 61, 18, 76, 208, 229, 96, 14, 85, 252, 29, 0]);
    }

    #[test]
    fn test_xor() {
        // Just some random values for secret/public to check we agree with previous implementation.
        let secret =
            Secret::copy_from_str(&"01a400760945613ff6a46383b250bf27493bfe679f05274916182776f09b28f1").unwrap();
        let h = [56, 242, 184, 93, 221, 158, 68, 46, 153, 138, 12, 152, 135, 63, 27, 151, 136, 30, 18, 171, 49, 150, 97, 219, 68, 55, 148, 72, 124, 63, 140, 230];
        assert_eq!(
            secret.xor(&h),
            [57, 86, 184, 43, 212, 219, 37, 17, 111, 46, 111, 27, 53, 111, 164, 176, 193, 37, 236, 204, 174, 147, 70, 146, 82, 47, 179, 62, 140, 164, 164, 23]
        );
    }

    #[test]
    fn test_secret_as_ref() {
        // Just some random values for secret/public to check we agree with previous implementation.
        let secret =
            Secret::copy_from_str(&"b71c71a67e1177ad4e901695e1b4b9ee17ae16c6668d313eac2f96dbcda3f291").unwrap();
        assert_eq!(
            secret.as_ref(),
            [183, 28, 113, 166, 126, 17, 119, 173, 78, 144, 22, 149, 225, 180, 185, 238, 23, 174, 22, 198, 102, 141, 49, 62, 172, 47, 150, 219, 205, 163, 242, 145]
        );
    }
}