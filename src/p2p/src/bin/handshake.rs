use std::error::Error;
use std::thread;
use std::time::Duration;
use tokio::net::TcpStream;
use common::{Public, random_h256};
use p2p::{Connection, Handshake};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let remote = TcpStream::connect("18.138.108.67:30303").await?;
    let connection = Connection::new(remote);

    let remote_node_pub = Public::from_str("d860a01f9722d78051619d1e2351aba3f43f943f6f00718d1b9baa4101932a1f5011f16bb2b1bb35db20d6fe28fa0bf09636d26a87d31de9ec6203eeedb1f666").unwrap();
    let nonce = random_h256();
    let handshake = Handshake::new(remote_node_pub, connection, nonce);
    handshake.start(true).await.unwrap();

    thread::sleep(Duration::from_millis(40000));
    Ok(())
}