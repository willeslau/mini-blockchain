use common::{H256, Public};
use p2p::{Connection, Handshake};
use std::error::Error;
use std::thread;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let remote = timeout(Duration::from_millis(10000),TcpStream::connect("52.231.165.108:30303")).await??;
    println!("connected");
    let connection = Connection::new(remote);

    let remote_node_pub = Public::from_str("715171f50508aba88aecd1250af392a45a330af91d7b90701c436b618c86aaa1589c9184561907bebbb56439b8f8787bc01f49a7c77276c58c1b09822d75e8e8").unwrap();
    let nonce = H256::random();
    let handshake = Handshake::new(remote_node_pub, connection, nonce);
    handshake.start(true).await.unwrap();

    thread::sleep(Duration::from_millis(60000));
    Ok(())
}
