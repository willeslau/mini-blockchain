use common::{from_vec, to_vec, Hash};
use serde::{Deserialize, Serialize};

const CHILD_SIZE: usize = 17;
type HashNode = Vec<u8>;
type ValueNode = Vec<u8>;

/// The node flags for extra configuration
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct NodeFlag {
    hash: Option<HashNode>,
    dirty: bool,
}

impl NodeFlag {
    pub fn new(dirty: bool) -> Self {
        Self { hash: None, dirty }
    }
}

/// CachedNode is all the information we know about a single cached trie node
/// in the memory database write layer.
pub struct CachedNode {
    node: Node,
    size: u32,
    parents: u32,
}

impl CachedNode {}

/// The Node in the MPT.
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Node {
    Empty,
    FullNode {
        children: [Option<Box<Node>>; CHILD_SIZE],
        flags: NodeFlag,
    },
    ShortNode {
        key: Hash,
        val: Box<Node>,
        flags: NodeFlag,
    },
    HashNode(Vec<u8>),
    ValueNode(Vec<u8>),
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
