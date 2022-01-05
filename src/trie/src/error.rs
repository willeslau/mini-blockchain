#[derive(Debug)]
pub enum Error {
    /// The key to be inserted should not be zero length
    KeyCannotBeEmpty,
    /// The value to be inserted should not be zero length
    ValueCannotBeEmpty,
    /// The node location passed is invalid
    InvalidNodeLocation,
    /// The state of the trie is invalid
    InvalidTrieState,
    /// The key is not found in the trie
    KeyNotExists,
}
