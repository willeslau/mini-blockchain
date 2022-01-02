use crate::RLPStream;
use crate::traits::Encodable;

impl Encodable for &str {
    fn encode(&self, stream: &mut RLPStream) {
        stream.write_iter(self.bytes())
    }
}

impl Encodable for Vec<u8> {
    fn encode(&self, stream: &mut RLPStream) {
        stream.write_iter(self.iter().cloned())
    }
}

impl Encodable for common::Hash {
    fn encode(&self, stream: &mut RLPStream) {
        stream.write_iter(self.iter().cloned())
    }
}