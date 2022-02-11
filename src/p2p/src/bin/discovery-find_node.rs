use common::{KeyPair, Public};
use p2p::{Discovery, HostInfo, NodeEndpoint, NodeEntry, NodeTable};
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

    let local_endpoint = NodeEndpoint {
        address: SocketAddr::V4(SocketAddrV4::new("0.0.0.0".parse().unwrap(), 30304)),
        udp_port: 30304,
    };
    let key_pair = KeyPair::random();
    let info = HostInfo::new(key_pair, local_endpoint);

    let node_table = Arc::new(RwLock::new(NodeTable::new_in_memory()));
    let mut discovery = Discovery::start(&info, node_table).await.unwrap();

    let target_endpoint = NodeEndpoint::new("0.0.0.0", 30303);
    let target_id = Public::from_slice(&[
        214, 205, 211, 59, 119, 131, 177, 238, 37, 99, 193, 231, 37, 139, 109, 165, 185, 165, 10,
        10, 175, 155, 156, 84, 241, 86, 34, 59, 197, 137, 66, 192, 102, 70, 254, 157, 112, 69, 86,
        148, 197, 82, 246, 119, 226, 247, 26, 162, 15, 43, 119, 128, 203, 229, 76, 64, 217, 91,
        210, 195, 46, 26, 115, 170,
    ]);
    let target_entry = NodeEntry::new(target_id, target_endpoint);

    discovery.add_node(target_entry.clone()).await.unwrap();
    thread::sleep(Duration::from_millis(2000));

    // need to wait a bit for ping/pong to finish
    let find_id = Public::from_str("d860a01f9722d78051619d1e2351aba3f43f943f6f00718d1b9baa4101932a1f5011f16bb2b1bb35db20d6fe28fa0bf09636d26a87d31de9ec6203eeedb1f666").unwrap();
    discovery.find_node(find_id, target_entry).await.unwrap();
    thread::sleep(Duration::from_millis(100_000));

    Ok(())
}
