use secp256k1::{Message, PublicKey, SecretKey};
use secp256k1::ecdh::SharedSecret;
use secp256k1::ecdsa::{RecoverableSignature, RecoveryId};
use crate::error::Error;
use crate::crypto::keypair::{Public, Secret};
use crate::{H256, H520, SECP256K1};

/// Create a shared secret for message exchange.
/// See https://en.wikipedia.org/wiki/Diffie%E2%80%93Hellman_key_exchange#cite_note-imperfectfs-4
pub fn agree(secret: &Secret, public: &Public) -> Result<Secret, Error> {
    let pdata = {
        let mut temp = [4u8; 65];
        (&mut temp[1..65]).copy_from_slice(&public.as_ref()[0..64]);
        temp
    };

    let publ = PublicKey::from_slice(&pdata)?;
    let sec = SecretKey::from_slice(secret.as_bytes())?;
    let shared = SharedSecret::new_with_hash(&publ, &sec, |x, _| x.into());

    Secret::import_key(&shared[0..32]).map_err(|_| Error::Secp256k1(secp256k1::Error::InvalidSecretKey))
}

/// Recovers the public key from the signature for the message
pub fn recover(signature: &H520, message: &H256) -> Result<Public, Error> {
    let rsig = RecoverableSignature::from_compact(&signature[0..64], RecoveryId::from_i32(signature[64] as i32)?)?;

    let pubkey = &SECP256K1.recover_ecdsa(&Message::from_slice(&message[..])?, &rsig)?;
    let serialized = pubkey.serialize_uncompressed();

    Ok(Public::from_slice(&serialized[1..65]))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use crate::crypto::ecdh::agree;
    use crate::{Public, Secret};

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