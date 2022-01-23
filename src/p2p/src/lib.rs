#![feature(exclusive_range_pattern)]

mod config;
mod connection;
mod enode;
mod error;
mod handshake;
mod host;
mod nat;
mod peer;
mod protocol;
mod service;

pub use connection::Connection;
pub use handshake::Handshake;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
