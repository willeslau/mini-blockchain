use crate::connection::{Bytes, Connection};
use crate::enode::{pubkey_to_idv4, NodeId};
use crate::error::Error;
use common::{agree, decrypt, encrypt, sign, KeyPair, Public, H256, H520, recover};
use rand::Rng;
use rlp::{Rlp, RLPStream};
use std::sync::Arc;
use tokio::sync::RwLock;

const V4_AUTH_PACKET_SIZE: usize = 307;
// const V4_ACK_PACKET_SIZE: usize = 210;
const V4_ACK_PACKET_SIZE: usize = 210;
const PROTOCOL_VERSION: u64 = 4;
// Amount of bytes added when encrypting with encryptECIES.
const ECIES_OVERHEAD: usize = 113;

/// The different states during a handshake
#[derive(PartialEq, Eq, Debug)]
pub(crate) enum HandshakeState {
    /// Just created
    New,
    // /// Waiting for auth packet
    // ReadingAuth,
    // /// Waiting for extended auth packet
    // ReadingAuthEip8,
    /// Waiting for ack packet
    ReadingAck,
    // /// Waiting for extended ack packet
    // ReadingAckEip8,
    // /// Ready to start a session
    StartSession,
}

/// Struct to handle the handshake with other eth nodes
pub struct Handshake {
    inner: Arc<RwLock<HandshakeInner>>,
}

impl Handshake {
    pub fn new(remote_node_pub: Public, connection: Connection, nonce: H256) -> Self {
        let remote_node_id = pubkey_to_idv4(&remote_node_pub);
        let inner = HandshakeInner::new(remote_node_id, remote_node_pub, nonce, connection);

        Self {
            inner: Arc::new(RwLock::new(inner))
        }
    }

