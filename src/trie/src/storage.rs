use crate::error::Error;
use crate::node::{CachedNode, Node};
use common::{to_vec, Hash};
use kv_storage::HashDB;
use std::collections::{HashMap, VecDeque};

pub type CacheIndex = usize;

/// Enum indicating where the node is currently stored
pub enum NodeLocation {
    /// Currently in the persistence storage
    Persistence(Hash),
    /// Currently in memory
    Memory(CacheIndex),
}

/// The memory slot type for nodes stored in memory
pub enum MemorySlot {
    /// The memory slot is updated, we need to flush it
    Updated(Node),
    /// Memory slot is just loaded from persistence, no changes made
    Loaded(Hash, Node),
}

/// In memory storage location for nodes
pub struct Cache {
    /// Data and references relationships of dirty trie nodes
    slots: Vec<MemorySlot>,
    /// Free index
    free_index: VecDeque<CacheIndex>,
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            slots: vec![],
            free_index: VecDeque::new()
        }
    }

    pub fn insert(&mut self, storage: MemorySlot) -> CacheIndex {
        if let Some(idx) = self.free_index.pop_front() {
            self.slots[idx] = storage;
            idx
        } else {
            self.slots.push(storage);
            self.slots.len() - 1
        }
    }

    pub fn get_mut(&mut self, index: CacheIndex) -> &mut MemorySlot {
        self.slots.get_mut(index).unwrap()
    }

    pub fn replace(&mut self, index: CacheIndex, storage_slot: MemorySlot) {
        self.slots[index] = storage_slot;
    }
}
