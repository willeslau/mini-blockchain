use crate::config::HostInfo;
use crate::connection::Bytes;
use crate::error::Error;
use crate::node::{NodeEndpoint, NodeEntry, NodeId};
use crate::node_table::NodeTable;
use crate::PROTOCOL_VERSION;
use common::{keccak, recover, sign, Secret, H256, H520};
use rlp::{RLPStream, Rlp};
use std::collections::hash_map::Entry;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;

const ADDRESS_BYTES_SIZE: usize = 32;
const ADDRESS_BITS_SIZE: usize = ADDRESS_BYTES_SIZE * 8;
// Size of address type in bytes.
const MAX_NODES_PING: usize = 32;
// Max nodes to add/ping at once
const UDP_MAX_PACKET_SIZE: usize = 1500;
const EXPIRY_TIME: Duration = Duration::from_secs(20);
const BUCKET_SIZE: usize = 16; // Denoted by k in [Kademlia]. Number of nodes stored in each bucket.

const PACKET_PING: u8 = 1;
const PACKET_PONG: u8 = 2;
const PACKET_FIND_NODE: u8 = 3;
const PACKET_NEIGHBOURS: u8 = 4;

#[derive(Debug)]
pub struct BucketEntry {
    pub node: NodeEntry,
    pub id_hash: H256,
    pub last_seen: Instant,
    backoff_until: Instant,
    fail_count: usize,
}

impl BucketEntry {
    fn new(node: NodeEntry) -> Self {
        let now = Instant::now();
        BucketEntry {
            id_hash: keccak(node.id().as_bytes()),
            node,
            last_seen: now,
            backoff_until: now,
            fail_count: 0,
        }
    }
}

/// The metadata of the target nodes being pinged
struct PingNodeRequest {
    node: NodeEntry,
    reason: PingReason,
    timestamp: Instant,
    hash: H256,
}

/// Find node request
struct FindNodeRequest {
    sent_at: Instant,
    response_count: usize,
    answered: bool,
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

#[derive(Clone, Debug)]
pub enum Request {
    AddNode(NodeEntry),
    AddNodes(Vec<NodeEntry>),
    FindNode(NodeId, NodeEntry),
    /// Stop the discovery service
    Stop,
}

pub struct Discovery {
    is_stop: bool,
    handle: Option<JoinHandle<()>>,
    request_tx: Arc<mpsc::Sender<Request>>,
}

impl Discovery {
    pub async fn start(info: &HostInfo, node_table: Arc<RwLock<NodeTable>>) -> Result<Self, Error> {
        let (udp_tx, mut udp_rx) = mpsc::channel(1024);
        let (request_tx, mut request_rx) = mpsc::channel(1024);

        log::info!(
            "discovery starting udp at {:}",
            info.public_endpoint().udp_address()
        );

        let socket = UdpSocket::bind(info.public_endpoint().udp_address()).await?;
        let mut discovery = DiscoveryInner::new(info, node_table, udp_tx);
        let handle = tokio::spawn(async move {
            // tricky, need to 0 init, otherwise udp socket will return empty
            let mut buf = vec![0; UDP_MAX_PACKET_SIZE];

            loop {
                tokio::select! {
                    Some((bytes, target)) = udp_rx.recv() => {
                        match socket.send_to(&bytes, target).await {
                            Ok(_) => {},
                            Err(e) => log::error!("error sending udp {:?}", e),
                        }
                    }
                    Ok((size, peer)) = socket.recv_from(&mut buf) => {
                        let data = &buf[..size];
                        match discovery.on_packet(data, peer).await {
                            Ok(_) => {},
                            Err(e) => log::error!("error processing packet {:?}", e),
                        }
                    }
                    Some(request) = request_rx.recv() => {
                        log::debug!("received request: {:?}", request);
                        if let Request::Stop = request { break; }
                        discovery.handle(request).await;
                    }
                }
            }
            log::info!("discovery ended");
        });

        Ok(Self {
            is_stop: false,
            handle: Some(handle),
            request_tx: Arc::new(request_tx),
        })
    }

    pub async fn stop(&mut self) {
        if self.is_stop {
            return;
        }
        self.is_stop = true;
        self.request_tx.send(Request::Stop).await.unwrap_or_default();
    }

