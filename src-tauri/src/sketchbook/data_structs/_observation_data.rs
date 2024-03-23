use crate::sketchbook::observations::Observation;
use crate::sketchbook::DatasetId;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// Structure for sending data about `Observation` to the frontend.
///
/// Contains also ID of the corresponding dataset. Some fields are further simplified compared to
/// original typesafe versions (e.g., pure `Strings` are used instead of more complex typesafe
/// structs) to allow for easier (de)serialization.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ObservationData {
    pub id: String,
    pub dataset: String,
    pub values: String,
}

impl ObservationData {
    /// Create new `ObservationData` object given `id` and values string slices.
    pub fn new(obs_id: &str, dataset_id: &str, values: &str) -> ObservationData {
        ObservationData {
            id: obs_id.to_string(),
            dataset: dataset_id.to_string(),
            values: values.to_string(),
        }
    }

    /// Create new `ObservationData` object given a reference to a observation, and ID of
    /// its dataset.
    pub fn from_obs(obs: &Observation, dataset_id: &DatasetId) -> ObservationData {
        ObservationData::new(
            obs.get_id().as_str(),
            dataset_id.as_str(),
            &obs.to_values_string(),
        )
    }

    /// Extract the corresponding `Observation` from the `ObservationData`.
    /// There is a syntax check just to make sure that the data are valid.
    pub fn to_observation(&self) -> Result<Observation, String> {
        Observation::try_from_str(self.values.clone(), &self.id)
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
