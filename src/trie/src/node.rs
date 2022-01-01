use crate::storage::NodeLocation;
use common::{from_vec, Hash, Hasher, to_vec};
use serde::{Deserialize, Serialize};
use crate::encoding::hex_to_compact;
use crate::rstd;
use rlp::RPLStream;

// The length of children is 17 because of the termination symbol
pub(crate) const CHILD_SIZE: usize = 17;

/// The Node in the MPT.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Node {
    Empty,
    Full {
        children: Box<[NodeLocation; CHILD_SIZE]>,
    },
    Short {
        key: Vec<u8>,
        val: NodeLocation,
    },
    Value(Vec<u8>),
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
    pub fn encode<H, F>(self, mut child_hf: F) -> Vec<u8>
        where
            H: Hasher,
            F: FnMut(NodeLocation) -> ChildReference
    {
        match self {
            // todo: handle empty node
            Node::Empty => H::hash(&vec![]),
            Node::Full { .. } => H::hash(&vec![]),
            Node::Short { mut key, val: node_loc } => {
                let nkey = hex_to_compact(&key);
                let c_ref = child_hf(node_loc);

                H::hash(&vec![])
            },
            Node::Value(_) => H::hash(&vec![]),
        };
        vec![]
    }
}

pub enum ChildReference {
    Hash(Hash),
    Value(Vec<u8>)
}

struct HashCodec;

impl HashCodec {
    pub fn short_node(key: Vec<u8>, child_ref: ChildReference) -> ChildReference {
        match child_ref {
            ChildReference::Hash(hash) => {

            },
            ChildReference::Value(val) => {
                // RLP key, val
                let mut rlp = RPLStream::new();
                rlp.write_iter(key.into_iter());
                rlp.write_iter(val.into_iter());
            }
        };

        ChildReference::Hash([0;32])
    }
}