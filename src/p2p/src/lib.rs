#![feature(exclusive_range_pattern)]

pub use connection::Connection;
pub use handshake::Handshake;
pub use discovery::Discovery;
pub use config::{ HostInfo, NetowkrConfig };
pub use node::{ NodeEntry, NodeEndpoint };

mod config;
mod connection;
mod error;
mod handshake;
mod discovery;
mod db;
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