    pub async fn start(&self, originate: bool) -> Result<(), Error> {
        // TODO: register timeout check in the event loop
        let h = Arc::clone(&self.inner);
        if originate {
            tokio::spawn(async move {
                let mut handshake = h.write().await;
                // let mut connection = c.write().await;
                handshake.write_auth().await.unwrap();
                handshake.read_ack().await.unwrap();
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
    /// Handshake public key
    pub remote_ephemeral: Public,
    /// Remote connection nonce.
    pub remote_nonce: H256,
    /// Remote `RLPx` protocol version.
    pub remote_version: u64,
    auth_cipher: Bytes,
    /// A copy of received encrypted ack packet
    ack_cipher: Bytes,
    state: HandshakeState,
    connection: Connection,
}

impl HandshakeInner {
    pub fn new(
        remote_node_id: NodeId,
        remote_node_pub: Public,
        nonce: H256,
        connection: Connection,
    ) -> Self {
        Self {
            remote_node_id,
            remote_node_pub,
            key_pair: KeyPair::random(),
            nonce,
            auth_cipher: Default::default(),
            ack_cipher: Default::default(),
            state: HandshakeState::New,
            remote_ephemeral: Public::default(),
            remote_nonce: H256::default(),
            remote_version: 0,
            connection
        }
    }

    async fn write_auth(&mut self) -> Result<(), Error> {
        let static_shared = agree(self.key_pair.secret(), &self.remote_node_pub)?;

        let mut rlp = RLPStream::new_list(4);
        rlp.append(&sign(self.key_pair.secret(), &(static_shared.as_ref() ^ &self.nonce))?.to_vec());
        rlp.append(self.key_pair.public());
        rlp.append(&self.nonce);
        rlp.append(&PROTOCOL_VERSION);
        let mut encoded = rlp.out();
        encoded.resize(encoded.len() + rand::thread_rng().gen_range(100..=301), 0);
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

    fn update_remote_id(&mut self, public: Public) {
        self.remote_node_pub = public;
        self.remote_node_id = pubkey_to_idv4(&self.remote_node_pub);
    }

    fn update_auth_meta(
        &mut self,
        sig: &[u8],
        remote_public: &[u8],
        remote_nonce: &[u8],
        remote_version: u64,
    ) -> Result<(), Error> {
        // if we are the originator, this should not have affected the existing values.
        self.update_remote_id(Public::from_slice(remote_public));
        self.remote_nonce = H256::from_slice(remote_nonce);
        self.remote_version = remote_version;
        let shared = agree(self.key_pair.secret(), &self.remote_node_pub)?;
        let signature = H520::from_slice(sig);
        let h: &H256 = shared.as_ref();
        self.remote_ephemeral = recover(&signature.into(), &(h ^ &self.remote_nonce))?;
        Ok(())
    }

    /// Parse and validate ack message
    async fn read_ack(&mut self) -> Result<(), Error> {
        log::info!(
            "parsing reading ack from remote: {:?}",
            self.remote_node_pub
        );

        let bytes = match self.connection.readable().await? {
            Some(v) => v,
            None => vec![],
        };
        log::info!("handshake ack data len received: {:?}", bytes.len());

        match bytes.len() {
            0..V4_ACK_PACKET_SIZE => Err(Error::BadProtocol),
            V4_ACK_PACKET_SIZE => {
                let ack = decrypt(self.key_pair.secret(), &[], &bytes)?;
                self.remote_ephemeral = Public::from_slice(&ack[0..64]);
                self.remote_nonce = H256::from_slice(&ack[64..(64 + 32)]);
                self.state = HandshakeState::StartSession;
                Ok(())
            },
            _ => {
                let ack = decrypt(self.key_pair.secret(), &bytes[0..2], &bytes[2..])?;

                let rlp = Rlp::new(&ack);
                self.remote_ephemeral = rlp.val_at(0)?;
                self.remote_nonce = rlp.val_at(1)?;
                self.remote_version = rlp.val_at(2)?;
                self.state = HandshakeState::StartSession;
                Ok(())
            }
        }
    }

    async fn read_auth(&mut self) -> Result<(), Error> {
        log::info!(
            "parsing reading auth from remote: {:?}",
            self.remote_node_pub
        );

        let bytes = match self.connection.readable().await? {
            Some(v) => v,
            None => vec![],
        };
        if bytes.len() != V4_AUTH_PACKET_SIZE {
            log::debug!("Wrong auth packet size, actual: {:}", bytes.len());
            return Err(Error::BadProtocol);
        }
        log::info!("data received: {:?}", bytes);

        self.auth_cipher = bytes;

        match decrypt(self.key_pair.secret(), &[], &self.auth_cipher) {
            Ok(auth) => {
                let (sig, rest) = auth.split_at(65);
                let (_, rest) = rest.split_at(32);
                let (pubk, rest) = rest.split_at(64);
                let (nonce, _) = rest.split_at(32);
                self.update_auth_meta(sig,pubk, nonce, PROTOCOL_VERSION)?;
                Ok(())
            }
            Err(_) => {
                // TODO: Try to interpret as EIP-8 packet
                Err(Error::NotImplemented)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::handshake::{PROTOCOL_VERSION};
    use common::{sign, KeyPair, Public, Secret, H256, agree};
    use rlp::{Rlp, RLPStream};
    use secp256k1::{PublicKey, Secp256k1, SecretKey};


    /// Helper function to perform RLP encoding on the some of the auth data
    fn rlp_encode(key_pair: &KeyPair, remote_pub: &Public, nonce: &H256, protocol: &u64) -> Vec<u8> {
        let static_shared = agree(key_pair.secret(), remote_pub).unwrap();
        let mut rlp = RLPStream::new_list(4);

        rlp.append(
            &sign(key_pair.secret(), &(static_shared.as_ref() ^ nonce))
                .unwrap()
                .to_vec(),
        );
        rlp.append(key_pair.public());
        rlp.append(nonce);
        rlp.append(protocol);

        rlp.out()
    }

    #[test]
    fn write_auth_works() {
        let secret = Secret::copy_from_str(
            "b71c71a67e1177ad4e901695e1b4b9ee17ae16c6668d313eac2f96dbcda3f291",
        )
        .unwrap();

        let s = secret.to_secp256k1_secret().unwrap();
        let kp = KeyPair::from_secret_key(s);

        let public = Public::from_str("e37f3cbb0d0601dc930b8d8aa56910dd5629f2a0979cc742418960573efc5c0ff96bc87f104337d8c6ab37e597d4f9ffbd57302bc98a825519f691b378ce13f5").unwrap();
        let nonce = [
            56, 242, 184, 93, 221, 158, 68, 46, 153, 138, 12, 152, 135, 63, 27, 151, 136, 30, 18,
            171, 49, 150, 97, 219, 68, 55, 148, 72, 124, 63, 140, 230,
        ];

        let v = rlp_encode(&kp, &public, &H256::from(nonce), &PROTOCOL_VERSION);
        assert_eq!(
            v,
            vec![
                248, 167, 184, 65, 82, 75, 241, 111, 123, 177, 70, 150, 169, 31, 140, 25, 127, 84,
                253, 146, 99, 79, 37, 200, 246, 58, 40, 85, 114, 161, 59, 131, 38, 218, 85, 246,
                112, 89, 246, 138, 8, 194, 35, 188, 224, 59, 179, 166, 142, 251, 66, 215, 77, 215,
                150, 159, 68, 194, 104, 110, 76, 208, 81, 218, 173, 220, 75, 139, 1, 184, 64, 202,
                99, 76, 174, 13, 73, 172, 180, 1, 216, 164, 198, 182, 254, 140, 85, 183, 13, 17,
                91, 244, 0, 118, 156, 193, 64, 15, 50, 88, 205, 49, 56, 117, 116, 7, 127, 48, 27,
                66, 27, 200, 77, 247, 38, 108, 68, 233, 230, 213, 105, 252, 86, 190, 0, 129, 41, 4,
                118, 123, 245, 204, 209, 252, 127, 160, 56, 242, 184, 93, 221, 158, 68, 46, 153,
                138, 12, 152, 135, 63, 27, 151, 136, 30, 18, 171, 49, 150, 97, 219, 68, 55, 148,
                72, 124, 63, 140, 230, 4
            ]
        );

        let ack = hex::encode("\
			049f8abcfa9c0dc65b982e98af921bc0ba6e4243169348a236abe9df5f93aa69d99cadddaa387662\
			b0ff2c08e9006d5a11a278b1b3331e5aaabf0a32f01281b6f4ede0e09a2d5f585b26513cb794d963\
			5a57563921c04a9090b4f14ee42be1a5461049af4ea7a7f49bf4c97a352d39c8d02ee4acc416388c\
			1c66cec761d2bc1c72da6ba143477f049c9d2dde846c252c111b904f630ac98e51609b3b1f58168d\
			dca6505b7196532e5f85b259a20c45e1979491683fee108e9660edbf38f3add489ae73e3dda2c71b\
			d1497113d5c755e942d1\
			");
        println!("{:?}", ack.as_bytes().len());
    }

    // use common::{sign, KeyPair, Public, Secret, H256, agree};
    // use rlp::RLPStream;
    // use secp256k1::{PublicKey, Secp256k1, SecretKey};
    // use crate::Rlp;

    #[test]
    fn test_rlp_works() {
        let v = vec![248, 100, 184, 64, 186, 92, 206, 211, 187, 200, 65, 210, 152, 97, 40, 173, 166, 44, 7, 110, 101, 42, 93, 126, 43, 3, 150, 175, 128, 227, 87, 65, 82, 51, 154, 192, 94, 220, 87, 207, 170, 2, 139, 177, 110, 193, 159, 237, 16, 78, 172, 88, 47, 112, 14, 209, 240, 176, 77, 237, 84, 17, 23, 154, 51, 108, 240, 40, 160, 215, 217, 94, 141, 43, 16, 124, 63, 11, 34, 168, 196, 53, 217, 254, 50, 126, 120, 82, 187, 77, 207, 174, 246, 105, 52, 120, 157, 101, 137, 38, 41, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        let rlp = Rlp::new(&v);

        let p: Public = rlp.val_at(0).unwrap();
        let f: H256 = rlp.val_at(1).unwrap();
        let u: u64 = rlp.val_at(2).unwrap();
    }
}
