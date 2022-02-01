use std::net::SocketAddr;
use common::{H256, H512, Hasher, KeccakHasher, Public};

/// Node public key
pub(crate) type NodeId = H512;

/// Node address info
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NodeEndpoint {
    /// IP(V4 or V6) address
    pub address: SocketAddr,
    /// Connection port.
    pub udp_rt: u16,
}

/// The node entry to store in database storage
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NodeEntry {
    id: NodeId,
    endpoint: NodeEndpoint,
}