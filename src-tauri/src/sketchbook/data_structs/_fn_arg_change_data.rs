use crate::sketchbook::model::{Essentiality, Monotonicity};
use crate::sketchbook::JsonSerde;
use serde::{Deserialize, Serialize};

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

impl JsonSerde<'_> for ChangeArgMonotoneData {}
impl JsonSerde<'_> for ChangeArgEssentialData {}

impl ChangeArgMonotoneData {
    /// Create new `ChangeArgMonotoneData` object given the arguments index and its new monotonicity.
    pub fn new(idx: usize, monotonicity: Monotonicity) -> ChangeArgMonotoneData {
        ChangeArgMonotoneData { idx, monotonicity }
    }
}

impl ChangeArgEssentialData {
    /// Create new `ChangeArgEssentialData` object given the arguments index and its new essentiality.
    pub fn new(idx: usize, essentiality: Essentiality) -> ChangeArgEssentialData {
        ChangeArgEssentialData { idx, essentiality }
    }
}
