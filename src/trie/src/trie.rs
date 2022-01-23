use crate::encoding::{key_bytes_to_hex, prefix_len, TERMINAL};
use crate::error::Error;
use crate::hasher::NodeHasher;
use crate::node::{DeleteItem, Node, CHILD_SIZE};
use crate::rstd::mem;
use crate::storage::{Cache, CacheIndex, MemorySlot, NodeLocation};
use common::{ensure, H256};
use kv_storage::DBStorage;
use log::debug;
use std::collections::HashSet;

type Prefix = Vec<u8>;

/// The Trie data type for storage
pub struct Trie<'a, H: DBStorage> {
    db: &'a mut H,
    root_loc: NodeLocation,
    cache: Cache,
    delete_items: HashSet<DeleteItem>,
    unhashed: u32,
    node_hasher: NodeHasher,
}

impl<'a, H: DBStorage> Trie<'a, H> {
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

    /// Try to get the bytes stored in the key. If key does not exist, return None.
    pub fn try_get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.get(&self.root_loc, &key_bytes_to_hex(key), 0)
    }

    fn get(&self, node_loc: &NodeLocation, key: &[u8], pos: usize) -> Option<Vec<u8>> {
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
                    self.get(&val, key, pos + matchlen)
                }
            }
            Node::Full { children } => self.get(&children[key[pos] as usize], key, pos + 1),
            Node::Value(val) => {
                if key.len() != pos {
                    None
                } else {
                    Some(val)
                }
            }
        }
    }

    /// Try to delete the key, returns corresponding errors
    pub fn try_delete(&mut self, key: &[u8]) -> Result<(), Error> {
        ensure!(!key.is_empty(), Error::KeyCannotBeEmpty)?;
        self.unhashed += 1;
        self.root_loc = self.delete(self.root_loc(), &key_bytes_to_hex(key))?;
        Ok(())
    }

    fn delete(&mut self, node_loc: NodeLocation, key: &[u8]) -> Result<NodeLocation, Error> {
        // let (_, node) = self.take_node_loc(node_loc)?;
        let (_, node) = self.get_node_loc_mut(&node_loc)?;
        match node {
            Node::Empty => Err(Error::KeyNotExists),
            Node::Full { children } => {
                debug!("delete full node for key: {:?}", key);

                // need need to delete the stored values and re-insert
                let mut children = mem::take(children);
                self.destroy(&node_loc)?;

                let sliceidx = key[0] as usize;
                let child = children[sliceidx];
                let child_loc = self.delete(child, &key[1..])?;
                children[sliceidx] = child_loc;

                // Because node is Full node, we require at least 2 children.
                // If child_loc is not None, that means there are at least
                // 2 children. If there is only one children, we should reduce
                // it to a Short node. If there are no children, implementation
                // is wrong, PANIC!
                let n = if !matches!(child_loc, NodeLocation::None) {
                    debug!(
                        "children after deletion not null, return full node for key: {:?}",
                        key
                    );
                    Node::Full { children }
                } else {
                    // Locate the next position to delete in children.
                    // If pos = -1, all children are empty
                    // If pos = -2, more than 1 children are non-empty
                    // Else only one child is not empty and pos points to it
                    let mut pos = -1;
                    for (i, c) in children.iter().enumerate() {
                        if !matches!(c, NodeLocation::None) {
                            if pos == -1 {
                                pos = i as i8;
                            } else {
                                pos = -2;
                                break;
                            }
                        }
                    }
                    if pos > -1 {
                        let pos = pos as u8;
                        if pos != TERMINAL {
                            // Now we have only one children. We need to take the key, val of the
                            // children and append the key to the pos char
                            let (_, n) = self.get_node_loc_mut(&children[pos as usize])?;
                            if matches!(n, Node::Short { .. }) {
                                let (_, n) = self.take_node_loc(children[pos as usize])?;
                                match n {
                                    Node::Short { mut key, val } => {
                                        let mut k = vec![pos];
                                        k.append(&mut key);
                                        Node::Short { key: k, val }
                                    }
                                    _ => Node::Short {
                                        key: vec![pos],
                                        val: children[pos as usize],
                                    },
                                }
                            } else {
                                Node::Short {
                                    key: vec![pos],
                                    val: children[pos as usize],
                                }
                            }
                        } else {
                            Node::Short {
                                key: vec![pos],
                                val: children[pos as usize],
                            }
                        }
                    } else {
                        Node::Full { children }
                    }
                };
                Ok(NodeLocation::Memory(
                    self.cache.insert(MemorySlot::Updated(n)),
                ))
            }
            Node::Short {
                key: nkey,
                val: nval,
            } => {
                let matchlen = prefix_len(nkey, key);
                debug!(
                    "delete match len for nkey: {:?} and key: {:?}: {}",
                    nkey, key, matchlen
                );

                // no match for key and node key, return directly
                if matchlen < nkey.len() {
                    debug!("key does not exist: {:?}", key);
                    return Err(Error::KeyNotExists);
                } else if matchlen == key.len() {
                    self.destroy(&node_loc)?; // TODO: double release here
                    debug!("exact key match: {:?}, purge in memory", key);
                    return Ok(NodeLocation::None);
                }

                let mut nkey = mem::take(nkey);
                let nval = mem::take(nval);
                self.destroy(&node_loc)?;

                let child_loc = self.delete(nval, &key[matchlen..])?;

                // Here child_loc cannot be empty. The reason is the child can only be one of
                // value node (which is handled above), at lease two items Full node
                // (otherwise we can make it into a Short node or value node), or a reduced
                // Short node from one item Full node. Short node link to another short node,
                // would be reduced to a single Short node.
                ensure!(
                    !matches!(child_loc, NodeLocation::None),
                    Error::InvalidNodeLocation
                )?;

                let (_, child) = self.get_node_loc_mut(&child_loc)?;
                let n = match child {
                    Node::Short {
                        key: ckey,
                        val: cval,
                    } => {
                        debug!("merge short node children with key: {:?}", ckey);

                        nkey.append(ckey);
                        let val = mem::take(cval);
                        debug!("purge children {:?} in memory, new key {:?}", ckey, nkey);
                        self.destroy(&child_loc)?;

                        Node::Short { key: nkey, val }
                    }
                    _ => Node::Short {
                        key: nkey,
                        val: child_loc,
                    },
                };
                Ok(NodeLocation::Memory(
                    self.cache.insert(MemorySlot::Updated(n)),
                ))
            }
            _ => panic!("invalid state"),
        }
    }

    fn destroy(&mut self, node_loc: &NodeLocation) -> Result<(), Error> {
        match node_loc {
            NodeLocation::None => Ok(()),
            NodeLocation::Persistence(_) => Err(Error::InvalidNodeLocation),
            NodeLocation::Memory(cache_index) => {
                let d = match self.cache.take(*cache_index) {
                    MemorySlot::Updated(n) => DeleteItem::Node(n),
                    MemorySlot::Loaded(h, _) => DeleteItem::Hash(h),
                };
                self.delete_items.insert(d);
                Ok(())
            }
        }
    }

    /// Try to update the key with provided value
    pub fn try_update(&mut self, key: &[u8], val: &[u8]) -> Result<(), Error> {
        ensure!(!key.is_empty(), Error::KeyCannotBeEmpty)?;
        if val.is_empty() {
            return self.try_delete(key);
        }

        self.unhashed += 1;
        self.root_loc = self.insert(
            self.root_loc(),
            Prefix::default(),
            &*key_bytes_to_hex(key),
            Vec::from(val),
        )?;

        Ok(())
    }

    fn insert(
        &mut self,
        node_loc: NodeLocation,
        prefix: Prefix,
        key: &[u8],
        val: Vec<u8>,
    ) -> Result<NodeLocation, Error> {
        let val_node = Node::Value(val);
        let val_loc = NodeLocation::Memory(self.cache.insert(MemorySlot::Updated(val_node)));
        self.insert_inner(node_loc, prefix, key, val_loc)
    }

    fn insert_inner(
        &mut self,
        node_loc: NodeLocation,
        mut prefix: Prefix,
        key: &[u8],
        value: NodeLocation,
    ) -> Result<NodeLocation, Error> {
        if matches!(node_loc, NodeLocation::None) {
            if key.is_empty() {
                return Ok(value);
            }
            let idx = self.cache.insert(MemorySlot::Updated(Node::Short {
                key: Vec::from(key),
                val: value,
            }));
            return Ok(NodeLocation::Memory(idx));
        }

        // All these round trips because of not familiar with
        // rust... See readme of this module.
        let (cache_index, node) = self.take_node_loc(node_loc)?;

        if key.is_empty() {
            return Ok(value);
        }

        match node {
            Node::Empty => {
                let n = Node::Short {
                    key: Vec::from(key),
                    val: value,
                };
                self.cache.replace(cache_index, MemorySlot::Updated(n));
                Ok(NodeLocation::Memory(cache_index))
            }
            Node::Short {
                key: nkey,
                val: nval,
            } => {
                let matchlen = prefix_len(&nkey, key);

                // This means the two keys match exactly
                if matchlen == nkey.len() {
                    // TODO: are these 2 steps necessary?
                    // update the prefix to include the matched key
                    prefix.append(&mut (key[..matchlen].to_vec()));

                    // update the child with a new value
                    let new = self.insert_inner(nval, prefix, &key[matchlen..], value)?;

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

                // One more index of matchlen because the branch node will consume one position.
                // So the prefix is effectively extended to [..matchlen+1]
                nprefix.append(&mut (nkey[..matchlen + 1]).to_vec());
                prefix.append(&mut (key[..matchlen + 1].to_vec()));

                children[c1] =
                    self.insert_inner(NodeLocation::None, prefix, &key[matchlen + 1..], value)?;
                children[c2] =
                    self.insert_inner(NodeLocation::None, nprefix, &nkey[matchlen + 1..], nval)?;

                let branch = Node::Full {
                    children: Box::new(children),
                };
                let idx = self.cache.insert(MemorySlot::Updated(branch));

                if matchlen == 0 {
                    Ok(NodeLocation::Memory(idx))
                } else {
                    let idx = self.cache.insert(MemorySlot::Updated(Node::Short {
                        key: nkey[..matchlen].to_vec(),
                        val: NodeLocation::Memory(idx),
                    }));
                    Ok(NodeLocation::Memory(idx))
                }
            }
            Node::Full { children: mut ch } => {
                prefix.push(key[0]);

                let i = key[0] as usize;
                ch[i] = self.insert_inner(ch[i], prefix, &key[1..], value)?;

                let branch = Node::Full { children: ch };
                Ok(NodeLocation::Memory(
                    self.cache.insert(MemorySlot::Updated(branch)),
                ))
            }
            _ => panic!("not implemented"),
        }
    }

    /// Commit cached node changes to underlying database. Update trie hash as well.
    pub fn commit(&mut self) -> Result<H256, Error> {
        // TODO: remove items in self.delete_items in db
        let node_loc = self.root_loc();
        let h = match node_loc {
            NodeLocation::None => H256::default(),
            NodeLocation::Persistence(h) => H256::from(h),
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

    fn extract_cache_index(&mut self, node_loc: &NodeLocation) -> Result<CacheIndex, Error> {
        match node_loc {
            NodeLocation::Persistence(h) => Ok(self.load_to_cache(&H256::from_slice(h))),
            NodeLocation::Memory(i) => Ok(*i),
            _ => Err(Error::InvalidNodeLocation),
        }
    }

    fn get_node_loc_mut(
        &mut self,
        node_loc: &NodeLocation,
    ) -> Result<(CacheIndex, &mut Node), Error> {
        let cache_index = self.extract_cache_index(node_loc)?;

        // Always fetch the node from cache
        let node = match self.cache.get_mut(cache_index) {
            MemorySlot::Updated(node) => node,
            MemorySlot::Loaded(_, node) => node,
        };

        Ok((cache_index, node))
    }

    fn take_node_loc(&mut self, node_loc: NodeLocation) -> Result<(CacheIndex, Node), Error> {
        let cache_index = self.extract_cache_index(&node_loc)?;

        // Always fetch the node from cache
        let node = match self.cache.take(cache_index) {
            MemorySlot::Updated(node) => node,
            MemorySlot::Loaded(_, node) => node,
        };

        Ok((cache_index, node))
    }

    fn load_to_cache(&mut self, h: &H256) -> CacheIndex {
        let node = match self.db.get(h.as_bytes()) {
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
    use common::H256;
    use crate::storage::NodeLocation;
    use crate::trie::Trie;
    use kv_storage::MemoryDB;

    const TEST_HASH: [u8; 32] = [
        0x65, 0x5a, 0x75, 0x4, 0xda, 0x98, 0xaa, 0xca, 0x39, 0xf2, 0x38, 0x85, 0xb2, 0xb2, 0x32,
        0xd4, 0xa4, 0x95, 0x31, 0x5d, 0x63, 0x87, 0x38, 0xcd, 0x6e, 0xa0, 0x84, 0xa9, 0x26, 0xf3,
        0xa3, 0x7,
    ];

    #[test]
    fn root_handle() {
        let mut hash_db = MemoryDB::new();
        let trie = Trie::new(&mut hash_db);

        assert_eq!(trie.root_loc(), NodeLocation::None);
    }

    #[test]
    fn delete_short_node_works() {
        let mut hash_db = MemoryDB::new();
        let mut trie = Trie::new(&mut hash_db);

        trie.try_update(b"foo", b"bar").unwrap();
        trie.try_update(b"test", b"barr").unwrap();

        trie.try_delete(b"test").unwrap();
        assert_eq!(trie.try_get(b"test"), None);

        assert_eq!(trie.try_get(b"foo"), Some(b"bar".to_vec()));
    }

    #[test]
    fn delete_full_node_with_terminal_works() {
        let mut hash_db = MemoryDB::new();
        let mut trie = Trie::new(&mut hash_db);

        trie.try_update(b"foo", b"bar").unwrap();
        trie.try_update(b"fook", b"barr").unwrap();

        trie.try_delete(b"fook").unwrap();
        assert_eq!(trie.try_get(b"fook"), None);

        assert_eq!(trie.try_get(b"foo"), Some(b"bar".to_vec()));
    }
    #[test]
    fn delete_works() {
        let mut hash_db = MemoryDB::new();
        let mut trie = Trie::new(&mut hash_db);
        log::info!("here");
        trie.try_update(b"foo", b"bar").unwrap();
        trie.try_update(b"fook", b"barr").unwrap();
        trie.try_update(b"fooo", b"bar").unwrap();
        trie.try_update(b"foooks", b"bar").unwrap();
        trie.try_update(b"fooks", b"bar").unwrap();

        trie.try_delete(b"foooks").unwrap();
        assert_eq!(trie.try_get(b"foooks"), None);
        trie.try_delete(b"fooks").unwrap();
        assert_eq!(trie.try_get(b"fooks"), None);

        let out = trie.commit().unwrap();
        assert_eq!(out, H256::from(TEST_HASH));
    }

    #[test]
    fn insert_works() {
        let mut hash_db = MemoryDB::new();
        let mut trie = Trie::new(&mut hash_db);

        trie.try_update(&vec![1, 2, 3], &[2]).unwrap();
        assert_eq!(trie.try_get(&vec![1, 2, 3]), Some(vec![2]));

        trie.try_update(&vec![1, 2, 3], &[3]).unwrap();
        assert_eq!(trie.try_get(&vec![1, 2, 3]), Some(vec![3]));

        trie.try_update(&vec![1, 2, 3, 4], &[3]).unwrap();
        assert_eq!(trie.try_get(&vec![1, 2, 3]), Some(vec![3]));
        assert_eq!(trie.try_get(&vec![1, 2, 3, 4]), Some(vec![3]));

        assert_eq!(trie.try_get(&vec![1, 2, 3, 5]), None);
    }

    #[test]
    fn commit_works() {
        let mut hash_db = MemoryDB::new();
        let mut trie = Trie::new(&mut hash_db);

        trie.try_update(b"foo", b"bar").unwrap();
        trie.try_update(b"fook", b"barr").unwrap();
        assert_eq!(trie.try_get(b"fook"), Some(b"barr".to_vec()));
        trie.try_update(b"fooo", b"bar").unwrap();
        let out = trie.commit().unwrap();
        assert_eq!(out, H256::from(TEST_HASH));
    }
}
