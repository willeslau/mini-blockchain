//! Transaction test transaction deserialization.
use serde::Deserialize;
use crate::{bytes::Bytes, hash::Address, maybe::MaybeEmpty, uint::Uint};

/// Transaction test transaction deserialization.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    /// Transaction data.
    pub data: Bytes,
    /// Gas limit.
    pub gas_limit: Uint,
    /// Gas price.
    pub gas_price: Uint,
    /// Nonce.
    pub nonce: Uint,
    /// To.
    pub to: MaybeEmpty<Address>,
    /// Value.
    pub value: Uint,
    /// R.
    pub r: Uint,
    /// S.
    pub s: Uint,
    /// V.
    pub v: Uint,
}

#[cfg(test)]
mod tests {
    use super::Transaction;
    use serde_json;

    #[test]
    fn transaction_deserialization() {
        let s = r#"{
			"data" : "0x",
			"gasLimit" : "0xf388",
			"gasPrice" : "0x09184e72a000",
			"nonce" : "0x00",
			"r" : "0x2c",
			"s" : "0x04",
			"to" : "",
			"v" : "0x1b",
			"value" : "0x00"
		}"#;
        let _deserialized: Transaction = serde_json::from_str(s).unwrap();
        // TODO: validate all fields
    }
}
