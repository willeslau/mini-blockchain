use common::{Hash, Hasher, KeccakHasher};
use kv_storage::HashDB;
use rlp::{Encodable, RLPStream};
use crate::encoding::hex_to_compact;
use crate::node::{CHILD_SIZE, Node};
use crate::storage::{Cache, NodeLocation};

pub(crate) struct NodeHasher<'a, H: HashDB> {
    db: &'a mut H,
    cache: Cache,
    hash_count: usize,
}

impl<'a, H: HashDB> NodeHasher<'a, H> {
    pub fn hash<F>(&mut self, node: Node, mut locator: F) -> ChildReference where F: FnMut(NodeLocation) -> Node {
        match node {
            // TODO: add empty node hash
            Node::Empty => { ChildReference::Hash(Hash::default()) }
            Node::Full { children } => { ChildReference::Hash(Hash::default()) }
            Node::Short {
                mut key,
                val: node_loc,
            } => self.hash_short_node_children(key, locator(node_loc), locator),
            // Should not process this type as Short node branch should have handled it.
            // Well, this might not be the best way to do this. The issue here is the key
            // to the value node is not stored, hence we need to get the key from Short node.
            Node::Value { .. } => panic!("invalid state"),
        };
        ChildReference::Hash(Hash::default())
    }

    fn hash_short_node_children<F>(&mut self, key: Vec<u8>, node: Node, mut locator: F) -> ChildReference where F: FnMut(NodeLocation) -> Node {
        let encoded = if let Node::Value { key: _, val: nval } = node {
            Encoder::value_node(key, nval)
        } else {
            Encoder::short_node(key, self.hash(node, locator))
        };
        self.process_encoded(encoded)
    }

    // fn hash_full_node_children<F>(&mut self, mut children: Box<[NodeLocation; CHILD_SIZE]>, locator: F) -> ChildReference where F: FnMut(NodeLocation) -> Node {
    //     let refs = children
    //         .iter_mut()
    //         .map(Option::take)
    //         .map(|c| {
    //         let node = locator(c);
    //         match node {
    //             Node::Empty => None,
    //             _ => Some(self.hash(node, locator))
    //         }
    //     });
    //     self.process_encoded(Encoder::full_node(Box::new(refs)))
    // }

    fn process_encoded(&mut self, encoded: Vec<u8>) -> ChildReference {
        if encoded.len() >= KeccakHasher::LENGTH {
            let hash = KeccakHasher::hash(&encoded);
            self.db.insert(Vec::from(hash.clone()), encoded);
            self.hash_count += 1;
            ChildReference::Hash(hash)
        } else {
            ChildReference::Inline(encoded)
        }
    }
}

/// This is a helper enum for hashing the nodes. During hashing of the nodes, i.e. Full node,
/// we need to hash the children first. This would require us to have sth that holds the hash
/// of the children. ChildReference is that sth.
pub(crate) enum ChildReference {
    Hash(Hash),
    Inline(Vec<u8>),
}

/// The encoder used to convert node to bytes
struct Encoder;

impl Encoder {
    pub fn full_node(children: Box<[Option<ChildReference>; CHILD_SIZE]>) -> Vec<u8> {
        let mut stream = RLPStream::new_list(CHILD_SIZE);
        for c in children.iter() {
            match c {
                None => stream.append_empty(),
                Some(r) => Self::handle_ref(&mut stream, r)
            }
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
            ChildReference::Inline(inline_data) => stream.append(&inline_data)
        };
    }
}
