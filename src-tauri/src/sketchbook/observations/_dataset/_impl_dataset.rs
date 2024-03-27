use crate::sketchbook::observations::{DataCategory, Dataset, Observation, VarValue};
use crate::sketchbook::utils::assert_ids_unique;
use crate::sketchbook::{ObservationId, VarId};
use std::collections::HashMap;

/// Creating new `Dataset` instances.
impl Dataset {
    /// Create new dataset from a list of observations, variables, and type of the observations.
    ///
    /// Length of each observation and number of variables must match.
    /// Observation IDs must be valid identifiers and must be unique.
    pub fn new(
        observations: Vec<Observation>,
        var_names: Vec<&str>,
        category: DataCategory,
    ) -> Result<Self, String> {
        // check that all variables are unique and valid, same for observation IDs
        let variables = Self::try_convert_vars(&var_names)?;
        assert_ids_unique(&variables)?;
        let observation_ids: Vec<&ObservationId> =
            observations.iter().map(|o| o.get_id()).collect();
        assert_ids_unique(&observation_ids)?;

        // Check that number of variables is the same as the length of observations.
        if !observations
            .iter()
            .all(|obs| obs.num_values() == var_names.len())
        {
            return Err("Number of variables and length of observations differ.".to_string());
        }
        let mut index_map = HashMap::new();
        for (i, obs) in observations.iter().enumerate() {
            index_map.insert(obs.get_id().clone(), i); // uniqueness of observations checked before
        }

        Ok(Self {
            observations,
            variables,
            category,
            index_map,
        })
    }

    /// Shorthand to create new `empty` dataset from given variables and observations category.
    pub fn new_empty(var_names: Vec<&str>, category: DataCategory) -> Result<Self, String> {
        Self::new(Vec::new(), var_names, category)
    }

    /// Shorthand to create a new dataset with `unspecified` type of observations, given
    /// a list of observations, variables.
    ///
    /// Length of observations and number of variables must match.
    /// Lists of observations and variables must not be empty.
    pub fn new_unspecified(
        observations: Vec<Observation>,
        var_names: Vec<&str>,
    ) -> Result<Self, String> {
        Dataset::new(observations, var_names, DataCategory::Unspecified)
    }

