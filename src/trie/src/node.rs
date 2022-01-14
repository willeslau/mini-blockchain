use crate::storage::NodeLocation;
use common::{from_vec, to_vec, H256};
use serde::{Deserialize, Serialize};

// The length of children is 17 because of the termination symbol
pub(crate) const CHILD_SIZE: usize = 17;

/// The item to delete
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) enum DeleteItem {
    /// It's a node type, need to hash first
    Node(Node),
    /// It's a hash, can delete from DB
    Hash(H256),
}

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
