use serde::{Serialize, Deserialize};
use crate::Executable;
use primitives::StringSerializable;

/// Our mock transaction type, just a simple place hold for the description
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MockedExecutable {
    value: String,
}

impl MockedExecutable {
    pub fn new(value: String) -> Self {
        MockedExecutable { value }
    }
}

impl Executable for MockedExecutable {
    fn is_valid(&self) -> bool {
        true
    }

    fn execute(&self) -> Result<(), ()> {
        Ok(())
    }
}

impl StringSerializable for MockedExecutable {
    fn serialize(&self) -> Box<str> {
        Box::from(serde_json::to_string(self).unwrap())
    }

    fn deserialize(data: &str) -> Self {
        serde_json::from_str(data).unwrap()
    }
}