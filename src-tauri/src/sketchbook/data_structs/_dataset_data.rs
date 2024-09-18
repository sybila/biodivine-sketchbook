use crate::sketchbook::data_structs::ObservationData;
use crate::sketchbook::ids::DatasetId;
use crate::sketchbook::observations::{Dataset, Observation};
use crate::sketchbook::JsonSerde;
use serde::{Deserialize, Serialize};

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

/// Structure for sending *metadata* about `Dataset`. This includes id, variable names,
/// but excludes all observations.
///
/// Some fields simplified compared to original typesafe versions (e.g., pure `Strings` are used
/// instead of more complex typesafe structs) to allow for easier (de)serialization.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DatasetMetaData {
    pub id: String,
    pub variables: Vec<String>,
}

impl<'de> JsonSerde<'de> for DatasetData {}
impl<'de> JsonSerde<'de> for DatasetMetaData {}

impl DatasetData {
    /// Create new `DatasetData` object given a reference to a dataset and its ID.
    pub fn from_dataset(id: &DatasetId, dataset: &Dataset) -> DatasetData {
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
        Dataset::new(observations, variables)
    }
}

impl DatasetMetaData {
    /// Create new `DatasetMetaData` object given a reference to a dataset and its ID.
    pub fn from_dataset(id: &DatasetId, dataset: &Dataset) -> DatasetMetaData {
        let variables = dataset.variables().iter().map(|v| v.to_string()).collect();
        DatasetMetaData {
            id: id.to_string(),
            variables,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::data_structs::DatasetData;
    use crate::sketchbook::ids::DatasetId;
    use crate::sketchbook::observations::{Dataset, Observation};

    #[test]
    /// Test converting between `Dataset` and `DatasetData`.
    fn test_converting() {
        let dataset_id = DatasetId::new("d").unwrap();
        let obs1 = Observation::try_from_str("*1", "o1").unwrap();
        let obs2 = Observation::try_from_str("00", "o2").unwrap();
        let dataset_before = Dataset::new(vec![obs1, obs2], vec!["a", "b"]).unwrap();
        let dataset_data = DatasetData::from_dataset(&dataset_id, &dataset_before);
        let dataset_after = dataset_data.to_dataset().unwrap();

        assert_eq!(dataset_before, dataset_after);
    }
}
