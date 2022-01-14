use secp256k1::PublicKey;
use crate::protocol::Cap;

pub(crate) const BASE_PROTOCOL_VERSION: u64 = 5;
pub(crate) const BASE_PROTOCOL_LENGTH: u64 = 16u64;
pub(crate) const BASE_PROTOCOL_MAX_MSG_SIZE: usize = 2 * 1024;
pub(crate) const SNAPPY_PROTOCOL_VERSION: u8 = 5;

/// Local (temporary) peer session ID.
pub type PeerId = usize;

/// ProtoHandshake is the RLP structure of the protocol handshake.
pub(crate) struct ProtoHandshake {
    pub version: u64,
    pub name: String,
    pub caps: Vec<Cap>,
    pub listen_port: Option<u64>,
    pub id: PublicKey
}

impl ProtoHandshake {
    pub fn new(version: u64, name: String, id: PublicKey) -> Self {
        ProtoHandshake {
            version,
            name,
            caps: vec![],
            listen_port: None,
            id
        }
    }

    pub fn append_cap(&mut self, cap: Cap) {
        self.caps.push(cap);
    }
}