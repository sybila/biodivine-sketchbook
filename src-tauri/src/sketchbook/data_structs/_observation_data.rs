use crate::sketchbook::observations::Observation;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// Structure for sending data about `Observation` to the frontend.
///
/// Some fields simplified compared to original typesafe versions (e.g., pure `Strings` are used
/// instead of more complex typesafe structs) to allow for easier (de)serialization.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ObservationData {
    pub id: String,
    pub values: String,
}

impl ObservationData {
    /// Create new `ObservationData` object given `id` and values string slices.
    pub fn new(id: &str, values: &str) -> ObservationData {
        ObservationData {
            id: id.to_string(),
            values: values.to_string(),
        }
    }

    /// Create new `ObservationData` object given a reference to a observation.
    pub fn from_obs(obs: &Observation) -> ObservationData {
        ObservationData::new(obs.get_id().as_str(), &obs.to_values_string())
    }
}

impl Display for ObservationData {
    /// Use json serialization to convert `ObservationData` to string.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl FromStr for ObservationData {
    type Err = String;

    /// Use json de-serialization to construct `ObservationData` from string.
    fn from_str(s: &str) -> Result<ObservationData, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}
