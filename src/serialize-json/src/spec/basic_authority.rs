//! Authority params deserialization.

use super::ValidatorSet;
use crate::uint::Uint;

/// Authority params deserialization.
#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct BasicAuthorityParams {
    /// Block duration.
    pub duration_limit: Uint,
    /// Valid authorities
    pub validators: ValidatorSet,
}

/// Authority engine deserialization.
#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BasicAuthority {
    /// Ethash params.
    pub params: BasicAuthorityParams,
}

#[cfg(test)]
mod tests {
    use crate::{
        hash::Address,
        spec::{basic_authority::BasicAuthority, validator_set::ValidatorSet},
        uint::Uint,
    };
    use common::{H160, U256};
    use serde_json;
    use std::str::FromStr;

    #[test]
    fn basic_authority_deserialization() {
        let s = r#"{
			"params": {
				"durationLimit": "0x0d",
				"validators" : {
					"list": ["0xc6d9d2cd449a754c494264e1809c50e34d64562b"]
				}
			}
		}"#;

        let deserialized: BasicAuthority = serde_json::from_str(s).unwrap();

        assert_eq!(deserialized.params.duration_limit, Uint(U256::from(0x0d)));
        let vs = ValidatorSet::List(vec![Address(
            H160::from_str("c6d9d2cd449a754c494264e1809c50e34d64562b").unwrap(),
        )]);
        assert_eq!(deserialized.params.validators, vs);
    }
}
