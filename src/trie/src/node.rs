use crate::encoding::hex_to_compact;
use crate::storage::NodeLocation;
use common::{from_vec, to_vec, Hash, Hasher};
use rlp::RLPStream;
use serde::{Deserialize, Serialize};

// The length of children is 17 because of the termination symbol
pub(crate) const CHILD_SIZE: usize = 17;

/// The Node in the MPT.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub(crate) enum Node {
    Empty,
    Full {
        children: Box<[NodeLocation; CHILD_SIZE]>,
    },
    Short {
        key: Vec<u8>,
        val: NodeLocation,
    },
    Value {
        key: Vec<u8>,
        val: Vec<u8>
    }
}

#[cfg(any(feature = "std"))]
impl From<Node> for Vec<u8> {
    fn from(n: Node) -> Self {
        to_vec(&n).unwrap()
    }
}

#[cfg(any(feature = "std"))]
impl From<Vec<u8>> for Node {
    fn from(n: Vec<u8>) -> Self {
        from_vec(&n).unwrap()
    }
}

impl Node {
    /// Encode node into
    /// * `ch` - The method that processes the child of the node
    pub fn encode<'a, H, FM, FL>(self, mut ch: FM, mut node_locator: FL) -> Vec<u8>
    where
        H: Hasher,
        FM: FnMut(NodeLocation) -> ChildReference,
        FL: FnMut(NodeLocation) -> Node,
    {
        match self {
            // todo: handle empty node
            Node::Empty => vec![],
            Node::Full { .. } => vec![],
            Node::Short {
                mut key,
                val: node_loc,
            } => {
                let child = node_locator(node_loc.clone());
                if let Node::Value {key: _, val: nval} = child {
                    Encoder::value_node(key, nval)
                } else {
                    Encoder::short_node(hex_to_compact(&key), ch(node_loc))
                }
            },
            Node::Value { key, val} => Encoder::value_node(hex_to_compact(&key), val),
        }
    }

    pub fn update_key(&mut self, k: &[u8]) {
        match self {
            Node::Short { key, .. } => *key = k.clone().to_vec(),
            Node::Value { key, .. } => *key = k.clone().to_vec(),
            _ => {}
        }
    }
}

/// This is a helper enum for hashing the nodes. During hashing of the nodes, i.e. Full node,
/// we need to hash the children first. This would require us to have sth that holds the hash
/// of the children. ChildReference is that sth.
pub(crate) enum ChildReference {
    Hash(Hash),
    Inline(Hash, usize)
}

/// The encoder used to convert node to bytes
struct Encoder;

impl Encoder {
    pub fn short_node(key: Vec<u8>, child_ref: ChildReference) -> Vec<u8> {
        match child_ref {
            ChildReference::Hash(hash) => vec![],
            ChildReference::Inline(..) => { vec![] }
        }
    }

    pub fn value_node(key: Vec<u8>, val: Vec<u8>) -> Vec<u8> {
        let mut rlp = RLPStream::new();
        rlp.append(&key);
        rlp.append(&val);
        rlp.into()
    }
}
