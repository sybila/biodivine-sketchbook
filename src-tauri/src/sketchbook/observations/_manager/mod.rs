use crate::sketchbook::ids::DatasetId;
use crate::sketchbook::observations::Dataset;
use crate::sketchbook::{JsonSerde, Manager};
use std::collections::HashMap;
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// **(internal)** Implementation of the safe identifier generating.
mod _impl_id_generating;
/// **(internal)** Functionality for loading datasets from file.
mod _impl_load_dataset;
/// **(internal)** Basic utility methods for `ObservationManager`.
mod _impl_manager;
/// **(internal)** Implementation of [Serialize] and [Deserialize] traits for `ObservationManager`.
mod _impl_serde;
/// **(internal)** Implementation of event-based API for the [SessionState] trait.
mod _impl_session_state;

/// Class to manage all observations and datasets.
///
/// `ObservationManager` can be managed through its classical Rust API, as well as
/// through the external events (as it implements the `SessionState` event).
#[derive(Clone, Debug, PartialEq)]
pub struct ObservationManager {
    datasets: HashMap<DatasetId, Dataset>,
}

impl<'de> JsonSerde<'de> for ObservationManager {}

impl Manager for ObservationManager {}

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
