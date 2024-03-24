use crate::sketchbook::data_structs::ObservationData;
use crate::sketchbook::observations::{Dataset, Observation, ObservationType};
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
    pub data_type: ObservationType,
}

impl DatasetData {
    /// Create new `DatasetData` object given a reference to a dataset and its ID.
    pub fn from_dataset(dataset: &Dataset, id: &DatasetId) -> DatasetData {
        let observations = dataset
            .observations()
            .iter()
            .map(|o| ObservationData::from_obs(o, id))
            .collect();
        let variables = dataset.variables().iter().map(|v| v.to_string()).collect();
        DatasetData {
            id: id.to_string(),
            observations,
            variables,
            data_type: *dataset.data_type(),
        }
    }

    /// Convert the `DatasetData` to the corresponding `Dataset`.
    /// There is a syntax check just to make sure that the data are valid.
    pub fn to_dataset(&self) -> Result<Dataset, String> {
        let observations = self
            .observations
            .iter()
            .map(|o| o.to_observation())
            .collect::<Result<Vec<Observation>, String>>()?;
        let variables = self.variables.iter().map(|v| v.as_str()).collect();
        Dataset::new(observations, variables, self.data_type)
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

#[cfg(test)]
mod tests {
    use crate::sketchbook::data_structs::DatasetData;
    use crate::sketchbook::observations::{Dataset, Observation, ObservationType};
    use crate::sketchbook::DatasetId;

    #[test]
    /// Test converting between `Dataset` and `DatasetData`.
    fn test_converting() {
        let dataset_id = DatasetId::new("d").unwrap();
        let obs1 = Observation::try_from_str("*1", "o1").unwrap();
        let obs2 = Observation::try_from_str("00", "o2").unwrap();
        let dataset_before =
            Dataset::new(vec![obs1, obs2], vec!["a", "b"], ObservationType::Attractor).unwrap();
        let dataset_data = DatasetData::from_dataset(&dataset_before, &dataset_id);
        let dataset_after = dataset_data.to_dataset().unwrap();

        assert_eq!(dataset_before, dataset_after);
    }
}