    /// **(internal)** Try converting variables string slices into VarIDs.
    fn try_convert_vars(var_names: &[&str]) -> Result<Vec<VarId>, String> {
        var_names
            .iter()
            .map(|v| VarId::new(v))
            .collect::<Result<Vec<VarId>, String>>()
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
                self.index_map.insert(o.get_id().clone(), i - 1);
            }
        });

        let obs = self.observations.remove(idx);
        self.index_map.remove(obs.get_id());
        Ok(())
    }

    /// Add observation to a given index in the dataset.
    ///
    /// This operation might be very costly, as we must reindex all subsequent observations.
    pub fn insert_observation(&mut self, index: usize, obs: Observation) -> Result<(), String> {
        // check that inputs are valid
        self.assert_no_observation(obs.get_id())?;
        if index > self.num_observations() {
            return Err("Index is larger than number of observations.".to_string());
        }

        self.index_map.insert(obs.get_id().clone(), index);
        self.observations.insert(index, obs);
        // re-index everything after the new observation
        self.observations.iter().enumerate().for_each(|(i, o)| {
            if i > index {
                self.index_map.insert(o.get_id().clone(), i + 1);
            }
        });
        Ok(())
    }

    /// Swap value vector for an observation with given ID.
    /// The new vector of values must be of the same length as the original.
    pub fn swap_observation_data(
        &mut self,
        id: &ObservationId,
        new_values: Vec<VarValue>,
    ) -> Result<(), String> {
        let idx = self.get_observation_index(id)?;
        self.observations[idx].set_all_values(new_values)
    }

    /// Set the id of variable with `original_id` to `new_id`.
    pub fn set_var_id(&mut self, original_id: &VarId, new_id: VarId) -> Result<(), String> {
        self.assert_valid_variable(original_id)?;
        self.assert_no_variable(&new_id)?;

        // we already checked that the variable must exist on some position
        if let Some(idx) = self.variables.iter().position(|v| v == original_id) {
            self.variables[idx] = new_id;
        }
        Ok(())
    }

    /// Set the id of variable given by string `original_id` to `new_id`.
    pub fn set_var_id_by_str(&mut self, original_id: &str, new_id: &str) -> Result<(), String> {
        let original_id = VarId::new(original_id)?;
        let new_id = VarId::new(new_id)?;
        self.set_var_id(&original_id, new_id)
    }

    /// Set the id of an observation with `original_id` to `new_id`.
    pub fn set_obs_id(
        &mut self,
        original_id: &ObservationId,
        new_id: ObservationId,
    ) -> Result<(), String> {
        self.assert_valid_observation(original_id)?;
        self.assert_no_observation(&new_id)?;

        // we must update both the observation instance and the index map
        let idx = self.get_observation_index(original_id)?;
        self.observations
            .get_mut(idx)
            .unwrap()
            .set_id(new_id.clone());
        self.index_map.remove(original_id);
        self.index_map.insert(new_id, idx);
        Ok(())
    }

    /// Set the id of observation given by string `original_id` to `new_id`.
    pub fn set_obs_id_by_str(&mut self, original_id: &str, new_id: &str) -> Result<(), String> {
        let original_id = ObservationId::new(original_id)?;
        let new_id = ObservationId::new(new_id)?;
        self.set_obs_id(&original_id, new_id)
    }

    /// Set the category of data for this dataset.
    pub fn set_category(&mut self, category: DataCategory) {
        self.category = category;
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
        self.variables.len()
    }

    /// Check if variable is tracked in this dataset.
    pub fn is_valid_variable(&self, var: &VarId) -> bool {
        self.variables.contains(var)
    }

    /// Check if observation is present in this dataset.
    pub fn is_valid_observation(&self, id: &ObservationId) -> bool {
        self.index_map.contains_key(id)
    }

    /// Observation on given index (indexing starts at 0).
    pub fn get_observation_on_idx(&self, index: usize) -> Result<&Observation, String> {
        if index >= self.num_observations() {
            return Err("Index is larger than number of observations.".to_string());
        }
        Ok(&self.observations[index])
    }

    /// Observation with given ID.
    pub fn get_observation(&self, id: &ObservationId) -> Result<&Observation, String> {
        let obs_idx = self.get_observation_index(id)?;
        self.get_observation_on_idx(obs_idx)
    }

    /// ID of an observation on given index.
    pub fn get_observation_id(&self, index: usize) -> &ObservationId {
        self.observations[index].get_id()
    }

    /// Get index of given observation, or None (if not present).
    /// Indexing starts at 0.
    pub fn get_observation_index(&self, id: &ObservationId) -> Result<usize, String> {
        self.assert_valid_observation(id)?;
        Ok(*self.index_map.get(id).unwrap())
    }

    /// Vector of all observations.
    pub fn observations(&self) -> &Vec<Observation> {
        &self.observations
    }

    /// Vector of all variables.
    pub fn variables(&self) -> &Vec<VarId> {
        &self.variables
    }

    /// Get `VarId` for a corresponding string identifier, if it is valid.
    pub fn get_var_id(&self, id: &str) -> Result<&VarId, String> {
        // there is at max one var with given id
        if let Some(var_id) = self.variables.iter().find(|&v| v.as_str() == id) {
            Ok(var_id)
        } else {
            Err(format!(
                "Variable with ID {id} does not exist in this dataset."
            ))
        }
    }

    /// Variable on given index.
    pub fn get_variable_on_idx(&self, index: usize) -> Result<&VarId, String> {
        if index >= self.num_variables() {
            return Err("Index is larger than number of variables.".to_string());
        }
        Ok(&self.variables[index])
    }

    /// Category of the data.
    pub fn category(&self) -> &DataCategory {
        &self.category
    }

    /// Make a string describing this `Dataset` in a human-readable format.
    /// If `list_all` is set to `true`, all observation vectors are listed. Otherwise, just
    /// a summary is given (number of observations).
    ///
    /// This is mainly for debug purposes, as it is different than classical string serialization.
    pub fn to_debug_string(&self, list_all: bool) -> String {
        let len = self.observations.len();
        let category = self.category.to_string();

        let mut var_string = String::new();
        for variable in &self.variables {
            var_string.push_str(format!("{variable}, ").as_str());
        }
        var_string = var_string.strip_suffix(", ").unwrap().to_string();

        if !list_all {
            return format!("{len} `{category}` observations with vars [{var_string}]");
        }

        let mut obs_string = String::new();
        for observation in &self.observations {
            obs_string.push_str(format!("{}, ", observation.to_debug_string()).as_str());
        }
        obs_string = obs_string.strip_suffix(", ").unwrap().to_string();

        format!("{len} `{category}` observations with vars [{var_string}]: [{obs_string}]")
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

    /// **(internal)** Utility method to ensure there is no variable with given Id yet.
    fn assert_no_variable(&self, var_id: &VarId) -> Result<(), String> {
        if self.is_valid_variable(var_id) {
            Err(format!(
                "Variable with id {var_id} already exists in this dataset."
            ))
        } else {
            Ok(())
        }
    }

    /// **(internal)** Utility method to ensure there is a variable with given Id.
    fn assert_valid_variable(&self, var_id: &VarId) -> Result<(), String> {
        if self.is_valid_variable(var_id) {
            Ok(())
        } else {
            Err(format!(
                "Variable with id {var_id} does not exist in this dataset."
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::observations::{DataCategory, Dataset, Observation};

    #[test]
    /// Test that valid datasets are created correctly.
    fn test_new_dataset() {
        let obs1 = Observation::try_from_str("*11", "i1").unwrap();
        let obs2 = Observation::try_from_str("000", "i2").unwrap();
        let obs_list = vec![obs1, obs2];
        let var_names = vec!["a", "b", "c"];
        let data_type = DataCategory::FixedPoint;

        let dataset = Dataset::new_empty(var_names.clone(), data_type).unwrap();
        assert_eq!(dataset.num_observations(), 0);
        assert_eq!(dataset.num_variables(), 3);

        let dataset = Dataset::new(obs_list.clone(), var_names.clone(), data_type).unwrap();
        assert_eq!(dataset.num_observations(), 2);
        assert_eq!(dataset.num_variables(), 3);
        assert_eq!(dataset.category(), &data_type);

        let dataset = Dataset::new_unspecified(obs_list.clone(), var_names.clone()).unwrap();
        assert_eq!(dataset.num_observations(), 2);
        assert_eq!(dataset.num_variables(), 3);
        assert_eq!(dataset.category(), &DataCategory::Unspecified);
    }

    #[test]
    /// Test that invalid datasets cannot be created.
    fn test_invalid_dataset() {
        let obs1 = Observation::try_from_str("*1", "i1").unwrap();
        let obs2 = Observation::try_from_str("000", "i2").unwrap();
        let var_names = vec!["a", "b"];
        let data_type = DataCategory::Unspecified;

        // two cases where length of observation and number variables differs
        let observations = vec![obs2.clone()];
        let obs_list = Dataset::new(observations, var_names.clone(), data_type);
        assert!(obs_list.is_err());

        let observations = vec![obs1.clone(), obs2.clone()];
        let obs_list = Dataset::new(observations, var_names.clone(), data_type);
        assert!(obs_list.is_err());

        // trying to add observation with the same id twice
        let observations = vec![obs1.clone(), obs1.clone()];
        let obs_list = Dataset::new(observations, var_names.clone(), data_type);
        assert!(obs_list.is_err());
    }

    #[test]
    /// Test adding/removing/editing observations in a dataset (both valid and invalid cases).
    fn test_manipulate_observations() {
        let obs1 = Observation::try_from_str("*1", "o").unwrap();
        let obs2 = Observation::try_from_str("00", "p").unwrap();
        let obs3 = Observation::try_from_str("11", "q").unwrap();

        let data_type = DataCategory::Attractor;
        let initial_obs_list = vec![obs1.clone(), obs2.clone()];
        let mut dataset = Dataset::new(initial_obs_list, vec!["a", "b"], data_type).unwrap();

        // add observation
        dataset.push_observation(obs3.clone()).unwrap();
        let all_three_obs = vec![obs1.clone(), obs2.clone(), obs3.clone()];
        assert_eq!(dataset.observations(), &all_three_obs);

        // try adding same observation again (should fail)
        assert!(dataset.push_observation(obs3.clone()).is_err());
        assert_eq!(dataset.observations(), &all_three_obs);

        // remove observation that is not last
        let obs1_id = obs1.get_id();
        dataset.remove_observation(obs1_id).unwrap();
        assert_eq!(dataset.observations(), &vec![obs2.clone(), obs3.clone()]);

        // remove last observation
        dataset.pop_observation();
        assert_eq!(dataset.observations(), &vec![obs2.clone()]);

        // finally, try re-adding one of the removed observations
        dataset.push_observation(obs1.clone()).unwrap();
        assert_eq!(dataset.observations(), &vec![obs2.clone(), obs1.clone()]);
    }

    #[test]
    /// Test changing observation's ID (both valid and invalid cases).
    fn test_set_observation_id() {
        let obs1 = Observation::try_from_str("*1", "o").unwrap();
        let obs2 = Observation::try_from_str("00", "p").unwrap();
        let data_type = DataCategory::Attractor;
        let mut dataset = Dataset::new(vec![obs1, obs2], vec!["a", "b"], data_type).unwrap();

        // valid case
        dataset.set_obs_id_by_str("o", "o2").unwrap();
        assert_eq!(
            dataset.get_observation_on_idx(0).unwrap().get_id().as_str(),
            "o2"
        );

        // invalid case, ID already in use
        assert!(dataset.set_obs_id_by_str("p", "o2").is_err());
    }

    #[test]
    /// Test changing variable's ID (both valid and invalid cases).
    fn test_set_var_id() {
        let data_type = DataCategory::Attractor;
        let mut dataset = Dataset::new_empty(vec!["a", "b"], data_type).unwrap();

        // valid case
        dataset.set_var_id_by_str("a", "a2").unwrap();
        assert_eq!(dataset.get_variable_on_idx(0).unwrap().as_str(), "a2");

        // invalid case, ID already in use
        assert!(dataset.set_obs_id_by_str("p", "o2").is_err());
    }

    #[test]
    /// Test displaying of string description of datasets.
    fn test_debug_str() {
        let obs1 = Observation::try_from_str("*1", "o").unwrap();
        let obs2 = Observation::try_from_str("00", "p").unwrap();
        let data_type = DataCategory::Attractor;
        let dataset = Dataset::new(vec![obs1, obs2], vec!["a", "b"], data_type).unwrap();

        let full_str = "2 `Attractor` observations with vars [a, b]: [o(*1), p(00)]";
        let short_str = "2 `Attractor` observations with vars [a, b]";
        assert_eq!(dataset.to_debug_string(true), full_str.to_string());
        assert_eq!(dataset.to_debug_string(false), short_str.to_string());
    }
}
