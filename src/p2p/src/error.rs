pub enum Error {
    StdError(std::io::Error),

    // =========== Socket Related ==========
    SocketNotReady,
    /// The read/write operation was interrupted half way
    Interrupted,
    /// Not all bytes are written to the socket
    IncompleteWrite,
    /// Connection reset by peer
    ConnectionResetByPeer
}