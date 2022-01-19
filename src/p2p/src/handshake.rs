use rand::Rng;
use common::{ecdh, H256, KeyPair, Public, Secret, sign, xor};
use rlp::RLPStream;
use crate::connection::Connection;
use crate::enode::NodeId;
use crate::error::Error;

const PROTOCOL_VERSION: u64 = 4;
// Amount of bytes added when encrypting with encryptECIES.
const ECIES_OVERHEAD: usize = 113;

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
    pub fn new(remote_node_id: NodeId, remote_node_pub: Public, connection: Connection, nonce: H256) -> Self {
        Self {
            remote_node_id,
            remote_node_pub,
            key_pair: KeyPair::random(),
            nonce,
            connection
        }
    }

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
        rlp.append(&sign(self.key_pair.secret(), &static_shared.xor(&self.nonce))?.to_vec());
        rlp.append(self.key_pair.public());
        rlp.append(&self.nonce);
        rlp.append(&PROTOCOL_VERSION);
        let mut encoded = rlp.out();
        encoded.resize(encoded.len() + rand::thread_rng().gen_range(100, 301), 0);
        let len = (encoded.len() + ECIES_OVERHEAD) as u16;
        let prefix = len.to_be_bytes();
        let message = ecies::encrypt(&self.id, &prefix, &encoded)?;
        self.auth_cipher.extend_from_slice(&prefix);
        self.auth_cipher.extend_from_slice(&message);
        self.connection.send(io, self.auth_cipher.clone());
        self.connection.expect(V4_ACK_PACKET_SIZE);
        Ok(())
    }
}

/// Helper function to perform RLP encoding on the some of the auth data
fn rlp_encode(key_pair: &KeyPair, remote_pub: &Public, nonce: &H256, protocol: &u64) -> Vec<u8> {
    let static_shared = ecdh::agree(key_pair.secret(), remote_pub).unwrap();
    let mut rlp = RLPStream::new_list(4);

    rlp.append(&sign(key_pair.secret(), &static_shared.xor(nonce)).unwrap().to_vec());
    rlp.append(key_pair.public());
    rlp.append(nonce);
    rlp.append(protocol);

    rlp.out()
}

#[cfg(test)]
mod tests {
    use secp256k1::{PublicKey, Secp256k1, SecretKey};
    use common::{ecdh, KeyPair, Public, Secret, sign};
    use rlp::RLPStream;
    use crate::handshake::{PROTOCOL_VERSION, rlp_encode};

    #[test]
    fn write_auth_works() {
        let secret = Secret::copy_from_str("b71c71a67e1177ad4e901695e1b4b9ee17ae16c6668d313eac2f96dbcda3f291").unwrap();

        let s = secret.to_secp256k1_secret().unwrap();
        let kp = KeyPair::from_secret_key(s);

        let public= Public::from_str("e37f3cbb0d0601dc930b8d8aa56910dd5629f2a0979cc742418960573efc5c0ff96bc87f104337d8c6ab37e597d4f9ffbd57302bc98a825519f691b378ce13f5").unwrap();
        let nonce = [56, 242, 184, 93, 221, 158, 68, 46, 153, 138, 12, 152, 135, 63, 27, 151, 136, 30, 18, 171, 49, 150, 97, 219, 68, 55, 148, 72, 124, 63, 140, 230];

        let v = rlp_encode(&kp, &public, &nonce, &PROTOCOL_VERSION);
        assert_eq!(
            v,
            vec![248, 167, 184, 65, 82, 75, 241, 111, 123, 177, 70, 150, 169, 31, 140, 25, 127, 84, 253, 146, 99, 79, 37, 200, 246, 58, 40, 85, 114, 161, 59, 131, 38, 218, 85, 246, 112, 89, 246, 138, 8, 194, 35, 188, 224, 59, 179, 166, 142, 251, 66, 215, 77, 215, 150, 159, 68, 194, 104, 110, 76, 208, 81, 218, 173, 220, 75, 139, 1, 184, 64, 202, 99, 76, 174, 13, 73, 172, 180, 1, 216, 164, 198, 182, 254, 140, 85, 183, 13, 17, 91, 244, 0, 118, 156, 193, 64, 15, 50, 88, 205, 49, 56, 117, 116, 7, 127, 48, 27, 66, 27, 200, 77, 247, 38, 108, 68, 233, 230, 213, 105, 252, 86, 190, 0, 129, 41, 4, 118, 123, 245, 204, 209, 252, 127, 160, 56, 242, 184, 93, 221, 158, 68, 46, 153, 138, 12, 152, 135, 63, 27, 151, 136, 30, 18, 171, 49, 150, 97, 219, 68, 55, 148, 72, 124, 63, 140, 230, 4]
        )
    }
}