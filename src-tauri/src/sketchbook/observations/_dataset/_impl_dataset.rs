use crate::sketchbook::observations::{Dataset, Observation, ObservationType, VarValue};
use crate::sketchbook::ObservationId;
use std::collections::HashMap;

/// Creating new `Dataset` instances.
impl Dataset {
    /// Create new dataset from a list of observations, variables, and type of the observations.
    ///
    /// Length of each observation and number of variables must match.
    /// Observation IDs must be unique.
    pub fn new(
        observations: Vec<Observation>,
        var_names: Vec<String>,
        data_type: ObservationType,
    ) -> Result<Self, String> {
        // Check that number of variables is the same as the length of observations.
        if !observations
            .iter()
            .all(|obs| obs.num_values() == var_names.len())
        {
            return Err("Number of variables and length of observations differ.".to_string());
        }
        let mut index_map = HashMap::new();
        for (i, obs) in observations.iter().enumerate() {
            if index_map.insert(obs.get_id().clone(), i).is_some() {
                return Err(format!("Duplicate observation ID found {}.", obs.get_id()));
            }
        }
        Ok(Self {
            observations,
            var_names,
            data_type,
            index_map,
        })
    }

    /// Create new empty dataset from given variables and observations type.
    pub fn new_empty(var_names: Vec<String>, data_type: ObservationType) -> Result<Self, String> {
        Ok(Self {
            observations: Vec::new(),
            var_names,
            data_type,
            index_map: HashMap::new(),
        })
    }

    /// Shorthand to create a new dataset with `unspecified` type of observations, given
    /// a list of observations, variables.
    ///
    /// Length of observations and number of variables must match.
    /// Lists of observations and variables must not be empty.
    pub fn new_unspecified(
        observations: Vec<Observation>,
        var_names: Vec<String>,
    ) -> Result<Self, String> {
        Dataset::new(observations, var_names, ObservationType::Unspecified)
    }
}

/// Editing `Dataset` instances.
impl Dataset {
    /// Add observation at the end of the dataset.
    ///
    /// The observation must have the same length as is the number of dataset's variables, and its
    /// id must not be already present in the dataset.
    pub fn push_observation(&mut self, obs: Observation) -> Result<(), String> {
        self.assert_no_observation(obs.get_id())?;
        self.index_map
            .insert(obs.get_id().clone(), self.observations.len());
        self.observations.push(obs);
        Ok(())
    }

    /// Remove observation from the end of the dataset.
    /// If no observations, nothing happens.
    pub fn pop_observation(&mut self) {
        if let Some(obs) = self.observations.pop() {
            self.index_map.remove(obs.get_id());
        }
    }

    /// Remove observation with given ID from the dataset. The ID must be valid
    ///
    /// This operation might be very costly, as we must reindex all subsequent observations.
    pub fn remove_observation(&mut self, id: &ObservationId) -> Result<(), String> {
        let idx = self.get_observation_index(id)?;
        // re-index everything after the to-be-deleted observation
        self.observations.iter().enumerate().for_each(|(i, o)| {
            if i > idx {
                self.index_map.insert(o.get_id().clone(), i + 1);
            }
        });

        let obs = self.observations.remove(idx);
        self.index_map.remove(obs.get_id());
        Ok(())
    }

    /// Swap value vector for an observation with given ID.
    /// The new vector of values must be of the same length as the original.
    pub fn update_observation(
        &mut self,
        id: &ObservationId,
        new_values: Vec<VarValue>,
    ) -> Result<(), String> {
        let idx = self.get_observation_index(id)?;
        self.observations[idx].set_all_values(new_values)
    }
}

/// Observing `Dataset` instances.
impl Dataset {
    /// Number of observations in the dataset.
    pub fn num_observations(&self) -> usize {
        self.observations.len()
    }

    /// Number of variables tracked by the dataset.
    pub fn num_variables(&self) -> usize {
        self.var_names.len()
    }

    /// Check if variable is tracked in this dataset.
    pub fn is_valid_variable(&self, var: &String) -> bool {
        self.var_names.contains(var)
    }

    /// Check if observation is present in this dataset.
    pub fn is_valid_observation(&self, id: &ObservationId) -> bool {
        self.index_map.contains_key(id)
    }

    /// Observation on given index.
    pub fn get_observation_on_idx(&self, index: usize) -> &Observation {
        &self.observations[index]
    }

    /// Observation with given ID.
    pub fn get_observation(&self, id: &ObservationId) -> Result<&Observation, String> {
        let obs_idx = self.get_observation_index(id)?;
        Ok(self.get_observation_on_idx(obs_idx))
    }

    /// ID of an observation on given index.
    pub fn get_observation_id(&self, index: usize) -> &ObservationId {
        self.observations[index].get_id()
    }

    /// Get index of given observation, or None (if not present).
    pub fn get_observation_index(&self, id: &ObservationId) -> Result<usize, String> {
        self.assert_valid_observation(id)?;
        Ok(*self.index_map.get(id).unwrap())
    }

    /// Vector of all observations.
    pub fn observations(&self) -> &Vec<Observation> {
        &self.observations
    }

    /// Vector of all variables.
    pub fn variables(&self) -> &Vec<String> {
        &self.var_names
    }

    /// Variable on given index.
    pub fn get_variable(&self, index: usize) -> &String {
        &self.var_names[index]
    }

    /// Type of the data.
    pub fn data_type(&self) -> &ObservationType {
        &self.data_type
    }

    /// Make a string describing this `Dataset` in a human-readable format.
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
            obs_string.push_str(format!("{}, ", observation.to_debug_string()).as_str());
        }
        obs_string = obs_string.strip_suffix(", ").unwrap().to_string();

        format!("{len} `{data_type}` observations with vars [{var_string}]: [{obs_string}]")
    }

    /// **(internal)** Utility method to ensure there is no observation with given ID yet.
    fn assert_no_observation(&self, id: &ObservationId) -> Result<(), String> {
        if self.is_valid_observation(id) {
            Err(format!("Observation with id {id} already exists."))
        } else {
            Ok(())
        }
    }

    /// **(internal)** Utility method to ensure there is a observation with given ID.
    fn assert_valid_observation(&self, id: &ObservationId) -> Result<(), String> {
        if self.is_valid_observation(id) {
            Ok(())
        } else {
            Err(format!("Observation with id {id} does not exist."))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::observations::{Dataset, Observation, ObservationType};

    #[test]
    /// Test displaying of string description of observation lists.
    fn test_invalid_observation_list() {
        let obs1 = Observation::try_from_str("*1".to_string(), "id").unwrap();
        let obs2 = Observation::try_from_str("000".to_string(), "id").unwrap();
        let var_names = vec!["a".to_string(), "b".to_string()];
        let obs_type = ObservationType::Unspecified;

        // length of observation and number variables differs
        let observations = vec![obs2.clone()];
        let obs_list = Dataset::new(observations, var_names.clone(), obs_type);
        assert!(obs_list.is_err());

        let observations = vec![obs1.clone(), obs2.clone()];
        let obs_list = Dataset::new(observations, var_names.clone(), obs_type);
        assert!(obs_list.is_err());
    }

    #[test]
    /// Test displaying of string description of observation lists.
    fn test_debug_str() {
        let observation1 = Observation::try_from_str("*1".to_string(), "a").unwrap();
        let observation2 = Observation::try_from_str("00".to_string(), "b").unwrap();
        let observation_list = Dataset::new(
            vec![observation1, observation2],
            vec!["a".to_string(), "b".to_string()],
            ObservationType::Attractor,
        )
        .unwrap();

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
