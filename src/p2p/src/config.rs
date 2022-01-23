use common::{KeyPair, Public};
use std::net::SocketAddr;

/// Network service configuration
#[derive(Clone, Debug)]
pub struct Config {
    /// This field must be set to a valid secp256k1 private key.
    pub key_pair: KeyPair,
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

impl Config {
    pub fn public_key(&self) -> &Public {
        self.key_pair.public()
    }
}
