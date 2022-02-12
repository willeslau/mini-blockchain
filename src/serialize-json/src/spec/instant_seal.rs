//! Instant seal engine params deserialization.

/// Instant seal engine params deserialization.
#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct InstantSealParams {
    /// Whether to enable millisecond timestamp.
    #[serde(default)]
    pub millisecond_timestamp: bool,
}

/// Instant seal engine descriptor.
#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstantSeal {
    /// Instant seal parameters.
    pub params: InstantSealParams,
}
