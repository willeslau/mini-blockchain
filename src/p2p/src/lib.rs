#![feature(exclusive_range_pattern)]

pub use connection::Connection;
pub use handshake::Handshake;

mod config;
mod connection;
mod error;
mod handshake;
mod discovery;
mod db;
mod node;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
