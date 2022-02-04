use std::collections::{HashMap, HashSet, VecDeque};
use std::net::SocketAddr;
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{mpsc};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::task::JoinHandle;
use common::{H256, keccak, Secret, sign};
use rlp::RLPStream;
use crate::config::{HostInfo};
use crate::connection::Bytes;
use crate::db::Storage;
use crate::error::Error;
use crate::node::{NodeEndpoint, NodeEntry, NodeId};
use crate::PROTOCOL_VERSION;

const ADDRESS_BYTES_SIZE: usize = 32; // Size of address type in bytes.
const MAX_NODES_PING: usize = 32; // Max nodes to add/ping at once
const UDP_MAX_PACKET_SIZE: usize = 1024;
const EXPIRY_TIME: Duration = Duration::from_secs(20);

const PACKET_PING: u8 = 1;
// const PACKET_PONG: u8 = 2;
// const PACKET_FIND_NODE: u8 = 3;
// const PACKET_NEIGHBOURS: u8 = 4;

/// The metadata of the target nodes being pinged
struct InFlightMeta {
    node: NodeEntry,
    reason: PingReason,
    timestamp: Instant,
    hash: H256,
}

// #[derive(Clone, Copy, PartialEq)]
// enum NodeCategory {
//     Bucket,
//     Observed,
// }

// #[derive(Clone, Copy, PartialEq)]
// enum NodeValidity {
//     Ourselves,
//     ValidNode(NodeCategory),
//     ExpiredNode(NodeCategory),
//     UnknownNode,
// }

#[derive(Clone, Copy)]
enum PingReason {
    Default,
    // FromDiscoveryRequest(NodeId, NodeValidity),
}

#[derive(Clone, Debug)]
pub enum Request {
    AddNode(NodeEntry),
    AddNodes(Vec<NodeEntry>),
}

pub struct Discovery {
    is_stop: Arc<AtomicBool>,
    handle: JoinHandle<()>,
    request_tx: mpsc::Sender<Request>,
}

impl Discovery {
    pub async fn start(info: &HostInfo) -> Result<Self, Error> {
        // TODO: temporary, replace with persistence, using rocksdb
        let storage = Storage::new_memory_db();
        let (udp_tx, mut udp_rx) = mpsc::channel(1024);
        let (request_tx, mut request_rx) = mpsc::channel(1024);
        let is_stop = Arc::new(AtomicBool::new(false));

        log::info!("discovery starting udp at {:}", info.public_endpoint().udp_address());
        let socket = UdpSocket::bind(info.public_endpoint().udp_address()).await?;

        let mut discovery = DiscoveryInner{
            storage,
            id: info.key_pair().public().clone(),
            id_hash: keccak(info.key_pair().public().as_bytes()),
            secret: info.key_pair().secret().clone(),
            public_endpoint: info.public_endpoint(),
            buckets: (0..ADDRESS_BYTES_SIZE * 8).map(|_| VecDeque::new()).collect(),
            not_allowed: HashSet::new(),
            pinging_nodes: HashMap::new(),
            to_add: vec![],
            sender: udp_tx
        };

        let stop = Arc::clone(&is_stop);
        let handle = tokio::spawn(async move {
            // tricky, need to 0 init, otherwise udp socket will return empty
            let mut buf = vec![0; UDP_MAX_PACKET_SIZE];

            while !stop.load(Ordering::SeqCst) {
                tokio::select! {
                    Some((bytes, target)) = udp_rx.recv() => {
                        log::debug!("sending bytes: {:} to target {:?}", bytes.len(), target);
                        match socket.send_to(&bytes, target).await {
                            Ok(..) => log::debug!("send bytes: {:} to target", bytes.len()),
                            Err(e) => log::error!("error sending udp {:?}", e),
                        }
                    }
                    Ok((size, peer)) = socket.recv_from(&mut buf) => {
                        let data = &buf[..size];
                        log::debug!("data: {:?} from peer {:?}, size: {:}", data, peer, size);
                    }
                    Some(request) = request_rx.recv() => {
                        discovery.handle(request).await;
                    }
                }
            }
            log::info!("discovery ended");
        });

        Ok(Self {
            is_stop,
            handle,
            request_tx
        })
    }

    pub fn stop(&mut self) {
        if self.is_stop.load(Ordering::SeqCst) { return; }
        self.is_stop.store(true, Ordering::SeqCst);
    }