    /// Add a new node to discovery table. Pings the node.
    pub async fn add_node(&mut self, e: NodeEntry) -> Result<(), SendError<Request>> {
        self.request_tx.send(Request::AddNode(e)).await
    }

    /// Add a list of nodes. Pings a few nodes each round
    pub async fn add_node_list(&mut self, nodes: Vec<NodeEntry>) -> Result<(), SendError<Request>> {
        self.request_tx.send(Request::AddNodes(nodes)).await
    }

    /// Find nodes that are closest to the `to_find` from `from`
    pub async fn find_node(
        &mut self,
        to_find: NodeId,
        from: NodeEntry,
    ) -> Result<(), SendError<Request>> {
        self.request_tx.send(Request::FindNode(to_find, from)).await
    }
}

impl Drop for Discovery {
    fn drop(&mut self) {
        let tx = Arc::clone(&self.request_tx);
        if let Some(join_handler) = self.handle.take() {
            futures::executor::block_on(async move {
                tx.send(Request::Stop).await.unwrap_or_default();
                join_handler.await.unwrap_or_default();
            });
        }
    }
}

/// The inner struct of Discovery, handles all the processing logic
struct DiscoveryInner {
    /// Node storage for external nodes in the P2P
    node_table: Arc<RwLock<NodeTable>>,
    /// The node id of self
    id: NodeId,
    /// The hash of self node id
    id_hash: H256,
    /// The secret of self
    secret: Secret,
    /// The self public endpoint
    public_endpoint: NodeEndpoint,
    /// The buckets that hold the external nodes
    buckets: Vec<VecDeque<BucketEntry>>,
    /// Not allowed node ids
    not_allowed: HashSet<NodeId>,
    /// The nodes that is currently being pinged
    pinging_nodes: HashMap<NodeId, PingNodeRequest>,
    /// The nodes that is currently being `find`
    finding_nodes: HashMap<NodeId, FindNodeRequest>,
    /// The node entries to be added
    to_add: Vec<NodeEntry>,

    sender: mpsc::Sender<(Bytes, SocketAddr)>,
}

impl DiscoveryInner {
    pub fn new(
        info: &HostInfo,
        node_table: Arc<RwLock<NodeTable>>,
        udp_tx: mpsc::Sender<(Bytes, SocketAddr)>,
    ) -> Self {
        Self {
            node_table,
            id: info.key_pair().public().clone(),
            id_hash: keccak(info.key_pair().public().as_bytes()),
            secret: info.key_pair().secret().clone(),
            public_endpoint: info.public_endpoint(),
            buckets: (0..ADDRESS_BYTES_SIZE * 8)
                .map(|_| VecDeque::new())
                .collect(),
            not_allowed: HashSet::new(),
            pinging_nodes: HashMap::new(),
            finding_nodes: HashMap::new(),
            to_add: vec![],
            sender: udp_tx,
        }
    }

    // ========= Handling Requests =========

    /// Handling different requests
    async fn handle(&mut self, request: Request) {
        let r = match request {
            Request::AddNode(e) => self.add_node(e).await,
            Request::AddNodes(ns) => self.add_node_list(ns).await,
            Request::FindNode(id, node) => self.find_node(id, &node).await,
            _ => Ok(()),
        };
        match r {
            Ok(_) => {}
            Err(e) => log::error!("error handling request: {:?}", e),
        }
    }

    /// Add a new node to discovery table. Pings the node
    async fn add_node(&mut self, e: NodeEntry) -> Result<(), Error> {
        log::debug!("attempt to add node: {:?}", e);
        let node_hash = keccak(e.id().as_bytes());
        match Self::distance(&self.id_hash, &node_hash) {
            Some(d) => {
                if self.buckets[d].iter().any(|bn| bn.node.id() == e.id()) {
                    return Ok(());
                }
                self.try_ping(e, PingReason::Default).await
            }
            None => Err(Error::InvalidNodeDistance),
        }
    }

    /// Add a list of nodes. Pings a few nodes each round
    async fn add_node_list(&mut self, nodes: Vec<NodeEntry>) -> Result<(), Error> {
        for n in nodes {
            self.add_node(n).await?;
        }
        Ok(())
    }

