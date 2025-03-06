use crate::sketchbook::JsonSerde;
use serde::{Deserialize, Serialize};

/// Structure for receiving data about network sampling details from the frontend.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SamplingData {
    pub count: usize,
    pub seed: Option<u64>,
    pub path: String,
}

impl JsonSerde<'_> for SamplingData {}

impl SamplingData {
    /// Create new `SamplingData` object given all its fields.
    pub fn new(count: usize, seed: Option<u64>, path: &str) -> SamplingData {
        SamplingData {
            count,
            seed,
            path: path.to_string(),
        }
    }
}
