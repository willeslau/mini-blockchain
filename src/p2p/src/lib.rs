#![feature(exclusive_range_pattern)]
#![feature(async_closure)]

pub use config::{HostInfo, NetowkrConfig};
pub use connection::Connection;
pub use discovery::Discovery;
pub use handshake::Handshake;
pub use node::{NodeEndpoint, NodeEntry};
pub use node_table::NodeTable;

mod config;
mod connection;
mod discovery;
mod error;
mod handshake;
mod node;
mod node_table;

const PROTOCOL_VERSION: u32 = 5;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
