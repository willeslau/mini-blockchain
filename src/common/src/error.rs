#[derive(Debug)]
pub enum Error {
    Secp256k1(secp256k1::Error),
    FromHexError(fixed_hash::rustc_hex::FromHexError),

    // P2P network errors
    InvalidNodeDistance,
    NodeBlocked,

    InvalidLength,
    CannotParseHexString,
    /// Invalid message for decryption
    InvalidMessage
}