    /// Add a new node to discovery table. Pings the node.
    pub async fn add_node(&mut self, e: NodeEntry) -> Result<(), Error>{
        self.request_tx.send(Request::AddNode(e)).await?;
        Ok(())
    }

    /// Add a list of nodes. Pings a few nodes each round
    pub async fn add_node_list(&mut self, nodes: Vec<NodeEntry>) -> Result<(), Error>{
        self.request_tx.send(Request::AddNodes(nodes)).await?;
        Ok(())
    }
}

/// The inner struct of Discovery, handles all the processing logic
struct DiscoveryInner {
    /// Node storage for external nodes in the P2P
    storage: Storage,
    /// The node id of self
    id: NodeId,
    /// The hash of self node id
    id_hash: H256,
    /// The secret of self
    secret: Secret,
    /// The self public endpoint
    public_endpoint: NodeEndpoint,
    /// The buckets that hold the external nodes
    buckets: Vec<VecDeque<NodeId>>,
    /// Not allowed node ids
    not_allowed: HashSet<NodeId>,
    /// The nodes that is currently being pinged
    pinging_nodes: HashMap<NodeId, InFlightMeta>,
    /// The node entries to be added
    to_add: Vec<NodeEntry>,

    sender: mpsc::Sender<(Bytes, SocketAddr)>
}

impl DiscoveryInner {
    async fn handle(&mut self, request: Request) {
        let r = match request {
            Request::AddNode(e) => self.add_node(e).await,
            Request::AddNodes(ns) => self.add_node_list(ns).await,
         };
        match r {
            Ok(_) => {}
            Err(e) => log::error!("error handling request: {:?}", e),
        }

    }

    /// Add a new node to discovery table. Pings the node
    async fn add_node(&mut self, e: NodeEntry) -> Result<(), Error>{
        let node_hash = keccak(e.id().as_bytes());
        match Self::distance(&self.id_hash, &node_hash) {
            Some(d) => {
                if self.buckets[d].iter().any(|nid| nid == e.id()) {
                    return Ok(());
                }
                self.try_ping(e, PingReason::Default).await
            },
            None => Err(Error::InvalidNodeDistance)
        }
    }

    /// Add a list of nodes. Pings a few nodes each round
    async fn add_node_list(&mut self, nodes: Vec<NodeEntry>) -> Result<(), Error> {
        for n in nodes {
            self.add_node(n).await?;
        }
        Ok(())
    }

    async fn try_ping(&mut self, e: NodeEntry, reason: PingReason) -> Result<(), Error> {
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
            self.ping(e, reason).await
        } else {
            log::info!("pinging nodes full, add node id {} to pending nodes", e.id());
            self.to_add.push(e);
            Ok(())
        }
    }

    async fn ping(&mut self, e: NodeEntry, reason: PingReason) -> Result<(), Error> {
        // The ping packet: https://github.com/ethereum/devp2p/blob/master/discv4.md#ping-packet-0x01
        let mut rlp = RLPStream::new_list(4);
        rlp.append(&PROTOCOL_VERSION);
        self.public_endpoint.to_rlp_list(&mut rlp);
        e.endpoint().to_rlp_list(&mut rlp);
        append_expiration(&mut rlp);

        let packet = assemble_packet(PACKET_PING, &rlp.out(), &self.secret)?;
        let hash = H256::from_slice(&packet[..32]);

        // send to the channel for processing
        self.sender.send((packet, e.endpoint().udp_address())).await?;

        // save the metadata for Pong
        self.pinging_nodes.insert(
            *e.id(),
            InFlightMeta {
                node: e,
                reason,
                timestamp: Instant::now(),
                hash
            }
        );

        Ok(())
    }

    /// Checks if the node_id is allowed for connection
    fn is_allowed(&self, node_id: &NodeId) -> bool {
        !self.not_allowed.contains(node_id)
    }

    fn distance(a: &H256, b: &H256) -> Option<usize> {
        let mut lz = 0;
        for i in 0..ADDRESS_BYTES_SIZE {
            let d: u8 = a[i] ^ b[i];
            if d == 0 {
                lz += 8;
            } else {
                lz += d.leading_zeros() as usize;
                return Some(ADDRESS_BYTES_SIZE * 8 - lz - 1); // -1 as index
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
        assert_eq!(result, Some(247));
    }
}
