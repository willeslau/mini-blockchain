use core::slice;
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use common::{H512};
use rlp::RLPStream;

/// Node public key
pub(crate) type NodeId = H512;

/// Node address info
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NodeEndpoint {
    /// IP(V4 or V6) address
    pub address: SocketAddr,
    /// Connection port.
    pub udp_port: u16,
}

impl NodeEndpoint {
    pub fn udp_address(&self) -> SocketAddr {
        match self.address {
            SocketAddr::V4(a) => SocketAddr::V4(SocketAddrV4::new(*a.ip(), self.udp_port)),
            SocketAddr::V6(a) => SocketAddr::V6(SocketAddrV6::new(
                *a.ip(),
                self.udp_port,
                a.flowinfo(),
                a.scope_id(),
            )),
        }
    }

    pub fn to_rlp(&self, rlp: &mut RLPStream) {
        match self.address {
            SocketAddr::V4(a) => {
                rlp.append(&(&a.ip().octets()[..]));
            }
            SocketAddr::V6(a) => unsafe {
                let o: *const u8 = a.ip().segments().as_ptr() as *const u8;
                rlp.append(&slice::from_raw_parts(o, 16));
            },
        };
        rlp.append(&self.udp_port);
        rlp.append(&self.address.port());
    }

    pub fn to_rlp_list(&self, rlp: &mut RLPStream) {
        rlp.begin_list(3);
        self.to_rlp(rlp);
    }
}

/// The node entry to store in database storage
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NodeEntry {
    id: NodeId,
    endpoint: NodeEndpoint,
}

impl NodeEntry {
    pub fn id(&self) -> &NodeId { &self.id }
    pub fn endpoint(&self) -> &NodeEndpoint { &self.endpoint }
}