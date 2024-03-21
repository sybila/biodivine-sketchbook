use crate::sketchbook::observations::Dataset;
use crate::sketchbook::DatasetId;
use std::collections::HashMap;
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// **(internal)** Basic utility methods for `ObservationManager`.
mod _impl_manager;
/// **(internal)** Implementation of [Serialize] and [Deserialize] traits for `ObservationManager`.
mod _impl_serde;
/// **(internal)** Implementation of event-based API for the [SessionState] trait.
mod _impl_session_state;

#[derive(Clone, Debug, PartialEq)]
pub struct ObservationManager {
    datasets: HashMap<DatasetId, Dataset>,
}

impl Default for ObservationManager {
    /// Default manager instance with no datasets.
    fn default() -> ObservationManager {
        ObservationManager::new_empty()
    }
}

impl Display for ObservationManager {
    /// Use json serialization to convert `ObservationManager` to string.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl FromStr for ObservationManager {
    type Err = String;

    /// Use json de-serialization to construct `ObservationManager` from string.
    fn from_str(s: &str) -> Result<ObservationManager, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}
