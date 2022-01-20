#[derive(Debug)]
pub enum Error {
    Secp256k1(secp256k1::Error),
    InvalidLength,
    CannotParseHexString
}