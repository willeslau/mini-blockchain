use crate::rlp::RLPStream;

/// RPL encodable trait. Encode Self into bytes and append to end of stream.
pub trait Encodable {
    fn encode(&self, stream: &mut RLPStream);
}

/// RPL decodable trait. Decode from the stream to Self. Read from start of stream.
pub trait Decodable {
    fn decode(&self, stream: &mut RLPStream);
}
