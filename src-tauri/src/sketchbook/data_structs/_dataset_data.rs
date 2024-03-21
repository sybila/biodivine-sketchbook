use crate::sketchbook::data_structs::ObservationData;
use crate::sketchbook::observations::Dataset;
use crate::sketchbook::DatasetId;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// Structure for sending data about `Dataset` .
///
/// Some fields simplified compared to original typesafe versions (e.g., pure `Strings` are used
/// instead of more complex typesafe structs) to allow for easier (de)serialization.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DatasetData {
    pub id: String,
    pub observations: Vec<ObservationData>,
    pub variables: Vec<String>,
}

impl DatasetData {
    /// Create new `DatasetData` object given a reference to a dataset and its ID.
    pub fn from_dataset(dataset: &Dataset, id: &DatasetId) -> DatasetData {
        let observations = dataset
            .observations()
            .iter()
            .map(ObservationData::from_obs)
            .collect();
        DatasetData {
            id: id.to_string(),
            observations,
            variables: dataset.variables().clone(),
        }
    }
}

impl Display for DatasetData {
    /// Use json serialization to convert `DatasetData` to string.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl FromStr for DatasetData {
    type Err = String;

    /// Use json de-serialization to construct `DatasetData` from string.
    fn from_str(s: &str) -> Result<DatasetData, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}
