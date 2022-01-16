use std::io;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter, Interest};
use tokio::net::TcpStream;
use tokio_util::codec::{Decoder, Encoder};
use crate::enode::NodeId;
use crate::error::Error;

const BUFFER_CAPACITY: usize = 4 * 1024;
const DEFAULT_INTERESTS: Interest = Interest::READABLE.add(Interest::WRITABLE);

pub type Bytes = Vec<u8>;

/// The generic frame used for the connection
pub trait Frame: Sized {
    fn parse_frame(bytes: &mut BytesMut) -> Result<Option<Self>, Error>;
}

/// This represents a connection to a peer
pub struct Connection {
    /// The socket containter
    socket: TcpStream,
    /// The buffer for reading frames.
    buffer: BytesMut,
    registered: AtomicBool,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            socket: stream,
            buffer: BytesMut::with_capacity(BUFFER_CAPACITY),
            registered: AtomicBool::new(false)
        }
    }
    //
    // pub fn register_socket(&mut self, poll: &mut Poll) -> Result<(), Error> {
    //     if self.registered.load(Ordering::SeqCst) {
    //         return Ok(());
    //     }
    //
    //     poll.registry()
    //         .register(&mut self.socket, self.token, DEFAULT_INTERESTS)?;
    //
    //     self.registered.store(true, Ordering::SeqCst);
    //     Ok(())
    // }
    //
    // pub fn reregister_socket(&mut self, poll: &mut Poll) -> Result<(), Error> {
    //     poll.registry().reregister(&mut self.socket, self.token, DEFAULT_INTERESTS)?;
    //     Ok(())
    // }
    //
    // /// Get remote peer address
    // pub fn remote_addr(&self) -> Result<SocketAddr, Error> {
    //     let addr = self.socket.peer_addr()?;
    //     Ok(addr)
    // }
    //
    // /// Get remote peer address string
    // pub fn remote_addr_str(&self) -> String {
    //     self.socket
    //         .peer_addr()
    //         .map(|a| a.to_string())
    //         .unwrap_or_else(|_| "Unknown".to_owned())
    // }
    //
    // /// Get local peer address string
    // pub fn local_addr_str(&self) -> String {
    //     self.socket
    //         .local_addr()
    //         .map(|a| a.to_string())
    //         .unwrap_or_else(|_| "Unknown".to_owned())
    // }

    /// Read from the socket. Caller ensure the socket is readable
    pub async fn readable<F: Frame>(&mut self) -> Result<Option<F>, Error>{
        loop {
            if let Some(frame) = F::parse_frame(&mut self.buffer)? {
                return Ok(Some(frame));
            }
            if 0 == self.socket.read_buf(&mut self.buffer).await? {
                return if self.buffer.is_empty() {
                    Ok(None)
                } else {
                    Err(Error::ConnectionResetByPeer)
                }
            }

            // TODO: check max capacity
        }
    }

    /// Write to the socket. Caller ensure the socket is writable
    pub async fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        match self.socket.write(data).await {
            Ok(n) if n < data.len() => Err(Error::IncompleteWrite),
            Ok(_) => Ok(()),
            Err(err) => return Err(err.into()),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::StdError(e)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}