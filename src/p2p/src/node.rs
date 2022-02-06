use crate::error::Error;
use common::H512;
use core::slice;
use core::str::FromStr;
use rlp::{RLPStream, Rlp};
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

/// Node public key
pub type NodeId = H512;

/// Node address info
#[derive(Debug, Clone, PartialEq)]
pub struct NodeEndpoint {
    /// IP(V4 or V6) address
    pub address: SocketAddr,
    /// Connection port.
    pub udp_port: u16,
}

impl NodeEndpoint {
    pub fn new(ip: &str, udp_port: u16) -> Self {
        Self {
            address: SocketAddr::from_str(&*format!("{:}:{:}", ip, udp_port))
                .expect("invalid endpoint"),
            udp_port,
        }
    }

    pub fn from_socket(address: SocketAddr, udp_port: u16) -> Self {
        Self { address, udp_port }
    }

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

    pub fn from_rlp(rlp: &Rlp) -> Result<Self, Error> {
        let udp_port: u16 = rlp.val_at(1)?;
        let tcp_port: u16 = rlp.val_at(2)?;
        let bytes = rlp.at(0)?.data()?;
        let socket = match bytes.len() {
            4 => SocketAddr::V4(SocketAddrV4::new(
                Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]),
                tcp_port,
            )),
            16 => unsafe {
                let ptr: *const u16 = bytes.as_ptr() as *const u16;
                let o = slice::from_raw_parts(ptr, 8);
                SocketAddr::V6(SocketAddrV6::new(
                    Ipv6Addr::new(o[0], o[1], o[2], o[3], o[4], o[5], o[6], o[7]),
                    tcp_port,
                    0,
                    0,
                ))
            },
            _ => return Err(Error::InvalidPacket),
        };

        Ok(NodeEndpoint::from_socket(socket, udp_port))
    }

    pub fn to_rlp_list(&self, rlp: &mut RLPStream) {
        rlp.begin_list(3);
        self.to_rlp(rlp);
    }

    /// Validates that the udp port is not 0 and address IP is specified
    pub fn is_valid_discovery_node(&self) -> bool {
        self.udp_port != 0
            && match self.address {
                SocketAddr::V4(a) => !a.ip().is_unspecified(),
                SocketAddr::V6(a) => !a.ip().is_unspecified(),
            }
    }

    /// Validates that the tcp port is not 0 and that the node is a valid discovery node (i.e. `is_valid_discovery_node()` is true).
    /// Sync happens over tcp.
    pub fn is_valid_sync_node(&self) -> bool {
        self.is_valid_discovery_node() && self.address.port() != 0
    }
}

/// The node entry to store in database storage
#[derive(Debug, Clone, PartialEq)]
pub struct NodeEntry {
    id: NodeId,
    endpoint: NodeEndpoint,
}

impl NodeEntry {
    pub fn new(id: NodeId, endpoint: NodeEndpoint) -> Self {
        Self { id, endpoint }
    }
    pub fn id(&self) -> &NodeId {
        &self.id
    }
    pub fn endpoint(&self) -> &NodeEndpoint {
        &self.endpoint
    }
    pub fn into(self) -> (NodeId, NodeEndpoint) {
        (self.id, self.endpoint)
    }
}
