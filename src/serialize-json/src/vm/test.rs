//! Vm test deserializer.

use crate::vm::Vm;
use serde_json::{self, Error};
use std::{collections::BTreeMap, io::Read};

/// Vm test deserializer.
#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct Test(BTreeMap<String, Vm>);

impl IntoIterator for Test {
    type Item = <BTreeMap<String, Vm> as IntoIterator>::Item;
    type IntoIter = <BTreeMap<String, Vm> as IntoIterator>::IntoIter;

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
