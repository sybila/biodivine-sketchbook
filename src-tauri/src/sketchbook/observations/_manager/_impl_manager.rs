use crate::sketchbook::ids::{DatasetId, ObservationId, VarId};
use crate::sketchbook::observations::{Dataset, DatasetIterator, Observation, ObservationManager};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

/// Creating instances of `ObservationManager`.
impl ObservationManager {
    /// Instantiate `ObservationManager` with empty list of datasets.
    pub fn new_empty() -> ObservationManager {
        ObservationManager {
            datasets: HashMap::new(),
        }
    }

    /// Instantiate `ObservationManager` with given list of ID-dataset pairs.
    pub fn from_datasets(datasets: Vec<(&str, Dataset)>) -> Result<ObservationManager, String> {
        let mut manager = ObservationManager::new_empty();

        let prop_id_set = datasets.iter().map(|pair| pair.0).collect::<HashSet<_>>();
        if prop_id_set.len() != datasets.len() {
            return Err(format!("Datasets {:?} contain duplicate IDs.", datasets));
        }

        for (id, dataset) in datasets {
            let dataset_id = DatasetId::new(id)?;
            manager.datasets.insert(dataset_id, dataset);
        }
        Ok(manager)
    }
}

/// Editing `ObservationManager`.
impl ObservationManager {
    /// Add a new dataset with given `id` to this `ObservationManager`.
    ///
    /// The ID must be valid identifier that is not already used by some other dataset.
    /// Returns `Err` in case the `id` is already being used.
    pub fn add_dataset(&mut self, id: DatasetId, dataset: Dataset) -> Result<(), String> {
        self.assert_no_dataset(&id)?;
        self.datasets.insert(id, dataset);
        Ok(())
    }

    /// Add a new dataset with given string `id` to this `ObservationManager`.
    ///
    /// The ID must be valid identifier that is not already used by some other dataset.
    /// Returns `Err` in case the `id` is already being used.
    pub fn add_dataset_by_str(&mut self, id: &str, dataset: Dataset) -> Result<(), String> {
        let dataset_id = DatasetId::new(id)?;
        self.add_dataset(dataset_id, dataset)
    }

    /// Shorthand to add a list of new datasets with given string IDs to this manager.
    ///
    /// The ID must be valid identifier that is not already used by some other dataset.
    /// Returns `Err` in case the `id` is already being used.
    pub fn add_multiple_datasets(
        &mut self,
        id_name_pairs: Vec<(&str, Dataset)>,
    ) -> Result<(), String> {
        // before making any changes, check if all IDs are actually valid
        for (id, _) in &id_name_pairs {
            let dataset_id = DatasetId::new(id)?;
            self.assert_no_dataset(&dataset_id)?;
        }
        for (id, name) in id_name_pairs {
            self.add_dataset_by_str(id, name)?;
        }
        Ok(())
    }

    /// Swap content of a dataset with given `id`. The ID must be valid identifier.
    pub fn swap_dataset_content(
        &mut self,
        id: &DatasetId,
        new_content: Dataset,
    ) -> Result<(), String> {
        self.assert_valid_dataset(id)?;
        self.datasets.insert(id.clone(), new_content);
        Ok(())
    }

    /// Swap content of a dataset with given `id`. The ID must be valid identifier.
    pub fn swap_dataset_content_by_str(
        &mut self,
        id: &str,
        new_content: Dataset,
    ) -> Result<(), String> {
        let dataset_id = DatasetId::new(id)?;
        self.swap_dataset_content(&dataset_id, new_content)
    }

    /// Set the id of dataset with `original_id` to `new_id`.
    pub fn set_dataset_id(
        &mut self,
        original_id: &DatasetId,
        new_id: DatasetId,
    ) -> Result<(), String> {
        self.assert_valid_dataset(original_id)?;
        self.assert_no_dataset(&new_id)?;

        if let Some(dataset) = self.datasets.remove(original_id) {
            self.datasets.insert(new_id.clone(), dataset);
        } else {
            panic!("Error when modifying dataset's id in the dataset map.");
        }
        Ok(())
    }

    /// Set the id of dataset with `original_id` to `new_id`.
    pub fn set_dataset_id_by_str(&mut self, original_id: &str, new_id: &str) -> Result<(), String> {
        let original_id = DatasetId::new(original_id)?;
        let new_id = DatasetId::new(new_id)?;
        self.set_dataset_id(&original_id, new_id)
    }

    /// Set the id of a variable with `original_id` (in a given dataset) to `new_id`.
    pub fn set_var_id(
        &mut self,
        dataset_id: &DatasetId,
        original_id: &VarId,
        new_id: VarId,
    ) -> Result<(), String> {
        self.assert_valid_dataset(dataset_id)?;
        self.datasets
            .get_mut(dataset_id)
            .unwrap()
            .set_var_id(original_id, new_id)
    }

