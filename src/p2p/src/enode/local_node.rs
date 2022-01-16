use std::cell::RefCell;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use common::{KeyPair};
use crate::config::Config;
use crate::enode::DB;
use crate::enode::node::NodeId;
use crate::enode::url_v4::*;

const DEFAULT_LISTEN_PORT: u16 = 30303;

pub(crate) struct NodeEndpoint {
    address: SocketAddr,
    udp_port: u16,
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
    endpoint: NodeEndpoint,
}

impl LocalNode {
    pub fn new(config: &Config, db: Arc<Mutex<DB>>) -> Self {
        let id = pubkey_to_idv4(config.key_pair.public());
        let seq = { db.lock().unwrap().local_seq(&id) };

        // create endpoint
        let listen_address = match config.listen_address {
            None => SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), DEFAULT_LISTEN_PORT)),
            Some(addr) => addr,
        };
        let udp_port = config.udp_port.unwrap_or_else(|| listen_address.port());

        // create the node itself
        let mut ln = Self {
            cur: None,
            id,
            key_pair: config.key_pair.clone(),
            db,
            seq,
            entries: vec![],
            endpoint: NodeEndpoint { address: listen_address, udp_port }
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