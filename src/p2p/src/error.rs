use std::net::SocketAddr;
use tokio::sync::mpsc::error::SendError;
use crate::discovery::Request;

/// Errors occur in this package
#[derive(Debug)]
pub enum Error {
    StdError(std::io::Error),
    NotImplemented,

    // ======== External Package Error========
    CommonError(common::Error),
    RlpError(rlp::Error),
    TokioVecError(tokio::sync::mpsc::error::SendError<Vec<u8>>),
    TokioVecSocketError(tokio::sync::mpsc::error::SendError<(Vec<u8>, std::net::SocketAddr)>),
    TokioRequestError(tokio::sync::mpsc::error::SendError<Request>),

    // =========== Socket Related ==========
    SocketNotReady,
    /// The read/write operation was interrupted half way
    Interrupted,
    /// Not all bytes are written to the socket
    IncompleteWrite,
    /// Connection reset by peer
    ConnectionResetByPeer,

    // ========== P2P network errors ==========
    InvalidNodeDistance,
    NodeBlocked,

    // =========== Handshake Related ==========
    BadProtocol,
    ExpectedReceivedSizeNotSet,
}

impl From<common::Error> for Error {
    fn from(e: common::Error) -> Self {
        Error::CommonError(e)
    }
}

impl From<rlp::Error> for Error {
    fn from(e: rlp::Error) -> Self {
        Error::RlpError(e)
    }
}

impl From<tokio::sync::mpsc::error::SendError<Vec<u8>>> for Error {
    fn from(e: SendError<Vec<u8>>) -> Self {
        Error::TokioVecError(e)
    }
}

impl From<tokio::sync::mpsc::error::SendError<(Vec<u8>, std::net::SocketAddr)>> for Error {
    fn from(e: SendError<(Vec<u8>, SocketAddr)>) -> Self {
        Error::TokioVecSocketError(e)
    }
}

impl From<tokio::sync::mpsc::error::SendError<Request>> for Error {
    fn from(e: SendError<Request>) -> Self {
        Error::TokioRequestError(e)
    }
}