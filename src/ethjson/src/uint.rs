//! Lenient uint json deserialization for test json files.

use serde::{
    de::{Error, Unexpected, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{fmt, str::FromStr};
use common::U256;

/// Lenient uint json deserialization for test json files.
#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Uint(pub U256);

impl Into<U256> for Uint {
    fn into(self) -> U256 {
        self.0
    }
}

impl Into<u64> for Uint {
    fn into(self) -> u64 {
        self.0.low_u64()
    }
}

impl Into<usize> for Uint {
    fn into(self) -> usize {
        self.0.low_u64() as usize
    }
}
impl Into<u8> for Uint {
    fn into(self) -> u8 {
        self.0.low_u64() as u8
    }
}

impl Serialize for Uint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.to_string().serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for Uint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        deserializer.deserialize_any(UintVisitor)
    }
}

struct UintVisitor;

impl<'a> Visitor<'a> for UintVisitor {
    type Value = Uint;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a hex encoded or decimal uint")
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Uint(U256::from(value)))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let value = match value.len() {
            0 => U256::from(0),
            2 if value.starts_with("0x") => U256::from(0),
            _ if value.starts_with("0x") => U256::from_str(&value[2..]).map_err(|e| {
                Error::custom(format!("Invalid hex value {}: {}", value, e).as_str())
            })?,
            _ => U256::from_dec_str(value).map_err(|e| {
                Error::custom(format!("Invalid decimal value {}: {:?}", value, e).as_str())
            })?,
        };

        Ok(Uint(value))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_str(value.as_ref())
    }
}

/// Deserialize and validate that the value is non-zero
pub fn validate_non_zero<'de, D>(d: D) -> Result<Uint, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Uint::deserialize(d)?;

    if value == Uint(U256::from(0)) {
        return Err(Error::invalid_value(
            Unexpected::Unsigned(value.into()),
            &"a non-zero value",
        ));
    }

    Ok(value)
}

/// Deserialize and validate that the value is non-zero
pub fn validate_optional_non_zero<'de, D>(d: D) -> Result<Option<Uint>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<Uint> = Option::deserialize(d)?;

    if let Some(value) = value {
        if value == Uint(U256::from(0)) {
            return Err(Error::invalid_value(
                Unexpected::Unsigned(value.into()),
                &"a non-zero value",
            ));
        }
    }

    Ok(value)
}

#[cfg(test)]
mod test {
    use crate::uint::Uint;
    use common::U256;
    use serde_json;

    #[test]
    fn uint_deserialization() {
        let s = r#"["0xa", "10", "", "0x", 0]"#;
        let deserialized: Vec<Uint> = serde_json::from_str(s).unwrap();
        assert_eq!(
            deserialized,
            vec![
                Uint(U256::from(10)),
                Uint(U256::from(10)),
                Uint(U256::from(0)),
                Uint(U256::from(0)),
                Uint(U256::from(0))
            ]
        );
    }

    #[test]
    fn uint_into() {
        assert_eq!(U256::from(10), Uint(U256::from(10)).into());
    }
}
