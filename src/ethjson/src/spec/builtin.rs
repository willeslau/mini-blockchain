//! Spec builtin deserialization.

use crate::uint::Uint;
use serde::Deserialize;
use std::collections::BTreeMap;

/// Linear pricing.
#[derive(Debug, PartialEq, serde::Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Linear {
    /// Base price.
    pub base: u64,
    /// Price for word.
    pub word: u64,
}

/// Pricing for modular exponentiation.
#[derive(Debug, PartialEq, serde::Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Modexp {
    /// Price divisor.
    pub divisor: u64,
}

/// Pricing for EIP2565 modular exponentiation.
#[derive(Debug, PartialEq, serde::Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Modexp2565 {}

/// Pricing for constant alt_bn128 operations (ECADD and ECMUL)
#[derive(Debug, PartialEq, serde::Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AltBn128ConstOperations {
    /// price
    pub price: u64,
}

/// Pricing for alt_bn128_pairing.
#[derive(Debug, PartialEq, serde::Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AltBn128Pairing {
    /// Base price.
    pub base: u64,
    /// Price per point pair.
    pub pair: u64,
}

/// Bls12 pairing price
#[derive(Debug, PartialEq, serde::Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Bls12Pairing {
    /// Price per final exp
    pub base: u64,
    /// Price per pair (Miller loop)
    pub pair: u64,
}

/// Pricing for constant Bls12 operations (ADD and MUL in G1 and G2, as well as mappings)
#[derive(Debug, PartialEq, serde::Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Bls12ConstOperations {
    /// Fixed price.
    pub price: u64,
}

/// Pricing for constant Bls12 operations (ADD and MUL in G1, as well as mappings)
#[derive(Debug, PartialEq, serde::Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Bls12G1Multiexp {
    /// Base const of the operation (G1 or G2 multiplication)
    pub base: u64,
}

/// Pricing for constant Bls12 operations (ADD and MUL in G2, as well as mappings)
#[derive(Debug, PartialEq, serde::Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Bls12G2Multiexp {
    /// Base const of the operation (G1 or G2 multiplication)
    pub base: u64,
}

/// Pricing variants.
#[derive(Debug, PartialEq, serde::Deserialize, Clone)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
pub enum Pricing {
    /// Pricing for Blake2 compression function: each call costs the same amount per round.
    Blake2F {
        /// Price per round of Blake2 compression function.
        gas_per_round: u64,
    },
    /// Linear pricing.
    Linear(Linear),
    /// Pricing for EIP198 modular exponentiation.
    Modexp(Modexp),
    /// Pricing for EIP2565 modular exponentiation.
    Modexp2565(Modexp2565),
    /// Pricing for alt_bn128_pairing exponentiation.
    AltBn128Pairing(AltBn128Pairing),
    /// Pricing for constant alt_bn128 operations
    AltBn128ConstOperations(AltBn128ConstOperations),
    /// Pricing of constant price bls12_381 operations
    Bls12ConstOperations(Bls12ConstOperations),
    /// Pricing of pairing bls12_381 operation
    Bls12Pairing(Bls12Pairing),
    /// Pricing of bls12_381 multiexp operations in G1
    Bls12G1Multiexp(Bls12G1Multiexp),
    /// Pricing of bls12_381 multiexp operations in G2
    Bls12G2Multiexp(Bls12G2Multiexp),
}

/// Builtin compability layer
#[derive(Debug, PartialEq, serde::Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct BuiltinCompat {
    /// Builtin name.
    name: String,
    /// Builtin pricing.
    pricing: PricingCompat,
    /// Activation block.
    activate_at: Option<Uint>,
}

/// Spec builtin.
#[derive(Debug, PartialEq, Clone)]
pub struct Builtin {
    /// Builtin name.
    pub name: String,
    /// Builtin pricing.
    pub pricing: BTreeMap<u64, PricingAt>,
}

impl From<BuiltinCompat> for Builtin {
    fn from(legacy: BuiltinCompat) -> Self {
        let pricing = match legacy.pricing {
            PricingCompat::Single(pricing) => {
                let mut map = BTreeMap::new();
                let activate_at: u64 = legacy.activate_at.map_or(0, Into::into);
                map.insert(
                    activate_at,
                    PricingAt {
                        info: None,
                        price: pricing,
                    },
                );
                map
            }
            PricingCompat::Multi(pricings) => {
                pricings.into_iter().map(|(a, p)| (a.into(), p)).collect()
            }
        };
        Self {
            name: legacy.name,
            pricing,
        }
    }
}

/// Compability layer for different pricings
#[derive(Debug, PartialEq, serde::Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
enum PricingCompat {
    /// Single builtin
    Single(Pricing),
    /// Multiple builtins
    Multi(BTreeMap<Uint, PricingAt>),
}