    async fn find_node(&mut self, target: NodeId, node: &NodeEntry) -> Result<(), Error> {
        let mut rlp = RLPStream::new_list(2);
        rlp.append(&target);
        append_expiration(&mut rlp);

        self.send_packet(PACKET_FIND_NODE, &rlp.out(), node.endpoint().udp_address())
            .await?;
        log::debug!("sent FindNode to {:?}", node);

        self.finding_nodes.insert(
            *node.id(),
            FindNodeRequest {
                sent_at: Instant::now(),
                response_count: 0,
                answered: false,
            },
        );

        Ok(())
    }

    // ========= On Receiving Packets =========
    /// Responds when there is a packet
    async fn on_packet(&mut self, packet: &[u8], from: SocketAddr) -> Result<(), Error> {
        // validate packet
        if packet.len() < 32 + 65 + 4 + 1 {
            return Err(Error::BadProtocol);
        }

        // check hash of package
        let hash_signed = keccak(&packet[32..]);
        if hash_signed[..] != packet[0..32] {
            log::error!("signature of packet does not match, packet size: {:}", packet.len());
            return Err(Error::PacketHashNotMatch);
        }

        // recover message sender node id
        let signed = &packet[(32 + 65)..];
        let signature = H520::from_slice(&packet[32..(32 + 65)]);
        let node_id = recover(&signature.into(), &keccak(signed))?;

        // handle the actual data
        let packet_id = signed[0];
        match packet_id {
            PACKET_PING => {
                self.on_ping(&signed[1..], node_id, from, hash_signed.as_bytes())
                    .await
            }
            PACKET_PONG => self.on_pong(&signed[1..], node_id, from).await,
            // TODO: implement find node
            PACKET_FIND_NODE => self.on_find_node(&signed[1..], node_id, from).await,
            PACKET_NEIGHBOURS => self.on_neighbours(&signed[1..], node_id, from).await,
            _ => {
                log::debug!("Unknown UDP packet: {}", packet_id);
                Ok(())
            }
        }
    }

    async fn on_find_node(
        &mut self,
        bytes: &[u8],
        from_node: NodeId,
        from_socket: SocketAddr,
    ) -> Result<(), Error> {
        // step 1. parse the bytes
        log::debug!("got find node from {:?} ; node_id={:#x}", &from_socket, from_node);

        let rlp = Rlp::new(bytes);
        let target: NodeId = rlp.val_at(0)?;
        let expire: u64 = rlp.val_at(1)?;
        self.check_expired(expire)?;

        // let dis = match Self::distance(&self.id_hash, &keccak(target.as_bytes())) {
        //     None => { return Ok(()); }
        //     Some(dis) => dis
        // };
        let nodes = self.closest_node(&target).await;

        Ok(())
    }

    async fn on_neighbours(
        &mut self,
        bytes: &[u8],
        node_id: NodeId,
        from: SocketAddr,
    ) -> Result<(), Error> {
        log::debug!("got neighbours from {:?} ; node_id={:#x}", &from, node_id);

        let rlp = Rlp::new(bytes);

        let nodes_count = rlp.at(0)?.item_count()?;
        let is_expected = match self.finding_nodes.entry(node_id) {
            Entry::Occupied(mut entry) => {
                let expected = {
                    let request = entry.get_mut();
                    if request.response_count + nodes_count <= BUCKET_SIZE {
                        request.response_count += nodes_count;
                        true
                    } else {
                        log::debug!("got unexpected Neighbors from {:?} ; oversized packet ({} + {}) node_id={:#x}", &from, request.response_count, nodes_count, node_id);
                        false
                    }
                };

                // TODO: we should have some sort of timeout checks,
                // TODO: ensure that it's not dangling messages.
                if entry.get().response_count == BUCKET_SIZE {
                    entry.remove();
                }
                expected
            }
            Entry::Vacant(_) => false,
        };

        if !is_expected {
            return Ok(());
        }

        // now we parse the packet with the following:
        // packet-data = [nodes, expiration, ...]
        // nodes = [[ip, udp-port, tcp-port, node-id], ...]
        let _expiration: u64 = rlp.val_at(1)?;
        let mut nodes = vec![];
        for r in rlp.at(0)?.iter() {
            let id: NodeId = r.val_at(3)?;

            // not processing self
            if id == self.id { continue; }

            let endpoint = NodeEndpoint::from_rlp(&r)?;
            if !endpoint.is_valid_discovery_node() {
                log::debug!("invalid address: {:?}", endpoint);
                continue;
            }

            if !self.is_allowed(&id) {
                log::debug!("node id not allowed: {:?}", id);
                continue;
            }

            let entry = NodeEntry::new(id, endpoint);
            nodes.push(entry);
        }

        // // avoid Send in rlp, just make Rust happy
        // for n in nodes {
        //     self.add_node(n).await?;
        // }

        Ok(())
    }

