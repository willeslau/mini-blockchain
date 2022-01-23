use std::sync::Arc;
use bytes::BytesMut;
use rand::Rng;
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use common::{agree, decrypt, encrypt, H256, KeyPair, Public, Secret, sign, xor};
use rlp::RLPStream;
use crate::connection::{Bytes, Connection, FrameCodec};
use crate::enode::{NodeId, pubkey_to_idv4};
use crate::error::Error;

const V4_AUTH_PACKET_SIZE: usize = 307;
const V4_ACK_PACKET_SIZE: usize = 210;
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

/// The codec that converts bytes to IngressFrame
struct IngressFrameCodec<'a> {
    inner: Arc<&'a HandshakeInner>,
}

impl FrameCodec for IngressFrameCodec {
    fn parse_frame<T>(&mut self, bytes: &mut BytesMut) -> Result<Option<T>, Error> {
        match self.inner.state {
            HandshakeState::New => Ok(None),
            HandshakeState::ReadingAck => self.read_auth(bytes),

            // TODO: handle these states
            HandshakeState::ReadingAuth => Ok(None),
            HandshakeState::ReadingAuthEip8 => Ok(None),
            HandshakeState::ReadingAckEip8 => Ok(None),
            HandshakeState::StartSession => Ok(None),
        }
    }
}

impl <'a> IngressFrameCodec<'a> {
    fn read_auth(&mut self, bytes: &mut BytesMut) -> Result<Option<IngressFrame<'a>>, Error> {
        log::info!("parsing reading auth from remote: {:?}", self.inner.remote_node_pub);
        if bytes.len() != V4_AUTH_PACKET_SIZE {
            log::debug!("Wrong auth packet size, actual: {:}", bytes.len());
            return Err(Error::BadProtocol);
        }
        self.inner.auth_cipher = bytes.to_vec();
        match decrypt(self.inner.key_pair.secret(), &[], bytes) {
            Ok(auth) => {
                let (sig, rest) = auth.split_at(65);
                let (_, rest) = rest.split_at(32);
                let (pubk, rest) = rest.split_at(64);
                let (nonce, _) = rest.split_at(32);

                // self.set_auth(secret, sig, pubk, nonce, PROTOCOL_VERSION)?;
                // self.write_ack(io)?;

                Ok(Some(IngressFrame::AuthAck {sig, pubk, nonce}))
            }
            Err(_) => {
                // TODO: Try to interpret as EIP-8 packet
                Err(Error::NotImplemented)
            }
        }
    }
}

/// The incoming frames used during the handshakes
enum IngressFrame<'a> {
    AuthAck{ sig: &'a [u8], pubk: &'a [u8], nonce: &'a [u8]},
}

/// Struct to handle the handshake with other eth nodes
pub struct Handshake {
    inner: Arc<HandshakeInner>,
}

impl Handshake {
    pub fn new(remote_node_pub: Public, stream: TcpStream, nonce: H256) -> Self {

        let remote_node_id = pubkey_to_idv4(&remote_node_pub);
        let inner = HandshakeInner::new(remote_node_id, remote_node_pub, connection, nonce);
        Self {
            inner: Arc::new(inner)
        }
    }

    pub async fn start(&self, originate: bool) -> Result<(), Error> {
        // TODO: register timeout check in the event loop
        let mut handshake = Arc::clone(&self.inner);
        if originate {
            tokio::spawn(async move {
                handshake.write_auth().await.unwrap();
                handshake.read_auth().await.unwrap();
            });
        }

        Ok(())
    }
}

/// The inner structure for Handshake
pub(crate) struct HandshakeInner {
    /// Remote node id, i.e. hash of public key
    remote_node_id: NodeId,
    /// Remote node public key
    remote_node_pub: Public,
    /// Local node key pair
    key_pair: KeyPair,
    nonce: H256,
    /// Underlying connection
    connection: Connection<IngressFrame>,
    auth_cipher: Bytes,
    /// A copy of received encrypted ack packet
    ack_cipher: Bytes,
    state: HandshakeState,
}

impl HandshakeInner {
    pub fn new(remote_node_id: NodeId, remote_node_pub: Public, connection: Connection<IngressFrame>, nonce: H256) -> Self {
        Self {
            remote_node_id,
            remote_node_pub,
            key_pair: KeyPair::random(),
            nonce,
            connection,
            auth_cipher: Default::default(),
            ack_cipher: Default::default(),
            state: HandshakeState::New,
        }
    }

    async fn read_auth(&mut self) -> Result<(), Error> {
        self.connection.readable::<IngressFrame>().await?;
        Ok(())
    }

    async fn write_auth(&mut self) -> Result<(), Error>{
        let static_shared = agree(self.key_pair.secret(), &self.remote_node_pub)?;

        let mut rlp = RLPStream::new_list(4);
        rlp.append(&sign(self.key_pair.secret(), &static_shared.xor(&self.nonce))?.to_vec());
        rlp.append(self.key_pair.public());
        rlp.append(&self.nonce);
        rlp.append(&PROTOCOL_VERSION);
        let mut encoded = rlp.out();

        encoded.resize(encoded.len() + rand::thread_rng().gen_range(100..301), 0);
        let len = (encoded.len() + ECIES_OVERHEAD) as u16;
        let prefix = len.to_be_bytes();
        let message = encrypt(&self.remote_node_pub, &prefix, &encoded)?;

        self.auth_cipher.extend_from_slice(&prefix);
        self.auth_cipher.extend_from_slice(&message);

        self.connection.write(&self.auth_cipher).await?;
        self.connection.expect(V4_ACK_PACKET_SIZE);

        self.state = HandshakeState::ReadingAck;

        Ok(())
    }
}

/// Helper function to perform RLP encoding on the some of the auth data
fn rlp_encode(key_pair: &KeyPair, remote_pub: &Public, nonce: &H256, protocol: &u64) -> Vec<u8> {
    let static_shared = agree(key_pair.secret(), remote_pub).unwrap();
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
    use common::{ KeyPair, Public, Secret, sign};
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