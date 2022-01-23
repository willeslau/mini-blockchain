/// Errors occur in this package
#[derive(Debug)]
pub enum Error {
    StdError(std::io::Error),
    NotImplemented,

    // ======== External Package Error========
    CommonError(common::Error),
    RlpError(rlp::Error),

    // =========== Socket Related ==========
    SocketNotReady,
    /// The read/write operation was interrupted half way
    Interrupted,
    /// Not all bytes are written to the socket
    IncompleteWrite,
    /// Connection reset by peer
    ConnectionResetByPeer,

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