    /// Set the id of a variable with `original_id` (in a given dataset) to `new_id`.
    pub fn set_var_id_by_str(
        &mut self,
        dataset_id: &str,
        original_id: &str,
        new_id: &str,
    ) -> Result<(), String> {
        let dataset_id = DatasetId::new(dataset_id)?;
        let original_id = VarId::new(original_id)?;
        let new_id = VarId::new(new_id)?;
        self.set_var_id(&dataset_id, &original_id, new_id)
    }

    /// Remove variable and all the values corresponding to it from a dataset (decrementing
    /// dimension of the dataset in process).
    pub fn remove_var(&mut self, dataset_id: &DatasetId, var_id: &VarId) -> Result<(), String> {
        self.assert_valid_dataset(dataset_id)?;
        self.datasets
            .get_mut(dataset_id)
            .unwrap()
            .remove_var(var_id)
    }

    /// Remove variable and all the values corresponding to it from a dataset (decrementing
    /// dimension of the dataset in process).
    pub fn remove_var_by_str(&mut self, dataset_id: &str, id: &str) -> Result<(), String> {
        let dataset_id = DatasetId::new(dataset_id)?;
        let var_id = VarId::new(id)?;
        self.remove_var(&dataset_id, &var_id)
    }

    /// Remove the dataset with given `id` from this manager.
    /// Returns `Err` in case the `id` is not a valid dataset's identifier.
    pub fn remove_dataset(&mut self, id: &DatasetId) -> Result<(), String> {
        self.assert_valid_dataset(id)?;

        if self.datasets.remove(id).is_none() {
            panic!("Error when removing dataset {id} from the dataset map.")
        }
        Ok(())
    }

    /// Remove the dataset with given string `id` from this manager.
    /// Returns `Err` in case the `id` is not a valid dataset's identifier.
    pub fn remove_dataset_by_str(&mut self, id: &str) -> Result<(), String> {
        let dataset_id = DatasetId::new(id)?;
        self.remove_dataset(&dataset_id)
    }
}

/// Observing the `ObservationManager`.
impl ObservationManager {
    /// The number of datasets in this `ObservationManager`.
    pub fn num_datasets(&self) -> usize {
        self.datasets.len()
    }

    /// Check if there is a dataset with given Id.
    pub fn is_valid_dataset_id(&self, id: &DatasetId) -> bool {
        self.datasets.contains_key(id)
    }

    /// Return a valid dataset's `DatasetId` corresponding to the given str `id`.
    ///
    /// Return `Err` if such dataset does not exist (and the ID is invalid).
    pub fn get_dataset_id(&self, id: &str) -> Result<DatasetId, String> {
        let dataset_id = DatasetId::from_str(id)?;
        if self.is_valid_dataset_id(&dataset_id) {
            return Ok(dataset_id);
        }
        Err(format!("Dataset with ID {id} does not exist."))
    }

    /// Return a `Dataset` corresponding to a given `DatasetId`.
    ///
    /// Return `Err` if such dataset does not exist (the ID is invalid in this context).
    pub fn get_dataset(&self, id: &DatasetId) -> Result<&Dataset, String> {
        let dataset = self
            .datasets
            .get(id)
            .ok_or(format!("Dataset with ID {id} does not exist."))?;
        Ok(dataset)
    }

    /// Return a `Dataset` corresponding to a given id given as string.
    ///
    /// Return `Err` if such dataset does not exist (the ID is invalid in this context).
    pub fn get_dataset_by_str(&self, id: &str) -> Result<&Dataset, String> {
        let dataset_id = DatasetId::new(id)?;
        self.get_dataset(&dataset_id)
    }

    /// Shorthand to get `ObservationId` from a specified dataset.
    ///
    /// Return `Err` if such dataset does not exist (the ID is invalid in this context).
    pub fn get_obs_id(&self, dataset_id: &str, obs_id: &str) -> Result<ObservationId, String> {
        let dataset = self.get_dataset_by_str(dataset_id)?;
        dataset.get_obs_id_by_str(obs_id)
    }

    /// Shorthand to get `Observation` with a given id, from a specified dataset.
    ///
    /// Return `Err` if such dataset does not exist (the ID is invalid in this context).
    pub fn get_obs(
        &self,
        dataset_id: &DatasetId,
        obs_id: &ObservationId,
    ) -> Result<&Observation, String> {
        let dataset = self.get_dataset(dataset_id)?;
        dataset.get_obs(obs_id)
    }

