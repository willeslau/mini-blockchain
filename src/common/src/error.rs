#[derive(Debug)]
pub enum Error {
    Secp256k1(secp256k1::Error),
    FromHexError(fixed_hash::rustc_hex::FromHexError),

    InvalidLength,
    CannotParseHexString,
    /// Invalid message for decryption
    InvalidMessage
}