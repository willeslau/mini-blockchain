//! State test transaction deserialization.

use crate::{
    bytes::Bytes,
    hash::{Address, H256},
    maybe::MaybeEmpty,
    uint::Uint,
};

/// State test transaction deserialization.
#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    /// Transaction data.
    pub data: Bytes,
    /// Gas limit.
    pub gas_limit: Uint,
    /// Gas price.
    pub gas_price: Option<Uint>,
    /// Nonce.
    pub nonce: Uint,
    /// Secret key.
    #[serde(rename = "secretKey")]
    pub secret: Option<H256>,
    /// To.
    pub to: MaybeEmpty<Address>,
    /// Value.
    pub value: Uint,
    /// Max fee per gas.
    pub max_fee_per_gas: Option<Uint>,
    /// Max priority fee per gas.
    pub max_priority_fee_per_gas: Option<Uint>,
}

#[cfg(test)]
mod tests {
    use super::Transaction;
    use serde_json;

    #[test]
    fn transaction_deserialization() {
        let s = r#"{
			"data" : "",
			"accessLists": null,
			"gasLimit" : "0x2dc6c0",
			"gasPrice" : "0x01",
			"nonce" : "0x00",
			"secretKey" : "45a915e4d060149eb4365960e6a7a45f334393093061116b197e3240065ff2d8",
			"to" : "1000000000000000000000000000000000000000",
			"value" : "0x00"
		}"#;
        let _deserialized: Transaction = serde_json::from_str(s).unwrap();
        // TODO: validate all fields
    }
}
