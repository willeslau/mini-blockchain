use serde::{Serialize, Deserialize};
use crate::Executable;
use primitives::StringSerializable;

/// Our mock transaction type, just a simple place hold for the description
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MockTransaction {
    value: String,
}

impl MockTransaction {
    pub fn new(value: String) -> Self {
        MockTransaction { value }
    }
}

impl Executable for MockTransaction {
    fn is_valid() -> bool {
        true
    }

    fn execute() -> Result<(), ()> {
        Ok(())
    }
}

impl StringSerializable for MockTransaction {
    fn serialize(&self) -> Box<str> {
        Box::from(serde_json::to_string(self).unwrap())
    }

    fn deserialize(data: &str) -> Self {
        serde_json::from_str(data).unwrap()
    }
}