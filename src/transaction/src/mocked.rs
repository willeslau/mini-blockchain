use crate::Executable;

/// Our mock transaction type, just a simple place hold for the description
#[derive(Clone)]
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