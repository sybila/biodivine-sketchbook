use crate::sketchbook::{Essentiality, Monotonicity};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// Structure for receiving data about changes in monotonicity of uninterpreted fn's argument from the frontend.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChangeArgMonotoneData {
    pub idx: usize,
    pub monotonicity: Monotonicity,
}

/// Structure for receiving data about changes in essentiality of uninterpreted fn's argument from the frontend.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChangeArgEssentialData {
    pub idx: usize,
    pub essentiality: Essentiality,
}

impl ChangeArgMonotoneData {
    /// Create new `ChangeArgMonotoneData` object given the arguments index and its new monotonicity.
    pub fn new(idx: usize, monotonicity: Monotonicity) -> ChangeArgMonotoneData {
        ChangeArgMonotoneData { idx, monotonicity }
    }
}

impl Display for ChangeArgMonotoneData {
    /// Use json serialization to convert `ChangeArgMonotoneData` to string.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl FromStr for ChangeArgMonotoneData {
    type Err = String;

    /// Use json de-serialization to construct `ChangeArgMonotoneData` from string.
    fn from_str(s: &str) -> Result<ChangeArgMonotoneData, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}

impl ChangeArgEssentialData {
    /// Create new `ChangeArgEssentialData` object given the arguments index and its new essentiality.
    pub fn new(idx: usize, essentiality: Essentiality) -> ChangeArgEssentialData {
        ChangeArgEssentialData { idx, essentiality }
    }
}

impl Display for ChangeArgEssentialData {
    /// Use json serialization to convert `ChangeArgEssentialData` to string.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl FromStr for ChangeArgEssentialData {
    type Err = String;

    /// Use json de-serialization to construct `ChangeArgEssentialData` from string.
    fn from_str(s: &str) -> Result<ChangeArgEssentialData, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}
