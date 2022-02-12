//! Lenient hash json deserialization for test json files.

use common::{H160 as Hash160, H256 as Hash256, H520 as Hash520, H64 as Hash64};
use serde::{
    de::{Error, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use ethbloom::Bloom as EthBloom;
use std::{fmt, str::FromStr};

macro_rules! impl_hash {
    ($name: ident, $inner: ident) => {
        /// Lenient hash json deserialization for test json files.
        #[derive(Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone)]
        pub struct $name(pub $inner);

        impl From<$name> for $inner {
            fn from(other: $name) -> $inner {
                other.0
            }
        }

        impl From<$inner> for $name {
            fn from(i: $inner) -> Self {
                $name(i)
            }
        }

        impl<'a> Deserialize<'a> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'a>,
            {
                struct HashVisitor;

                impl<'b> Visitor<'b> for HashVisitor {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        write!(formatter, "a 0x-prefixed hex-encoded hash")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: Error,
                    {
                        let value = match value.len() {
                            0 => $inner::from_low_u64_be(0),
                            2 if value == "0x" => $inner::from_low_u64_be(0),
                            _ if value.starts_with("0x") => {
                                $inner::from_str(&value[2..]).map_err(|e| {
                                    Error::custom(
                                        format!("Invalid hex value {}: {}", value, e).as_str(),
                                    )
                                })?
                            }
                            _ => $inner::from_str(value).map_err(|e| {
                                Error::custom(
                                    format!("Invalid hex value {}: {}", value, e).as_str(),
                                )
                            })?,
                        };

                        Ok($name(value))
                    }

                    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
                    where
                        E: Error,
                    {
                        self.visit_str(value.as_ref())
                    }
                }

                deserializer.deserialize_any(HashVisitor)
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&format!("{:#x}", self.0))
            }
        }
    };
}

impl_hash!(H64, Hash64);
impl_hash!(Address, Hash160);
impl_hash!(H256, Hash256);
impl_hash!(H520, Hash520);
impl_hash!(Bloom, EthBloom);

#[cfg(test)]
mod test {
    use crate::hash::H256;
    use serde_json;
    use std::str::FromStr;

    #[test]
    fn hash_deserialization() {
        let s = r#"["", "5a39ed1020c04d4d84539975b893a4e7c53eab6c2965db8bc3468093a31bc5ae"]"#;
        let deserialized: Vec<H256> = serde_json::from_str(s).unwrap();
        assert_eq!(
            deserialized,
            vec![
                H256(common::H256::zero()),
                H256(
                    common::H256::from_str(
                        "5a39ed1020c04d4d84539975b893a4e7c53eab6c2965db8bc3468093a31bc5ae"
                    )
                    .unwrap()
                )
            ]
        );
    }

    // #[test]
    // fn hash_into() {
    //     assert_eq!(
    //         common::H256::zero(),
    //         H256(common::H256::zero()).into()
    //     );
    // }
}
