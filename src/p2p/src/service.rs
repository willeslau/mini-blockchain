use std::cell::RefCell;
use std::fmt::Error;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use crate::config::Config;
use crate::enode::{DB, LocalNode};
// use crate::peer::{BASE_PROTOCOL_VERSION, ProtoHandshake};
use crate::protocol::Protocol;

pub struct NetworkService {
    name: String,
    running: Arc<AtomicBool>,
    protocols: Vec<Box<dyn Protocol>>,
    node_db: Arc<Mutex<DB>>,
    local_node: LocalNode,

    config: Config,
}

impl NetworkService {
    pub fn start(name: String, config: Config) -> Result<Self, Error> {
        // setup local node
        // let public_key = config.public_key();
        // let proto_handshake = ProtoHandshake::new(BASE_PROTOCOL_VERSION, self.name.clone(), public_key);
        // proto_handshake.

        let mut db = if config.node_db.is_empty() {
            DB::new_memory_db()
        } else {
            panic!("not implemented");
        };
        let db = Arc::new(Mutex::new(db));

        let local_node = LocalNode::new(&config, db.clone());
        let s = Self {
            name,
            running: Arc::new(AtomicBool::new(false)),
            protocols: vec![],
            node_db: db,
            local_node,
            config,
        };

        // TODO: maybe to do these?
        // srv.localnode.SetFallbackIP(net.IP{127, 0, 0, 1})
        // for _, p := range srv.Protocols {
        //     for _, e := range p.Attributes {
        //         srv.localnode.Set(e)
        //     }
        // }

        Ok(s)
    }

    fn setup_local_node(&mut self) -> Result<(), Error> {
        Ok(())
    }
}
