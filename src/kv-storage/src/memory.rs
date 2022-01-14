use crate::{DBStorage};
use std::collections::HashMap;

/// In memory database storage. Use for testing purpose only.
pub struct MemoryDB {
    data: HashMap<Vec<u8>, Vec<u8>>,
}

impl MemoryDB {
    pub fn new() -> Self {
        MemoryDB {
            data: HashMap::new(),
        }
    }
}

impl DBStorage for MemoryDB {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        match self.data.get(key) {
            None => None,
            Some(v) => Some(Vec::from(v.clone())),
        }
    }

    fn contains(&self, key: &[u8]) -> bool {
        self.data.get(key).is_some()
    }

    fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.data.insert(key, value);
    }

    fn remove(&mut self, key: &[u8]) {
        self.data.remove(key);
    }
}
