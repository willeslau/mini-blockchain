use crate::node::Node;
use crate::rstd;
use common::H256;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub type CacheIndex = usize;

// TODO: remove Copy
/// Enum indicating where the node is currently stored
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum NodeLocation {
    /// Currently in the persistence storage
    Persistence([u8; 32]),
    /// Currently in memory
    Memory(CacheIndex),
    /// Not stored anywhere
    None,
}

impl Default for NodeLocation {
    fn default() -> Self {
        NodeLocation::None
    }
}

/// The memory slot type for nodes stored in memory
pub(crate) enum MemorySlot {
    /// The memory slot is updated, we need to flush it
    Updated(Node),
    /// Memory slot is just loaded from persistence, no changes made
    Loaded(H256, Node),
}

/// In memory storage location for nodes
pub(crate) struct Cache {
    /// Data and references relationships of dirty trie nodes
    slots: Vec<MemorySlot>,
    /// Free index
    free_indices: VecDeque<CacheIndex>,
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            slots: vec![],
            free_indices: VecDeque::new(),
        }
    }

    pub fn insert(&mut self, storage: MemorySlot) -> CacheIndex {
        if let Some(idx) = self.free_indices.pop_front() {
            self.slots[idx] = storage;
            idx
        } else {
            self.slots.push(storage);
            self.slots.len() - 1
        }
    }

    /// Get the node at index
    /// Note: this method could be dangerous as index might be a freed index.
    pub fn get_node(&self, index: CacheIndex) -> Node {
        match self.slots.get(index) {
            None => Node::Empty,
            Some(slot) => match slot {
                MemorySlot::Updated(node) => node.clone(),
                MemorySlot::Loaded(_, node) => node.clone(),
            },
        }
    }

    pub fn get_mut(&mut self, index: CacheIndex) -> &mut MemorySlot {
        self.slots.get_mut(index).unwrap()
    }

    pub fn replace(&mut self, index: CacheIndex, storage_slot: MemorySlot) {
        self.slots[index] = storage_slot;
    }

    /// Take the item out of the cache. Assume user pass valid index.
    pub fn take(&mut self, index: CacheIndex) -> MemorySlot {
        self.free_indices.push_back(index);
        rstd::mem::replace(&mut self.slots[index], MemorySlot::Updated(Node::Empty))
    }
}
