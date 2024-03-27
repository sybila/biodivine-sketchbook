use crate::sketchbook::observations::Observation;
use crate::sketchbook::{DatasetId, JsonSerde};
use serde::{Deserialize, Serialize};

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

impl<'de> JsonSerde<'de> for ObservationData {}

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
        Observation::try_from_str(&self.values.clone(), &self.id)
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::data_structs::ObservationData;
    use crate::sketchbook::observations::Observation;
    use crate::sketchbook::DatasetId;

    #[test]
    /// Test converting between `Observation` and `ObservationData`.
    fn test_converting() {
        let dataset_id = DatasetId::new("d").unwrap();
        let obs_before = Observation::try_from_str("0011*", "o").unwrap();
        let obs_data = ObservationData::from_obs(&obs_before, &dataset_id);
        let obs_after = obs_data.to_observation().unwrap();

        assert_eq!(obs_before, obs_after);
    }
}
