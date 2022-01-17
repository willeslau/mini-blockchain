use crate::connection::Connection;
use crate::enode::NodeId;
use crate::error::Error;

/// The different states during a handshake
#[derive(PartialEq, Eq, Debug)]
enum HandshakeState {
    /// Just created
    New,
    /// Waiting for auth packet
    ReadingAuth,
    /// Waiting for extended auth packet
    ReadingAuthEip8,
    /// Waiting for ack packet
    ReadingAck,
    /// Waiting for extended ack packet
    ReadingAckEip8,
    /// Ready to start a session
    StartSession,
}

/// The incoming frames used during the handshakes
pub(crate) enum IngressFrame {

}

/// Struct to handle the handshake with other eth nodes
pub(crate) struct Handshake {
    /// Remote node public key
    pub id: NodeId,
    /// Underlying connection
    pub connection: Connection,
}

impl Handshake {
    // pub fn new(id: NodeId, socket: TcpStream, token: Token) -> Self {
    //     Self {
    //         id,
    //         connection: Connection::new(socket, token),
    //     }
    // }
    //
    pub fn start(&mut self, originate: bool) -> Result<(), Error> {
        // TODO: register timeout check in the event loop

        if originate {
            self.write_auth()
        }
        Ok(())
    }
}