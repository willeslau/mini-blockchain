pub enum Error {
    InvalidTokenSize,
    IOError(std::io::Error)
}