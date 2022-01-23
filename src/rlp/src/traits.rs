use crate::Error;
use crate::rlp::RLPStream;
use crate::rlpin::Rlp;

/// RPL encodable trait. Encode Self into bytes and append to end of stream.
pub trait Encodable {
    fn encode(&self, stream: &mut RLPStream);
}

/// RPL decodable trait. Decode from the stream to Self. Read from start of stream.
pub trait Decodable: Sized {
    /// Decode a value from RLP bytes
    fn decode(rlp: &Rlp) -> Result<Self, Error>;
}
