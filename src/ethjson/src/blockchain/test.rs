//! Blockchain test deserializer.

use crate::blockchain::blockchain::BlockChain;
use serde_json::{self, Error};
use std::{collections::BTreeMap, io::Read};

/// Blockchain test deserializer.
#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct Test(BTreeMap<String, BlockChain>);

impl IntoIterator for Test {
    type Item = <BTreeMap<String, BlockChain> as IntoIterator>::Item;
    type IntoIter = <BTreeMap<String, BlockChain> as IntoIterator>::IntoIter;

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
