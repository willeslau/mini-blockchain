// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Functions for ECIES scheme encryption and decryption
use std::borrow::Borrow;
use subtle::ConstantTimeEq;
use aes::Aes128Ctr;
use aes::cipher::{NewCipher, StreamCipher};
use aes::cipher::errors::InvalidLength;
use sha2::{Digest, Sha256};
use crate::{Error, h128_from, hmac_sha256, KeyPair, Public, random_h128, Secret, sha256};
use crate::crypto::ecdh;

const ENC_VERSION: u8 = 0x04;

/// Encrypt a message with a public key, writing an HMAC covering both
/// the plaintext and authenticated data.
///
/// Authenticated data may be empty.
pub fn encrypt(public: &Public, auth_data: &[u8], plain: &[u8]) -> Result<Vec<u8>, Error> {
	let r = KeyPair::random();
	let z = ecdh::agree(r.secret(), public)?;

	let mut key = [0u8; 32];
	kdf(&z, &[0u8; 0], &mut key);

	let ekey = h128_from(&key[0..16]); // for encryption
	let mkey = sha256(&key[16..32]); // for signature

	// 1: ENC_VERSION, 1-65: Public key, 65-81: iv, 81-..: plain data, rest is hmac signature
	let mut msg = vec![0u8; secp256k1::constants::UNCOMPRESSED_PUBLIC_KEY_SIZE + 16 + plain.len() + 32];
	let iv = random_h128();

	msg[0] = ENC_VERSION;
	msg[1..65].copy_from_slice(r.public().as_ref());
	msg[65..81].copy_from_slice(&iv);
	msg[81..plain.len()+81].copy_from_slice(plain);

	// now perform encryption
	let mut encryptor = Aes128Ctr::new_from_slices(&ekey, &iv)?;
	encryptor.apply_keystream(&mut msg[81..81+plain.len()]);

	// perform hmac_sha256
	let sig = hmac_sha256(
		&mkey,
		&msg[65..plain.len()+81],
		auth_data,
	);
	msg[81+plain.len()..].copy_from_slice(&sig);

	Ok(msg)
}

/// Decrypt a message with a secret key, checking HMAC for ciphertext
/// and authenticated data validity.
pub fn decrypt(secret: &Secret, auth_data: &[u8], encrypted: &[u8]) -> Result<Vec<u8>, Error> {
	const META_LEN: usize = 1 + 64 + 16 + 32;
	let enc_version = encrypted[0];
	if encrypted.len() < META_LEN || enc_version < 2 || enc_version > 4 {
		return Err(Error::InvalidMessage);
	}

	let e = &encrypted[1..];
	let p = Public::from_slice(&e[0..64]);
	let z = ecdh::agree(secret, &p)?;
	let mut key = [0u8; 32];
	kdf(&z, &[0u8; 0], &mut key);

	let ekey = &key[0..16];
	let mkey = sha256(&key[16..32]);

	let cipher_text_len = encrypted.len() - META_LEN;
	let cipher_with_iv = &e[64..(64 + 16 + cipher_text_len)];
	let cipher_iv = &cipher_with_iv[0..16];
	let cipher_enc_text = &cipher_with_iv[16..];
	let msg_mac = &e[(64 + 16 + cipher_text_len)..];

	// Verify tag
	let mac = hmac_sha256(
		&mkey,
		cipher_with_iv,
		auth_data,
	);
	if mac.ct_eq(msg_mac).unwrap_u8() == 0 {
		return Err(Error::InvalidMessage);
	}

	let mut msg = cipher_enc_text.to_vec();
	let mut encryptor = Aes128Ctr::new_from_slices(&ekey, &cipher_iv)?;
	encryptor.apply_keystream(&mut msg);

	Ok(msg)
}

fn kdf(secret: &Secret, s1: &[u8], dest: &mut [u8]) {
	// SEC/ISO/Shoup specify counter size SHOULD be equivalent
	// to size of hash output, however, it also notes that
	// the 4 bytes is okay. NIST specifies 4 bytes.
	let mut ctr = 1_u32;
	let mut written = 0_usize;
	while written < dest.len() {
		let mut hasher = Sha256::default();
		let ctrs = [
			(ctr >> 24) as u8,
			(ctr >> 16) as u8,
			(ctr >> 8) as u8,
			ctr as u8,
		];
		hasher.update(&ctrs);
		hasher.update(secret.as_bytes());
		hasher.update(s1);
		let d = hasher.finalize();
		dest[written..(written + 32)].copy_from_slice(&d);
		written += 32;
		ctr += 1;
	}
}

impl From<aes::cipher::errors::InvalidLength> for Error {
	fn from(_: InvalidLength) -> Self {
		Error::InvalidLength
	}
}

#[cfg(test)]
mod tests {
	use crate::{KeyPair, Secret};
	use super::super::{ecies};

	#[test]
	fn ecies_shared() {
		let secret = Secret::copy_from_str("b71c71a67e1177ad4e901695e1b4b9ee17ae16c6668d313eac2f96dbcda3f291").unwrap();
		let kp = KeyPair::from_secret_key(secret.to_secp256k1_secret().unwrap());

		let message = b"So many books, so little time";

		let shared = b"shared";
		let wrong_shared = b"incorrect";
		let encrypted = ecies::encrypt(kp.public(), shared, message).unwrap();

		// TODO: check encrypt implementation
		assert_ne!(encrypted[..], message[..]);

		assert!(ecies::decrypt(kp.secret(), wrong_shared, &encrypted).is_err());
		let decrypted = ecies::decrypt(kp.secret(), shared, &encrypted).unwrap();
		assert_eq!(decrypted[..message.len()], message[..]);
	}
}
