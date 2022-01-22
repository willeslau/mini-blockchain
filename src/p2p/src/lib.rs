mod config;
mod service;
mod peer;
mod protocol;
mod enode;
mod error;
mod nat;
mod handshake;
mod connection;
mod host;

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
