pub enum Error {
    StdError(std::io::Error),

    // ======== Common Package Error========
    CommonError(common::Error),

    // =========== Socket Related ==========
    SocketNotReady,
    /// The read/write operation was interrupted half way
    Interrupted,
    /// Not all bytes are written to the socket
    IncompleteWrite,
    /// Connection reset by peer
    ConnectionResetByPeer
}

impl From<common::Error> for Error {
    fn from(e: common::Error) -> Self {
        Error::CommonError(e)
    }
}