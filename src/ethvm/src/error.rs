/// All vm related errors
#[derive(Debug)]
pub enum Error {
    OutOfGas,
    InvalidCommand,
    InvalidJump
}
