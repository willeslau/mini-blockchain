use common::{H256, Public};
use p2p::{Connection, Handshake};
use std::error::Error;
use std::str::FromStr;
use std::thread;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let remote = timeout(Duration::from_millis(10000),TcpStream::connect("18.138.108.67:30303")).await??;
    println!("connected");
    let connection = Connection::new(remote);

    let remote_node_pub = Public::from_str("d860a01f9722d78051619d1e2351aba3f43f943f6f00718d1b9baa4101932a1f5011f16bb2b1bb35db20d6fe28fa0bf09636d26a87d31de9ec6203eeedb1f666").unwrap();
    let nonce = H256::random();
    let handshake = Handshake::new(remote_node_pub, connection, nonce);
    handshake.start(true).await.unwrap();

    thread::sleep(Duration::from_millis(60000));
    Ok(())
}