    async fn on_ping(
        &mut self,
        bytes: &[u8],
        node_id: NodeId,
        from: SocketAddr,
        echo_hash: &[u8],
    ) -> Result<(), Error> {
        log::debug!("got ping from {:?} ; node_id={:#x}", &from, node_id);
        let rlp = Rlp::new(bytes);
        let ping_from = if let Ok(ne) = NodeEndpoint::from_rlp(&rlp.at(1)?) {
            ne
        } else {
            // If there are no endpoints returned, then most likely it's a
            // message from the Boot Nodes. Set the port to 0 as it should
            // not be used in syncing.
            let mut address = from.clone();
            address.set_port(0);
            NodeEndpoint::from_socket(address, from.port())
        };
        let ping_to = NodeEndpoint::from_rlp(&rlp.at(2)?)?;
        let timestamp: u64 = rlp.val_at(3)?;
        self.check_expired(timestamp)?;

        // now form the response packet, https://github.com/ethereum/devp2p/blob/master/discv4.md#pong-packet-0x02
        let mut response = RLPStream::new_list(3);
        ping_to.to_rlp_list(&mut response);
        response.append(&echo_hash);
        append_expiration(&mut response);

        self.send_packet(PACKET_PONG, &response.out(), from.clone())
            .await?;

        let pong_to = NodeEndpoint::from_socket(from.clone(), ping_from.udp_port);
        let entry = NodeEntry::new(node_id.clone(), pong_to);
        if !entry.endpoint().is_valid_discovery_node() {
            log::debug!("got bad address: {:?}", entry);
        } else if !self.is_allowed(&node_id) {
            log::debug!("address not allowed: {:?}", entry);
        } else {
            log::debug!("adding node on ping from: {:?}", entry);
            self.add_node(entry).await?;
        }
        Ok(())
    }

    async fn on_pong(
        &mut self,
        bytes: &[u8],
        node_id: NodeId,
        from: SocketAddr,
    ) -> Result<(), Error> {
        log::debug!("got pong from {:?} ; node_id={:#x}", &from, node_id);

        let rlp = Rlp::new(bytes);
        let echo_hash: H256 = rlp.val_at(1)?;
        let timestamp: u64 = rlp.val_at(2)?;
        self.check_expired(timestamp)?;

        match self.pinging_nodes.entry(node_id) {
            Entry::Occupied(entry) => {
                if echo_hash != entry.get().hash {
                    log::debug!("Hash doesn't match for node {:?} at {:?}", node_id, from);
                    return Ok(());
                }
                let meta = entry.remove();
                if let PingReason::FromDiscoveryRequest(node_id, _validity) = meta.reason {
                    log::info!("node id: {:?}", node_id);
                } else {
                    self.update_node(meta.node).await?;
                }
                Ok(())
            }
            Entry::Vacant(_) => Ok(()),
        }
    }

    // ========= Helper Functions =========
    async fn closest_node(&self, target: &NodeId) -> Vec<NodeId> {
        let mut nodes = BinaryHeap::with_capacity(BUCKET_SIZE);
        for bucket in &self.buckets {
            for entry in bucket {
                if nodes.len() < BUCKET_SIZE {
                    nodes.push(entry);
                } else if  {

                }
            }
        }
        let mut nodes = Vec::with_capacity(BUCKET_SIZE);
        let append_nodes = |nodes: &mut Vec<NodeId>, bucket: &VecDeque<BucketEntry>| {
            let mut candidates: Vec<BucketEntry> = bucket.iter().collect();
            candidates.sort_unstable_by_key(|a, b| a.id_hash ^ b.id_hash);

            let vacancies = nodes.capacity() - nodes.len();
            let mut to_append = if vacancies >= candidates.len() {
                candidates.iter().map(|e| e.node.id().clone()).collect()
            } else {
                candidates[..vacancies].iter().map(|e| e.node.id().clone()).collect()
            };
            nodes.append(&mut to_append);

            nodes.len() == BUCKET_SIZE
        };

        let target_distance = self.id_hash ^ keccak(target.as_bytes());

        for i in 0..ADDRESS_BITS_SIZE {
            // the distance bit at i is set
            if target_distance[i / 8] & 1 << i % 8 == 1 {
                // target distance at index 0 corresponds to
                // the max 8 bits of the bucket, need to reverse
                let bucket = &self.buckets[ADDRESS_BITS_SIZE - 1 - i];
                if !bucket.is_empty() && append_nodes(&mut nodes, bucket) {
                    return nodes;
                }
            }
        }

        for i in (0..ADDRESS_BITS_SIZE).rev() {
            // the distance bit at i is set
            if target_distance[i / 8] & 1 << i % 8 == 0 {
                let bucket = &self.buckets[ADDRESS_BITS_SIZE - 1 - i];
                if !bucket.is_empty() && append_nodes(&mut nodes, bucket) {
                    return nodes;
                }
            }
        }
        vec![]
    }