    /// Shorthand to get `Observation` with a given string id, from a specified dataset.
    ///
    /// Return `Err` if such dataset (or observation) does not exist (the ID is invalid
    /// in this context).
    pub fn get_obs_by_str(&self, dataset_id: &str, obs_id: &str) -> Result<&Observation, String> {
        let dataset_id = DatasetId::new(dataset_id)?;
        let obs_id = ObservationId::new(obs_id)?;
        self.get_obs(&dataset_id, &obs_id)
    }

    /// Return an iterator over all datasets of this model.
    pub fn datasets(&self) -> DatasetIterator {
        self.datasets.iter()
    }

    /// **(internal)** Utility method to ensure there is no dataset with given ID yet.
    fn assert_no_dataset(&self, id: &DatasetId) -> Result<(), String> {
        if self.is_valid_dataset_id(id) {
            Err(format!("Dataset with id {id} already exists."))
        } else {
            Ok(())
        }
    }

    /// **(internal)** Utility method to ensure there is a dataset with given ID.
    fn assert_valid_dataset(&self, id: &DatasetId) -> Result<(), String> {
        if self.is_valid_dataset_id(id) {
            Ok(())
        } else {
            Err(format!("Dataset with id {id} does not exist."))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::observations::{Dataset, Observation, ObservationManager};

    #[test]
    /// Test that valid manager instances are created correctly, and invalid case is handled.
    fn test_new_manager() {
        let manager = ObservationManager::new_empty();
        assert_eq!(manager.num_datasets(), 0);

        let d1 = Dataset::new(vec![], vec!["a", "b"]).unwrap();
        let d2 = Dataset::new(vec![], vec!["a", "c"]).unwrap();
        let dataset_list = vec![("d1", d1.clone()), ("d2", d2.clone())];
        let manager = ObservationManager::from_datasets(dataset_list).unwrap();
        assert_eq!(manager.num_datasets(), 2);

        // test also invalid, with non-unique IDs
        let dataset_list = vec![("d", d1.clone()), ("d", d2.clone())];
        assert!(ObservationManager::from_datasets(dataset_list).is_err());
    }

    #[test]
    /// Test adding/removing datasets.
    fn test_manipulate_datasets() {
        let o1 = Observation::try_from_str("*", "o").unwrap();
        let o2 = Observation::try_from_str("0", "p").unwrap();

        let d1 = Dataset::new(vec![o1, o2], vec!["a"]).unwrap();
        let d2 = Dataset::new(vec![], vec!["a", "c"]).unwrap();
        let dataset_list = vec![("d1", d1.clone()), ("d2", d2.clone())];

        let mut manager = ObservationManager::from_datasets(dataset_list).unwrap();
        assert_eq!(manager.num_datasets(), 2);

        // add dataset
        let d3 = Dataset::new(vec![], vec!["a", "c"]).unwrap();
        manager.add_dataset_by_str("d3", d3.clone()).unwrap();
        assert_eq!(manager.num_datasets(), 3);

        // try adding dataset with the same ID again (should fail)
        let d3 = Dataset::new(vec![], vec!["a", "c"]).unwrap();
        assert!(manager.add_multiple_datasets(vec![("d3", d3)]).is_err());
        assert_eq!(manager.num_datasets(), 3);

        // remove a dataset
        manager.remove_dataset_by_str("d2").unwrap();
        assert_eq!(manager.num_datasets(), 2);

        // try removing dataset with invalid (already removed) ID
        assert!(manager.remove_dataset_by_str("d2").is_err());
        assert_eq!(manager.num_datasets(), 2);
    }

    #[test]
    /// Test changing a dataset's ID or content.
    fn test_edit_dataset() {
        let o1 = Observation::try_from_str("*1", "o").unwrap();
        let o2 = Observation::try_from_str("00", "p").unwrap();
        let d1 = Dataset::new(vec![o1, o2], vec!["a", "b"]).unwrap();
        let dataset_list = vec![("dataset1", d1.clone())];
        let mut manager = ObservationManager::from_datasets(dataset_list).unwrap();

        // try setting ID
        manager.set_dataset_id_by_str("dataset1", "d1").unwrap();
        assert!(manager.get_dataset_id("dataset1").is_err());
        assert!(manager.get_dataset_id("d1").is_ok());

        // try setting content
        let new_dataset = Dataset::new(vec![], vec!["a", "b"]).unwrap();
        manager
            .swap_dataset_content_by_str("d1", new_dataset.clone())
            .unwrap();
        let d1 = manager.get_dataset_id("d1").unwrap();
        assert_eq!(manager.get_dataset(&d1).unwrap(), &new_dataset);
    }
}
