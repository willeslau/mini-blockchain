use crate::encoding::{key_bytes_to_hex, prefix_len};
use crate::error::Error;
use crate::node::{Node, NodeFlag};
use crate::storage::{Cache, CacheIndex, NodeLocation, MemorySlot};
use common::{bytes_to_hash, ensure, Hash, Hasher, KeccakHasher};
use kv_storage::HashDB;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;

pub const EMPTY_TRIE: u8 = 0;

struct Prefix {
    idx: usize,
    key: Hash,
}

pub struct Trie<'a, H: HashDB> {
    db: &'a mut H,
    root_loc: NodeLocation,
    cache: Cache,
    unhashed: u32,
}

impl<'a, H: HashDB> Trie<'a, H> {
    /// The root_hash needs to be the empty node hash
    pub fn new(db: &'a mut H) -> Self {
        Self {
            db,
            root_loc: NodeLocation::Persistence(KeccakHasher::hash(&[EMPTY_TRIE])),
            cache: Cache::new(),
            unhashed: 0,
        }
    }

    // // pub fn new_from_existing(db: &'db DB, root_hash: &[u8]) -> Self {
    // //
    // // }

    /// Try to update the key with provided value
    pub fn try_update(&mut self, key: &Vec<u8>, val: Vec<u8>) -> Result<(), Error> {
        ensure!(val.len() > 0, Error::ValueCannotBeEmpty)?;

        self.unhashed += 1;
        let k = key_bytes_to_hex(key);
        let root = self.root_handle();
        let mut prefix = Prefix {
            idx: 0,
            key: Hash::default(),
        };
        self.root_loc =
            self.insert(root, &mut prefix, bytes_to_hash(&k), Node::ValueNode(val))?;
        Ok(())
    }

    fn insert(
        &mut self,
        node_loc: NodeLocation,
        prefix: &mut Prefix,
        key: Hash,
        value: Node,
    ) -> Result<NodeLocation, Error> {
        ensure!(key.len() != 0, Error::KeyCannotBeEmpty)?;

        // All these round trips because of not familiar with
        // rust... See readme of this module.
        let cache_index = match node_loc {
            NodeLocation::Persistence(h) => self.load_to_cache(&h),
            NodeLocation::Memory(i) => i,
        };

        let node = match self.cache.get_mut(cache_index) {
            MemorySlot::Updated(node) => node,
            MemorySlot::Loaded(_, node) => node,
        };

        match node {
            Node::Empty => {
                let n = Node::ShortNode {
                    key,
                    val: Box::new(value),
                    flags: NodeFlag::new(true),
                };
                self.cache.replace(cache_index, MemorySlot::Updated(n));
                return Ok(NodeLocation::Memory(cache_index));
            }
            Node::ShortNode { key: nkey, .. } => {
                let plen = prefix_len(nkey, &key);

                /*
                   matchlen := prefixLen(key, n.Key)
                   // If the whole key matches, keep this short node as is
                   // and only update the value.
                   if matchlen == len(n.Key) {
                       dirty, nn, err := t.insert(n.Val, append(prefix, key[:matchlen]...), key[matchlen:], value)
                       if !dirty || err != nil {
                           return false, n, err
                       }
                       return true, &shortNode{n.Key, nn, t.newFlag()}, nil
                   }
                   // Otherwise branch out at the index where they differ.
                   branch := &fullNode{flags: t.newFlag()}
                   var err error
                   _, branch.Children[n.Key[matchlen]], err = t.insert(nil, append(prefix, n.Key[:matchlen+1]...), n.Key[matchlen+1:], n.Val)
                   if err != nil {
                       return false, nil, err
                   }
                   _, branch.Children[key[matchlen]], err = t.insert(nil, append(prefix, key[:matchlen+1]...), key[matchlen+1:], value)
                   if err != nil {
                       return false, nil, err
                   }
                   // Replace this shortNode with the branch if it occurs at index 0.
                   if matchlen == 0 {
                       return true, branch, nil
                   }
                   // Otherwise, replace it with a short node leading up to the branch.
                   return true, &shortNode{key[:matchlen], branch, t.newFlag()}, nil
                */
            }
            _ => {}
        }
        Ok(NodeLocation::Memory(100000))
    }

    fn load_to_cache(&mut self, h: &Hash) -> CacheIndex {
        let node = match self.db.get(h) {
            None => Node::Empty,
            Some(bytes) => Node::from(bytes),
        };
        self.cache.insert(MemorySlot::Loaded(h.clone(), node))
    }

    // a hack to get the root node's handle
    fn root_handle(&self) -> NodeLocation {
        match self.root_loc {
            NodeLocation::Persistence(h) => NodeLocation::Persistence(h),
            NodeLocation::Memory(x) => NodeLocation::Memory(x),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::Cache;
    use crate::trie::Trie;
    use kv_storage::MemoryDB;

    #[test]
    fn insert_works() {
        let mut hash_db = MemoryDB::new();
        let mut trie = Trie::new(&mut hash_db);

        trie.try_update(&vec![1,2,3], vec![2]);
    }
}
