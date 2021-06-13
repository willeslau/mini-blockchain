pub trait StringSerializable {
    /// Serialize to string
    fn serialize(&self) -> Box<str>;
    /// Deserialize from string
    fn deserialize(data: &str) -> Self;
}

