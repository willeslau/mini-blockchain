#![feature(exclusive_range_pattern)]

pub use config::{HostInfo, NetowkrConfig};
pub use connection::Connection;
pub use discovery::Discovery;
pub use handshake::Handshake;
pub use node::{NodeEndpoint, NodeEntry};

mod config;
mod connection;
mod db;
mod discovery;
mod error;
mod handshake;
mod node;

const PROTOCOL_VERSION: u32 = 5;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