/// Price for a builtin, with the block number to activate it on
#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct PricingAt {
    /// Description of the activation, e.g. "PunyPony HF, March 12, 2025".
    pub info: Option<String>,
    /// Builtin pricing.
    pub price: Pricing,
}

#[cfg(test)]
mod tests {
    use super::{ Builtin, BuiltinCompat };
    use serde_json;

    #[test]
    fn builtin_deserialization() {
        let s = r#"{
			"name": "ecrecover",
			"pricing": { "linear": { "base": 3000, "word": 0 } }
		}"#;
        let builtin: Builtin = serde_json::from_str::<BuiltinCompat>(s).unwrap().into();
        assert_eq!(builtin.name, "ecrecover");
    }

    #[test]
    fn deserialize_multiple_pricings() {
        let s = r#"{
			"name": "ecrecover",
			"pricing": {
				"0": {
					"price": {"linear": { "base": 3000, "word": 0 }}
				},
				"500": {
					"info": "enable fake EIP at block 500",
					"price": {"linear": { "base": 10, "word": 0 }}
				}
			}
		}"#;
        let builtin: Builtin = serde_json::from_str::<BuiltinCompat>(s).unwrap().into();
        assert_eq!(builtin.name, "ecrecover");
    }

    #[test]
    fn deserialization_blake2_f_builtin() {
        let s = r#"{
			"name": "blake2_f",
			"activate_at": "0xffffff",
			"pricing": { "blake2_f": { "gas_per_round": 123 } }
		}"#;
        let builtin: Builtin = serde_json::from_str::<BuiltinCompat>(s).unwrap().into();
        assert_eq!(builtin.name, "blake2_f");
    }

    #[test]
    fn deserialization_alt_bn128_const_operations() {
        let s = r#"{
			"name": "alt_bn128_mul",
			"pricing": {
				"100500": {
					"price": { "alt_bn128_const_operations": { "price": 123 }}
				}
			}
		}"#;
        let builtin: Builtin = serde_json::from_str::<BuiltinCompat>(s).unwrap().into();
        assert_eq!(builtin.name, "alt_bn128_mul");
    }

    #[test]
    fn activate_at() {
        let s = r#"{
			"name": "late_start",
			"activate_at": 100000,
			"pricing": { "modexp": { "divisor": 5 } }
		}"#;

        let builtin: Builtin = serde_json::from_str::<BuiltinCompat>(s).unwrap().into();
        assert_eq!(builtin.name, "late_start");
    }

    #[test]
    fn deserialization_bls12_381_multiexp_operation() {
        let s = r#"{
			"name": "bls12_381_g1_multiexp",
			"pricing": {
				"10000000": {
					"price": { "bls12_g1_multiexp": { "base": 12000}}
				}
			}
		}"#;
        let builtin: Builtin = serde_json::from_str::<BuiltinCompat>(s).unwrap().into();
        assert_eq!(builtin.name, "bls12_381_g1_multiexp");
        // assert_eq!(
        //     builtin.pricing,
        //     btreemap![
        //         10000000 => PricingAt {
        //             info: None,
        //             price: Pricing::Bls12G1Multiexp(Bls12G1Multiexp{
        //                     base: 12000
        //             }),
        //         }
        //     ]
        // );
    }

    #[test]
    fn deserialization_bls12_381_multiexp_operation_in_g2() {
        let s = r#"{
			"name": "bls12_381_g2_multiexp",
			"pricing": {
				"10000000": {
					"price": { "bls12_g2_multiexp": { "base": 55000}}
				}
			}
		}"#;
        let builtin: Builtin = serde_json::from_str::<BuiltinCompat>(s).unwrap().into();
        assert_eq!(builtin.name, "bls12_381_g2_multiexp");
        // assert_eq!(
        //     builtin.pricing,
        //     btreemap![
        //         10000000 => PricingAt {
        //             info: None,
        //             price: Pricing::Bls12G2Multiexp(Bls12G2Multiexp{
        //                     base: 55000
        //             }),
        //         }
        //     ]
        // );
    }

    #[test]
    fn deserialization_modexp2565() {
        let s = r#"{
			"name": "modexp",
			"pricing": {
				"10000000": {
					"price": { "modexp2565": {  } }
				}
			}
		}"#;
        let builtin: Builtin = serde_json::from_str::<BuiltinCompat>(s).unwrap().into();
        assert_eq!(builtin.name, "modexp");
        // assert_eq!(
        //     builtin.pricing,
        //     btreemap![
        //         10000000 => PricingAt {
        //             info: None,
        //             price: Pricing::Modexp2565(Modexp2565{}),
        //         }
        //     ]
        // );
    }
}
