//! Vm environment.
use crate::{hash::Address, uint::Uint};

/// Vm environment.
#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct Env {
    /// Address.
    #[serde(rename = "currentCoinbase")]
    pub author: Address,
    /// Difficulty
    #[serde(rename = "currentDifficulty")]
    pub difficulty: Uint,
    /// Gas limit.
    #[serde(rename = "currentGasLimit")]
    pub gas_limit: Uint,
    /// Number.
    #[serde(rename = "currentNumber")]
    pub number: Uint,
    /// Timestamp.
    #[serde(rename = "currentTimestamp")]
    pub timestamp: Uint,
    /// Block base fee.
    #[serde(rename = "currentBaseFee")]
    pub base_fee: Option<Uint>,
}

#[cfg(test)]
mod tests {
    use super::Env;
    use serde_json;

    #[test]
    fn env_deserialization() {
        let s = r#"{
			"currentCoinbase" : "2adc25665018aa1fe0e6bc666dac8fc2697ff9ba",
			"currentDifficulty" : "0x0100",
			"currentGasLimit" : "0x0f4240",
			"currentNumber" : "0x00",
			"currentTimestamp" : "0x01"
		}"#;
        let _deserialized: Env = serde_json::from_str(s).unwrap();
        // TODO: validate all fields
    }
}
