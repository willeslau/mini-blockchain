//! TransactionTest test deserializer.

use crate::transaction::TransactionTest;
use serde_json::{self, Error};
use serde::Deserialize;
use std::{collections::BTreeMap, io::Read};

/// TransactionTest test deserializer.
#[derive(Debug, Deserialize)]
pub struct Test(BTreeMap<String, TransactionTest>);

impl IntoIterator for Test {
    type Item = <BTreeMap<String, TransactionTest> as IntoIterator>::Item;
    type IntoIter = <BTreeMap<String, TransactionTest> as IntoIterator>::IntoIter;

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
