//! Blockchain test state deserializer.

use crate::{
    bytes::Bytes,
    hash::Address,
    spec::{Account, Builtin},
};
use std::collections::BTreeMap;

/// Blockchain test state deserializer.
#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State(BTreeMap<Address, Account>);

impl State {
    /// Returns all builtins.
    pub fn builtins(&self) -> BTreeMap<Address, Builtin> {
        self.0
            .iter()
            .filter_map(|(add, ref acc)| acc.builtin.clone().map(|b| (add.clone(), b.into())))
            .collect()
    }

    /// Returns all constructors.
    pub fn constructors(&self) -> BTreeMap<Address, Bytes> {
        self.0
            .iter()
            .filter_map(|(add, ref acc)| acc.constructor.clone().map(|b| (add.clone(), b)))
            .collect()
    }
}

impl IntoIterator for State {
    type Item = <BTreeMap<Address, Account> as IntoIterator>::Item;
    type IntoIter = <BTreeMap<Address, Account> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
