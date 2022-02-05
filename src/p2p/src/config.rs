use crate::node::NodeEndpoint;
use common::KeyPair;
use std::net::{SocketAddr, SocketAddrV4};

pub struct HostInfo {
    /// This field must be set to a valid secp256k1 private key.
    pub key_pair: Option<KeyPair>,
    // /// Local address + discovery port
    // pub local_endpoint: NodeEndpoint,
    /// Public address + discovery port
    pub public_endpoint: Option<NodeEndpoint>,
}

impl HostInfo {
    pub fn key_pair(&self) -> KeyPair {
        match &self.key_pair {
            None => KeyPair::random(),
            Some(key_pair) => key_pair.clone(),
        }
    }

    pub fn public_endpoint(&self) -> NodeEndpoint {
        match &self.public_endpoint {
            None => NodeEndpoint {
                address: SocketAddr::V4(SocketAddrV4::new("0.0.0.0".parse().unwrap(), 30303)),
                udp_port: 30303,
            },
            Some(public_endpoint) => public_endpoint.clone(),
        }
    }
}

impl Default for HostInfo {
    fn default() -> Self {
        Self {
            key_pair: Some(KeyPair::random()),
            public_endpoint: None,
        }
    }
}

/// Network service configuration
#[derive(Clone, Debug)]
pub struct NetowkrConfig {
    /// The path to the local database storage
    pub node_db: String,
    /// IP address to listen for incoming connections. Listen to all connections by default
    pub listen_address: Option<SocketAddr>,
    /// Port for UDP connections, same as TCP by default
    pub udp_port: Option<u16>,

    /// Directory path to store general network configuration. None means nothing will be saved
    pub config_path: Option<String>,
    /// Directory path to store network-specific configuration. None means nothing will be saved
    pub net_config_path: Option<String>,
    /// IP address to advertise. Detected automatically if none.
    pub public_address: Option<SocketAddr>,
    // /// TODO: Enable NAT configuration
    // pub nat: Box<dyn crate::nat::Interface>,
    /// Enable discovery
    pub discovery_enabled: bool,
    /// List of initial node addresses
    pub boot_nodes: Vec<String>,
    /// Minimum number of connected peers to maintain
    pub min_peers: u32,
    /// Maximum allowed number of peers
    pub max_peers: u32,
    /// Maximum handshakes
    pub max_handshakes: u32,
    /// List of reserved node addresses.
    pub reserved_nodes: Vec<String>,
    /// Client identifier
    pub client_version: String,
}
