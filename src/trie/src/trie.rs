use crate::encoding::{key_bytes_to_hex, prefix_len};
use crate::error::Error;
use crate::hasher::NodeHasher;
use crate::node::{Node, CHILD_SIZE};
use crate::storage::{Cache, CacheIndex, MemorySlot, NodeLocation};
use common::{ensure, Hash};
use kv_storage::HashDB;
use std::collections::HashSet;

type Prefix = Vec<u8>;

pub struct Trie<'a, H: HashDB> {
    db: &'a mut H,
    root_loc: NodeLocation,
    cache: Cache,
    delete_items: HashSet<Node>,
    unhashed: u32,
    node_hasher: NodeHasher,
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
            node_hasher: NodeHasher::new(),
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
        if key.is_empty() {
            return None;
        }

        let node = match node_loc {
            NodeLocation::Persistence(h) => match self.db.get(h) {
                None => Node::Empty,
                Some(bytes) => Node::from(bytes),
            },
            NodeLocation::Memory(cache_index) => self.cache.get_node(*cache_index),
            NodeLocation::None => Node::Empty,
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
            }
            Node::Full { children } => self.get_inner(&children[key[pos] as usize], key, pos + 1),
            Node::Value(val) => {
                if key.len() != pos {
                    None
                } else {
                    Some(val)
                }
            }
        }
    }

    /// Try to update the key with provided value
    pub fn try_update(&mut self, key: &[u8], val: &[u8]) -> Result<(), Error> {
        ensure!(!key.is_empty(), Error::KeyCannotBeEmpty)?;
        ensure!(!val.is_empty(), Error::ValueCannotBeEmpty)?;

        self.unhashed += 1;
        let k = key_bytes_to_hex(key);
        let root = self.root_loc();
        let prefix = Prefix::default();
        self.root_loc = self.insert(root, prefix, k, Vec::from(val))?;
        Ok(())
    }

    pub fn try_delete(&mut self, key: &[u8]) -> Result<(), Error> {
        Ok(())
    }

    fn insert(
        &mut self,
        node_loc: NodeLocation,
        prefix: Prefix,
        key: Vec<u8>,
        val: Vec<u8>,
    ) -> Result<NodeLocation, Error> {
        let val_node = Node::Value(val);
        let val_loc = NodeLocation::Memory(self.cache.insert(MemorySlot::Updated(val_node)));
        self.insert_inner(node_loc, prefix, key, val_loc)
    }

    fn take_node_loc(&mut self, node_loc: NodeLocation) -> Result<(CacheIndex, Node), Error> {
        let cache_index = match node_loc {
            NodeLocation::Persistence(h) => self.load_to_cache(&h),
            NodeLocation::Memory(i) => i,
            _ => {
                return Err(Error::InvalidNodeLocation);
            }
        };

        // Always fetch the node from cache
        let node = match self.cache.take(cache_index) {
            MemorySlot::Updated(node) => node,
            MemorySlot::Loaded(_, node) => node,
        };

        Ok((cache_index, node))
    }

    fn destroy(&mut self, node_loc: &NodeLocation) -> Result<(), Error> {
        match node_loc {
            NodeLocation::None => Ok(()),
            NodeLocation::Persistence(_) => Err(Error::InvalidNodeLocation),
            NodeLocation::Memory(cache_index) => {
                match self.cache.take(*cache_index) {
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
        let (cache_index, node) = self.take_node_loc(node_loc)?;

        if key.is_empty() { return Ok(value); }

        match node {
            Node::Empty => {
                let n = Node::Short { key, val: value };
                self.cache.replace(cache_index, MemorySlot::Updated(n));
                Ok(NodeLocation::Memory(cache_index))
            }
            Node::Short {
                key: nkey,
                val: nval,
            } => {
                let matchlen = prefix_len(&nkey, &key);

                // This means the two keys match exactly
                if matchlen == nkey.len() {
                    // TODO: are these 2 steps necessary?
                    // update the prefix to include the matched key
                    prefix.append(&mut (key[..matchlen].to_vec()));

                    // update the child with a new value
                    let new = self.insert_inner(nval, prefix, key[matchlen..].to_vec(), value)?;

                    // now we update the slot
                    let idx = self.cache.insert(MemorySlot::Updated(Node::Short {
                        key: nkey,
                        val: new,
                    }));
                    return Ok(NodeLocation::Memory(idx));
                }

                let mut children = [NodeLocation::None; CHILD_SIZE];
                let mut nprefix = prefix.clone();

                let c1 = key[matchlen] as usize;
                let c2 = nkey[matchlen] as usize;

                let k1 = key[matchlen + 1..].to_vec();
                let k2 = nkey[matchlen + 1..].to_vec();

                // A trick to avoid borrow mutable
                let nv = nval;
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

                if matchlen == 0 {
                    Ok(NodeLocation::Memory(idx))
                } else {
                    let idx = self.cache.insert(MemorySlot::Updated(Node::Short {
                        key: nk,
                        val: NodeLocation::Memory(idx),
                    }));
                    Ok(NodeLocation::Memory(idx))
                }
            }
            Node::Full { children: mut ch } => {
                prefix.push(key[0]);
                let i = key[0] as usize;
                ch[i] = self.insert_inner(ch[i], prefix, key[1..].to_vec(), value)?;
                let branch = Node::Full { children: ch };
                let idx = self.cache.insert(MemorySlot::Updated(branch));
                Ok(NodeLocation::Memory(idx))
            }
            _ => panic!("not implemented"),
        }
    }

    /// Commit cached node changes to underlying database. Update trie hash as well.
    pub fn commit(&mut self) -> Result<Hash, Error> {
        // TODO: remove items in self.delete_items in db
        let node_loc = self.root_loc();
        let h = match node_loc {
            NodeLocation::None => Hash::default(),
            NodeLocation::Persistence(h) => h,
            NodeLocation::Memory(x) => {
                match self.cache.take(x) {
                    MemorySlot::Updated(node) => {
                        self.node_hasher.hash(node, self.db, &mut self.cache)
                    }
                    // If the slot is just loaded from DB and not updated,
                    // we should not have the need to process it again.
                    MemorySlot::Loaded(h, _) => h,
                }
            }
        };
        Ok(h)
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
    use crate::storage::NodeLocation;
    use crate::trie::Trie;
    use kv_storage::MemoryDB;

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

        trie.try_update(&vec![1, 2, 3], &[2]).unwrap();
        assert_eq!(trie.get(&vec![1, 2, 3]), Some(vec![2]));

        trie.try_update(&vec![1, 2, 3], &[3]).unwrap();
        assert_eq!(trie.get(&vec![1, 2, 3]), Some(vec![3]));

        trie.try_update(&vec![1, 2, 3, 4], &[3]).unwrap();
        assert_eq!(trie.get(&vec![1, 2, 3]), Some(vec![3]));
        assert_eq!(trie.get(&vec![1, 2, 3, 4]), Some(vec![3]));
    }

    #[test]
    fn commit_works() {
        let mut hash_db = MemoryDB::new();
        let mut trie = Trie::new(&mut hash_db);

        trie.try_update(b"foo", b"bar").unwrap();
        trie.try_update(b"fook", b"barr").unwrap();
        trie.try_update(b"fooo", b"bar").unwrap();
        let out = trie.commit().unwrap();
        assert_eq!(
            out,
            [
                0x65, 0x5a, 0x75, 0x4, 0xda, 0x98, 0xaa, 0xca, 0x39, 0xf2, 0x38, 0x85, 0xb2, 0xb2,
                0x32, 0xd4, 0xa4, 0x95, 0x31, 0x5d, 0x63, 0x87, 0x38, 0xcd, 0x6e, 0xa0, 0x84, 0xa9,
                0x26, 0xf3, 0xa3, 0x7
            ]
        );
    }
}