    async fn update_node(&mut self, n: NodeEntry) -> Result<(), Error> {
        match self.update_bucket(n) {
            Err(Error::NodeIsSelf) => {}
            Err(Error::NodeNotFoundInBucket { entry, distance }) => {
                log::debug!(
                    "adding node: {:?} with distance {:?} to bucket",
                    entry,
                    distance
                );

                self.buckets[distance].push_front(BucketEntry::new(entry.clone()));

                // When BUCKET_SIZE, the least recently seen node in the bucket needs to be
                // revalidated by sending a Ping packet. If no reply is received, it is
                // considered dead, removed and N₁ added to the front of the bucket.
                if self.buckets[distance].len() > BUCKET_SIZE {
                    self.try_ping(
                        // unwrap should be safe
                        node_to_ping(&self.buckets[distance]).unwrap(),
                        PingReason::Default,
                    )
                    .await?;
                }

                if entry.endpoint().is_valid_discovery_node() {
                    let mut table = self.node_table.write().await;
                    table.upsert(vec![entry]);
                }
            }
            _ => {}
        };
        Ok(())
    }

    /// Only update the entries in bucket
    fn update_bucket(&mut self, n: NodeEntry) -> Result<(), Error> {
        let hash = keccak(n.id().as_bytes());
        let distance = Self::distance(&hash, &self.id_hash).ok_or(Error::NodeIsSelf)?;
        self.buckets[distance]
            .iter_mut()
            .find(|v| v.node.id() == n.id())
            .map_or(
                Err(Error::NodeNotFoundInBucket {
                    entry: n.clone(),
                    distance,
                }),
                |mut v| {
                    v.node = n;
                    v.backoff_until = Instant::now();
                    v.last_seen = Instant::now();
                    v.fail_count = 0;
                    Ok(())
                },
            )
    }

    fn check_expired(&self, timestamp: u64) -> Result<(), Error> {
        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if current_timestamp < timestamp {
            Ok(())
        } else {
            Err(Error::PongExpired)
        }
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
            log::info!(
                "pinging nodes full, add node id {} to pending nodes",
                e.id()
            );
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

        let hash = self
            .send_packet(PACKET_PING, &rlp.out(), e.endpoint().udp_address())
            .await?;

        // save the metadata for Pong
        self.pinging_nodes.insert(
            *e.id(),
            PingNodeRequest {
                node: e,
                reason,
                timestamp: Instant::now(),
                hash,
            },
        );

        Ok(())
    }

    async fn send_packet(
        &self,
        packet_type: u8,
        packet_bytes: &[u8],
        socket: SocketAddr,
    ) -> Result<H256, Error> {
        let packet = assemble_packet(packet_type, packet_bytes, &self.secret)?;
        let hash = H256::from_slice(&packet[..32]);
        // send to the channel for processing
        self.sender.send((packet, socket)).await?;
        Ok(hash)
    }

    /// Checks if the node_id is allowed for connection
    fn is_allowed(&self, node_id: &NodeId) -> bool {
        !self.not_allowed.contains(node_id)
    }

