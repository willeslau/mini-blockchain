use std::cell::RefCell;
use std::net::{SocketAddr};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use common::{KeyPair};
use crate::enode::DB;
use crate::enode::node::NodeId;
use crate::enode::url_v4::*;

pub(crate) struct LnEndpoint  {
    static_ip: SocketAddr,
    fallback_ip: SocketAddr,
    fallback_udp: u16,
}

/// LocalNode produces the signed node record of a local node, i.e. a node run in the
/// current process. Setting ENR entries via the Set method updates the record. A new version
/// of the record is signed on demand when the Node method is called.
pub(crate) struct LocalNode {
    /// holds a non-nil node pointer while the record is up-to-date.
    cur: Option<NodeId>,
    id: NodeId,
    key_pair: KeyPair,
    db: Arc<Mutex<DB>>,

    // everything below is protected by a lock
    seq: u64,
    entries: Vec<u8>,
    // endpoint: LnEndpoint,
}

impl LocalNode {
    pub fn new(key_pair: KeyPair, db: Arc<Mutex<DB>>) -> Self {
        let id = pubkey_to_idv4(key_pair.public());
        let mut seq;
        {
            let d = db.lock().unwrap();
            seq = d.local_seq(&id);
        }
        let mut ln = Self {
            cur: None,
            id,
            key_pair,
            db,
            seq,
            entries: vec![],
            // endpoint: LnEndpoint {},
        };

        ln.invalidate();
        ln
    }

    pub fn invalidate(&mut self) {
        self.cur = None;
    }
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use common::KeyPair;

    #[test]
    fn it_works() {
        let secret = "bacd06016aea4280e14efd7182ba18cd98bf11701943d3d47d76b04bb7baad19";
        let s = secp256k1::SecretKey::from_str(secret).unwrap();
        let kp = KeyPair::from_secret_key(s);
        println!("{:x?}", kp.public());
    }
}