//! Blockchain test state deserializer.

use crate::{
    bytes::Bytes,
    hash::{Address, H256},
    spec::{Account, Builtin},
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(untagged)]
pub enum HashOrMap {
    /// When the `postState` is large, tests sometimes just include the state root of the last
    /// successful block here.
    Hash(H256),
    /// The expected `postState` of a test
    Map(BTreeMap<Address, Account>),
}

/// Blockchain state deserializer.
#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State(pub HashOrMap);

impl State {
    /// Returns all builtins.
    pub fn builtins(&self) -> BTreeMap<Address, Builtin> {
        match &self.0 {
            HashOrMap::Hash(_) => BTreeMap::default(),
            HashOrMap::Map(map) => map
                .iter()
                .filter_map(|(add, ref acc)| acc.builtin.clone().map(|b| (add.clone(), b.into())))
                .collect(),
        }
    }

    /// Returns all constructors.
    pub fn constructors(&self) -> BTreeMap<Address, Bytes> {
        match &self.0 {
            HashOrMap::Hash(_) => BTreeMap::default(),
            HashOrMap::Map(map) => map
                .iter()
                .filter_map(|(add, ref acc)| acc.constructor.clone().map(|b| (add.clone(), b)))
                .collect(),
        }
    }
}

impl IntoIterator for State {
    type Item = <BTreeMap<Address, Account> as IntoIterator>::Item;
    type IntoIter = <BTreeMap<Address, Account> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        if let HashOrMap::Map(m) = self.0 {
            m.into_iter()
        } else {
            BTreeMap::default().into_iter()
        }
    }
}
