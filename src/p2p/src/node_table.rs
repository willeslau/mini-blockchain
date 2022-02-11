use crate::node::NodeId;
use crate::{NodeEndpoint, NodeEntry};
use kv_storage::{DBStorage, MemoryDB};
use std::collections::HashMap;
// use std::time::SystemTime;

/// The different types of a Peer
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub(crate) enum PeerType {
    _Required,
    Optional,
}

/// A type for representing an interaction (contact) with a node at a given time
/// that was either a success or a failure.
#[derive(Clone, Copy, Debug)]
pub(crate) enum NodeContact {
    // Success(SystemTime),
    // Failure(SystemTime),
}

pub struct Node {
    id: NodeId,
    endpoint: NodeEndpoint,
    peer_type: PeerType,
    last_contact: Option<NodeContact>,
}

impl Node {
    pub fn new(id: NodeId, endpoint: NodeEndpoint) -> Self {
        Self {
            id,
            endpoint,
            peer_type: PeerType::Optional,
            last_contact: None,
        }
    }
}

pub struct NodeTable {
    nodes: HashMap<NodeId, Node>,
    storage: Box<dyn DBStorage>,
}

impl NodeTable {
    pub fn new(storage: Box<dyn DBStorage>) -> Self {
        Self {
            nodes: HashMap::with_capacity(1024),
            storage,
        }
    }

    pub fn new_in_memory() -> Self {
        let inner = MemoryDB::new();
        Self::new(Box::new(inner))
    }

    // pub fn remove(&mut self, nodes: Vec<NodeEntry>) {}

    pub fn upsert(&mut self, entries: Vec<NodeEntry>) {
        for e in entries {
            let (id, endpoint) = e.into();
            let n = Node::new(id, endpoint);
            self.nodes.insert(n.id, n);
        }
    }

    /// Flush in memory nodes to db
    pub fn flush(&mut self) {}
}
