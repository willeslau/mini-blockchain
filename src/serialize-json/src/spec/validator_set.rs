//! Validator set deserialization.

use crate::{hash::Address, uint::Uint};
use std::collections::BTreeMap;

/// Different ways of specifying validators.
#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub enum ValidatorSet {
    /// A simple list of authorities.
    List(Vec<Address>),
    /// Address of a contract that indicates the list of authorities.
    SafeContract(Address),
    /// Address of a contract that indicates the list of authorities and enables reporting of theor misbehaviour using transactions.
    Contract(Address),
    /// A map of starting blocks for each validator set.
    Multi(BTreeMap<Uint, ValidatorSet>),
}

#[cfg(test)]
mod tests {
    use crate::{hash::Address, spec::validator_set::ValidatorSet, uint::Uint};
    use common::{H160, U256};
    use serde_json;
    use std::str::FromStr;

    #[test]
    fn validator_set_deserialization() {
        let s = r#"[{
			"list": ["0xc6d9d2cd449a754c494264e1809c50e34d64562b"]
		}, {
			"safeContract": "0xc6d9d2cd449a754c494264e1809c50e34d64562b"
		}, {
			"contract": "0xc6d9d2cd449a754c494264e1809c50e34d64562b"
		}, {
			"multi": {
				"0": { "list": ["0xc6d9d2cd449a754c494264e1809c50e34d64562b"] },
				"10": { "list": ["0xd6d9d2cd449a754c494264e1809c50e34d64562b"] },
				"20": { "contract": "0xc6d9d2cd449a754c494264e1809c50e34d64562b" }
			}
		}]"#;

        let deserialized: Vec<ValidatorSet> = serde_json::from_str(s).unwrap();
        assert_eq!(deserialized.len(), 4);

        assert_eq!(
            deserialized[0],
            ValidatorSet::List(vec![Address(
                H160::from_str("c6d9d2cd449a754c494264e1809c50e34d64562b").unwrap()
            )])
        );
        assert_eq!(
            deserialized[1],
            ValidatorSet::SafeContract(Address(
                H160::from_str("c6d9d2cd449a754c494264e1809c50e34d64562b").unwrap()
            ))
        );
        assert_eq!(
            deserialized[2],
            ValidatorSet::Contract(Address(
                H160::from_str("c6d9d2cd449a754c494264e1809c50e34d64562b").unwrap()
            ))
        );
        match deserialized[3] {
            ValidatorSet::Multi(ref map) => {
                assert_eq!(map.len(), 3);
                assert!(map.contains_key(&Uint(U256::from(0))));
                assert!(map.contains_key(&Uint(U256::from(10))));
                assert!(map.contains_key(&Uint(U256::from(20))));
            }
            _ => assert!(false),
        }
    }
}
