use std::collections::HashSet;
use crate::encoding::{key_bytes_to_hex, prefix_len};
use crate::error::Error;
use crate::node::{Node, CHILD_SIZE};
use crate::rstd::mem;
use crate::storage::{Cache, CacheIndex, MemorySlot, NodeLocation};
use common::{ensure, Hash};
use kv_storage::HashDB;

type Prefix = Vec<u8>;

pub struct Trie<'a, H: HashDB> {
    db: &'a mut H,
    root_loc: NodeLocation,
    cache: Cache,
    delete_items: HashSet<Node>,
    unhashed: u32,
}

impl<'a, H: HashDB> Trie<'a, H> {
    /// The root_hash needs to be the empty node hash
    pub fn new(db: &'a mut H) -> Self {
        Self {
            db,
            root_loc: NodeLocation::None,
            cache: Cache::new(),
            delete_items: Default::default(),
            unhashed: 0,
        }
    }

    // // pub fn new_from_existing(db: &'db DB, root_hash: &[u8]) -> Self {
    // //
    // // }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        None
    }

    /// Try to update the key with provided value
    pub fn try_update(&mut self, key: &[u8], val: Vec<u8>) -> Result<(), Error> {
        ensure!(!key.is_empty(), Error::KeyCannotBeEmpty)?;
        ensure!(!val.is_empty(), Error::ValueCannotBeEmpty)?;

        self.unhashed += 1;
        let k = key_bytes_to_hex(key);
        let root = self.root_handle();
        let prefix = Prefix::default();
        self.root_loc = self.insert(root, prefix, k, val)?;
        Ok(())
    }

    fn insert(
        &mut self,
        node_loc: NodeLocation,
        prefix: Prefix,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<NodeLocation, Error> {
        let val_node = Node::Value(value);
        let val_loc = NodeLocation::Memory(self.cache.insert(MemorySlot::Updated(val_node)));
        self.insert_inner(node_loc, prefix, key, val_loc)
    }

    fn parse_node_loc(&mut self, node_loc: NodeLocation) -> Result<(CacheIndex, &mut Node), Error> {
        let cache_index = match node_loc {
            NodeLocation::Persistence(h) => self.load_to_cache(&h),
            NodeLocation::Memory(i) => i,
            _ => {
                return Err(Error::InvalidNodeLocation);
            }
        };

        // Always fetch the node from cache
        let node = match self.cache.get_mut(cache_index) {
            MemorySlot::Updated(node) => node,
            MemorySlot::Loaded(_, node) => node,
        };

        Ok((cache_index, node))
    }

    fn destroy(&mut self, node_loc: NodeLocation) -> Result<(), Error> {
        match node_loc {
            NodeLocation::None => Ok(()),
            NodeLocation::Persistence(_) => Err(Error::InvalidNodeLocation),
            NodeLocation::Memory(cache_index) => {
                match self.cache.take(cache_index) {
                    // since it's still in memory, no need to do anything
                    MemorySlot::Updated(n) => self.delete_items.insert(n),
                    MemorySlot::Loaded(_, n) => self.delete_items.insert(n),
                };
                Ok(())
            }
        }
    }

    fn insert_inner(
        &mut self,
        node_loc: NodeLocation,
        mut prefix: Prefix,
        key: Vec<u8>,
        value: NodeLocation,
    ) -> Result<NodeLocation, Error> {
        if matches!(node_loc, NodeLocation::None) {
            if key.is_empty() {
                return Ok(value);
            }
            let idx = self
                .cache
                .insert(MemorySlot::Updated(Node::Leaf { key, val: value }));
            return Ok(NodeLocation::Memory(idx));
        }

        // All these round trips because of not familiar with
        // rust... See readme of this module.
        let (cache_index, node) = self.parse_node_loc(node_loc)?;

        // require key not empty, maybe this case needs to be handled
        ensure!(!key.is_empty(), Error::KeyCannotBeEmpty)?;

        match node {
            Node::Empty => {
                let n = Node::Leaf { key, val: value };
                self.cache.replace(cache_index, MemorySlot::Updated(n));
                return Ok(NodeLocation::Memory(cache_index));
            }
            Node::Leaf {
                key: nkey,
                val: nval
            } => {
                let matchlen = prefix_len(nkey, &key);

                // This means the two keys match exactly
                if matchlen == nkey.len() {
                    // TODO: are these 2 steps necessary?
                    // update the prefix to include the matched key
                    prefix.append(&mut (key[..matchlen].to_vec()));
                    *nkey = prefix;

                    // Replace and delete old value
                    let v = mem::replace(nval, value);
                    self.destroy(v)?;

                    // now we update the slot
                    let slot = self.cache.take(cache_index);
                    let idx = self.cache.insert(slot.into_updated());
                    return Ok(NodeLocation::Memory(idx));
                }

                let mut children = [NodeLocation::None; CHILD_SIZE];
                let mut nprefix = prefix.clone();

                let c1 = key[matchlen] as usize;
                let c2 = nkey[matchlen] as usize;

                let k1 = key[matchlen + 1..].to_vec();
                let k2 = nkey[matchlen + 1..].to_vec();

                // A trick to avoid borrow mutable
                let nval = mem::replace(nval, NodeLocation::None);
                // One more index of matchlen because the branch node will consume one position.
                // So the prefix is effectively extended to [..matchlen+1]
                nprefix.append(&mut (nkey[..matchlen + 1]).to_vec());
                prefix.append(&mut (key[..matchlen + 1].to_vec()));

                let n1 = self.insert_inner(NodeLocation::None, prefix, k1, value)?;
                let n2 = self.insert_inner(NodeLocation::None, nprefix, k2, nval)?;

                children[c1] = n1;
                children[c2] = n2;

                let branch = Node::Branch {
                    children: Box::new(children),
                };
                let idx = self.cache.insert(MemorySlot::Updated(branch));

                return Ok(NodeLocation::Memory(idx));
            }
            _ => {}
        }
        Ok(NodeLocation::None)
    }

    fn load_to_cache(&mut self, h: &Hash) -> CacheIndex {
        let node = match self.db.get(h) {
            None => Node::Empty,
            Some(bytes) => Node::from(bytes),
        };
        self.cache.insert(MemorySlot::Loaded(*h, node))
    }

    // a hack to get the root node's handle
    fn root_handle(&self) -> NodeLocation {
        match self.root_loc {
            NodeLocation::Persistence(h) => NodeLocation::Persistence(h),
            NodeLocation::Memory(x) => NodeLocation::Memory(x),
            // this should never have happened
            NodeLocation::None => NodeLocation::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::trie::Trie;
    use kv_storage::MemoryDB;
    use crate::storage::NodeLocation;

    #[test]
    fn root_handle() {
        let mut hash_db = MemoryDB::new();
        let trie = Trie::new(&mut hash_db);

        assert_eq!(trie.root_handle(), NodeLocation::None);
    }

    #[test]
    fn insert_works() {
        let mut hash_db = MemoryDB::new();
        let mut trie = Trie::new(&mut hash_db);

        trie.try_update(&vec![1, 2, 3], vec![2]).unwrap();
        trie.try_update(&vec![1, 2, 3], vec![3]).unwrap();

        trie.try_update(&vec![1, 2, 3, 4], vec![3]).unwrap();
    }
}
