//! Null engine params deserialization.

use crate::uint::Uint;

/// Authority params deserialization.
#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct NullEngineParams {
    /// Block reward.
    pub block_reward: Option<Uint>,
    /// Immediate finalization.
    pub immediate_finalization: Option<bool>,
}

/// Null engine descriptor
#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NullEngine {
    /// Ethash params.
    pub params: NullEngineParams,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uint::Uint;
    use common::U256;
    use serde_json;

    #[test]
    fn null_engine_deserialization() {
        let s = r#"{
			"params": {
				"blockReward": "0x0d"
			}
		}"#;

        let deserialized: NullEngine = serde_json::from_str(s).unwrap();
        assert_eq!(
            deserialized.params.block_reward,
            Some(Uint(U256::from(0x0d)))
        );
    }
}
