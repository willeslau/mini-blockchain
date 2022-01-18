use common::{ecdh, H256, KeyPair, Public, sign, xor};
use rlp::RLPStream;
use crate::connection::Connection;
use crate::enode::NodeId;
use crate::error::Error;

const PROTOCOL_VERSION: u64 = 4;

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
    /// Remote node id, i.e. hash of public key
    remote_node_id: NodeId,
    /// Remote node public key
    remote_node_pub: Public,
    /// Local node key pair
    key_pair: KeyPair,
    nonce: H256,
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
            self.write_auth()?
        }

        Ok(())
    }

    fn write_auth(&mut self) -> Result<(), Error>{
        let static_shared = ecdh::agree(self.key_pair.secret(), &self.remote_node_pub)?;
        let mut rlp = RLPStream::new_list(4);

        // todo: check Encodable
        rlp.append(&sign(self.key_pair.secret(), &static_shared.xor(&self.nonce))?.to_vec());
        rlp.append(self.key_pair.public());
        rlp.append(&self.nonce);
        rlp.append(&PROTOCOL_VERSION);
        let mut encoded = rlp.out();
        Ok(())
    }
}