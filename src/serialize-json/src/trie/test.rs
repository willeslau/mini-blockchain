//! TransactionTest test deserializer.

use crate::trie::Trie;
use serde_json::{self, Error};
use serde::Deserialize;
use std::{collections::BTreeMap, io::Read};

/// TransactionTest test deserializer.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Test(BTreeMap<String, Trie>);

impl IntoIterator for Test {
    type Item = <BTreeMap<String, Trie> as IntoIterator>::Item;
    type IntoIter = <BTreeMap<String, Trie> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Test {
    /// Loads test from json.
    pub fn load<R>(reader: R) -> Result<Self, Error>
    where
        R: Read,
    {
        serde_json::from_reader(reader)
    }
}
