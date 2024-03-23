use crate::sketchbook::observations::{Observation, ObservationType};
use crate::sketchbook::ObservationId;
use std::collections::HashMap;
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// **(internal)** Basic utility methods for `Dataset`.
mod _impl_dataset;
/// **(internal)** Implementation of partial event-based API.
mod _impl_events;
/// **(internal)** Implementation of [Serialize] and [Deserialize] traits for `Dataset`.
mod _impl_serde;

/// An ordered list of observations (of potentially specified type) for given variables.
/// The order is important for some datasets, for example, to be able to capture time series.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dataset {
    /// List of binarized observations.
    observations: Vec<Observation>,
    /// Variables captured by the observations.
    var_names: Vec<String>,
    /// Type of this dataset.
    data_type: ObservationType,
    /// Index map from observation IDs to their index in vector, for faster searching.
    index_map: HashMap<ObservationId, usize>,
}

impl Display for Dataset {
    /// Use json serialization to convert `Dataset` to string.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl FromStr for Dataset {
    type Err = String;

    /// Use json de-serialization to construct `Dataset` from string.
    fn from_str(s: &str) -> Result<Dataset, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}
