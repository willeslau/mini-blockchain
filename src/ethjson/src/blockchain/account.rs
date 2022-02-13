//! Blockchain test account deserializer.

use crate::{bytes::Bytes, uint::Uint};
use std::collections::BTreeMap;

/// Blockchain test account deserializer.
#[derive(Debug, PartialEq, serde::Deserialize, Clone)]
pub struct Account {
    /// Balance.
    pub balance: Uint,
    /// Code.
    pub code: Bytes,
    /// Nonce.
    pub nonce: Uint,
    /// Storage.
    pub storage: BTreeMap<Uint, Uint>,
}

#[cfg(test)]
mod tests {
    use crate::blockchain::account::Account;
    use serde_json;

    #[test]
    fn account_deserialization() {
        let s = r#"{
			"balance" : "0x09184e72a078",
			"code" : "0x600140600155",
			"nonce" : "0x00",
			"storage" : {
				"0x01" : "0x9a10c2b5bb8f3c602e674006d9b21f09167df57c87a78a5ce96d4159ecb76520"
			}
		}"#;
        let _deserialized: Account = serde_json::from_str(s).unwrap();
        // TODO: validate all fields
    }
}
