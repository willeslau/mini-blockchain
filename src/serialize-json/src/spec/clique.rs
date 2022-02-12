//! Clique params deserialization.

use std::num::NonZeroU64;

/// Clique params deserialization.
#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct CliqueParams {
    /// period as defined in EIP
    pub period: Option<u64>,
    /// epoch length as defined in EIP
    pub epoch: Option<NonZeroU64>,
}

/// Clique engine deserialization.
#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct Clique {
    /// CliqueEngine params
    pub params: CliqueParams,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn clique_deserialization() {
        let s = r#"{
			"params": {
				"period": 5,
				"epoch": 30000
			}
		}"#;

        let deserialized: Clique = serde_json::from_str(s).unwrap();
        assert_eq!(deserialized.params.period, Some(5u64));
        assert_eq!(deserialized.params.epoch, NonZeroU64::new(30000));
    }
}
