use std::collections::HashSet;
use crate::encoding::{key_bytes_to_hex, prefix_len};
use crate::error::Error;
use crate::node::{Node, CHILD_SIZE, ChildReference};
use crate::rstd::mem;
use crate::storage::{Cache, CacheIndex, MemorySlot, NodeLocation};
use common::{ensure, Hash, KeccakHasher};
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
        let k = key_bytes_to_hex(key);
        self.get_inner(&self.root_loc, &k, 0)
    }

    fn get_inner(&self, node_loc: &NodeLocation, key: &[u8], pos: usize) -> Option<Vec<u8>> {
        if key.is_empty() { return None; }

        let node = match node_loc {
            NodeLocation::Persistence(h) => {
                match self.db.get(h) {
                    None => Node::Empty,
                    Some(bytes) => Node::from(bytes),
                }
            },
            NodeLocation::Memory(cache_index) => self.cache.get_node(*cache_index),
            NodeLocation::None => Node::Empty
        };

        match node {
            Node::Empty => None,
            Node::Short { key: nkey, val } => {
                let matchlen = prefix_len(&nkey, &key[pos..]);
                if matchlen != nkey.len() {
                    None
                } else {
                    self.get_inner(&val, key, pos + matchlen)
                }
            },
            Node::Full { children } => {
                self.get_inner(&children[key[pos] as usize], key, pos + 1)
            },
            Node::Value(v) => {
                if key.len() != pos { None }
                else { Some(v) }
            }
        }
    }

    /// Try to update the key with provided value
    pub fn try_update(&mut self, key: &[u8], val: Vec<u8>) -> Result<(), Error> {
        ensure!(!key.is_empty(), Error::KeyCannotBeEmpty)?;
        ensure!(!val.is_empty(), Error::ValueCannotBeEmpty)?;

        self.unhashed += 1;
        let k = key_bytes_to_hex(key);
        let root = self.root_loc();
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
                .insert(MemorySlot::Updated(Node::Short { key, val: value }));
            return Ok(NodeLocation::Memory(idx));
        }

        // All these round trips because of not familiar with
        // rust... See readme of this module.
        let (cache_index, node) = self.parse_node_loc(node_loc)?;

        // require key not empty, maybe this case needs to be handled
        ensure!(!key.is_empty(), Error::KeyCannotBeEmpty)?;

        match node {
            Node::Empty => {
                let n = Node::Short { key, val: value };
                self.cache.replace(cache_index, MemorySlot::Updated(n));
                return Ok(NodeLocation::Memory(cache_index));
            }
            Node::Short {
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
                let nv = mem::replace(nval, NodeLocation::None);
                let nk = nkey[..matchlen].to_vec();

                // One more index of matchlen because the branch node will consume one position.
                // So the prefix is effectively extended to [..matchlen+1]
                nprefix.append(&mut (nkey[..matchlen + 1]).to_vec());
                prefix.append(&mut (key[..matchlen + 1].to_vec()));

                let n1 = self.insert_inner(NodeLocation::None, prefix, k1, value)?;
                let n2 = self.insert_inner(NodeLocation::None, nprefix, k2, nv)?;

                children[c1] = n1;
                children[c2] = n2;

                let branch = Node::Full {
                    children: Box::new(children),
                };
                let idx = self.cache.insert(MemorySlot::Updated(branch));

                self.cache.take(cache_index);
                let idx = self.cache.insert(MemorySlot::Updated(Node::Short {
                    key: nk,
                    val: NodeLocation::Memory(idx),
                }));
                return Ok(NodeLocation::Memory(idx));
            }
            _ => {}
        }
        Ok(NodeLocation::None)
    }

    /// Commit cached node changes to underlying database. Update trie hash as well.
    pub fn commit(&mut self) -> Result<Hash, Error> {
        // TODO: remove items in self.delete_items in db
        let node_loc = self.root_loc();
        self.commit_child(node_loc);
        Ok(Hash::default())
    }

    fn commit_child(&mut self, node_loc: NodeLocation) -> ChildReference {
        match node_loc {
            // The root is still in persistence or not exists, nothing to do.
            NodeLocation::Persistence(_) | NodeLocation::None => ChildReference::Hash(Hash::default()),
            NodeLocation::Memory(x) => {
                match self.cache.take(x) {
                    MemorySlot::Updated(node) => {
                        node.encode::<KeccakHasher, _>(|node_loc| self.commit_child(node_loc));
                        ChildReference::Hash(Hash::default())
                        // match node {
                        //     Node::Empty => Err(Error::InvalidTrieState),
                        //     Node::Full { children } => {
                        //         for c in children.to_vec() {
                        //             self.commit_child(c)?;
                        //         }
                        //         Ok(Hash::default())
                        //     }
                        //     Node::Short { key, val } => {
                        //         Ok(Hash::default())
                        //     }
                        //     Node::Value(_) => { Ok(Hash::default()) }
                        // }
                    },
                    // If the slot is just loaded from DB and not updated,
                    // we should not have the need to process it again.
                    MemorySlot::Loaded(h, _) => ChildReference::Hash(h)
                }
            },
        }
    }

    fn load_to_cache(&mut self, h: &Hash) -> CacheIndex {
        let node = match self.db.get(h) {
            None => Node::Empty,
            Some(bytes) => Node::from(bytes),
        };
        self.cache.insert(MemorySlot::Loaded(*h, node))
    }

    // a hack to get the root node's handle
    fn root_loc(&self) -> NodeLocation {
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

        assert_eq!(trie.root_loc(), NodeLocation::None);
    }

    #[test]
    fn insert_works() {
        let mut hash_db = MemoryDB::new();
        let mut trie = Trie::new(&mut hash_db);

        trie.try_update(&vec![1, 2, 3], vec![2]).unwrap();
        assert_eq!(trie.get(&vec![1, 2, 3]), Some(vec![2]));

        trie.try_update(&vec![1, 2, 3], vec![3]).unwrap();
        assert_eq!(trie.get(&vec![1, 2, 3]), Some(vec![3]));

        trie.try_update(&vec![1, 2, 3, 4], vec![3]).unwrap();
        assert_eq!(trie.get(&vec![1, 2, 3]), Some(vec![3]));
        assert_eq!(trie.get(&vec![1, 2, 3, 4]), Some(vec![3]));
    }
}
