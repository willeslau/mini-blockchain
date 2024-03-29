//! Executed transaction.
use crate::{bytes::Bytes, hash::Address, uint::Uint};

/// Executed transaction.
#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    /// Contract address.
    pub address: Address,
    /// Transaction sender.
    #[serde(rename = "caller")]
    pub sender: Address,
    /// Contract code.
    pub code: Bytes,
    /// Input data.
    pub data: Bytes,
    /// Gas.
    pub gas: Uint,
    /// Gas price.
    pub gas_price: Uint,
    /// Transaction origin.
    pub origin: Address,
    /// Sent value.
    pub value: Uint,
}

#[cfg(test)]
mod tests {
    use super::Transaction;
    use serde_json;

    #[test]
    fn transaction_deserialization() {
        let s = r#"{
			"address" : "0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6",
			"caller" : "cd1722f2947def4cf144679da39c4c32bdc35681",
			"code" : "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01600055",
			"data" : "0x",
			"gas" : "0x0186a0",
			"gasPrice" : "0x5af3107a4000",
			"origin" : "cd1722f2947def4cf144679da39c4c32bdc35681",
			"value" : "0x0de0b6b3a7640000"
		}"#;
        let _deserialized: Transaction = serde_json::from_str(s).unwrap();
    }
}
