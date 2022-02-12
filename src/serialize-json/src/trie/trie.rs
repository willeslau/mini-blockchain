//! Trie test deserialization.
use serde::Deserialize;
use crate::{hash::H256, trie::Input};

/// Trie test deserialization.
#[derive(Debug, Deserialize, PartialEq)]
pub struct Trie {
    /// Trie test input.
    #[serde(rename = "in")]
    pub input: Input,
    /// Trie root hash.
    pub root: H256,
}
