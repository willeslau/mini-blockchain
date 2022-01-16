use common::{H256, keccak, KeyPair, Secret};
use crate::config::Config;
use crate::enode::{NodeEndpoint, NodeId, pubkey_to_idv4};
use crate::protocol::CapabilityInfo;

/// Shared host information
pub(crate) struct HostInfo {
    /// Our private and public keys.
    keys: KeyPair,
    /// Current network configuration
    config: Config,
    /// Connection nonce.
    nonce: H256,
    /// RLPx protocol version
    pub protocol_version: u32,
    /// Registered capabilities (handlers)
    pub capabilities: Vec<CapabilityInfo>,
    /// Local address + discovery port
    pub local_endpoint: NodeEndpoint,
    /// Public address + discovery port
    pub public_endpoint: Option<NodeEndpoint>,
}

impl HostInfo {
    fn next_nonce(&mut self) -> H256 {
        self.nonce = keccak(&self.nonce);
        self.nonce
    }

    pub(crate) fn client_version(&self) -> &str {
        &self.config.client_version
    }

    pub(crate) fn secret(&self) -> &Secret {
        self.keys.secret()
    }

    pub(crate) fn id(&self) -> NodeId {
        pubkey_to_idv4(self.keys.public())
    }
}