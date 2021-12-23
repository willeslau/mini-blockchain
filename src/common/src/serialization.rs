use bincode::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum SerializationError {
    BincodeError(bincode::Error)
}

impl From<bincode::Error> for SerializationError {
    fn from(e: Error) -> Self {
        Self::BincodeError(e)
    }
}

pub fn to_vec<T>(value: &T) -> Result<Vec<u8>, SerializationError>
    where
        T: Serialize,
{
    let v = bincode::serialize(value)?;
    Ok(v)
}

pub fn from_vec<'a, T>(raw: &'a Vec<u8>) -> Result<T, SerializationError>
    where
        T: Deserialize<'a>,
{
    let v = bincode::deserialize(raw)?;
    Ok(v)
}