use crate::encoding::hex_to_compact;
use crate::node::{Node, CHILD_SIZE};
use crate::storage::{Cache, MemorySlot, NodeLocation};
use common::{H256, Hasher, KeccakHasher};
use kv_storage::DBStorage;
use rlp::RLPStream;

pub(crate) struct NodeHasher {
    hash_count: usize,
}

impl NodeHasher {
    pub fn new() -> Self {
        Self { hash_count: 0 }
    }

    pub fn hash<H: DBStorage>(&mut self, node: Node, db: &mut H, cache: &mut Cache) -> H256 {
        match self.hash_inner(node, db, cache) {
            ChildReference::Hash(h) => h,
            ChildReference::Inline(v) => self.insert_db_raw(v, db),
            _ => panic!("invalid state"),
        }
    }

    pub fn hash_inner<H: DBStorage>(
        &mut self,
        node: Node,
        db: &mut H,
        cache: &mut Cache,
    ) -> ChildReference {
        match node {
            // TODO: add empty node hash
            Node::Empty => ChildReference::Hash(H256::default()),
            Node::Full { children } => self.hash_full_node_children(children, db, cache),
            Node::Short { key, val: node_loc } => {
                let nd = self.take_node_loc(&node_loc, cache);
                self.hash_short_node_children(key, db, nd, cache)
            }
            // Should not process this type as Short node branch should have handled it.
            // Well, this might not be the best way to do this. The issue here is the key
            // to the value node is not stored, hence we need to get the key from Short node.
            _ => panic!("invalid state"),
        }
    }

    fn take_node_loc(&mut self, node_loc: &NodeLocation, cache: &mut Cache) -> NodeData {
        match node_loc {
            NodeLocation::Persistence(h) => NodeData::Hash(H256::from_slice(h)),
            NodeLocation::Memory(i) => match cache.take(*i) {
                MemorySlot::Updated(node) => NodeData::Node(node),
                MemorySlot::Loaded(h, _) => NodeData::Hash(h),
            },
            NodeLocation::None => NodeData::Node(Node::Empty),
        }
    }

    fn hash_short_node_children<H: DBStorage>(
        &mut self,
        key: Vec<u8>,
        db: &mut H,
        nd: NodeData,
        cache: &mut Cache,
    ) -> ChildReference {
        let k = hex_to_compact(&key);
        let encoded = match nd {
            NodeData::Hash(h) => Encoder::short_node(k, ChildReference::Hash(h)),
            NodeData::Node(node) => {
                if let Node::Value(val) = node {
                    Encoder::value_node(k, val)
                } else {
                    Encoder::short_node(k, self.hash_inner(node, db, cache))
                }
            }
        };
        self.insert_encoded(encoded, db)
    }

    fn hash_full_node_children<H: DBStorage>(
        &mut self,
        children: Box<[NodeLocation; CHILD_SIZE]>,
        db: &mut H,
        cache: &mut Cache,
    ) -> ChildReference {
        let mut refs = Vec::with_capacity(CHILD_SIZE);
        for i in 0..CHILD_SIZE - 1 {
            let c = &children[i];
            match self.take_node_loc(c, cache) {
                NodeData::Hash(h) => refs.push(Some(ChildReference::Hash(h))),
                NodeData::Node(node) => match node {
                    Node::Empty => refs.push(None),
                    _ => {
                        refs.push(Some(self.hash_inner(node, db, cache)));
                    }
                },
            }
        }

        // process the 17th element, the terminal
        let tm = &children[CHILD_SIZE - 1];
        match self.take_node_loc(tm, cache) {
            NodeData::Hash(h) => refs.push(Some(ChildReference::Hash(h))),
            NodeData::Node(node) => match node {
                Node::Empty => refs.push(None),
                Node::Value(v) => refs.push(Some(ChildReference::Value(v))),
                _ => panic!("invalid state"),
            },
        }
        self.insert_encoded(Encoder::full_node(refs), db)
    }

    /// Hash the encoded node or keep the raw data if len is short
    fn insert_encoded<H: DBStorage>(&mut self, encoded: Vec<u8>, db: &mut H) -> ChildReference {
        if encoded.len() >= KeccakHasher::LENGTH {
            ChildReference::Hash(self.insert_db_raw(encoded, db))
        } else {
            ChildReference::Inline(encoded)
        }
    }

    fn insert_db_raw<H: DBStorage>(&mut self, encoded: Vec<u8>, db: &mut H) -> H256 {
        let hash = KeccakHasher::hash(&encoded);
        db.insert(Vec::from(hash.as_bytes()), encoded);
        self.hash_count += 1;
        hash
    }
}

pub(crate) enum NodeData {
    Hash(H256),
    Node(Node),
}

/// This is a helper enum for hashing the nodes. During hashing of the nodes, i.e. Full node,
/// we need to hash the children first. This would require us to have sth that holds the hash
/// of the children. ChildReference is that sth.
#[derive(Debug, Clone)]
pub(crate) enum ChildReference {
    Hash(H256),
    Inline(Vec<u8>),
    Value(Vec<u8>),
}

/// The encoder used to convert node to bytes
struct Encoder;

impl Encoder {
    pub fn full_node(children: Vec<Option<ChildReference>>) -> Vec<u8> {
        let mut stream = RLPStream::new_list(CHILD_SIZE);
        for c in children {
            match c {
                None => {
                    stream.append_empty();
                }
                Some(r) => {
                    Self::handle_ref(&mut stream, r);
                }
            };
        }
        stream.into()
    }

    pub fn short_node(key: Vec<u8>, child_ref: ChildReference) -> Vec<u8> {
        let mut stream = RLPStream::new_list(2);
        stream.append(&key);
        Self::handle_ref(&mut stream, child_ref);
        stream.into()
    }

    pub fn value_node(key: Vec<u8>, val: Vec<u8>) -> Vec<u8> {
        let mut rlp = RLPStream::new_list(2);
        rlp.append(&key);
        rlp.append(&val);
        rlp.into()
    }

    fn handle_ref(stream: &mut RLPStream, child_ref: ChildReference) {
        match child_ref {
            ChildReference::Hash(h) => stream.append(&h),
            ChildReference::Inline(inline_data) => stream.append_raw(&inline_data),
            ChildReference::Value(v) => stream.append(&v),
        };
    }
}
