use crate::node::NodeId;
use common::vec_to_u64_le;
use kv_storage::{DBStorage, MemoryDB};

const DB_LOCAL_SEQ: &str = "seq";
const DB_LOCAL_PREFIX: &str = "local:";

pub(crate) struct Storage {
    inner: Box<dyn DBStorage>,
}

impl Storage {
    pub fn new(storage: Box<dyn DBStorage>) -> Self {
        Self { inner: storage }
    }

    pub fn new_memory_db() -> Self {
        let inner = MemoryDB::new();
        Self::new(Box::new(inner))
    }

    pub fn store_node(&mut self) {}

    pub fn local_seq(&self, id: &NodeId) -> u64 {
        let k = local_item_key(id, DB_LOCAL_SEQ);
        self.fetch_u64(&k)
    }

    /// Retrieves an integer associated with a particular key.
    fn fetch_u64(&self, key: &[u8]) -> u64 {
        self.inner
            .get(key)
            .map(|v| {
                // directly invoke `expect` should be ok here as
                // input/output is done by the code.
                // if cannot parse, then sth is seriously wrong.
                vec_to_u64_le(v).expect("cannot parse to u64")
            })
            .unwrap_or(0)
    }

    /// Stores an integer in the given key.
    fn store_u64(&mut self, key: &[u8], n: u64) {
        self.inner.insert(key.to_vec(), n.to_le_bytes().to_vec());
    }
}

/// Returns the key of a local node item.
fn local_item_key(id: &NodeId, field: &str) -> Vec<u8> {
    let mut v = vec![];
    v.extend(DB_LOCAL_PREFIX.as_bytes());
    v.extend(id.as_bytes());
    v.push(b':');
    v.extend(field.as_bytes());
    v
}

// #[cfg(test)]
// mod tests {
//     use common::H256;
//     use crate::db::local_item_key;
//     use crate::enode::DB;
//
//     #[test]
//     fn store_fetch_u64_works() {
//         let mut db = DB::new_memory_db();
//         db.store_u64(&vec![1, 2, 3], 123);
//         assert_eq!(db.fetch_u64(&vec![1, 2, 3]), 123);
//     }
//
//     #[test]
//     fn local_item_key_works() {
//         let mut bytes = [0; 32];
//         bytes[0] = b'a';
//         bytes[1] = b'b';
//         bytes[2] = b'c';
//         assert_eq!(
//             hex::encode(local_item_key(&H256::from(bytes), "seq")),
//             "6c6f63616c3a61626300000000000000000000000000000000000000000000000000000000003a736571"
//         );
//     }
// }
