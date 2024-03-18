use crate::sketchbook::observations::{Observation, ObservationType};
use serde::{Deserialize, Serialize};

/// An ordered list of observations. The order is important for some datasets, for example,
/// to be able to capture time series.
/// Contains binarized observations' data, names for variables, and type of data.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ObservationList {
    /// List of binarized observations.
    pub observations: Vec<Observation>,
    /// Variables captured by the observations.
    pub var_names: Vec<String>,
    /// Type of this dataset.
    pub data_type: ObservationType,
}

impl ObservationList {
    /// Generate new `ObservationList`.
    ///
    /// Length of observations and number of variables must match.
    /// Lists of observations and variables must not be empty.
    pub fn new(
        observations: Vec<Observation>,
        var_names: Vec<String>,
        data_type: ObservationType,
    ) -> Result<Self, String> {
        // Check that number of variables is the same as the length of observations.
        if observations.is_empty() {
            return Err("List of observations cannot be empty.".to_string());
        }
        if var_names.is_empty() {
            return Err("List of variables cannot be empty.".to_string());
        }
        if !observations
            .iter()
            .all(|obs| obs.num_values() == var_names.len())
        {
            return Err("Number of variables and length of observations differs.".to_string());
        }
        Ok(Self {
            observations,
            var_names,
            data_type,
        })
    }

    /// Make a string describing this `ObservationList` in a human-readable format.
    /// If `list_all` is set to `true`, all observation vectors are listed. Otherwise, just
    /// a summary is given (number of observations).
    ///
    /// This is mainly for debug purposes, as it is different than classical string serialization.
    pub fn to_debug_string(&self, list_all: bool) -> String {
        let len = self.observations.len();
        let data_type = self.data_type.to_string();

        let mut var_string = String::new();
        for variable in &self.var_names {
            var_string.push_str(format!("{variable}, ").as_str());
        }
        var_string = var_string.strip_suffix(", ").unwrap().to_string();

        if !list_all {
            return format!("{len} `{data_type}` observations with vars [{var_string}]");
        }

        let mut obs_string = String::new();
        for observation in &self.observations {
            obs_string.push_str(format!("{}, ", observation.to_debug_string(false)).as_str());
        }
        obs_string = obs_string.strip_suffix(", ").unwrap().to_string();

        format!("{len} `{data_type}` observations with vars [{var_string}]: [{obs_string}]")
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::observations::{Observation, ObservationList, ObservationType};

    #[test]
    /// Test displaying of string description of observation lists.
    fn test_invalid_observation_list() {
        let obs1 = Observation::try_from_str("*1".to_string(), "id").unwrap();
        let obs2 = Observation::try_from_str("000".to_string(), "id").unwrap();
        let var_names = vec!["a".to_string(), "b".to_string()];
        let obs_type = ObservationType::Unspecified;

        // length of observation and number variables differs
        let observations = vec![obs2.clone()];
        let obs_list = ObservationList::new(observations, var_names.clone(), obs_type);
        assert!(obs_list.is_err());

        let observations = vec![obs1.clone(), obs2.clone()];
        let obs_list = ObservationList::new(observations, var_names.clone(), obs_type);
        assert!(obs_list.is_err());

        // empty observations list
        let obs_list = ObservationList::new(vec![], var_names.clone(), obs_type);
        assert!(obs_list.is_err());

        // empty variables list
        let obs_list = ObservationList::new(vec![obs1], vec![], obs_type);
        assert!(obs_list.is_err());
    }

    #[test]
    /// Test displaying of string description of observation lists.
    fn test_debug_str() {
        let observation1 = Observation::try_from_str("*1".to_string(), "a").unwrap();
        let observation2 = Observation::try_from_str("00".to_string(), "b").unwrap();
        let observation_list = ObservationList {
            observations: vec![observation1, observation2],
            var_names: vec!["a".to_string(), "b".to_string()],
            data_type: ObservationType::Attractor,
        };

        let full_description = "2 `Attractor` observations with vars [a, b]: [a(*1), b(00)]";
        let short_description = "2 `Attractor` observations with vars [a, b]";
        assert_eq!(
            observation_list.to_debug_string(true),
            full_description.to_string()
        );
        assert_eq!(
            observation_list.to_debug_string(false),
            short_description.to_string()
        );
    }
}
