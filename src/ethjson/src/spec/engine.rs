//! Engine deserialization.

use super::{AuthorityRound, BasicAuthority, Clique, Ethash, InstantSeal, NullEngine};

/// Engine deserialization.
#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub enum Engine {
    /// Null engine.
    Null(NullEngine),
    /// Instantly sealing engine.
    InstantSeal(Option<InstantSeal>),
    /// Ethash engine.
    #[serde(rename = "Ethash")]
    Ethash(Ethash),
    /// BasicAuthority engine.
    BasicAuthority(BasicAuthority),
    /// AuthorityRound engine.
    AuthorityRound(AuthorityRound),
    /// Clique engine.
    Clique(Clique),
}

#[cfg(test)]
mod tests {
    use crate::spec::Engine;
    use serde_json;

    #[test]
    fn engine_deserialization() {
        let s = r#"{
			"null": {
				"params": {
					"blockReward": "0x0d"
				}
			}
		}"#;

        let deserialized: Engine = serde_json::from_str(s).unwrap();
        match deserialized {
            Engine::Null(_) => {} // unit test in its own file.
            _ => panic!(),
        }

        let s = r#"{
			"instantSeal": {"params": {}}
		}"#;

        let deserialized: Engine = serde_json::from_str(s).unwrap();
        match deserialized {
            Engine::InstantSeal(_) => {} // instant seal is unit tested in its own file.
            _ => panic!(),
        };

        let s = r#"{
			"instantSeal": null
		}"#;

        let deserialized: Engine = serde_json::from_str(s).unwrap();
        match deserialized {
            Engine::InstantSeal(_) => {} // instant seal is unit tested in its own file.
            _ => panic!(),
        };

        let s = r#"{
			"Ethash": {
				"params": {
					"minimumDifficulty": "0x020000",
					"difficultyBoundDivisor": "0x0800",
					"durationLimit": "0x0d",
					"homesteadTransition" : "0x",
					"daoHardforkTransition": "0xffffffffffffffff",
					"daoHardforkBeneficiary": "0x0000000000000000000000000000000000000000",
					"daoHardforkAccounts": []
				}
			}
		}"#;

        let deserialized: Engine = serde_json::from_str(s).unwrap();
        match deserialized {
            Engine::Ethash(_) => {} // ethash is unit tested in its own file.
            _ => panic!(),
        };

        let s = r#"{
			"basicAuthority": {
				"params": {
					"durationLimit": "0x0d",
					"validators" : {
						"list": ["0xc6d9d2cd449a754c494264e1809c50e34d64562b"]
					}
				}
			}
		}"#;
        let deserialized: Engine = serde_json::from_str(s).unwrap();
        match deserialized {
            Engine::BasicAuthority(_) => {} // basicAuthority is unit tested in its own file.
            _ => panic!(),
        };

        let s = r#"{
			"authorityRound": {
				"params": {
					"stepDuration": "0x02",
					"validators": {
						"list" : ["0xc6d9d2cd449a754c494264e1809c50e34d64562b"]
					},
					"startStep" : 24,
					"validateStepTransition": 150
				}
			}
		}"#;
        let deserialized: Engine = serde_json::from_str(s).unwrap();
        match deserialized {
            Engine::AuthorityRound(_) => {} // AuthorityRound is unit tested in its own file.
            _ => panic!(),
        };

        let s = r#"{
			"clique": {
				"params": {
					"period": 15,
					"epoch": 30000
				}
			}
		}"#;
        let deserialized: Engine = serde_json::from_str(s).unwrap();
        match deserialized {
            Engine::Clique(_) => {} // Clique is unit tested in its own file.
            _ => panic!(),
        };
    }
}
