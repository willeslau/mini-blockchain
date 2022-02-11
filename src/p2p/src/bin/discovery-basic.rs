use common::{KeyPair, Public};
use p2p::{Discovery, HostInfo, NodeEndpoint, NodeEntry, NodeTable};
use secp256k1::SecretKey;
use std::error::Error;
use std::net::{SocketAddr, SocketAddrV4};
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let target_endpoint = NodeEndpoint::new("18.138.108.67", 30303);
    let target_id = Public::from_str("d860a01f9722d78051619d1e2351aba3f43f943f6f00718d1b9baa4101932a1f5011f16bb2b1bb35db20d6fe28fa0bf09636d26a87d31de9ec6203eeedb1f666").unwrap();
    let target_entry = NodeEntry::new(target_id, target_endpoint);

    let local_endpoint = NodeEndpoint {
        address: SocketAddr::V4(SocketAddrV4::new("0.0.0.0".parse().unwrap(), 30303)),
        udp_port: 30303,
    };
    let secret = SecretKey::from_slice(&[
        58, 196, 243, 141, 203, 11, 246, 29, 108, 239, 211, 154, 212, 227, 72, 164, 174, 0, 88,
        124, 6, 76, 117, 8, 190, 84, 234, 51, 49, 67, 23, 209,
    ])
    .unwrap();
    let key_pair = KeyPair::from_secret_key(secret);
    let info = HostInfo::new(key_pair, local_endpoint);

    let node_table = Arc::new(RwLock::new(NodeTable::new_in_memory()));
    let mut discovery = Discovery::start(&info, node_table).await.unwrap();

    discovery.add_node(target_entry.clone()).await.unwrap();
    thread::sleep(Duration::from_millis(1000_000));

    Ok(())
}
