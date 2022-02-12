//! Blockchain test transaction deserialization.

use crate::{bytes::Bytes, uint::Uint};
use common::{H160, H256};

/// Blockchain test transaction deserialization.
#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    #[serde(rename = "type")]
    pub transaction_type: Option<Uint>,
    pub data: Bytes,
    pub gas_limit: Uint,
    pub gas_price: Option<Uint>,
    pub nonce: Uint,
    pub r: Uint,
    pub s: Uint,
    pub v: Uint,
    pub value: Uint,
    pub chain_id: Option<Uint>,
    pub access_list: Option<AccessList>,
    pub max_fee_per_gas: Option<Uint>,
    pub max_priority_fee_per_gas: Option<Uint>,
    pub hash: Option<H256>,
}

#[derive(Debug, PartialEq, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccessListItem {
    pub address: H160,
    pub storage_keys: Vec<H256>,
}

pub type AccessList = Vec<AccessListItem>;
