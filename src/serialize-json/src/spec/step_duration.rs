//! Step duration configuration parameter

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::uint::Uint;

/// Step duration can be specified either as a `Uint` (in seconds), in which case it will be
/// constant, or as a list of pairs consisting of a timestamp of type `Uint` and a duration, in
/// which case the duration of a step will be determined by a mapping arising from that list.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum StepDuration {
    /// Duration of all steps.
    Single(Uint),
    /// Step duration transitions: a mapping of timestamp to step durations.
    Transitions(BTreeMap<Uint, Uint>),
}