    /// Calculate the node distances based on XOR
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

fn node_to_ping(nodes: &VecDeque<BucketEntry>) -> Option<NodeEntry> {
    let now = Instant::now();
    nodes
        .iter()
        .filter(|n| n.backoff_until < now)
        .min_by_key(|n| n.last_seen)
        .map(|n| n.node.clone())
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
    use crate::discovery::{DiscoveryInner, ADDRESS_BYTES_SIZE};
    use crate::node::NodeId;
    use crate::{HostInfo, NodeTable};
    use common::{keccak, H256};
    use std::collections::{HashMap, HashSet, VecDeque};
    use std::net::SocketAddr;
    use std::str::FromStr;
    use std::sync::Arc;
    use tokio::net::UdpSocket;
    use tokio::sync::{mpsc, RwLock};

    fn mock_discovery_inner() -> DiscoveryInner {
        let info = HostInfo::default();
        let node_table = Arc::new(RwLock::new(NodeTable::new_in_memory()));

        let (udp_tx, _) = mpsc::channel(1024);
        DiscoveryInner::new(&info, node_table, udp_tx)
    }

    #[test]
    fn distance_works() {
        let a = H256::from_slice(&[
            228, 104, 254, 227, 239, 33, 109, 25, 223, 95, 27, 195, 177, 52, 50, 204, 76, 30, 147,
            218, 216, 159, 47, 146, 236, 13, 163, 128, 250, 160, 17, 192,
        ]);
        let b = H256::from_slice(&[
            228, 214, 227, 65, 84, 85, 107, 82, 209, 81, 68, 106, 172, 254, 164, 105, 92, 23, 184,
            27, 10, 90, 228, 69, 143, 90, 18, 117, 49, 186, 231, 5,
        ]);

        let result = DiscoveryInner::distance(&a, &b);
        assert_eq!(result, Some(247));
    }

    #[tokio::test]
    async fn on_neighbour_works() {
        let packet = [
            249, 1, 68, 249, 1, 60, 248, 77, 132, 3, 19, 109, 47, 130, 118, 95, 130, 118, 95, 184,
            64, 230, 28, 44, 31, 17, 212, 63, 86, 36, 199, 104, 56, 23, 82, 99, 186, 245, 219, 174,
            61, 13, 25, 190, 148, 196, 184, 10, 105, 3, 191, 73, 179, 121, 110, 29, 4, 50, 126,
            134, 0, 83, 62, 189, 219, 232, 109, 83, 170, 123, 187, 136, 187, 111, 51, 203, 132,
            149, 185, 31, 245, 123, 197, 151, 2, 248, 77, 132, 65, 21, 192, 96, 130, 118, 95, 130,
            118, 95, 184, 64, 82, 233, 15, 53, 126, 77, 181, 92, 103, 75, 98, 101, 126, 45, 184,
            75, 124, 209, 142, 35, 18, 167, 215, 83, 26, 224, 175, 60, 47, 104, 18, 91, 45, 211,
            99, 224, 19, 137, 35, 252, 122, 71, 232, 57, 109, 226, 200, 29, 91, 173, 14, 10, 26,
            100, 158, 148, 88, 51, 162, 68, 112, 98, 124, 227, 248, 77, 132, 161, 97, 125, 179,
            130, 118, 95, 130, 118, 95, 184, 64, 72, 188, 36, 62, 107, 189, 108, 83, 153, 76, 97,
            214, 128, 114, 134, 209, 126, 109, 133, 223, 188, 187, 74, 166, 187, 240, 102, 35, 196,
            168, 85, 205, 179, 172, 152, 90, 164, 106, 208, 193, 249, 83, 236, 201, 251, 130, 14,
            196, 224, 245, 158, 39, 229, 209, 27, 210, 112, 90, 193, 6, 162, 167, 188, 160, 248,
            77, 132, 142, 132, 140, 165, 130, 118, 95, 130, 118, 95, 184, 64, 152, 157, 113, 208,
            148, 119, 140, 96, 115, 45, 118, 125, 22, 249, 38, 91, 84, 238, 116, 252, 7, 37, 181,
            45, 29, 80, 24, 78, 214, 96, 199, 139, 149, 10, 208, 29, 83, 67, 204, 145, 36, 129,
            106, 67, 170, 131, 144, 138, 201, 131, 203, 81, 84, 106, 106, 48, 115, 50, 207, 175,
            82, 134, 89, 58, 132, 97, 255, 181, 207,
        ];
        let mut mock_inner = mock_discovery_inner();
        mock_inner
            .on_neighbours(
                &packet,
                NodeId::random(),
                SocketAddr::from_str("0.0.0.0:30303").unwrap(),
            )
            .await.unwrap();
    }
}
