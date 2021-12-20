pub enum Error {
    /// The key to be inserted should not be zero length
    KeyCannotBeEmpty,
    /// The value to be inserted should not be zero length
    ValueCannotBeEmpty,
}