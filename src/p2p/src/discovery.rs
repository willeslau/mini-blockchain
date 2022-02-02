use std::collections::{HashMap, HashSet};
use common::{Error, H256, keccak};
use crate::db::Storage;
use crate::node::{NodeEntry, NodeId};

const ADDRESS_BYTES_SIZE: usize = 32; // Size of address type in bytes.
const MAX_NODES_PING: usize = 32; // Max nodes to add/ping at once

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

    fn ping(&mut self, _e: NodeEntry, _reason: PingReason) -> Result<(), Error> {
        Ok(())
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
