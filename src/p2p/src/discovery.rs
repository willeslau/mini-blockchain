use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use common::{Error, H256, keccak, Secret, sign};
use rlp::RLPStream;
use crate::connection::Bytes;
use crate::db::Storage;
use crate::node::{NodeEndpoint, NodeEntry, NodeId};
use crate::PROTOCOL_VERSION;

const ADDRESS_BYTES_SIZE: usize = 32; // Size of address type in bytes.
const MAX_NODES_PING: usize = 32; // Max nodes to add/ping at once

const EXPIRY_TIME: Duration = Duration::from_secs(20);

const PACKET_PING: u8 = 1;
const PACKET_PONG: u8 = 2;
const PACKET_FIND_NODE: u8 = 3;
const PACKET_NEIGHBOURS: u8 = 4;

/// The metadata of the target nodes being pinged
struct InFlightMeta {
    node: NodeEntry,
}

#[derive(Clone, Copy, PartialEq)]
enum NodeCategory {
    Bucket,
    Observed,
}

#[derive(Clone, Copy, PartialEq)]
enum NodeValidity {
    Ourselves,
    ValidNode(NodeCategory),
    ExpiredNode(NodeCategory),
    UnknownNode,
}

#[derive(Clone, Copy)]
enum PingReason {
    Default,
    FromDiscoveryRequest(NodeId, NodeValidity),
}

pub(crate) struct Discovery {

}

impl Discovery {
    /// Add a new node to discovery table. Pings the node.
    pub fn add_node(&mut self, _e: NodeEntry) {}

    /// Add a list of nodes. Pings a few nodes each round
    pub fn add_node_list(&mut self, _nodes: Vec<NodeEntry>) {}
}

/// The inner struct of Discovery, handles all the processing logic
struct DiscoveryInner<'a> {
    /// Node storage for external nodes in the P2P
    storage: &'a mut Storage,
    /// The node id of self
    id: NodeId,
    /// The hash of self node id
    id_hash: H256,
    /// The self public endpoint
    public_endpoint: NodeEndpoint,
    /// The buckets that hold the external nodes
    buckets: [Vec<NodeId>; ADDRESS_BYTES_SIZE * 8],
    /// Not allowed node ids
    not_allowed: HashSet<NodeId>,
    /// The nodes that is currently being pinged
    pinging_nodes: HashMap<NodeId, InFlightMeta>,
    /// The node entries to be added
    to_add: Vec<NodeEntry>,
}

impl <'a> DiscoveryInner<'a> {
    /// Add a new node to discovery table. Pings the node
    fn add_node(&mut self, e: NodeEntry) -> Result<(), Error>{
        let node_hash = keccak(e.id().as_bytes());
        match Self::distance(&self.id_hash, &node_hash) {
            Some(d) => {
                if self.buckets[d].iter().any(|nid| nid == e.id()) {
                    return Ok(());
                }
                self.try_ping(e, PingReason::Default)
            },
            None => Err(Error::InvalidNodeDistance)
        }
    }

    /// Add a list of nodes. Pings a few nodes each round
    fn add_node_list(&mut self, nodes: Vec<NodeEntry>) -> Result<(), Error> {
        for n in nodes {
            self.add_node(n)?;
        }
        Ok(())
    }

    fn try_ping(&mut self, e: NodeEntry, reason: PingReason) -> Result<(), Error> {
        if !self.is_allowed(e.id()) {
            log::info!("node id {} not allowed", e.id());
            return Err(Error::NodeBlocked);
        }

        // Currently pinging, return directly.
        // TODO: maybe perform timeout check?
        if self.pinging_nodes.contains_key(e.id()) {
            log::debug!("node id {} is being pinged", e.id());
            return Ok(());
        }

        if self.pinging_nodes.len() < MAX_NODES_PING {
            log::info!("pinging node id {}", e.id());
            self.ping(e, reason)
        } else {
            log::info!("pinging nodes full, add node id {} to pending nodes", e.id());
            self.to_add.push(e);
            Ok(())
        }
    }

    fn ping(&mut self, e: NodeEntry, _reason: PingReason) -> Result<(), Error> {
        // The ping packet: https://github.com/ethereum/devp2p/blob/master/discv4.md#ping-packet-0x01
        let mut rlp = RLPStream::new_list(4);
        rlp.append(&PROTOCOL_VERSION);
        self.public_endpoint.to_rlp_list(&mut rlp);
        e.endpoint().to_rlp_list(&mut rlp);
        append_expiration(&mut rlp);

        let hash = keccak(rlp.as_bytes());
        let hash = self.send(PACKET_PING, &e.endpoint().udp_address(), &rlp.out())?;
        Ok(())
    }

    fn send(&self, packet_type: u8, socket_addr: &SocketAddr, data: &[u8]) -> Result<H256, Error> {
        Ok(H256::random())
    }

    /// Checks if the node_id is allowed for connection
    fn is_allowed(&self, node_id: &NodeId) -> bool {
        self.not_allowed.contains(node_id)
    }

    fn distance(a: &H256, b: &H256) -> Option<usize> {
        let mut lz = 0;
        for i in 0..ADDRESS_BYTES_SIZE {
            let d: u8 = a[i] ^ b[i];
            if d == 0 {
                lz += 8;
            } else {
                lz += d.leading_zeros() as usize;
                return Some(ADDRESS_BYTES_SIZE * 8 - lz);
            }
        }
        None
    }
}

fn append_expiration(rlp: &mut RLPStream) {
    let expiry = SystemTime::now() + EXPIRY_TIME;
    let timestamp = expiry
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as u32;
    rlp.append(&timestamp);
}

/// Prepare the package: [hash_of_signature_and_bytes, signature, bytes]
fn assemble_packet(packet_id: u8, bytes: &[u8], secret: &Secret) -> Result<Bytes, Error> {
    let mut packet = Bytes::with_capacity(bytes.len() + 32 + 65 + 1);
    packet.resize(32 + 65, 0); // Filled in below
    packet.push(packet_id);
    packet.extend_from_slice(bytes);

    let hash = keccak(&packet[(32 + 65)..]);
    let signature = match sign(secret, &hash) {
        Ok(s) => s,
        Err(e) => {
            log::warn!("Error signing UDP packet");
            return Err(Error::from(e));
        }
    };
    packet[32..(32 + 65)].copy_from_slice(&signature[..]);
    let signed_hash = keccak(&packet[32..]);
    packet[0..32].copy_from_slice(signed_hash.as_bytes());
    Ok(packet)
}

#[cfg(test)]
mod tests {
    use common::H256;
    use crate::discovery::{DiscoveryInner};

    #[test]
    fn distance_works() {
        let a= H256::from_slice(&[228, 104, 254, 227, 239, 33, 109, 25, 223, 95, 27, 195, 177, 52, 50, 204, 76, 30, 147, 218, 216, 159, 47, 146, 236, 13, 163, 128, 250, 160, 17, 192]);
        let b= H256::from_slice(&[228, 214, 227, 65, 84, 85, 107, 82, 209, 81, 68, 106, 172, 254, 164, 105, 92, 23, 184, 27, 10, 90, 228, 69, 143, 90, 18, 117, 49, 186, 231, 5]);

        let result = DiscoveryInner::distance(&a, &b);
        assert_eq!(result, Some(248));
    }
}
